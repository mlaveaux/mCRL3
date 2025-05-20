use mcrl3_aterm::Protected;
use mcrl3_aterm::Term;
use mcrl3_aterm::ThreadTermPool;
use mcrl3_data::DataExpression;
use mcrl3_data::DataExpressionRef;

pub type DataSubstitutionBuilder = Protected<Vec<DataExpressionRef<'static>>>;

/// This function substitutes the term 't' at the position 'p' with 'new_subterm', see [substitute].
pub fn data_substitute(tp: &ThreadTermPool, t: &DataExpressionRef<'_>, new_subterm: DataExpression, p: &[usize]) -> DataExpression {
    let mut args = Protected::new(vec![]);
    substitute_rec(tp, t, new_subterm, p, &mut args, 0)
}

/// This is the same as [data_substitute], but it uses a [SubstitutionBuilder] to store the arguments.
pub fn data_substitute_with(
    builder: &mut DataSubstitutionBuilder,
    tp: &ThreadTermPool,
    t: &DataExpressionRef<'_>,
    new_subterm: DataExpression,
    p: &[usize],
) -> DataExpression {
    substitute_rec(tp, t, new_subterm, p, builder, 0)
}

/// The recursive implementation for subsitute
///
/// 'depth'         -   Used to keep track of the depth in 't'. Function should be called with
///                     'depth' = 0.
fn substitute_rec(
    tp: &ThreadTermPool,
    t: &DataExpressionRef<'_>,
    new_subterm: DataExpression,
    p: &[usize],
    args: &mut DataSubstitutionBuilder,
    depth: usize,
) -> DataExpression {
    if p.len() == depth {
        // in this case we have arrived at the place where 'new_subterm' needs to be injected
        new_subterm
    } else {
        // else recurse deeper into 't'
        let new_child_index = p[depth] - 1;
        let new_child = substitute_rec(tp, &t.data_arg(new_child_index), new_subterm, p, args, depth + 1);

        let mut write_args = args.write();
        for (index, arg) in t.arguments().enumerate() {
            if index == new_child_index {
                let t = write_args.protect(&new_child);
                write_args.push(t.into());
            } else {
                let t = write_args.protect(&arg);
                write_args.push(t.into());
            }
        }

        let result = tp.create_term(&t.get_head_symbol(), &write_args);
        drop(write_args);

        // TODO: When write is dropped we check whether all terms where inserted, but this clear violates that assumption.
        args.write().clear();
        result.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ahash::AHashSet;
    use mcrl3_aterm::ATerm;
    use mcrl3_aterm::THREAD_TERM_POOL;
    use mcrl3_data::to_untyped_data_expression;

    use crate::utilities::DataPosition;
    use crate::utilities::DataPositionIndexed;

    #[test]
    fn test_data_substitute() {
        let t = to_untyped_data_expression(&ATerm::from_string("s(s(a))").unwrap(), &AHashSet::new());
        let t0 = to_untyped_data_expression(&ATerm::from_string("0").unwrap(), &AHashSet::new());

        // substitute the a for 0 in the term s(s(a))
        let result = THREAD_TERM_POOL.with_borrow(|tp| data_substitute(tp, &t.copy(), t0.clone(), &vec![1, 1]));

        // Check that indeed the new term as a 0 at position 1.1.
        assert_eq!(t0, result.get_data_position(&DataPosition::new(&vec![1, 1])).protect());
    }
}
