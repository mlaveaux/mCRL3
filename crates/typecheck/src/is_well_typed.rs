use std::ops::ControlFlow;

use thiserror::Error;

use merc_syntax::SortDescend;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::try_visit_sort_expr_with;
use merc_syntax::visit_sort_expr;
use merc_utilities::MercError;

use crate::InferenceError;
use crate::nonempty_sorts;
use crate::target_sort;

/// Checks if a signature is well-typed, i.e. it satisfies the conditions of 15.1.7.
///
/// Runs on the normalized specification as a syntactic safety net behind
/// `query_signature` (which checks before alias expansion, for error messages
/// in the user's terms); equation-variable sorts and the sort-emptiness check
/// are covered only here.
pub(crate) fn is_well_typed(spec: &UntypedDataSpecification) -> Result<(), WellTypedError> {
    are_constructors_and_mappings_disjoint(spec)?;

    // A product sort only has meaning as the domain of a function sort, but the
    // grammar cannot enforce that (`#` and `->` share the sort-expression
    // syntax), so `map f: Pos -> (Pos # Pos);` parses and must be rejected
    // here. Sorts on binders inside equation bodies are checked later, as part
    // of data-expression type checking.
    for sort in spec.sort_declarations.iter().filter_map(|decl| decl.expr.as_ref()) {
        check_products_within_domains(sort)?;
    }
    for decl in spec.constructor_declarations.iter().chain(&spec.map_declarations) {
        check_products_within_domains(&decl.sort)?;
    }
    for equation in &spec.equation_declarations {
        for var in &equation.variables {
            check_products_within_domains(&var.sort)?;
        }
    }

    // Check that there are no constructors defined for the basic sorts.
    for constructor in &spec.constructor_declarations {
        let sort = target_sort(&constructor.sort);

        // There are not more constructors for basic sorts.
        if is_basic_sort(sort) {
            return Err(WellTypedError::ConstructorForBasicSort {
                constructor: constructor.identifier.clone(),
                sort: sort.to_string(),
            });
        }

        // Function sorts are not constructor sorts. Both forms are matched
        // because flattening rewrites `Function` into `FlattenedFunction`, so
        // after the pipeline's early passes only the latter occurs here.
        if matches!(
            sort,
            SortExpression::Function { .. } | SortExpression::FlattenedFunction { .. }
        ) {
            return Err(WellTypedError::ConstructorForFunctionSort {
                constructor: constructor.identifier.clone(),
                sort: sort.to_string(),
            });
        }
    }

    // Check that all sorts are syntactically non-empty. `nonempty_sorts` already
    // assumes sorts without constructors (abstract sorts and aliases) to be
    // non-empty, so only genuine constructor sorts are reported here, as in
    // mCRL2's check_for_empty_constructor_domains.
    let nonempty = nonempty_sorts(spec);
    for sort in &spec.sort_declarations {
        let id = sort.id.expect("The sorts must be resolved");
        if !nonempty.contains(&id) {
            return Err(WellTypedError::EmptySort {
                sort: sort.identifier.clone(),
            });
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum WellTypedError {
    #[error("Constructor '{}' and mapping '{}' have the same identifier", constructor, map)]
    ConstructorAndMappingConflict { constructor: String, map: String },

    #[error(
        "Constructors cannot be defined for basic sorts, but constructor '{}' is defined for sort '{}'",
        constructor,
        sort
    )]
    ConstructorForBasicSort { constructor: String, sort: String },

    #[error(
        "Constructors cannot be defined for function sorts, but constructor '{}' is defined for sort '{}'",
        constructor,
        sort
    )]
    ConstructorForFunctionSort { constructor: String, sort: String },

    #[error("Sort '{}' is syntactically empty", sort)]
    EmptySort { sort: String },

    #[error("A product sort '{}' may only appear as the domain of a function sort", sort)]
    ProductSortOutsideFunctionDomain { sort: String },

    #[error("Alias cycle detected: {:?}", sorts)]
    AliasCycle { sorts: Vec<String> },

    #[error("Sort '{sort}' is recursively defined via a function sort, or a set or a bag type container")]
    RecursiveAliasThroughFunctionSort { sort: String },

    #[error("Error: '{0}'")]
    Custom(MercError),

    /// A Phase-3 sort inference error in a user equation.
    #[error(transparent)]
    Inference(#[from] InferenceError),

    // These are name resolution errors, but we include them here to avoid having to define a separate error type for name resolution.
    #[error("Duplicate sort declaration: '{}'", sort)]
    DuplicateSortDeclaration { sort: String },

    #[error("Undefined sort: '{}'", sort)]
    UndefinedSort { sort: String },
}

