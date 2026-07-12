use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use merc_syntax::DefId;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;

use crate::CONTAINER_TEMPLATES;
use crate::ResolvedSortId;
use crate::Signature;
use crate::TypeckContext;
use crate::WellTypedError;
use crate::push_overload;
use crate::query_sort_of_def;

/// The display names of the system-internal sorts (`@NatPair`, ...), keyed by
/// the fresh nominal [DefId]s that [resolve_system_signature] assigned to them.
///
/// These ids are numbered past the user declarations, so they never collide
/// with a [DefId] from name resolution — but they index nothing: they exist for
/// interning and display only, and must never be passed to `query_sort_of_def`.
#[derive(Debug)]
pub(crate) struct SystemSortNames {
    names: HashMap<DefId, String>,
}

impl SystemSortNames {
    /// The name of a system-internal sort, or `None` for a user [DefId].
    // Consumed by the sort rendering of inference errors (docs/typecheck.md §9).
    #[allow(dead_code)]
    pub(crate) fn name(&self, def: DefId) -> Option<&str> {
        self.names.get(&def).map(String::as_str)
    }
}

/// Resolves the constructor and mapping declarations of the system-defined
/// specification onto the interned sort lattice, giving Phase-3 inference the
/// overload sets of the built-in operators (`&&`, `+`, `|>`, ...). Stored as
/// [TypeckContext::system_signature]; the returned [SystemSortNames] name the
/// fresh ids minted for the system-internal sorts.
///
/// Unlike `query_signature` this runs no well-typedness checks: the system
/// specification is trusted content, and legitimately declares things a user
/// cannot, such as constructors for the basic sorts (`@c0: Nat`).
///
/// Requires `system` to be built by `build_system_defined_specification` from
/// the normalized `user_spec`: sorts substituted into the Appendix-B templates
/// are then `Resolved` nodes of the user specification, so only the
/// system-internal names (`@NatPair`) remain as `Reference` nodes.
pub(crate) fn resolve_system_signature(
    ctx: &mut TypeckContext,
    user_spec: &UntypedDataSpecification,
    system: &UntypedDataSpecification,
) -> Result<SystemSortNames, WellTypedError> {
    // The system specification re-declares the basic sorts (`sort Bool;`),
    // which already resolve as primitives; only the remaining declarations
    // denote system-internal nominal sorts.
    let mut sort_ids: HashMap<String, ResolvedSortId> = HashMap::new();
    let mut names = HashMap::new();
    for decl in &system.sort_declarations {
        if is_basic_sort_name(&decl.identifier) || sort_ids.contains_key(&decl.identifier) {
            continue;
        }
        debug_assert!(
            decl.expr.is_none(),
            "system-defined sorts are nominal, but '{}' has a body",
            decl.identifier
        );

        let def = DefId::new(user_spec.sort_declarations.len() + names.len());
        sort_ids.insert(decl.identifier.clone(), ctx.sorts.def(def));
        names.insert(def, decl.identifier.clone());
    }

    let mut signature = Signature {
        constructors: HashMap::new(),
        mappings: HashMap::new(),
    };

    for decl in &system.constructor_declarations {
        let id = resolve_system_sort(ctx, user_spec, &sort_ids, &decl.sort)?;
        push_overload(signature.constructors.entry(decl.identifier.clone()).or_default(), id);
    }
    for decl in &system.map_declarations {
        let id = resolve_system_sort(ctx, user_spec, &sort_ids, &decl.sort)?;
        push_overload(signature.mappings.entry(decl.identifier.clone()).or_default(), id);
    }

    ctx.system_signature = Some(Rc::new(signature));
    Ok(SystemSortNames { names })
}

fn is_basic_sort_name(name: &str) -> bool {
    matches!(name, "Bool" | "Pos" | "Nat" | "Int" | "Real")
}

