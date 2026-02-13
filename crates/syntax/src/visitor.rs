use std::ops::ControlFlow;

use merc_utilities::MercError;

use crate::RegFrm;
use crate::{SortExpression, StateFrm};

/// Applies the given function recursively to the state formula.
///
/// The substitution function takes a state formula and returns an optional new
/// formula. If it returns `Some(new_formula)`, the substitution is applied and
/// the new formula is returned. If it returns `None`, the substitution is not
/// applied and the function continues to traverse the formula tree.
pub fn apply_statefrm<F>(formula: StateFrm, mut function: F) -> Result<StateFrm, MercError>
where
    F: FnMut(&StateFrm) -> Result<Option<StateFrm>, MercError>,
{
    apply_statefrm_rec(formula, &mut function)
}

/// Visits the state formula and calls the given function on each subformula.
///
/// The visitor function takes a state formula and returns a `ControlFlow`. If
/// it returns `ControlFlow::Break(value)`, the traversal is stopped and the
/// value is returned. If it returns `ControlFlow::Continue(())`, the traversal
/// continues.
pub fn visit_statefrm<T, F>(formula: &StateFrm, mut visitor: F) -> Result<Option<T>, MercError>
where
    F: FnMut(&StateFrm) -> Result<ControlFlow<T>, MercError>,
{
    visit_statefrm_rec(formula, &mut visitor)
}

pub fn visit_sort_expr<T, F>(sort_expr: &SortExpression, mut visitor: F) -> Result<Option<T>, MercError>
where
    F: FnMut(&SortExpression) -> Result<ControlFlow<T>, MercError>,
{
    visit_sort_expr_rec(sort_expr, &mut visitor)
}

/// See [`apply_statefrm`].
fn apply_statefrm_rec(
    formula: StateFrm,
    apply: &mut F,
) -> Result<StateFrm, MercError> 
    where F: FnMut(&StateFrm) -> Result<Option<StateFrm>, MercError>
{
    if let Some(formula) = apply(&formula)? {
        // A substitution was made, return the new formula.
        return Ok(formula);
    }

    match formula {
        StateFrm::Binary { op, lhs, rhs } => {
            let new_lhs = apply_statefrm_rec(*lhs, apply)?;
            let new_rhs = apply_statefrm_rec(*rhs, apply)?;
            Ok(StateFrm::Binary {
                op,
                lhs: Box::new(new_lhs),
                rhs: Box::new(new_rhs),
            })
        }
        StateFrm::FixedPoint {
            operator,
            variable,
            body,
        } => {
            let new_body = apply_statefrm_rec(*body, apply)?;
            Ok(StateFrm::FixedPoint {
                operator,
                variable,
                body: Box::new(new_body),
            })
        }
        StateFrm::Bound { bound, variables, body } => {
            let new_body = apply_statefrm_rec(*body, apply)?;
            Ok(StateFrm::Bound {
                bound,
                variables,
                body: Box::new(new_body),
            })
        }
        StateFrm::Modality {
            operator,
            formula,
            expr,
        } => {
            let expr = apply_statefrm_rec(*expr, apply)?;
            Ok(StateFrm::Modality {
                operator,
                formula,
                expr: Box::new(expr),
            })
        }
        StateFrm::Quantifier {
            quantifier,
            variables,
            body,
        } => {
            let new_body = apply_statefrm_rec(*body, apply)?;
            Ok(StateFrm::Quantifier {
                quantifier,
                variables,
                body: Box::new(new_body),
            })
        }
        StateFrm::DataValExprRightMult(expr, data_val) => {
            let new_expr = apply_statefrm_rec(*expr, apply)?;
            Ok(StateFrm::DataValExprRightMult(Box::new(new_expr), data_val))
        }
        StateFrm::DataValExprLeftMult(data_val, expr) => {
            let new_expr = apply_statefrm_rec(*expr, apply)?;
            Ok(StateFrm::DataValExprLeftMult(data_val, Box::new(new_expr)))
        }
        StateFrm::Unary { op, expr } => {
            let new_expr = apply_statefrm_rec(*expr, apply)?;
            Ok(StateFrm::Unary {
                op,
                expr: Box::new(new_expr),
            })
        }
        StateFrm::Id(_, _)
        | StateFrm::True
        | StateFrm::False
        | StateFrm::Delay(_)
        | StateFrm::Yaled(_)
        | StateFrm::DataValExpr(_) => Ok(formula),
    }
}

