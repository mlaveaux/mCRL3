use std::collections::HashMap;
use std::rc::Rc;

use merc_syntax::UntypedDataSpecification;

use crate::ResolvedSort;
use crate::ResolvedSortId;
use crate::TypeckContext;
use crate::WellTypedError;
use crate::check_products_within_domains;
use crate::resolve_sort;
use crate::target_sort;

/// The (S, C, M) signature of a specification (Definition 15.1.5): the resolved
/// overload set of every constructor and mapping name, the lookup table for
/// Phase-3 overload resolution.
///
/// A symbol is a name together with its sort, so a name maps to one
/// [ResolvedSortId] per overload; duplicate declarations of the same symbol
/// collapse into one entry.
pub(crate) struct Signature {
    pub(crate) constructors: HashMap<String, Vec<ResolvedSortId>>,
    pub(crate) mappings: HashMap<String, Vec<ResolvedSortId>>,
}

/// Returns the signature of `spec`, running the signature-layer well-typedness
/// checks of 15.1.7 the first time it is called. Memoized on
/// [TypeckContext::signature].
///
/// Runs *before* `normalize_sorts`, so the errors refer to sorts as the user
/// wrote them (`D` rather than its alias expansion `Nat`); the semantic facts
/// are obtained through the interned sort lattice instead, which expands alias
/// indirection lazily via `query_sort_of_def`. Requires names to be resolved
/// and structured sorts to be desugared.
pub(crate) fn query_signature<'a>(
    ctx: &'a mut TypeckContext,
    spec: &UntypedDataSpecification,
) -> Result<&'a Signature, WellTypedError> {
    if ctx.signature.is_none() {
        let signature = compute_signature(ctx, spec)?;
        ctx.signature = Some(Rc::new(signature));
    }

    Ok(ctx.signature.as_deref().expect("the signature was just computed"))
}

fn compute_signature(ctx: &mut TypeckContext, spec: &UntypedDataSpecification) -> Result<Signature, WellTypedError> {
    // resolve_sort has no meaning for (and panics on) a product sort outside a
    // function domain, so every sort this query resolves is checked first: the
    // constructor and mapping sorts, and the alias bodies reachable from them
    // through query_sort_of_def.
    for sort in spec.sort_declarations.iter().filter_map(|decl| decl.expr.as_ref()) {
        check_products_within_domains(sort)?;
    }
    for sort in spec
        .constructor_declarations
        .iter()
        .map(|decl| &decl.sort)
        .chain(spec.map_declarations.iter().map(|decl| &decl.sort))
    {
        check_products_within_domains(sort)?;
    }

    let mut signature = Signature {
        constructors: HashMap::new(),
        mappings: HashMap::new(),
    };

    for decl in &spec.constructor_declarations {
        let id = resolve_sort(ctx, spec, &decl.sort);

        // The constructor targets the range of its (function) sort. The check
        // is semantic — an alias of `Nat` is rejected like `Nat` itself — but
        // the error reports the target as written. When the whole constructor
        // sort is an alias of a function sort, the written sort itself is the
        // closest the user came to writing the target.
        let target = match ctx.sorts.get(id) {
            ResolvedSort::Function { domain: _, range } => *range,
            _ => id,
        };
        match ctx.sorts.get(target) {
            ResolvedSort::Primitive(_) => {
                return Err(WellTypedError::ConstructorForBasicSort {
                    constructor: decl.identifier.clone(),
                    sort: target_sort(&decl.sort).to_string(),
                });
            }
            ResolvedSort::Function { .. } => {
                return Err(WellTypedError::ConstructorForFunctionSort {
                    constructor: decl.identifier.clone(),
                    sort: target_sort(&decl.sort).to_string(),
                });
            }
            _ => {}
        }

        push_overload(signature.constructors.entry(decl.identifier.clone()).or_default(), id);
    }

    for decl in &spec.map_declarations {
        let id = resolve_sort(ctx, spec, &decl.sort);

        // The constructors and mappings must be disjoint *as symbols*: the same
        // name under both `cons` and `map` conflicts exactly when the resolved
        // sorts coincide, which also catches sorts that only differ through an
        // alias. Overloading the name with a different sort remains allowed.
        if signature
            .constructors
            .get(&decl.identifier)
            .is_some_and(|overloads| overloads.contains(&id))
        {
            return Err(WellTypedError::ConstructorAndMappingConflict {
                constructor: decl.identifier.clone(),
                map: decl.identifier.clone(),
            });
        }

        push_overload(signature.mappings.entry(decl.identifier.clone()).or_default(), id);
    }

    Ok(signature)
}

/// Appends `id` unless it is already an overload, so duplicate declarations of
/// the same symbol collapse into one entry.
pub(crate) fn push_overload(overloads: &mut Vec<ResolvedSortId>, id: ResolvedSortId) {
    if !overloads.contains(&id) {
        overloads.push(id);
    }
}

#[cfg(test)]
mod tests {
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::Signature;
    use crate::TypeckContext;
    use crate::WellTypedError;
    use crate::query_signature;