/// The polymorphic signature of the container and function-update operations:
/// for each name, the overload sorts as written in the templates, with the
/// sort variables (`S`, `T`) still unresolved `Reference` nodes.
///
/// These operations exist for *every* element sort, so — like the comparison
/// schemes — inference looks them up here and instantiates the variables fresh
/// per occurrence, mirroring mCRL2's built-in polymorphic symbol table. Their
/// per-sort instantiations are deliberately *not* part of the resolved system
/// signature: listing an operation both ways would misreport ambiguity.
pub(crate) struct PolymorphicSignature {
    pub(crate) ops: HashMap<String, Vec<SortExpression>>,
}

/// The [PolymorphicSignature] of the bundled templates: the constructor and
/// mapping declarations of every raw template, collected once.
pub(crate) static POLYMORPHIC_SIGNATURE: LazyLock<PolymorphicSignature> = LazyLock::new(|| {
    let mut ops: HashMap<String, Vec<SortExpression>> = HashMap::new();
    for template in CONTAINER_TEMPLATES.all() {
        for decl in template
            .constructor_declarations
            .iter()
            .chain(&template.map_declarations)
        {
            let overloads = ops.entry(decl.identifier.clone()).or_default();
            if !overloads.contains(&decl.sort) {
                overloads.push(decl.sort.clone());
            }
        }
    }
    PolymorphicSignature { ops }
});

/// The system-defined counterpart of `resolve_sort`. It differs in two ways:
/// `Reference` nodes are looked up among the system-internal sorts (the system
/// specification never went through name resolution), and unknown references
/// are a clean error rather than a panic, so a template mistake in a
/// `spec/*.mcrl2` file cannot crash the checker.
fn resolve_system_sort(
    ctx: &mut TypeckContext,
    user_spec: &UntypedDataSpecification,
    sort_ids: &HashMap<String, ResolvedSortId>,
    sort: &SortExpression,
) -> Result<ResolvedSortId, WellTypedError> {
    match sort {
        SortExpression::Simple(sort) => Ok(ctx.sorts.primitive(*sort)),
        SortExpression::Complex(op, subsort) => {
            let subsort = resolve_system_sort(ctx, user_spec, sort_ids, subsort)?;
            Ok(ctx.sorts.generic(*op, subsort))
        }
        SortExpression::FlattenedFunction { domain, range } => {
            let domain = domain
                .iter()
                .map(|sort| resolve_system_sort(ctx, user_spec, sort_ids, sort))
                .collect::<Result<_, _>>()?;
            let range = resolve_system_sort(ctx, user_spec, sort_ids, range)?;
            Ok(ctx.sorts.function(domain, range))
        }
        // The system specification is parsed directly and never flattened, so
        // function sorts appear with a `Product` domain spine.
        SortExpression::Function { domain, range } => {
            let mut resolved_domain = Vec::new();
            resolve_system_function_domain(ctx, user_spec, sort_ids, domain, &mut resolved_domain)?;
            let range = resolve_system_sort(ctx, user_spec, sort_ids, range)?;
            Ok(ctx.sorts.function(resolved_domain, range))
        }
        // A sort substituted into an Appendix-B template comes from the
        // normalized user specification, so its `DefId` indexes `user_spec`.
        SortExpression::Resolved(_, id) => Ok(query_sort_of_def(ctx, user_spec, *id)),
        SortExpression::Reference(name) => match sort_ids.get(name) {
            Some(id) => Ok(*id),
            None => Err(WellTypedError::Custom(
                format!("the system-defined specification references the undeclared sort '{name}'").into(),
            )),
        },
        SortExpression::Struct { .. } => unreachable!("the system-defined specification has no structured sorts"),
        SortExpression::Product { .. } => {
            unreachable!("product sorts cannot occur outside a function domain")
        }
    }
}

