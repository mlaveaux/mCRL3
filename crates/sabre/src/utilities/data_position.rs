use std::collections::VecDeque;

use mcrl3_data::DataExpression;
use mcrl3_data::DataExpressionRef;

use super::ExplicitPosition;

/// A specialisation of the [PositionIndexed] trait for [DataExpression]. This is used to keep the indexing consistent.
pub trait DataPositionIndexed<'b> {
    type Target<'a>
    where
        Self: 'a,
        Self: 'b;

    /// Returns the Target at the given position.
    fn get_data_position(&'b self, position: &ExplicitPosition) -> Self::Target<'b>;
}

impl<'b> DataPositionIndexed<'b> for DataExpression {
    type Target<'a>
        = DataExpressionRef<'a>
    where
        Self: 'a;

    fn get_data_position(&'b self, position: &ExplicitPosition) -> Self::Target<'b> {
        let mut result = self.copy();

        for index in &position.indices {
            result = result.data_arg(*index).into(); // Note that positions are 1 indexed.
        }

        result
    }
}

impl<'b> DataPositionIndexed<'b> for DataExpressionRef<'b> {
    type Target<'a>
        = DataExpressionRef<'a>
    where
        Self: 'a;

    fn get_data_position(&'b self, position: &ExplicitPosition) -> Self::Target<'b> {
        let mut result = self.copy();

        for index in &position.indices {
            result = result.data_arg(*index).into(); // Note that positions are 1 indexed.
        }

        result
    }
}

/// An iterator over all (term, position) pairs of the given ATerm.
pub struct DataPositionIterator<'a> {
    queue: VecDeque<(DataExpressionRef<'a>, ExplicitPosition)>,
}

impl<'a> DataPositionIterator<'a> {
    pub fn new(t: DataExpressionRef<'a>) -> Self {
        Self {
            queue: VecDeque::from([(t, ExplicitPosition::empty_pos())]),
        }
    }
}

impl<'a> Iterator for DataPositionIterator<'a> {
    type Item = (DataExpressionRef<'a>, ExplicitPosition);

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            None
        } else {
            // Get a subterm to inspect
            let (term, pos) = self.queue.pop_front().unwrap();

            // Put subterms in the queue
            for (i, argument) in term.data_arguments().enumerate() {
                let mut new_position = pos.clone();
                new_position.indices.push(i + 1);
                self.queue.push_back((argument.into(), new_position));
            }

            Some((term.into(), pos))
        }
    }
}

#[cfg(test)]
mod tests {
    use ahash::AHashSet;
    use mcrl3_aterm::ATerm;
    use mcrl3_data::to_untyped_data_expression;

    use super::*;

    #[test]
    fn test_get_data_position() {
        let t = to_untyped_data_expression(&ATerm::from_string("f(g(a),b)").unwrap(), &AHashSet::new());
        let expected = to_untyped_data_expression(&ATerm::from_string("a").unwrap(),  &AHashSet::new());

        assert_eq!(t.get_data_position(&ExplicitPosition::new(&[1, 1])), expected.copy());
    }

    #[test]
    fn test_position_iterator() {
        let t = to_untyped_data_expression(&ATerm::from_string("f(g(a),b)").unwrap(), &AHashSet::new());

        for (term, pos) in DataPositionIterator::new(t.copy()) {
            assert_eq!(
                t.get_data_position(&pos),
                term,
                "The resulting (subterm, position) pair doesn't match the get_data_position implementation"
            );
        }

        assert_eq!(
            DataPositionIterator::new(t.copy()).count(),
            4,
            "The number of subterms doesn't match the expected value"
        );
    }
}