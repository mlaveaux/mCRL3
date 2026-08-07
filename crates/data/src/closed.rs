use ahash::AHashSet;
use merc_aterm::Term;

use crate::is_data_application;
use crate::is_data_function_symbol;
use crate::is_data_machine_number;
use crate::is_data_variable;

/// Returns true iff `term` contains no data variables, i.e. it is a ground term.
///
/// A closed term normalises to the same result under every substitution, which is what makes
/// it sound to cache its normal form across calls that pass different substitutions.
///
/// Panics for binders and where clauses, which have no flat argument list.
pub fn is_closed<'a, 'b, T: Term<'a, 'b>>(term: &'b T) -> bool {
    // Terms are maximally shared, so the same subterm is typically reachable along many paths.
    // Remembering the ones already seen keeps this linear in the size of the term graph rather
    // than the size of the tree it unfolds to. Keying on the term address is only valid because
    // no terms are created here, so no garbage collection can run during the traversal.
    let mut visited = AHashSet::new();
    let mut stack = vec![term.copy()];

    while let Some(t) = stack.pop() {
        if !visited.insert(t.index()) {
            continue;
        }

        if is_data_variable(&t) {
            return false;
        } else if is_data_function_symbol(&t) || is_data_machine_number(&t) {
            continue;
        } else if is_data_application(&t) {
            // The arguments of a data application include its head function symbol, which is
            // closed by definition, so it needs no special treatment here.
            stack.extend(t.arguments());
        } else {
            panic!("is_closed is not defined for binders and where clauses: {t}");
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use ahash::AHashSet;

    use crate::DataExpression;
    use crate::is_closed;

    #[test]
    fn test_is_closed_ground_term() {
        let term = DataExpression::from_string("s(s(a), b)").unwrap();
        assert!(is_closed(&term));
    }

    #[test]
    fn test_is_closed_with_variable() {
        let variables = AHashSet::from_iter(["x".to_string()]);
        let term = DataExpression::from_string_untyped("s(s(x), b)", &variables).unwrap();
        assert!(!is_closed(&term));
    }

    #[test]
    fn test_is_closed_bare_variable() {
        let variables = AHashSet::from_iter(["x".to_string()]);
        let term = DataExpression::from_string_untyped("x", &variables).unwrap();
        assert!(!is_closed(&term));
    }
}