/// Resolves the leaves of a `Product` domain spine in declaration order.
fn resolve_system_function_domain(
    ctx: &mut TypeckContext,
    user_spec: &UntypedDataSpecification,
    sort_ids: &HashMap<String, ResolvedSortId>,
    sort: &SortExpression,
    domain: &mut Vec<ResolvedSortId>,
) -> Result<(), WellTypedError> {
    match sort {
        SortExpression::Product { lhs, rhs } => {
            resolve_system_function_domain(ctx, user_spec, sort_ids, lhs, domain)?;
            resolve_system_function_domain(ctx, user_spec, sort_ids, rhs, domain)?;
        }
        _ => domain.push(resolve_system_sort(ctx, user_spec, sort_ids, sort)?),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use merc_syntax::ComplexSort;
    use merc_syntax::DefId;
    use merc_syntax::Sort;
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::ResolvedSort;
    use crate::SystemSortNames;
    use crate::TypeckContext;
    use crate::WellTypedError;
    use crate::resolve_system_signature;

    /// Type checks `text` and resolves the system signature of its
    /// system-defined specification in a fresh context.
    fn resolve(text: &str) -> (DataSpecification, TypeckContext, SystemSortNames) {
        let spec = DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()).unwrap();
        let mut ctx = TypeckContext::new();
        let names =
            resolve_system_signature(&mut ctx, spec.data_specification(), spec.system_defined_specification()).unwrap();
        (spec, ctx, names)
    }

    #[test]
    fn test_boolean_operators_are_resolved() {
        let (_, ctx, _) = resolve("map f: Bool;");
        let signature = ctx.system_signature.as_ref().unwrap();

        let bool_sort = ctx.sorts.primitive(Sort::Bool);
        let conjunction = ctx.sorts.get(signature.mappings["&&"][0]).clone();
        assert_eq!(
            conjunction,
            ResolvedSort::Function {
                domain: vec![bool_sort, bool_sort],
                range: bool_sort,
            }
        );
    }

    #[test]
    fn test_overloads_are_collected() {
        // Appendix B declares `max` for Pos # Nat, Nat # Pos and Nat # Nat
        // (and more through Int), all collected as one overloaded name.
        let (_, ctx, _) = resolve("map f: Nat;");
        let signature = ctx.system_signature.as_ref().unwrap();
        assert!(signature.mappings["max"].len() >= 3);
    }

    #[test]
    fn test_template_instantiation_carries_user_sorts() {
        // The list template is instantiated with the user sort `D`, so the
        // cons operator `|>` resolves to `D # List(D) -> List(D)`.
        let (spec, mut ctx, _) = resolve("sort D = struct s; map f: List(D);");
        let def = DefId::new(*spec.sorts().index("D").unwrap());
        let d = ctx.sorts.def(def);
        let d_list = ctx.sorts.generic(ComplexSort::List, d);
        let expected = ctx.sorts.function(vec![d, d_list], d_list);

        let signature = ctx.system_signature.as_ref().unwrap();
        assert!(signature.constructors["|>"].contains(&expected));
    }

    #[test]
    fn test_system_internal_sort_gets_fresh_def() {
        // `@NatPair` exists only in the system specification; it gets a nominal
        // id past the user declarations, and its name is kept for display.
        let (spec, ctx, names) = resolve("sort D; map f: D;");
        let signature = ctx.system_signature.as_ref().unwrap();

        let pair_constructor = signature.constructors["@cPair"][0];
        let ResolvedSort::Function { domain: _, range } = ctx.sorts.get(pair_constructor) else {
            panic!("expected a function sort");
        };
        let ResolvedSort::Def(def) = ctx.sorts.get(*range) else {
            panic!("expected a nominal sort");
        };
        assert!(**def >= spec.data_specification().sort_declarations.len());
        assert_eq!(names.name(*def), Some("@NatPair"));
    }

    #[test]
    fn test_unknown_reference_is_a_clean_error() {
        // A system specification referencing an undeclared sort must error
        // rather than panic; parse one directly to simulate a template mistake.
        let spec = DataSpecification::from_untyped(UntypedDataSpecification::parse("map f: Bool;").unwrap()).unwrap();
        let broken = UntypedDataSpecification::parse("map f: Unknown;").unwrap();

        let mut ctx = TypeckContext::new();
        match resolve_system_signature(&mut ctx, spec.data_specification(), &broken) {
            Err(WellTypedError::Custom(err)) => assert!(err.to_string().contains("Unknown")),
            other => panic!("expected a custom error, got {other:?}"),
        }
    }
}
