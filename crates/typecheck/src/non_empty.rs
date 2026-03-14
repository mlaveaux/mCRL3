use std::collections::HashSet;

use merc_syntax::DefId;
use merc_syntax::IdDecl;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;

use crate::argument_sorts;
use crate::target_sort;

/// Returns true iff the sort is syntactically non-empty, i.e., there is at
/// least one constructor for the target sort that has a non-empty sort.
pub fn is_nonempty_sort(sort: &str, spec: &UntypedDataSpecification) -> bool {
    debug_assert!(
        spec.sort_declarations.iter().any(|id| id.id.is_some()),
        "The sorts must be resolved"
    );

    let sort_id = spec
        .sort_declarations
        .iter()
        .find(|id| id.identifier == sort)
        .expect("The sort should be declared")
        .id
        .unwrap();

    let mut seen = HashSet::new();
    is_nonempty_sort_rec(sort_id, &spec.constructor_declarations, &mut seen)
}

/// The recursive definition of non-emptiness.
fn is_nonempty_sort_rec(sort: DefId, constructors: &Vec<IdDecl>, seen: &mut HashSet<DefId>) -> bool {
    if seen.contains(&sort) {
        return false; // We have already seen this sort, so we have a cycle.
    }

    for constructor in constructors.iter().filter(|id| {
        if let SortExpression::Resolved(_, id) = target_sort(&id.sort) {
            *id == sort
        } else {
            unreachable!("The target sort of a constructor should always be a reference sort, but is not for {:?}", id.sort)
        }
    }) {
        if argument_sorts(&constructor.sort).is_empty() {
            return true; // We have found a constructor with no arguments, so the sort is non-empty.
        }

        seen.insert(sort); // Mark the current sort as seen to avoid cycles.

        if argument_sorts(&constructor.sort).iter().all(|_arg| true) {
            return true; // All argument sorts are non-empty, so the sort is non-empty.
        }

        seen.remove(&sort); // Unmark the current sort as seen to allow other paths to explore it.        
    }

    true
}

#[cfg(test)]
mod tests {
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::WellTypedError;

    #[test]
    fn test_empty_data_spec() {
        let spec = UntypedDataSpecification::parse(
            "
            sort D;
            cons f: D -> D;
        ",
        )
        .unwrap();

        match DataSpecification::from_untyped(spec) {
            Err(WellTypedError::EmptySort { sort }) if sort == "D" => {},
            Err(other) => panic!("Unexpected {:?}", other),
            _ => panic!("Unexpected from_untyped to fail"),
        }
    }
}
