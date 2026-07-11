use merc_syntax::ComplexSort;
use merc_syntax::SortExpression;

/// Returns true iff the sort is finite.
// Reserved for finiteness-dependent checks; not consumed by a pass yet.
#[allow(dead_code)]
pub(crate) fn is_finite(sort: &SortExpression) -> bool {
    match sort {
        SortExpression::Product { lhs, rhs } => is_finite(lhs) && is_finite(rhs),
        SortExpression::Function { domain, range: _ } => is_finite(domain),
        SortExpression::Struct { inner } => inner
            .iter()
            .all(|decl| decl.args.iter().all(|(_, sort)| is_finite(sort))),
        SortExpression::Simple(sort) => match sort {
            merc_syntax::Sort::Bool => true,
            _ => false, // All number sorts are infinite.
        },
        SortExpression::Complex(complex_sort, sort_expression) => {
            (*complex_sort == ComplexSort::Set || *complex_sort == ComplexSort::FSet) && is_finite(sort_expression)
        }
        SortExpression::FlattenedFunction { domain, range: _ } => domain.iter().all(is_finite),
        SortExpression::Reference(_) | SortExpression::Resolved(_, _) => {
            unreachable!("is_finite should not be called on reference sorts")
        }
    }
}
