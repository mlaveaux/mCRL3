use std::ops::ControlFlow;

use crate::SortExpression;

type Result<B, C> = ControlFlow<B, C>;

trait SortExpressionVisitor<B, C> {
    fn visit_sort_expression(&mut self, expr: &SortExpression) -> Result<B, C> {
        match expr {
            SortExpression::Product { lhs, rhs } => {
                self.visit_sort_expression(lhs)?;
                self.visit_sort_expression(rhs)
            },
            SortExpression::Function { domain, range } => {
                self.visit_sort_expression(&domain)?;
                self.visit_sort_expression(&range)
            },
            SortExpression::Complex(complex_sort, sort_expression) => {
                self.visit_sort_expression(&sort_expression)
            },
            _ => {
                unreachable!();
            }
        }
    }

}