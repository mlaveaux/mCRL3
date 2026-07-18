use merc_utilities::MercError;

use crate::Assignment;
use crate::BagElement;
use crate::DataExpr;
use crate::DataExprKind;
use crate::DataExprUpdate;
use crate::RegFrm;
use crate::RegFrmKind;
use crate::SortExpression;
use crate::SortExpressionKind;
use crate::StateFrm;
use crate::StateFrmKind;

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

/// Applies the given function recursively to the sort expression.
pub fn apply_sort_expression<E, F>(sort_expr: SortExpression, mut function: F) -> Result<SortExpression, E>
where
    F: FnMut(&SortExpression) -> Result<Option<SortExpression>, E>,
{
    apply_sort_expression_rec(sort_expr, &mut function)
}

/// Rebuilds a data expression bottom-up: the subexpressions of every node are
/// mapped first, then `function` is applied to the node with its rebuilt
/// children. The expression returned by `function` is not traversed again, so
/// the mapping always terminates.
pub fn map_data_expr<F>(expr: DataExpr, mut function: F) -> DataExpr
where
    F: FnMut(DataExpr) -> DataExpr,
{
    map_data_expr_rec(expr, &mut function)
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

    let span = formula.span.clone();
    match formula.node {
        RegFrmKind::Iteration(reg_frm) => {
            let new_reg_frm = apply_regular_formula_rec(*reg_frm, apply)?;
            Ok(RegFrmKind::Iteration(Box::new(new_reg_frm)).spanned(span))
        }
        RegFrmKind::Plus(reg_frm) => {
            let new_reg_frm = apply_regular_formula_rec(*reg_frm, apply)?;
            Ok(RegFrmKind::Plus(Box::new(new_reg_frm)).spanned(span))
        }
        RegFrmKind::Sequence { lhs, rhs } => {
            let new_lhs = apply_regular_formula_rec(*lhs, apply)?;
            let new_rhs = apply_regular_formula_rec(*rhs, apply)?;
            Ok(RegFrmKind::Sequence {
                lhs: Box::new(new_lhs),
                rhs: Box::new(new_rhs),
            }
            .spanned(span))
        }
        RegFrmKind::Choice { lhs, rhs } => {
            let new_lhs = apply_regular_formula_rec(*lhs, apply)?;
            let new_rhs = apply_regular_formula_rec(*rhs, apply)?;
            Ok(RegFrmKind::Choice {
                lhs: Box::new(new_lhs),
                rhs: Box::new(new_rhs),
            }
            .spanned(span))
        }
        other => Ok(other.spanned(span)),
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

    let span = formula.span.clone();
    match formula.node {
        StateFrmKind::Binary { op, lhs, rhs } => {
            let new_lhs = apply_statefrm_rec(*lhs, apply)?;
            let new_rhs = apply_statefrm_rec(*rhs, apply)?;
            Ok(StateFrmKind::Binary {
                op,
                lhs: Box::new(new_lhs),
                rhs: Box::new(new_rhs),
            }
            .spanned(span))
        }
        StateFrmKind::FixedPoint {
            operator,
            variable,
            body,
        } => {
            let new_body = apply_statefrm_rec(*body, apply)?;
            Ok(StateFrmKind::FixedPoint {
                operator,
                variable,
                body: Box::new(new_body),
            }
            .spanned(span))
        }
        StateFrmKind::Bound { bound, variables, body } => {
            let new_body = apply_statefrm_rec(*body, apply)?;
            Ok(StateFrmKind::Bound {
                bound,
                variables,
                body: Box::new(new_body),
            }
            .spanned(span))
        }
        StateFrmKind::Modality {
            operator,
            formula,
            expr,
        } => {
            let expr = apply_statefrm_rec(*expr, apply)?;
            Ok(StateFrmKind::Modality {
                operator,
                formula,
                expr: Box::new(expr),
            }
            .spanned(span))
        }
        StateFrmKind::Quantifier {
            quantifier,
            variables,
            body,
        } => {
            let new_body = apply_statefrm_rec(*body, apply)?;
            Ok(StateFrmKind::Quantifier {
                quantifier,
                variables,
                body: Box::new(new_body),
            }
            .spanned(span))
        }
        StateFrmKind::DataValExprRightMult(expr, data_val) => {
            let new_expr = apply_statefrm_rec(*expr, apply)?;
            Ok(StateFrmKind::DataValExprRightMult(Box::new(new_expr), data_val).spanned(span))
        }
        StateFrmKind::DataValExprLeftMult(data_val, expr) => {
            let new_expr = apply_statefrm_rec(*expr, apply)?;
            Ok(StateFrmKind::DataValExprLeftMult(data_val, Box::new(new_expr)).spanned(span))
        }
        StateFrmKind::Unary { op, expr } => {
            let new_expr = apply_statefrm_rec(*expr, apply)?;
            Ok(StateFrmKind::Unary {
                op,
                expr: Box::new(new_expr),
            }
            .spanned(span))
        }
        other @ (StateFrmKind::Id(_, _)
        | StateFrmKind::True
        | StateFrmKind::False
        | StateFrmKind::Delay(_)
        | StateFrmKind::Yaled(_)
        | StateFrmKind::DataValExpr(_)) => Ok(other.spanned(span)),
    }
}

/// See [`map_data_expr`].
fn map_data_expr_rec<F>(expr: DataExpr, apply: &mut F) -> DataExpr
where
    F: FnMut(DataExpr) -> DataExpr,
{
    let DataExpr { node, span } = expr;
    let kind = match node {
        DataExprKind::Application { function, arguments } => DataExprKind::Application {
            function: Box::new(map_data_expr_rec(*function, apply)),
            arguments: arguments
                .into_iter()
                .map(|argument| map_data_expr_rec(argument, apply))
                .collect(),
        },
        DataExprKind::List(elements) => DataExprKind::List(
            elements
                .into_iter()
                .map(|element| map_data_expr_rec(element, apply))
                .collect(),
        ),
        DataExprKind::Set(elements) => DataExprKind::Set(
            elements
                .into_iter()
                .map(|element| map_data_expr_rec(element, apply))
                .collect(),
        ),
        DataExprKind::Bag(elements) => DataExprKind::Bag(
            elements
                .into_iter()
                .map(|element| BagElement {
                    expr: map_data_expr_rec(element.expr, apply),
                    multiplicity: map_data_expr_rec(element.multiplicity, apply),
                })
                .collect(),
        ),
        DataExprKind::SetBagComp { variable, predicate } => DataExprKind::SetBagComp {
            variable,
            predicate: Box::new(map_data_expr_rec(*predicate, apply)),
        },
        DataExprKind::Lambda { variables, body } => DataExprKind::Lambda {
            variables,
            body: Box::new(map_data_expr_rec(*body, apply)),
        },
        DataExprKind::Quantifier { op, variables, body } => DataExprKind::Quantifier {
            op,
            variables,
            body: Box::new(map_data_expr_rec(*body, apply)),
        },
        DataExprKind::Unary { op, expr } => DataExprKind::Unary {
            op,
            expr: Box::new(map_data_expr_rec(*expr, apply)),
        },
        DataExprKind::Binary { op, lhs, rhs } => DataExprKind::Binary {
            op,
            lhs: Box::new(map_data_expr_rec(*lhs, apply)),
            rhs: Box::new(map_data_expr_rec(*rhs, apply)),
        },
        DataExprKind::FunctionUpdate { expr, update } => DataExprKind::FunctionUpdate {
            expr: Box::new(map_data_expr_rec(*expr, apply)),
            update: Box::new(DataExprUpdate {
                expr: map_data_expr_rec(update.expr, apply),
                update: map_data_expr_rec(update.update, apply),
            }),
        },
        DataExprKind::Whr { expr, assignments } => DataExprKind::Whr {
            expr: Box::new(map_data_expr_rec(*expr, apply)),
            assignments: assignments
                .into_iter()
                .map(|assignment| Assignment {
                    identifier: assignment.identifier,
                    expr: map_data_expr_rec(assignment.expr, apply),
                })
                .collect(),
        },
        leaf @ (DataExprKind::Id(_)
        | DataExprKind::Number(_)
        | DataExprKind::Bool(_)
        | DataExprKind::EmptyList
        | DataExprKind::EmptySet
        | DataExprKind::EmptyBag) => leaf,
    };

    apply(kind.spanned(span))
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

    let span = sort_expr.span.clone();
    match sort_expr.node {
        SortExpressionKind::Product { lhs, rhs } => {
            let lhs = apply_sort_expression_rec(*lhs, apply)?;
            let rhs = apply_sort_expression_rec(*rhs, apply)?;
            Ok(SortExpressionKind::Product {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
            .spanned(span))
        }
        SortExpressionKind::Function { domain, range } => {
            let domain = apply_sort_expression_rec(*domain, apply)?;
            let range = apply_sort_expression_rec(*range, apply)?;
            Ok(SortExpressionKind::Function {
                domain: Box::new(domain),
                range: Box::new(range),
            }
            .spanned(span))
        }
        SortExpressionKind::Struct { mut inner } => {
            for decl in &mut inner {
                for (_, sort) in &mut decl.args {
                    *sort = apply_sort_expression_rec(sort.clone(), apply)?;
                }
            }

            Ok(SortExpressionKind::Struct { inner }.spanned(span))
        }
        SortExpressionKind::Complex(complex_sort, sort_expression) => {
            let inner = apply_sort_expression_rec(*sort_expression, apply)?;
            Ok(SortExpressionKind::Complex(complex_sort, Box::new(inner)).spanned(span))
        }
        SortExpressionKind::FlattenedFunction { domain, range } => {
            let domain = domain
                .into_iter()
                .map(|sort| apply_sort_expression_rec(sort, apply))
                .collect::<Result<Vec<SortExpression>, _>>()?;
            let range = apply_sort_expression_rec(*range, apply)?;
            Ok(SortExpressionKind::FlattenedFunction {
                domain,
                range: Box::new(range),
            }
            .spanned(span))
        }
        other @ (SortExpressionKind::Reference(_)
        | SortExpressionKind::Simple(_)
        | SortExpressionKind::Resolved(_, _)) => {
            // Ignored
            Ok(other.spanned(span))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::DataExpr;
    use crate::DataExprBinaryOp;
    use crate::DataExprKind;
    use crate::StateFrmKind;
    use crate::UntypedStateFrmSpec;

    use super::apply_statefrm;
    use super::map_data_expr;

    #[test]
    fn test_visit_state_frm_variables() {
        let input = UntypedStateFrmSpec::parse("mu X. [a]X && mu X. X && Y").unwrap();

        let mut variables = vec![];
        apply_statefrm(input.formula, |frm| {
            if let StateFrmKind::Id(name, _) = &frm.node {
                variables.push(name.clone());
            }

            Ok(None)
        })
        .unwrap();

        assert_eq!(variables, vec!["X", "X", "Y"]);
    }

    /// Children are mapped before their parent: rewriting the addition to its
    /// left operand yields the already-mapped operand.
    #[test]
    fn test_map_data_expr_maps_bottom_up() {
        let expr = DataExpr::parse("x + z").unwrap();

        let mapped = map_data_expr(expr, |expr| {
            let DataExpr { node, span } = expr;
            match node {
                DataExprKind::Id(name) if name == "x" => DataExprKind::Number("1".to_string()).into(),
                DataExprKind::Binary {
                    op: DataExprBinaryOp::Add,
                    lhs,
                    rhs: _,
                } => *lhs,
                other => other.spanned(span),
            }
        });

        assert_eq!(mapped, DataExprKind::Number("1".to_string()).into());
    }
}
