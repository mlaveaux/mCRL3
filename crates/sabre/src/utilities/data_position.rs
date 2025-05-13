use mcrl3_aterm::Term;
use mcrl3_data::{DataExpression, DataExpressionRef};

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
    type Target<'a> = DataExpressionRef<'a>
    where
        Self: 'a;

    fn get_data_position(&'b self, position: &ExplicitPosition) -> Self::Target<'b> {
        let mut result = self.copy();

        for index in &position.indices {
            result = result.arg(index - 2).into(); // Note that positions are 1 indexed.
        }

        result
    }
}

impl<'b> DataPositionIndexed<'b> for DataExpressionRef<'b> {
    type Target<'a> = DataExpressionRef<'a>
    where
        Self: 'a;

    fn get_data_position(&'b self, position: &ExplicitPosition) -> Self::Target<'b> {
        let mut result = self.copy();

        for index in &position.indices {
            result = result.arg(index - 2).into(); // Note that positions are 1 indexed.
        }

        result
    }
}