/// Checks that no *symbol* — an identifier together with its sort — is declared
/// as both a constructor and a mapping.
///
/// A name may still be overloaded across a constructor and a mapping when their
/// sorts differ (for example a `struct` constructor `area: … -> Area` alongside
/// a mapping `area: Instruction -> Area`); disambiguating such overloads is the
/// job of overload resolution.
fn are_constructors_and_mappings_disjoint(spec: &UntypedDataSpecification) -> Result<(), WellTypedError> {
    for constructor in &spec.constructor_declarations {
        if let Some(map) = spec
            .map_declarations
            .iter()
            .find(|map| map.identifier == constructor.identifier && map.sort == constructor.sort)
        {
            return Err(WellTypedError::ConstructorAndMappingConflict {
                constructor: constructor.identifier.clone(),
                map: map.identifier.clone(),
            });
        }
    }

    Ok(())
}

/// The set of basic sorts `BS` are exactly the sorts Bool, Pos, Int, Nat, and Real. Definition 15.1.2.
fn is_basic_sort(sort: &SortExpression) -> bool {
    matches!(sort, SortExpression::Simple(_))
}

/// Checks that every product sort occurs as (part of the spine of) a function
/// sort's domain, the only position where `A # B` has meaning.
///
/// The domain and range of a `Function` need different treatment, which the
/// visitor context cannot express (all children receive the same context), so
/// that case is handled manually and pruned.
pub(crate) fn check_products_within_domains(sort: &SortExpression) -> Result<(), WellTypedError> {
    try_visit_sort_expr_with::<WellTypedError, (), (), _>(sort, (), |expr, ()| match expr {
        SortExpression::Product { .. } => {
            Err(WellTypedError::ProductSortOutsideFunctionDomain { sort: expr.to_string() })
        }
        SortExpression::Function { domain, range } => {
            check_product_spine(domain)?;
            check_products_within_domains(range)?;
            Ok(ControlFlow::Continue(SortDescend::Prune))
        }
        _ => Ok(ControlFlow::Continue(SortDescend::Descend(()))),
    })
    .map(|_| ())
}

/// Walks the `Product` spine of a function domain, where products are the
/// domain separator, and checks the leaf sorts.
fn check_product_spine(sort: &SortExpression) -> Result<(), WellTypedError> {
    match sort {
        SortExpression::Product { lhs, rhs } => {
            check_product_spine(lhs)?;
            check_product_spine(rhs)
        }
        _ => check_products_within_domains(sort),
    }
}

/// Returns whether a binder sort inside an equation body can be resolved by
/// the pipeline today. An anonymous `struct` is not hoisted out of expressions
/// by `hoist_anonymous_structs`, and a bare product sort is not a sort (mCRL2
/// rejects it), so a construct binding either is deferred rather than resolved
/// (see G7/G8 in docs/typecheck.md).
pub(crate) fn is_supported_binder_sort(sort: &SortExpression) -> bool {
    let contains_struct = visit_sort_expr(sort, |expr| match expr {
        SortExpression::Struct { .. } => ControlFlow::Break(()),
        _ => ControlFlow::Continue(()),
    })
    .is_some();

    !contains_struct && check_products_within_domains(sort).is_ok()
}

#[cfg(test)]
mod tests {
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::WellTypedError;

    #[test]
    fn test_well_typed_spec() {
        let spec = UntypedDataSpecification::parse(
            "
            sort D;
            cons f: D -> Nat;
        ",
        )
        .unwrap();

        match DataSpecification::from_untyped(spec) {
            Err(WellTypedError::ConstructorForBasicSort { constructor, sort })
                if constructor == "f" && sort == "Nat" => {}
            Err(other) => panic!("Unexpected error {:?}", other),
            _ => panic!("Expected from_untyped to fail"),
        }
    }

    #[test]
    fn test_abstract_sort_is_allowed() {
        let spec = UntypedDataSpecification::parse(
            "
            sort D;
            map f: D -> D;
        ",
        )
        .unwrap();

        DataSpecification::from_untyped(spec).expect("a sort without constructors is assumed non-empty");
    }

    #[test]
    fn test_product_sort_outside_function_domain_is_rejected() {
        for text in [
            "map f: Pos -> (Pos # Pos);",
            "map f: List(Pos # Pos);",
            "sort D = Bool # Bool;",
            "map f: Nat; var x: Nat # Nat; eqn f = 0;",
            "map f: ((Pos # Pos) -> Bool) -> (Nat # Nat);",
        ] {
            match DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap()) {
                Err(WellTypedError::ProductSortOutsideFunctionDomain { .. }) => {}
                Err(other) => panic!("unexpected error {other:?} for {text}"),
                Ok(_) => panic!("expected {text} to be rejected"),
            }
        }
    }

    #[test]
    fn test_product_sort_in_function_domain_is_allowed() {
        // Parenthesized products inside a domain are still domain separators,
        // including in a nested higher-order function sort.
        for text in [
            "map f: (Pos # Pos) # Pos -> Bool;",
            "map f: ((Pos # Pos) -> Bool) -> Bool;",
        ] {
            DataSpecification::from_untyped(UntypedDataSpecification::parse(text).unwrap())
                .unwrap_or_else(|err| panic!("expected {text} to typecheck, got {err}"));
        }
    }
}
