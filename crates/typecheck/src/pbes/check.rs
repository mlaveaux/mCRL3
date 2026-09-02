//! The scoped walk over every PBES equation's formula and `init`: checks each `val(...)`
//! expression against `Bool`, resolves each `PropVarInst` against the equation table (name, arity,
//! and each argument's sort), and pushes/pops quantifier binders — accumulating each checked
//! expression's [`TypingInfo`] along the way.

use merc_syntax::PbesExpr;
use merc_syntax::PbesExprKind;
use merc_syntax::PropVarInst;
use merc_syntax::Span;
use merc_syntax::UntypedPbes;

use crate::DataSpecification;
use crate::ResolvedName;
use crate::ResolvedSortId;
use crate::TypingInfo;
use crate::checking::Scope;
use crate::checking::check_expression_against;
use crate::checking::collect_binder_sorts;
use crate::declared_span;

use super::PbesError;
use super::pbes_specification::DeclarationTables;
use super::pbes_specification::resolve_declared_sort;

/// Checks every equation's `formula` and `init` in `spec` against `tables` (already built by
/// [`super::pbes_specification::DeclarationTables::build`], which resolved every declared sort —
/// global variables' and each equation's own parameters'), returning every checked expression's
/// merged [`TypingInfo`] in declaration order.
pub(super) fn check_pbes_specification(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    spec: &UntypedPbes,
) -> Result<TypingInfo, PbesError> {
    let mut typing = TypingInfo::default();

    let globals: Vec<(Span, ResolvedSortId)> = spec
        .global_variables
        .iter()
        .zip(&tables.global_sorts)
        .map(|(decl, &sort)| (decl.span.clone(), sort))
        .collect();

    for (eqn, params) in spec.equations.iter().zip(&tables.equation_params) {
        let mut scope = globals.clone();
        // An equation's own parameters are in scope throughout its formula.
        scope.extend(
            eqn.variable
                .parameters
                .iter()
                .zip(params)
                .map(|(decl, &(_, sort))| (decl.span.clone(), sort)),
        );
        collect_scope(data, &eqn.formula, &mut scope)?;
        check_pbes_expr(data, tables, &scope, &eqn.formula, &mut typing)?;
    }

    // `init` is a bare `PropVarInst`, checked the same way as one appearing inside a formula —
    // scope = globals only, since it sits outside every equation's own parameter scope.
    check_prop_var_inst(data, tables, &globals, &spec.init, &mut typing)?;

    Ok(typing)
}

/// Resolves the declared sort of every `Quantifier` binder in `expr`, extending `scope` with each
/// — every variable occurrence in `expr` already names its own declaration's span
/// (`crate::resolve_pbes_variables`), so this only needs to run once, before checking any of
/// `expr`'s leaves, not interleaved with the check walk itself.
fn collect_scope(
    data: &mut DataSpecification,
    expr: &PbesExpr,
    scope: &mut Vec<(Span, ResolvedSortId)>,
) -> Result<(), PbesError> {
    match &expr.node {
        PbesExprKind::True | PbesExprKind::False | PbesExprKind::DataValExpr(_) | PbesExprKind::PropVarInst(_) => {
            Ok(())
        }
        PbesExprKind::Negation(inner) => collect_scope(data, inner, scope),
        PbesExprKind::Binary { lhs, rhs, .. } => {
            collect_scope(data, lhs, scope)?;
            collect_scope(data, rhs, scope)
        }
        PbesExprKind::Quantifier { variables, body, .. } => {
            collect_binder_sorts(data, scope, variables, resolve_declared_sort)?;
            collect_scope(data, body, scope)
        }
    }
}

fn check_pbes_expr(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &Scope,
    expr: &PbesExpr,
    typing: &mut TypingInfo,
) -> Result<(), PbesError> {
    match &expr.node {
        PbesExprKind::True | PbesExprKind::False => Ok(()),

        PbesExprKind::DataValExpr(data_expr) => {
            let bool_sort = data.context().sorts.bool_sort();
            check_expression_against::<PbesError>(data, scope, data_expr, bool_sort, typing)
        }

        PbesExprKind::PropVarInst(inst) => check_prop_var_inst(data, tables, scope, inst, typing),

        PbesExprKind::Negation(inner) => check_pbes_expr(data, tables, scope, inner, typing),

        PbesExprKind::Binary { lhs, rhs, .. } => {
            check_pbes_expr(data, tables, scope, lhs, typing)?;
            check_pbes_expr(data, tables, scope, rhs, typing)
        }

        PbesExprKind::Quantifier { body, .. } => check_pbes_expr(data, tables, scope, body, typing),
    }
}

/// Resolves `inst.identifier` against the equation table (`UndeclaredPropositionalVariable` if
/// missing), checks its argument count against the declared parameter count (`ArityMismatch`), and
/// checks each argument against its parameter's sort. On success, also pushes a
/// [`ResolvedName::PropositionalVariable`] at `inst`'s own span — see
/// [`docs/name_resolution.md`](../../../../docs/name_resolution.md): unlike an action/process name,
/// a PBES equation is never overloaded, so the equation table's single match is the answer.
fn check_prop_var_inst(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &Scope,
    inst: &PropVarInst,
    typing: &mut TypingInfo,
) -> Result<(), PbesError> {
    let Some(&index) = tables.equations_by_name.get(&inst.identifier) else {
        return Err(PbesError::UndeclaredPropositionalVariable {
            name: inst.identifier.clone(),
            span: inst.span.clone(),
        });
    };
    typing.push(
        inst.span.clone(),
        ResolvedName::PropositionalVariable {
            name: inst.identifier.clone(),
            declaration: declared_span(&tables.equation_decl_spans[index]),
        },
    );

    let params = &tables.equation_params[index];
    if inst.arguments.len() != params.len() {
        return Err(PbesError::ArityMismatch {
            name: inst.identifier.clone(),
            expected: params.len(),
            found: inst.arguments.len(),
            span: inst.span.clone(),
        });
    }

    for (arg, (_, sort)) in inst.arguments.iter().zip(params) {
        check_expression_against::<PbesError>(data, scope, arg, *sort, typing)?;
    }
    Ok(())
}
