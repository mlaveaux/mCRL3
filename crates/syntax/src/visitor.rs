use std::ops::ControlFlow;

use merc_utilities::MercError;

use crate::ActFrm;
use crate::RegFrm;
use crate::SortExpression;
use crate::StateFrm;

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
            if let Some(result) = visit_statefrm_rec(lhs, function)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_statefrm_rec(rhs, function)? {
                return Ok(Some(result));
            }
        }
        StateFrm::FixedPoint { body, .. } => {
            if let Some(result) = visit_statefrm_rec(body, function)? {
                return Ok(Some(result));
            }
        }
        StateFrm::Bound { body, .. } => {
            if let Some(result) = visit_statefrm_rec(body, function)? {
                return Ok(Some(result));
            }
        }
        StateFrm::Modality { expr, .. } => {
            if let Some(result) = visit_statefrm_rec(expr, function)? {
                return Ok(Some(result));
            }
        }
        StateFrm::Quantifier { body, .. } => {
            if let Some(result) = visit_statefrm_rec(body, function)? {
                return Ok(Some(result));
            }
        }
        StateFrm::DataValExprRightMult(expr, _data_val) => {
            if let Some(result) = visit_statefrm_rec(expr, function)? {
                return Ok(Some(result));
            }
        }
        StateFrm::DataValExprLeftMult(_data_val, expr) => {
            if let Some(result) = visit_statefrm_rec(expr, function)? {
                return Ok(Some(result));
            }
        }
        StateFrm::Unary { expr, .. } => {
            if let Some(result) = visit_statefrm_rec(expr, function)? {
                return Ok(Some(result));
            }
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
            if let Some(result) = visit_sort_expr_rec(lhs, function)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_sort_expr_rec(rhs, function)? {
                return Ok(Some(result));
            }
        }
        SortExpression::Function { domain, range } => {
            if let Some(result) = visit_sort_expr_rec(domain, function)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_sort_expr_rec(range, function)? {
                return Ok(Some(result));
            }
        }
        SortExpression::Struct { inner } => {
            for constructors in inner {
                for (_name, sort) in &constructors.args {
                    if let Some(result) = visit_sort_expr_rec(sort, function)? {
                        return Ok(Some(result));
                    }
                }
            }
        }
        SortExpression::Complex(_complex_sort, sort_expression) => {
            if let Some(result) = visit_sort_expr_rec(sort_expression, function)? {
                return Ok(Some(result));
            }
        }
        SortExpression::Reference(_) | SortExpression::Simple(_) => {}
    }

    // The visitor did not break the traversal.
    Ok(None)
}

/// Maps the given `function` recursively to the regular formula.
pub fn visit_regular_formula<T, F>(formula: &RegFrm, mut function: F) -> Result<Option<T>, MercError>
where
    F: FnMut(&RegFrm) -> Result<ControlFlow<T>, MercError>,
{
    visit_regular_formula_rec(formula, &mut function)
}

/// See [visit_regular_formula].
fn visit_regular_formula_rec<T, F>(formula: &RegFrm, visit: &mut F) -> Result<Option<T>, MercError>
where
    F: FnMut(&RegFrm) -> Result<ControlFlow<T>, MercError>,
{
    if let ControlFlow::Break(result) = visit(formula)? {
        // A substitution was made, return the new formula.
        return Ok(Some(result));
    }

    match formula {
        RegFrm::Iteration(reg_frm) => {
            if let Some(result) = visit_regular_formula_rec(reg_frm, visit)? {
                return Ok(Some(result));
            }
        }
        RegFrm::Plus(reg_frm) => {
            if let Some(result) = visit_regular_formula_rec(reg_frm, visit)? {
                return Ok(Some(result));
            }
        }
        RegFrm::Sequence { lhs, rhs } => {
            if let Some(result) = visit_regular_formula_rec(lhs, visit)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_regular_formula_rec(rhs, visit)? {
                return Ok(Some(result));
            }
        }
        RegFrm::Choice { lhs, rhs } => {
            if let Some(result) = visit_regular_formula_rec(lhs, visit)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_regular_formula_rec(rhs, visit)? {
                return Ok(Some(result));
            }
        }
        _ => {}
    }

    Ok(None)
}

/// Visitor for action formulas.
///
pub fn visit_action_formula<T, F>(formula: &ActFrm, mut visitor: F) -> Result<Option<T>, MercError>
where
    F: FnMut(&ActFrm) -> Result<ControlFlow<T>, MercError>,
{
    visit_action_formula_rec(formula, &mut visitor)
}

fn visit_action_formula_rec<T, F>(formula: &ActFrm, visitor: &mut F) -> Result<Option<T>, MercError>
where
    F: FnMut(&ActFrm) -> Result<ControlFlow<T>, MercError>,
{
    if let ControlFlow::Break(result) = visitor(formula)? {
        // The visitor requested to break the traversal.
        return Ok(Some(result));
    }

    match formula {
        ActFrm::Negation(act_frm) => {
            if let Some(result) = visit_action_formula_rec(act_frm, visitor)? {
                return Ok(Some(result));
            }
        }
        ActFrm::Quantifier {
            quantifier: _,
            variables: _,
            body,
        } => {
            if let Some(result) = visit_action_formula_rec(body, visitor)? {
                return Ok(Some(result));
            }
        }
        ActFrm::Binary { op: _, lhs, rhs } => {
            if let Some(result) = visit_action_formula_rec(lhs, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_action_formula_rec(rhs, visitor)? {
                return Ok(Some(result));
            }
        }
        ActFrm::True | ActFrm::False | ActFrm::MultAct(_) | ActFrm::DataExprVal(_) => {}
    }

    // The visitor did not break the traversal.
    Ok(None)
}
