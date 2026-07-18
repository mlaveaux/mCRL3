use std::convert::Infallible;
use std::ops::ControlFlow;

use merc_utilities::MercError;

use crate::ActFrm;
use crate::ActFrmKind;
use crate::DataExpr;
use crate::DataExprKind;
use crate::RegFrm;
use crate::RegFrmKind;
use crate::SortExpression;
use crate::SortExpressionKind;
use crate::StateFrm;
use crate::StateFrmKind;

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

/// Visits all sort expressions in the sort expression.
pub fn visit_sort_expr<T, F>(sort_expr: &SortExpression, mut visitor: F) -> Option<T>
where
    F: FnMut(&SortExpression) -> ControlFlow<T>,
{
    try_visit_sort_expr(sort_expr, |sort_expr| -> Result<_, Infallible> {
        Ok(visitor(sort_expr))
    })
    .expect("Inner function does not fail")
}

/// Visits all sort expressions in the sort expression, allowing the visitor to return an error.
pub fn try_visit_sort_expr<E, T, F>(sort_expr: &SortExpression, mut visitor: F) -> Result<Option<T>, E>
where
    F: FnMut(&SortExpression) -> Result<ControlFlow<T>, E>,
{
    visit_sort_expr_rec(sort_expr, &mut visitor)
}

/// Visits all subexpressions of a data expression in pre-order.
pub fn visit_data_expr<T, F>(expr: &DataExpr, mut visitor: F) -> Option<T>
where
    F: FnMut(&DataExpr) -> ControlFlow<T>,
{
    try_visit_data_expr(expr, |expr| -> Result<_, Infallible> { Ok(visitor(expr)) })
        .expect("Inner function does not fail")
}

/// Visits all subexpressions of a data expression in pre-order, allowing the
/// visitor to return an error.
pub fn try_visit_data_expr<E, T, F>(expr: &DataExpr, mut visitor: F) -> Result<Option<T>, E>
where
    F: FnMut(&DataExpr) -> Result<ControlFlow<T>, E>,
{
    visit_data_expr_rec(expr, &mut visitor)
}

/// Visits all subexpressions of a data expression in pre-order, allowing the
/// visitor to mutate each node in place. Children are visited after the
/// visitor ran on their parent, so they are the children of the possibly
/// mutated node.
pub fn try_visit_data_expr_mut<E, T, F>(expr: &mut DataExpr, mut visitor: F) -> Result<Option<T>, E>
where
    F: FnMut(&mut DataExpr) -> Result<ControlFlow<T>, E>,
{
    visit_data_expr_mut_rec(expr, &mut visitor)
}

/// Controls how [`try_visit_sort_expr_with`] proceeds below the current node.
pub enum SortDescend<C> {
    /// Visit the children, passing them the given context.
    Descend(C),
    /// Do not visit the children (the visitor handled them itself, or they are
    /// irrelevant).
    Prune,
}

/// Visits all sort expressions top-down while threading a visitor-chosen
/// context from each node to its children, and allowing subtrees to be pruned.
///
/// The context makes position-dependent checks expressible — e.g. "was a
/// function sort passed on the way here" — which the plain
/// [`try_visit_sort_expr`] cannot do. Note that all children of a node receive
/// the same context; if the children need different treatment, handle them in
/// the visitor and return [`SortDescend::Prune`].
pub fn try_visit_sort_expr_with<E, T, C, F>(sort_expr: &SortExpression, ctx: C, mut visitor: F) -> Result<Option<T>, E>
where
    C: Copy,
    F: FnMut(&SortExpression, C) -> Result<ControlFlow<T, SortDescend<C>>, E>,
{
    visit_sort_expr_with_rec(sort_expr, ctx, &mut visitor)
}