    fn typecheck(text: &str) -> DataSpecification {
        DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()).unwrap()
    }

    fn typecheck_err(text: &str) -> WellTypedError {
        match DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()) {
            Err(err) => err,
            Ok(_) => panic!("expected the specification to be rejected"),
        }
    }

    #[test]
    fn test_signature_collects_overloads() {
        let spec = typecheck("map f: Nat; f: Bool -> Bool;");
        assert_eq!(spec.signature().mappings["f"].len(), 2);
    }

    #[test]
    fn test_signature_matches_declaration_sorts() {
        // The signature is resolved before normalization and the declaration
        // sorts after; both must agree on the interned ids, also through an
        // alias chain onto a struct representative.
        let spec = typecheck("sort D; A = B; B = struct s; cons c: A; map g: D -> Bool;");
        let signature = spec.signature();
        assert_eq!(
            signature.constructors["c"],
            vec![spec.declaration_sorts().constructors[0]]
        );
        assert_eq!(signature.mappings["g"], vec![spec.declaration_sorts().mappings[0]]);
    }

    #[test]
    fn test_duplicate_declaration_is_one_symbol() {
        let spec = typecheck("map f: Nat; f: Nat;");
        assert_eq!(spec.signature().mappings["f"].len(), 1);
    }

    #[test]
    fn test_constructor_and_mapping_conflict() {
        match typecheck_err("sort D; cons c: D; map c: D;") {
            WellTypedError::ConstructorAndMappingConflict { constructor, map } => {
                assert_eq!(constructor, "c");
                assert_eq!(map, "c");
            }
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn test_constructor_and_mapping_overload_is_allowed() {
        // The name `c` is shared, but the sorts differ, so these are distinct
        // symbols to be disambiguated by overload resolution.
        let spec = typecheck("sort D; cons c: Bool -> D; d: D; map c: Nat -> D;");
        let signature = spec.signature();
        assert_eq!(signature.constructors["c"].len(), 1);
        assert_eq!(signature.mappings["c"].len(), 1);
        assert_ne!(signature.constructors["c"], signature.mappings["c"]);
    }

    #[test]
    fn test_conflict_through_alias_is_detected() {
        // `A` and `(Nat -> Bool)` denote the same sort, so the constructor and
        // mapping `c` are the same symbol even though their written sorts
        // differ; the conflict is decided on the interned sort ids.
        match typecheck_err("sort A = Nat -> Bool; sort D; cons c: A -> D; d: D; map c: (Nat -> Bool) -> D;") {
            WellTypedError::ConstructorAndMappingConflict { constructor, .. } => assert_eq!(constructor, "c"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn test_constructor_for_alias_of_basic_sort_reports_written_name() {
        // The target of `c` denotes the built-in `Nat` and is rejected, but the
        // error refers to the sort as the user wrote it, not to its expansion.
        match typecheck_err("sort D = Nat; cons c: D;") {
            WellTypedError::ConstructorForBasicSort { constructor, sort } => {
                assert_eq!(constructor, "c");
                assert_eq!(sort, "D");
            }
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn test_constructor_for_function_sort_is_rejected() {
        // The higher-order target is written directly, without alias indirection.
        match typecheck_err("cons c: Bool -> (Nat -> Bool);") {
            WellTypedError::ConstructorForFunctionSort { constructor, sort } => {
                assert_eq!(constructor, "c");
                assert_eq!(sort, "(Nat -> Bool)");
            }
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn test_constructor_for_alias_of_function_sort_reports_written_name() {
        match typecheck_err("sort A = Nat -> Bool; cons c: Bool -> A;") {
            WellTypedError::ConstructorForFunctionSort { constructor, sort } => {
                assert_eq!(constructor, "c");
                assert_eq!(sort, "A");
            }
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn test_constructor_whose_whole_sort_is_a_function_alias() {
        // `c: A` with `A = Nat -> Bool` makes `c` a constructor for the basic
        // sort `Bool`; the error reports the written sort `A`, the closest the
        // user came to writing the target.
        match typecheck_err("sort A = Nat -> Bool; cons c: A;") {
            WellTypedError::ConstructorForBasicSort { constructor, sort } => {
                assert_eq!(constructor, "c");
                assert_eq!(sort, "A");
            }
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn test_constructor_whose_function_alias_targets_a_declared_sort() {
        // As above, but the aliased function sort ranges over the declared sort
        // `D`, so `c` is a valid (unary) constructor for `D`.
        let spec = typecheck("sort D; A = Nat -> D; cons c: A; d: D;");
        assert_eq!(spec.signature().constructors["c"].len(), 1);
    }

    #[test]
    fn test_query_signature_is_memoized() {
        let spec = typecheck("sort D; cons c: D; map f: D -> Bool;");

        let mut ctx = TypeckContext::new();
        let first: *const Signature = query_signature(&mut ctx, spec.data_specification()).unwrap();
        let second: *const Signature = query_signature(&mut ctx, spec.data_specification()).unwrap();
        assert!(std::ptr::eq(first, second), "the second query must be a cache hit");
    }
}
