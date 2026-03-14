use merc_utilities::MercError;

use crate::RegFrm;
use crate::SortExpression;
use crate::StateFrm;

/// Applies the given function recursively to the state formula.
///
/// The substitution `function` takes a state formula and returns an optional new
/// formula. If it returns `Some(new_formula)`, the substitution is applied and
/// the new formula is returned. If it returns `None`, the substitution is not
/// applied and the function continues to traverse the formula tree.
pub fn apply_statefrm<F>(formula: StateFrm, mut function: F) -> Result<StateFrm, MercError>
where
    F: FnMut(&StateFrm) -> Result<Option<StateFrm>, MercError>,
{
    apply_statefrm_rec(formula, &mut function)
}

///// Applies the given function recursively to the sort expression.
pub fn apply_sort_expression<E, F>(sort_expr: SortExpression, mut function: F) -> Result<SortExpression, E>
where
    F: FnMut(&SortExpression) -> Result<Option<SortExpression>, E>,
{
    apply_sort_expression_rec(sort_expr, &mut function)
}

/// Applies the given `function` recursively to the regular formula.
///
/// # Details
///
/// The substitution function is a partial function, where `Some(formula)`
/// indicates that substitution should be applied.
pub fn apply_regular_formula<F>(formula: RegFrm, mut function: F) -> Result<RegFrm, MercError>
where
    F: FnMut(&RegFrm) -> Result<Option<RegFrm>, MercError>,
{
    apply_regular_formula_rec(formula, &mut function)
}

/// See [apply_regular_formula].
fn apply_regular_formula_rec<F>(formula: RegFrm, apply: &mut F) -> Result<RegFrm, MercError>
where
    F: FnMut(&RegFrm) -> Result<Option<RegFrm>, MercError>,
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

/// See [`apply_statefrm`].
fn apply_statefrm_rec<F>(formula: StateFrm, apply: &mut F) -> Result<StateFrm, MercError>
where
    F: FnMut(&StateFrm) -> Result<Option<StateFrm>, MercError>,
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

/// See [`apply_sort_expression`].
fn apply_sort_expression_rec<E, F>(sort_expr: SortExpression, apply: &mut F) -> Result<SortExpression, E>
where
    F: FnMut(&SortExpression) -> Result<Option<SortExpression>, E>,
{
    if let Some(sort_expr) = apply(&sort_expr)? {
        // A substitution was made, return the new sort expression.
        return Ok(sort_expr);
    }

    match sort_expr {
        SortExpression::Product { lhs, rhs } => {
            let lhs = apply_sort_expression_rec(*lhs, apply)?;
            let rhs = apply_sort_expression_rec(*rhs, apply)?;
            Ok(SortExpression::Product {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        SortExpression::Function { domain, range } => {
            let domain = apply_sort_expression_rec(*domain, apply)?;
            let range = apply_sort_expression_rec(*range, apply)?;
            Ok(SortExpression::Function {
                domain: Box::new(domain),
                range: Box::new(range),
            })
        }
        SortExpression::Struct { mut inner } => {
            for decl in &mut inner {
                for (_, sort) in &mut decl.args {
                    *sort = apply_sort_expression_rec(sort.clone(), apply)?;
                }
            }

            Ok(SortExpression::Struct { inner })
        }
        SortExpression::Complex(complex_sort, sort_expression) => {
            let inner = apply_sort_expression_rec(*sort_expression, apply)?;
            Ok(SortExpression::Complex(complex_sort, Box::new(inner)))
        }
        SortExpression::FlattenedFunction { domain, range } => {
            let domain = domain
                .into_iter()
                .map(|sort| apply_sort_expression_rec(sort, apply))
                .collect::<Result<Vec<SortExpression>, _>>()?;
            let range = apply_sort_expression_rec(*range, apply)?;
            Ok(SortExpression::FlattenedFunction {
                domain,
                range: Box::new(range),
            })
        }
        SortExpression::Reference(_) | SortExpression::Simple(_) | SortExpression::Resolved(_, _) => {
            // Ignored
            Ok(sort_expr)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::StateFrm;
    use crate::UntypedStateFrmSpec;

    use super::apply_statefrm;

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