/// See [`try_visit_sort_expr_with`].
fn visit_sort_expr_with_rec<E, T, C, F>(sort_expr: &SortExpression, ctx: C, visitor: &mut F) -> Result<Option<T>, E>
where
    C: Copy,
    F: FnMut(&SortExpression, C) -> Result<ControlFlow<T, SortDescend<C>>, E>,
{
    let ctx = match visitor(sort_expr, ctx)? {
        ControlFlow::Break(result) => return Ok(Some(result)),
        ControlFlow::Continue(SortDescend::Prune) => return Ok(None),
        ControlFlow::Continue(SortDescend::Descend(ctx)) => ctx,
    };

    match &sort_expr.node {
        SortExpressionKind::Product { lhs, rhs } => {
            if let Some(result) = visit_sort_expr_with_rec(lhs, ctx, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_sort_expr_with_rec(rhs, ctx, visitor)? {
                return Ok(Some(result));
            }
        }
        SortExpressionKind::Function { domain, range } => {
            if let Some(result) = visit_sort_expr_with_rec(domain, ctx, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_sort_expr_with_rec(range, ctx, visitor)? {
                return Ok(Some(result));
            }
        }
        SortExpressionKind::Struct { inner } => {
            for constructor in inner {
                for (_name, sort) in &constructor.args {
                    if let Some(result) = visit_sort_expr_with_rec(sort, ctx, visitor)? {
                        return Ok(Some(result));
                    }
                }
            }
        }
        SortExpressionKind::Complex(_complex_sort, sort_expression) => {
            if let Some(result) = visit_sort_expr_with_rec(sort_expression, ctx, visitor)? {
                return Ok(Some(result));
            }
        }
        SortExpressionKind::FlattenedFunction { domain, range } => {
            for domain_sort in domain {
                if let Some(result) = visit_sort_expr_with_rec(domain_sort, ctx, visitor)? {
                    return Ok(Some(result));
                }
            }
            if let Some(result) = visit_sort_expr_with_rec(range, ctx, visitor)? {
                return Ok(Some(result));
            }
        }
        SortExpressionKind::Reference(_) | SortExpressionKind::Simple(_) | SortExpressionKind::Resolved(_, _) => {}
    }

    Ok(None)
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

    match &formula.node {
        StateFrmKind::Binary { lhs, rhs, .. } => {
            if let Some(result) = visit_statefrm_rec(lhs, function)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_statefrm_rec(rhs, function)? {
                return Ok(Some(result));
            }
        }
        StateFrmKind::FixedPoint { body, .. } => {
            if let Some(result) = visit_statefrm_rec(body, function)? {
                return Ok(Some(result));
            }
        }
        StateFrmKind::Bound { body, .. } => {
            if let Some(result) = visit_statefrm_rec(body, function)? {
                return Ok(Some(result));
            }
        }
        StateFrmKind::Modality { expr, .. } => {
            if let Some(result) = visit_statefrm_rec(expr, function)? {
                return Ok(Some(result));
            }
        }
        StateFrmKind::Quantifier { body, .. } => {
            if let Some(result) = visit_statefrm_rec(body, function)? {
                return Ok(Some(result));
            }
        }
        StateFrmKind::DataValExprRightMult(expr, _data_val) => {
            if let Some(result) = visit_statefrm_rec(expr, function)? {
                return Ok(Some(result));
            }
        }
        StateFrmKind::DataValExprLeftMult(_data_val, expr) => {
            if let Some(result) = visit_statefrm_rec(expr, function)? {
                return Ok(Some(result));
            }
        }
        StateFrmKind::Unary { expr, .. } => {
            if let Some(result) = visit_statefrm_rec(expr, function)? {
                return Ok(Some(result));
            }
        }
        StateFrmKind::Id(_, _)
        | StateFrmKind::True
        | StateFrmKind::False
        | StateFrmKind::Delay(_)
        | StateFrmKind::Yaled(_)
        | StateFrmKind::DataValExpr(_) => {}
    }

    // The visitor did not break the traversal.
    Ok(None)
}

/// See [`visit_sort_expr`].
fn visit_sort_expr_rec<E, T, F>(sort_expr: &SortExpression, function: &mut F) -> Result<Option<T>, E>
where
    F: FnMut(&SortExpression) -> Result<ControlFlow<T>, E>,
{
    if let ControlFlow::Break(result) = function(sort_expr)? {
        // The visitor requested to break the traversal.
        return Ok(Some(result));
    }

    match &sort_expr.node {
        SortExpressionKind::Product { lhs, rhs } => {
            if let Some(result) = visit_sort_expr_rec(lhs, function)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_sort_expr_rec(rhs, function)? {
                return Ok(Some(result));
            }
        }
        SortExpressionKind::Function { domain, range } => {
            if let Some(result) = visit_sort_expr_rec(domain, function)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_sort_expr_rec(range, function)? {
                return Ok(Some(result));
            }
        }
        SortExpressionKind::Struct { inner } => {
            for constructors in inner {
                for (_name, sort) in &constructors.args {
                    if let Some(result) = visit_sort_expr_rec(sort, function)? {
                        return Ok(Some(result));
                    }
                }
            }
        }
        SortExpressionKind::Complex(_complex_sort, sort_expression) => {
            if let Some(result) = visit_sort_expr_rec(sort_expression, function)? {
                return Ok(Some(result));
            }
        }
        SortExpressionKind::FlattenedFunction { domain, range } => {
            for domain_sort in domain {
                if let Some(result) = visit_sort_expr_rec(domain_sort, function)? {
                    return Ok(Some(result));
                }
            }
            if let Some(result) = visit_sort_expr_rec(range, function)? {
                return Ok(Some(result));
            }
        }
        SortExpressionKind::Reference(_) | SortExpressionKind::Simple(_) | SortExpressionKind::Resolved(_, _) => {}
    }

    // The visitor did not break the traversal.
    Ok(None)
}

/// See [`try_visit_data_expr`].
fn visit_data_expr_rec<E, T, F>(expr: &DataExpr, visitor: &mut F) -> Result<Option<T>, E>
where
    F: FnMut(&DataExpr) -> Result<ControlFlow<T>, E>,
{
    if let ControlFlow::Break(result) = visitor(expr)? {
        // The visitor requested to break the traversal.
        return Ok(Some(result));
    }

    match &expr.node {
        DataExprKind::Application { function, arguments } => {
            if let Some(result) = visit_data_expr_rec(function, visitor)? {
                return Ok(Some(result));
            }
            for argument in arguments {
                if let Some(result) = visit_data_expr_rec(argument, visitor)? {
                    return Ok(Some(result));
                }
            }
        }
        DataExprKind::List(elements) | DataExprKind::Set(elements) => {
            for element in elements {
                if let Some(result) = visit_data_expr_rec(element, visitor)? {
                    return Ok(Some(result));
                }
            }
        }
        DataExprKind::Bag(elements) => {
            for element in elements {
                if let Some(result) = visit_data_expr_rec(&element.expr, visitor)? {
                    return Ok(Some(result));
                }
                if let Some(result) = visit_data_expr_rec(&element.multiplicity, visitor)? {
                    return Ok(Some(result));
                }
            }
        }
        DataExprKind::SetBagComp { variable: _, predicate } => {
            if let Some(result) = visit_data_expr_rec(predicate, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::Lambda { variables: _, body }
        | DataExprKind::Quantifier {
            op: _,
            variables: _,
            body,
        } => {
            if let Some(result) = visit_data_expr_rec(body, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::Unary { op: _, expr } => {
            if let Some(result) = visit_data_expr_rec(expr, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::Binary { op: _, lhs, rhs } => {
            if let Some(result) = visit_data_expr_rec(lhs, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_data_expr_rec(rhs, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::FunctionUpdate { expr, update } => {
            if let Some(result) = visit_data_expr_rec(expr, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_data_expr_rec(&update.expr, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_data_expr_rec(&update.update, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::Whr { expr, assignments } => {
            if let Some(result) = visit_data_expr_rec(expr, visitor)? {
                return Ok(Some(result));
            }
            for assignment in assignments {
                if let Some(result) = visit_data_expr_rec(&assignment.expr, visitor)? {
                    return Ok(Some(result));
                }
            }
        }
        DataExprKind::Id(_)
        | DataExprKind::Number(_)
        | DataExprKind::Bool(_)
        | DataExprKind::EmptyList
        | DataExprKind::EmptySet
        | DataExprKind::EmptyBag => {}
    }

    // The visitor did not break the traversal.
    Ok(None)
}

/// See [`try_visit_data_expr_mut`].
fn visit_data_expr_mut_rec<E, T, F>(expr: &mut DataExpr, visitor: &mut F) -> Result<Option<T>, E>
where
    F: FnMut(&mut DataExpr) -> Result<ControlFlow<T>, E>,
{
    if let ControlFlow::Break(result) = visitor(expr)? {
        // The visitor requested to break the traversal.
        return Ok(Some(result));
    }

    match &mut expr.node {
        DataExprKind::Application { function, arguments } => {
            if let Some(result) = visit_data_expr_mut_rec(function, visitor)? {
                return Ok(Some(result));
            }
            for argument in arguments {
                if let Some(result) = visit_data_expr_mut_rec(argument, visitor)? {
                    return Ok(Some(result));
                }
            }
        }
        DataExprKind::List(elements) | DataExprKind::Set(elements) => {
            for element in elements {
                if let Some(result) = visit_data_expr_mut_rec(element, visitor)? {
                    return Ok(Some(result));
                }
            }
        }
        DataExprKind::Bag(elements) => {
            for element in elements {
                if let Some(result) = visit_data_expr_mut_rec(&mut element.expr, visitor)? {
                    return Ok(Some(result));
                }
                if let Some(result) = visit_data_expr_mut_rec(&mut element.multiplicity, visitor)? {
                    return Ok(Some(result));
                }
            }
        }
        DataExprKind::SetBagComp { variable: _, predicate } => {
            if let Some(result) = visit_data_expr_mut_rec(predicate, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::Lambda { variables: _, body }
        | DataExprKind::Quantifier {
            op: _,
            variables: _,
            body,
        } => {
            if let Some(result) = visit_data_expr_mut_rec(body, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::Unary { op: _, expr } => {
            if let Some(result) = visit_data_expr_mut_rec(expr, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::Binary { op: _, lhs, rhs } => {
            if let Some(result) = visit_data_expr_mut_rec(lhs, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_data_expr_mut_rec(rhs, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::FunctionUpdate { expr, update } => {
            if let Some(result) = visit_data_expr_mut_rec(expr, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_data_expr_mut_rec(&mut update.expr, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_data_expr_mut_rec(&mut update.update, visitor)? {
                return Ok(Some(result));
            }
        }
        DataExprKind::Whr { expr, assignments } => {
            if let Some(result) = visit_data_expr_mut_rec(expr, visitor)? {
                return Ok(Some(result));
            }
            for assignment in assignments {
                if let Some(result) = visit_data_expr_mut_rec(&mut assignment.expr, visitor)? {
                    return Ok(Some(result));
                }
            }
        }
        DataExprKind::Id(_)
        | DataExprKind::Number(_)
        | DataExprKind::Bool(_)
        | DataExprKind::EmptyList
        | DataExprKind::EmptySet
        | DataExprKind::EmptyBag => {}
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

    match &formula.node {
        RegFrmKind::Iteration(reg_frm) => {
            if let Some(result) = visit_regular_formula_rec(reg_frm, visit)? {
                return Ok(Some(result));
            }
        }
        RegFrmKind::Plus(reg_frm) => {
            if let Some(result) = visit_regular_formula_rec(reg_frm, visit)? {
                return Ok(Some(result));
            }
        }
        RegFrmKind::Sequence { lhs, rhs } => {
            if let Some(result) = visit_regular_formula_rec(lhs, visit)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_regular_formula_rec(rhs, visit)? {
                return Ok(Some(result));
            }
        }
        RegFrmKind::Choice { lhs, rhs } => {
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

    match &formula.node {
        ActFrmKind::Negation(act_frm) => {
            if let Some(result) = visit_action_formula_rec(act_frm, visitor)? {
                return Ok(Some(result));
            }
        }
        ActFrmKind::Quantifier {
            quantifier: _,
            variables: _,
            body,
        } => {
            if let Some(result) = visit_action_formula_rec(body, visitor)? {
                return Ok(Some(result));
            }
        }
        ActFrmKind::Binary { op: _, lhs, rhs } => {
            if let Some(result) = visit_action_formula_rec(lhs, visitor)? {
                return Ok(Some(result));
            }
            if let Some(result) = visit_action_formula_rec(rhs, visitor)? {
                return Ok(Some(result));
            }
        }
        ActFrmKind::True | ActFrmKind::False | ActFrmKind::MultAct(_) | ActFrmKind::DataExprVal(_) => {}
    }

    // The visitor did not break the traversal.
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::ops::ControlFlow;

    use crate::DataExpr;
    use crate::DataExprKind;
    use crate::Sort;
    use crate::SortExpressionKind;

    use super::try_visit_data_expr_mut;
    use super::visit_data_expr;
    use super::visit_sort_expr;

    /// Regression test: the FlattenedFunction arm used to discard `Break`
    /// results from both the domain sorts and the range.
    #[test]
    fn test_visit_sort_expr_breaks_inside_flattened_function() {
        let sort = SortExpressionKind::FlattenedFunction {
            domain: vec![SortExpressionKind::Simple(Sort::Nat).into()],
            range: Box::new(SortExpressionKind::Simple(Sort::Bool).into()),
        }
        .into();

        let found = visit_sort_expr(&sort, |expr| match &expr.node {
            SortExpressionKind::Simple(Sort::Nat) => ControlFlow::Break("domain"),
            _ => ControlFlow::Continue(()),
        });
        assert_eq!(found, Some("domain"));

        let found = visit_sort_expr(&sort, |expr| match &expr.node {
            SortExpressionKind::Simple(Sort::Bool) => ControlFlow::Break("range"),
            _ => ControlFlow::Continue(()),
        });
        assert_eq!(found, Some("range"));
    }

    /// The easy-to-miss children (bag multiplicities and whr assignments) are
    /// visited as well.
    #[test]
    fn test_visit_data_expr_reaches_nested_children() {
        let expr = DataExpr::parse("f(v) whr v = { e: m } end").unwrap();

        for name in ["v", "e", "m"] {
            let found = visit_data_expr(&expr, |expr| match &expr.node {
                DataExprKind::Id(id) if id == name => ControlFlow::Break(()),
                _ => ControlFlow::Continue(()),
            });
            assert_eq!(found, Some(()), "identifier {name} was not visited");
        }
    }

    #[test]
    fn test_try_visit_data_expr_mut_rewrites_in_place() {
        let mut expr = DataExpr::parse("x + f(x)").unwrap();

        let result: Option<Infallible> = try_visit_data_expr_mut(&mut expr, |expr| {
            if let DataExprKind::Id(name) = &mut expr.node
                && name == "x"
            {
                *name = "y".to_string();
            }
            Ok::<_, Infallible>(ControlFlow::Continue(()))
        })
        .unwrap();

        assert!(result.is_none());
        assert_eq!(expr, DataExpr::parse("y + f(y)").unwrap());
    }
}
