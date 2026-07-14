use thiserror::Error;

use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_utilities::MercError;

use crate::is_nonempty_sort;
use crate::target_sort;

/// Checks if a signature is well-typed, i.e. it satisfies the conditions of 15.1.7.
pub fn is_well_typed(spec: &UntypedDataSpecification) -> Result<(), WellTypedError> {
    are_constructors_and_mappings_disjoint(spec)?;

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

        // Function sorts are not constructor sorts
        if matches!(sort, SortExpression::Function { domain: _, range: _ }) {
            return Err(WellTypedError::ConstructorForFunctionSort {
                constructor: constructor.identifier.clone(),
                sort: sort.to_string(),
            });
        }
    }

    // Check that all sorts are syntactically non-empty.
    for sort in &spec.sort_declarations {
        if !is_nonempty_sort(&sort.identifier, spec) {
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

    #[error("Alias cycle detected: {:?}", sorts)]
    AliasCycle { sorts: Vec<String> },

    #[error("Error: '{}'", 0)]
    Custom(MercError),

    // These are name resolution errors, but we include them here to avoid having to define a separate error type for name resolution.
    #[error("Duplicate sort declaration: '{}'", sort)]
    DuplicateSortDeclaration { sort: String },

    #[error("Undefined sort: '{}'", sort)]
    UndefinedSort { sort: String },
}

/// Checks that the constructors and mappings are disjoint, i.e. that no identifier is both a constructor and a mapping.
fn are_constructors_and_mappings_disjoint(spec: &UntypedDataSpecification) -> Result<(), WellTypedError> {
    for constructor in &spec.constructor_declarations {
        if let Some(map) = spec
            .map_declarations
            .iter()
            .find(|map| map.identifier == constructor.identifier)
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
}
