use merc_syntax::DefId;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;

use crate::ResolvedSortId;
use crate::TypeckContext;

/// The resolved sorts of every declaration in a checked specification, stored
/// positionally because constructor and map declarations carry no [DefId] (only
/// sort declarations do).
///
/// Covers the user specification only; the system-defined specification is
/// still unresolved content (see G3 in `docs/typecheck.md`).
pub(crate) struct DeclarationSorts {
    /// Parallel to `constructor_declarations`.
    pub(crate) constructors: Vec<ResolvedSortId>,
    /// Parallel to `map_declarations`.
    pub(crate) mappings: Vec<ResolvedSortId>,
    /// Parallel to `equation_declarations`; the inner vector is parallel to the
    /// equation's variable list.
    pub(crate) equation_variables: Vec<Vec<ResolvedSortId>>,
}

/// Resolves the sort of every constructor, map and equation variable in `spec`
/// onto the interned sort lattice of `ctx`.
///
/// Requires `spec` to have passed the `from_untyped` pipeline up to and
/// including `normalize_sorts`: names resolved, structured sorts desugared, and
/// alias indirection expanded.
pub(crate) fn resolve_declaration_sorts(ctx: &mut TypeckContext, spec: &UntypedDataSpecification) -> DeclarationSorts {
    let result = DeclarationSorts {
        constructors: spec
            .constructor_declarations
            .iter()
            .map(|decl| resolve_sort(ctx, spec, &decl.sort))
            .collect(),
        mappings: spec
            .map_declarations
            .iter()
            .map(|decl| resolve_sort(ctx, spec, &decl.sort))
            .collect(),
        equation_variables: spec
            .equation_declarations
            .iter()
            .map(|equation| {
                equation
                    .variables
                    .iter()
                    .map(|var| resolve_sort(ctx, spec, &var.sort))
                    .collect()
            })
            .collect(),
    };

    debug_assert_eq!(result.constructors.len(), spec.constructor_declarations.len());
    debug_assert_eq!(result.mappings.len(), spec.map_declarations.len());
    debug_assert_eq!(result.equation_variables.len(), spec.equation_declarations.len());
    result
}

/// Resolves a single sort expression to its interned [ResolvedSortId].
///
/// Requires names resolved and structured sorts desugared; alias indirection
/// need not be expanded, since a `Resolved` sort goes through
/// [query_sort_of_def], which resolves the alias body lazily. Note that
/// flattening does not recurse into a substituted function sort, so a nested
/// higher-order sort still appears as `Function` with a `Product` domain spine;
/// both forms resolve to the same interned function sort.
pub(crate) fn resolve_sort(
    ctx: &mut TypeckContext,
    spec: &UntypedDataSpecification,
    sort: &SortExpression,
) -> ResolvedSortId {
    match sort {
        SortExpression::Simple(sort) => ctx.sorts.primitive(*sort),
        SortExpression::Complex(op, subsort) => {
            let subsort = resolve_sort(ctx, spec, subsort);
            ctx.sorts.generic(*op, subsort)
        }
        SortExpression::FlattenedFunction { domain, range } => {
            let domain = domain.iter().map(|sort| resolve_sort(ctx, spec, sort)).collect();
            let range = resolve_sort(ctx, spec, range);
            ctx.sorts.function(domain, range)
        }
        // Unreachable through the pipeline today (it flattens every function
        // sort before resolution), but kept so the resolver accepts any
        // well-formed sort expression, such as binder sorts built during
        // inference.
        SortExpression::Function { domain, range } => {
            let mut resolved_domain = Vec::new();
            resolve_function_domain(ctx, spec, domain, &mut resolved_domain);
            let range = resolve_sort(ctx, spec, range);
            ctx.sorts.function(resolved_domain, range)
        }
        SortExpression::Resolved(_, id) => query_sort_of_def(ctx, spec, *id),
        SortExpression::Reference(_) => unreachable!("Names must have been resolved"),
        SortExpression::Struct { .. } => unreachable!("Structured sorts must have been desugared"),
        SortExpression::Product { .. } => {
            unreachable!("product sorts outside a function domain were rejected before resolution")
        }
    }
}

/// Resolves the leaves of a `Product` domain spine in declaration order, the
/// resolution counterpart of `flatten_function_domain_rec`.
fn resolve_function_domain(
    ctx: &mut TypeckContext,
    spec: &UntypedDataSpecification,
    sort: &SortExpression,
    domain: &mut Vec<ResolvedSortId>,
) {
    match sort {
        SortExpression::Product { lhs, rhs } => {
            resolve_function_domain(ctx, spec, lhs, domain);
            resolve_function_domain(ctx, spec, rhs, domain);
        }
        _ => domain.push(resolve_sort(ctx, spec, sort)),
    }
}

