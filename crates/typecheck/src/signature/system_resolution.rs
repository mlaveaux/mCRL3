use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use merc_syntax::DefId;
use merc_syntax::SortExpression;
use merc_syntax::SortExpressionKind;
use merc_syntax::UntypedDataSpecification;

use crate::BUILTIN_SCHEME_TEMPLATE;
use crate::CONTAINER_TEMPLATES;
use crate::ResolvedSortId;
use crate::Signature;
use crate::TypeCheckContext;
use crate::WellTypedError;
use crate::push_overload;
use crate::query_sort_of_def;

/// Resolves the constructor and mapping declarations of the *basic-sort* part
/// of the system-defined specification onto the interned sort lattice.
///
/// `system` must be the *basic-sort* specification ([basic_sort_data_specification]),
/// not the full system-defined specification `build_system_defined_specification`
/// produces: the container operations are looked up polymorphically instead
/// (`POLYMORPHIC_SIGNATURE`), because resolving their per-sort instantiations
/// here as well would misreport ambiguity (a name would have both a concrete
/// and a polymorphic candidate for the same sort).
///
/// Unlike `build_signature` this runs no well-typedness checks here — not
/// because the system specification is trusted, but because `build_signature`'s
/// checks would misfire on it: it legitimately declares things a user cannot,
/// such as constructors for the basic sorts (`@c0: Nat`). The system
/// specification's own well-formedness is instead verified separately and
/// extensively by `check_system_specification` (debug builds).
pub(crate) fn resolve_system_signature(
    ctx: &mut TypeCheckContext,
    user_spec: &UntypedDataSpecification,
    system: &UntypedDataSpecification,
) -> Result<(), WellTypedError> {
    // The system specification re-declares the basic sorts (`sort Bool;`),
    // which already resolve as primitives; only the remaining declarations
    // denote system-internal nominal sorts.
    //
    // Each system-internal sort gets a fresh DefId that continues the user
    // sorts' numbering: `user_spec.sort_declarations.len() + decl_index`. This
    // is the layout `TypeCheckContext::sort_name` relies on to recover such a
    // DefId's name from the system specification's declarations on demand.
    let mut sort_ids: HashMap<String, ResolvedSortId> = HashMap::new();
    for (decl_index, decl) in system.sort_declarations.iter().enumerate() {
        if is_basic_sort_name(&decl.identifier) || sort_ids.contains_key(&decl.identifier) {
            continue;
        }
        debug_assert!(
            decl.expr.is_none(),
            "system-defined sorts are nominal, but '{}' has a body",
            decl.identifier
        );

        let def = DefId::new(user_spec.sort_declarations.len() + decl_index);
        sort_ids.insert(decl.identifier.clone(), ctx.sorts.def(def));
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
    Ok(())
}

fn is_basic_sort_name(name: &str) -> bool {
    matches!(name, "Bool" | "Pos" | "Nat" | "Int" | "Real")
}

/// The polymorphic signature of the built-in operators that exist for *every*
/// sort: the container and function-update operations, plus the comparison
/// operators and `if`. For each name, the overload sorts as written in the
/// templates, with the sort variables (`S`, `T`) still unresolved `Reference`
/// nodes.
///
/// Inference looks a name up here and instantiates the variables fresh per
/// occurrence (`template_instance`), mirroring mCRL2's built-in polymorphic
/// symbol table. This one mechanism covers `|>` and `==` alike — the comparison
/// operators and `if` are just further schemes, carried by
/// [BUILTIN_SCHEME_TEMPLATE]. Their per-sort instantiations are deliberately
/// *not* part of the resolved system signature: listing an operation both ways
/// would misreport ambiguity.
pub(crate) struct PolymorphicSignature {
    pub(crate) ops: HashMap<String, Vec<SortExpression>>,
}

/// The [PolymorphicSignature] of the bundled container templates and the
/// built-in schemes: the constructor and mapping declarations of each, collected
/// once.
pub(crate) static POLYMORPHIC_SIGNATURE: LazyLock<PolymorphicSignature> = LazyLock::new(|| {
    let mut ops: HashMap<String, Vec<SortExpression>> = HashMap::new();
    for template in CONTAINER_TEMPLATES.all() {
        collect_overloads(&mut ops, template);
    }
    // The comparison operators and `if` are polymorphic in exactly the same way
    // as the container operations, so they join the same table rather than a
    // separate, hand-written scheme instantiation.
    collect_overloads(&mut ops, &BUILTIN_SCHEME_TEMPLATE);
    PolymorphicSignature { ops }
});

/// Collects the constructor and mapping declarations of `spec` into `ops`,
/// keyed by name, dropping an overload sort already recorded for that name.
fn collect_overloads(ops: &mut HashMap<String, Vec<SortExpression>>, spec: &UntypedDataSpecification) {
    for (identifier, sort) in spec
        .constructor_declarations
        .iter()
        .map(|decl| (&decl.identifier, &decl.sort))
        .chain(spec.map_declarations.iter().map(|decl| (&decl.identifier, &decl.sort)))
    {
        let overloads = ops.entry(identifier.clone()).or_default();
        if !overloads.contains(sort) {
            overloads.push(sort.clone());
        }
    }
}

/// The system-defined counterpart of `resolve_sort`. It differs in two ways:
/// `Reference` nodes are looked up among the system-internal sorts (the system
/// specification never went through name resolution), and unknown references
/// are a clean error rather than a panic, so a template mistake in a
/// `spec/*.mcrl2` file cannot crash the checker.
fn resolve_system_sort(
    ctx: &mut TypeCheckContext,
    user_spec: &UntypedDataSpecification,
    sort_ids: &HashMap<String, ResolvedSortId>,
    sort: &SortExpression,
) -> Result<ResolvedSortId, WellTypedError> {
    match &sort.node {
        SortExpressionKind::Simple(sort) => Ok(ctx.sorts.primitive(*sort)),
        SortExpressionKind::Complex(op, subsort) => {
            let subsort = resolve_system_sort(ctx, user_spec, sort_ids, subsort)?;
            Ok(ctx.sorts.generic(*op, subsort))
        }
        SortExpressionKind::FlattenedFunction { domain, range } => {
            let domain = domain
                .iter()
                .map(|sort| resolve_system_sort(ctx, user_spec, sort_ids, sort))
                .collect::<Result<_, _>>()?;
            let range = resolve_system_sort(ctx, user_spec, sort_ids, range)?;
            Ok(ctx.sorts.function(domain, range))
        }
        // The system specification is parsed directly and never flattened, so
        // function sorts appear with a `Product` domain spine.
        SortExpressionKind::Function { domain, range } => {
            let mut resolved_domain = Vec::new();
            resolve_system_function_domain(ctx, user_spec, sort_ids, domain, &mut resolved_domain)?;
            let range = resolve_system_sort(ctx, user_spec, sort_ids, range)?;
            Ok(ctx.sorts.function(resolved_domain, range))
        }
        // A sort substituted into an Appendix-B template comes from the
        // normalized user specification, so its `DefId` indexes `user_spec`.
        SortExpressionKind::Resolved(_, id) => Ok(query_sort_of_def(ctx, user_spec, *id)),
        SortExpressionKind::Reference(name) => match sort_ids.get(name) {
            Some(id) => Ok(*id),
            None => Err(WellTypedError::Custom(
                format!("the system-defined specification references the undeclared sort '{name}'").into(),
            )),
        },
        SortExpressionKind::Struct { .. } => unreachable!("the system-defined specification has no structured sorts"),
        SortExpressionKind::Product { .. } => {
            unreachable!("product sorts cannot occur outside a function domain")
        }
    }
}

/// Resolves the leaves of a `Product` domain spine in declaration order.
fn resolve_system_function_domain(
    ctx: &mut TypeCheckContext,
    user_spec: &UntypedDataSpecification,
    sort_ids: &HashMap<String, ResolvedSortId>,
    sort: &SortExpression,
    domain: &mut Vec<ResolvedSortId>,
) -> Result<(), WellTypedError> {
    match &sort.node {
        SortExpressionKind::Product { lhs, rhs } => {
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
    use crate::NumberEncoding;
    use crate::ResolvedSort;
    use crate::TypeCheckContext;
    use crate::WellTypedError;
    use crate::basic_sort_data_specification;
    use crate::resolve_system_signature;

    /// Type checks `text` and resolves the basic-sort system signature in a
    /// fresh context, as `DataSpecification::from_untyped` does.
    fn resolve(text: &str) -> (DataSpecification, TypeCheckContext) {
        let spec = DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()).unwrap();
        let mut ctx = TypeCheckContext::new();
        let basics = basic_sort_data_specification(NumberEncoding::Binary);
        resolve_system_signature(&mut ctx, spec.data_specification(), &basics).unwrap();
        (spec, ctx)
    }

    #[test]
    fn test_boolean_operators_are_resolved() {
        let (_, ctx) = resolve("map f: Bool;");
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
        let (_, ctx) = resolve("map f: Nat;");
        let signature = ctx.system_signature.as_ref().unwrap();
        assert!(signature.mappings["max"].len() >= 3);
    }

    #[test]
    fn test_template_instantiation_carries_user_sorts() {
        // `resolve_system_sort`'s handling of `Resolved` nodes, exercised
        // directly: production only ever feeds `resolve_system_signature` the
        // basic-sort spec (see its doc comment), so this instantiates the
        // full system-defined spec — containers included — in an isolated
        // context to check the substitution logic itself. The list template
        // instantiated with the user sort `D` should resolve `|>` to
        // `D # List(D) -> List(D)`.
        let spec = DataSpecification::from_untyped(
            UntypedDataSpecification::parse("sort D = struct s; map f: List(D);").unwrap(),
        )
        .unwrap();
        let mut ctx = TypeCheckContext::new();
        resolve_system_signature(&mut ctx, spec.data_specification(), spec.system_defined_specification()).unwrap();

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
        // DefId past the user declarations, and its name is recovered by
        // `sort_name`, which derives it from the system specification's
        // declarations on demand rather than from a stored table.
        let (spec, ctx) = resolve("sort D; map f: D;");
        let signature = ctx.system_signature.as_ref().unwrap();

        let pair_constructor = signature.constructors["@cPair"][0];
        let ResolvedSort::Function { domain: _, range } = ctx.sorts.get(pair_constructor) else {
            panic!("expected a function sort");
        };
        let ResolvedSort::Def(def) = ctx.sorts.get(*range) else {
            panic!("expected a nominal sort");
        };
        let user_len = spec.data_specification().sort_declarations.len();
        assert!(**def >= user_len);
        assert_eq!(
            ctx.sort_name(spec.data_specification(), spec.system_defined_specification(), *def),
            Some("@NatPair")
        );
    }

    #[test]
    fn test_unknown_reference_is_a_clean_error() {
        // A system specification referencing an undeclared sort must error
        // rather than panic; parse one directly to simulate a template mistake.
        let spec = DataSpecification::from_untyped(UntypedDataSpecification::parse("map f: Bool;").unwrap()).unwrap();
        let broken = UntypedDataSpecification::parse("map f: Unknown;").unwrap();

        let mut ctx = TypeCheckContext::new();
        match resolve_system_signature(&mut ctx, spec.data_specification(), &broken) {
            Err(WellTypedError::Custom(err)) => assert!(err.to_string().contains("Unknown")),
            other => panic!("expected a custom error, got {other:?}"),
        }
    }
}