/// See [`visit_statefrm`].
fn visit_statefrm_rec<T, F>(formula: &StateFrm, function: &mut F) -> Result<Option<T>, MercError>
where
    F: FnMut(&StateFrm) -> Result<ControlFlow<T>, MercError>,
{
    if let ControlFlow::Break(result) = function(formula)? {
        // The visitor requested to break the traversal.
        return Ok(Some(result));
    }

    match formula {
        StateFrm::Binary { lhs, rhs, .. } => {
            visit_statefrm_rec(lhs, function)?;
            visit_statefrm_rec(rhs, function)?;
        }
        StateFrm::FixedPoint { body, .. } => {
            visit_statefrm_rec(body, function)?;
        }
        StateFrm::Bound { body, .. } => {
            visit_statefrm_rec(body, function)?;
        }
        StateFrm::Modality { expr, .. } => {
            visit_statefrm_rec(expr, function)?;
        }
        StateFrm::Quantifier { body, .. } => {
            visit_statefrm_rec(body, function)?;
        }
        StateFrm::DataValExprRightMult(expr, _data_val) => {
            visit_statefrm_rec(expr, function)?;
        }
        StateFrm::DataValExprLeftMult(_data_val, expr) => {
            visit_statefrm_rec(expr, function)?;
        }
        StateFrm::Unary { expr, .. } => {
            visit_statefrm_rec(expr, function)?;
        }
        StateFrm::Id(_, _)
        | StateFrm::True
        | StateFrm::False
        | StateFrm::Delay(_)
        | StateFrm::Yaled(_)
        | StateFrm::DataValExpr(_) => {}
    }

    // The visitor did not break the traversal.
    Ok(None)
}

fn visit_sort_expr_rec<T, F>(sort_expr: &SortExpression, function: &mut F) -> Result<Option<T>, MercError>
where
    F: FnMut(&SortExpression) -> Result<ControlFlow<T>, MercError>,
{
    if let ControlFlow::Break(result) = function(sort_expr)? {
        // The visitor requested to break the traversal.
        return Ok(Some(result));
    }

    match sort_expr {
        SortExpression::Product { lhs, rhs } => {
            visit_sort_expr_rec(lhs, function)?;
            visit_sort_expr_rec(rhs, function)?;
        }
        SortExpression::Function { domain, range } => {
            visit_sort_expr_rec(domain, function)?;
            visit_sort_expr_rec(range, function)?;
        }
        SortExpression::Struct { inner } => {
            for constructors in inner {
                for (_name, sort) in &constructors.args {
                    visit_sort_expr_rec(&sort, function)?;
                }
            }
        }
        SortExpression::Complex(_complex_sort, sort_expression) => {
            visit_sort_expr_rec(sort_expression, function)?;
        }
        SortExpression::Reference(_) | SortExpression::Simple(_) => {}
    }

    // The visitor did not break the traversal.
    Ok(None)
}

/// Applies the given function recursively to the regular formula. The
/// substitution function takes a regular formula and returns an optional new
/// formula. If it returns `Some(new_formula)`, the substitution is applied and
/// the new formula is returned. If it returns `None`, the substitution is not
/// applied and the function continues to traverse the formula tree.
pub fn apply_regular_formula<F>(
    formula: RegFrm,
    mut function: F,
) -> Result<RegFrm, MercError> 
    where F: FnMut(&RegFrm) -> Result<Option<RegFrm>, MercError>
{
    apply_regular_formula_rec(formula, &mut function)
}

/// See [apply_regular_formula].
fn apply_regular_formula_rec<F>(formula: RegFrm, apply: &mut F) -> Result<RegFrm, MercError> 
    where F: FnMut(&RegFrm) -> Result<Option<RegFrm>, MercError>
{
    if let Some(formula) = apply(&formula)? {
        // A substitution was made, return the new formula.
        return Ok(formula);
    }

    match formula {
        RegFrm::Iteration(reg_frm) => {
            let new_reg_frm = apply_regular_formula_rec(*reg_frm, apply)?;
            Ok(RegFrm::Iteration(Box::new(new_reg_frm)))
        }
        RegFrm::Plus(reg_frm) => {
            let new_reg_frm = apply_regular_formula_rec(*reg_frm, apply)?;
            Ok(RegFrm::Plus(Box::new(new_reg_frm)))
        }
        RegFrm::Sequence { lhs, rhs } => {
            let new_lhs = apply_regular_formula_rec(*lhs, apply)?;
            let new_rhs = apply_regular_formula_rec(*rhs, apply)?;
            Ok(RegFrm::Sequence {
                lhs: Box::new(new_lhs),
                rhs: Box::new(new_rhs),
            })
        }
        RegFrm::Choice { lhs, rhs } => {
            let new_lhs = apply_regular_formula_rec(*lhs, apply)?;
            let new_rhs = apply_regular_formula_rec(*rhs, apply)?;
            Ok(RegFrm::Choice {
                lhs: Box::new(new_lhs),
                rhs: Box::new(new_rhs),
            })
        }
        _ => Ok(formula),
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::UntypedStateFrmSpec;

    use super::*;

    #[test]
    fn test_visit_state_frm_variables() {
        let input = UntypedStateFrmSpec::parse("mu X. [a]X && mu X. X && Y").unwrap();

        let mut variables = vec![];
        apply_statefrm(input.formula, |frm| {
            if let StateFrm::Id(name, _) = frm {
                variables.push(name.clone());
            }

            Ok(None)
        })
        .unwrap();

        assert_eq!(variables, vec!["X", "X", "Y"]);
    }
}