/// Returns the resolved sort denoted by a sort declaration: the nominal sort
/// for an abstract sort or struct representative (no alias body), or the
/// resolved body for an alias. Memoized on [TypeckContext::sort_of_def].
///
/// Requires `def` to originate from name resolution of `spec`, so it indexes
/// `sort_declarations`. Cyclic aliases were rejected by `check_aliases`, so the
/// query cannot re-enter itself, whether alias bodies are normalized or not.
pub(crate) fn query_sort_of_def(
    ctx: &mut TypeckContext,
    spec: &UntypedDataSpecification,
    def: DefId,
) -> ResolvedSortId {
    debug_assert!(
        spec.sort_declarations
            .get(*def)
            .is_some_and(|decl| decl.id == Some(def)),
        "DefId {def:?} does not originate from name resolution of this specification"
    );

    match ctx
        .sort_of_def
        .get_or_lock(def)
        .expect("check_aliases rejected cyclic aliases")
    {
        Some(id) => *id,
        None => {
            let id = match &spec.sort_declarations[*def].expr {
                None => ctx.sorts.def(def),
                Some(expr) => resolve_sort(ctx, spec, expr),
            };
            *ctx.sort_of_def.unlock(def, id)
        }
    }
}

#[cfg(test)]
mod tests {
    use merc_syntax::ComplexSort;
    use merc_syntax::DefId;
    use merc_syntax::Sort;
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::ResolvedSort;
    use crate::ResolvedSortId;
    use crate::TypeckContext;
    use crate::query_sort_of_def;

    /// Type checks `text`; the returned specification carries the resolved
    /// declaration sorts and the context that interned them.
    fn typecheck(text: &str) -> DataSpecification {
        DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()).unwrap()
    }

    /// The resolved sort of the `index`-th map declaration.
    fn mapping(spec: &DataSpecification, index: usize) -> ResolvedSortId {
        spec.declaration_sorts().mappings[index]
    }

    #[test]
    fn test_resolve_basic_sort() {
        let spec = typecheck("map f: Nat;");
        assert_eq!(mapping(&spec, 0), spec.context().sorts.primitive(Sort::Nat));
    }

    #[test]
    fn test_resolve_alias_inside_container() {
        // `D = Nat` is inlined by normalization, so `List(D)` resolves to `List(Nat)`.
        let spec = typecheck("sort D = Nat; map f: List(D);");
        let sorts = &spec.context().sorts;
        let ResolvedSort::Generic { op, subsort } = sorts.get(mapping(&spec, 0)) else {
            panic!("expected a container sort");
        };
        assert_eq!(*op, ComplexSort::List);
        assert_eq!(*subsort, sorts.primitive(Sort::Nat));
    }

    #[test]
    fn test_resolve_function_sort() {
        let spec = typecheck("map f: Nat # Bool -> Real;");
        let sorts = &spec.context().sorts;
        let ResolvedSort::Function { domain, range } = sorts.get(mapping(&spec, 0)) else {
            panic!("expected a function sort");
        };
        assert_eq!(*domain, vec![sorts.primitive(Sort::Nat), sorts.primitive(Sort::Bool)]);
        assert_eq!(*range, sorts.primitive(Sort::Real));
    }

    #[test]
    fn test_resolve_higher_order_function_sort() {
        // Flattening does not recurse into the substituted sort, so the inner
        // `Nat -> Bool` is still an un-flattened `Function`; both forms must
        // resolve to the same interned function sort.
        let spec = typecheck("map f: (Nat -> Bool) -> Bool; g: Nat -> Bool;");
        let sorts = &spec.context().sorts;
        let ResolvedSort::Function { domain, range } = sorts.get(mapping(&spec, 0)) else {
            panic!("expected a function sort");
        };
        assert_eq!(*range, sorts.primitive(Sort::Bool));
        assert_eq!(*domain, vec![mapping(&spec, 1)]);
    }

    #[test]
    fn test_resolve_struct_sort_is_nominal() {
        // A structured sort resolves to the nominal sort of its declaration,
        // and its desugared constructors target that same sort.
        let spec = typecheck("sort D = struct a | b; map f: D;");
        let def = DefId::new(*spec.sorts().index("D").expect("D should be declared"));
        assert_eq!(*spec.context().sorts.get(mapping(&spec, 0)), ResolvedSort::Def(def));
        assert_eq!(spec.declaration_sorts().constructors[0], mapping(&spec, 0));
    }

    #[test]
    fn test_interned_sorts_are_shared() {
        let spec = typecheck("map f: List(Nat); g: List(Nat);");
        assert_eq!(mapping(&spec, 0), mapping(&spec, 1));
    }

    #[test]
    fn test_resolve_equation_variables() {
        let spec = typecheck("map f: Nat -> Bool; var n: Nat; eqn f(n) = true;");
        assert_eq!(
            spec.declaration_sorts().equation_variables,
            vec![vec![spec.context().sorts.primitive(Sort::Nat)]]
        );
    }

    #[test]
    fn test_query_sort_of_def_expands_alias_and_memoizes() {
        // A directly-queried alias resolves to its expanded definition; the
        // second query is answered from the cache and yields the same id.
        let spec = typecheck("sort D = List(Nat); map f: D;");
        let def = DefId::new(*spec.sorts().index("D").expect("D should be declared"));

        let mut ctx = TypeckContext::new();
        let first = query_sort_of_def(&mut ctx, spec.data_specification(), def);
        assert_eq!(first, mapping(&spec, 0));
        assert_eq!(ctx.sort_of_def.get_or_lock(def), Ok(Some(&first)));
        assert_eq!(query_sort_of_def(&mut ctx, spec.data_specification(), def), first);
    }
}
