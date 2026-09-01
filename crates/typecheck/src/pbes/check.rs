//! The scoped walk over every PBES equation's formula and `init`: checks each `val(...)`
//! expression against `Bool`, resolves each `PropVarInst` against the equation table (name, arity,
//! and each argument's sort), and pushes/pops quantifier binders — accumulating each checked
//! expression's [`TypingInfo`] along the way.

use merc_syntax::IdDecl;
use merc_syntax::PbesExpr;
use merc_syntax::PbesExprKind;
use merc_syntax::PropVarInst;
use merc_syntax::UntypedPbes;

use crate::DataSpecification;
use crate::ResolvedSortId;
use crate::TypingInfo;
use crate::checking::Scope;
use crate::checking::check_expression_against;

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

    let globals: Vec<(&str, ResolvedSortId)> = spec
        .global_variables
        .iter()
        .zip(&tables.global_sorts)
        .map(|(decl, &sort)| (decl.identifier.as_str(), sort))
        .collect();

    for (eqn, params) in spec.equations.iter().zip(&tables.equation_params) {
        let mut scope = Scope::new(globals.clone());
        // An equation's own parameters shadow a global variable of the same name.
        scope
            .variables
            .extend(params.iter().map(|(name, sort)| (name.as_str(), *sort)));
        check_pbes_expr(data, tables, &mut scope, &eqn.formula, &mut typing)?;
    }

    // `init` is a bare `PropVarInst`, checked the same way as one appearing inside a formula —
    // scope = globals only, since it sits outside every equation's own parameter scope.
    let scope = Scope::new(globals);
    check_prop_var_inst(data, tables, &scope, &spec.init, &mut typing)?;

    Ok(typing)
}

fn check_pbes_expr<'a>(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &mut Scope<'a>,
    expr: &'a PbesExpr,
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

        PbesExprKind::Quantifier { variables, body, .. } => {
            let pushed = push_binders(data, scope, variables)?;
            let result = check_pbes_expr(data, tables, scope, body, typing);
            scope.variables.truncate(scope.variables.len() - pushed);
            result
        }
    }
}

/// Resolves each of `variables`' declared sorts and pushes them onto `scope`, returning how many
/// were pushed so the caller can `truncate` them back off once it's done with them — mirrors
/// `crate::process::check`'s `Sum`/`Dist` binder handling, for a quantifier here instead.
fn push_binders<'a>(
    data: &mut DataSpecification,
    scope: &mut Scope<'a>,
    variables: &'a [IdDecl],
) -> Result<usize, PbesError> {
    for var in variables {
        let sort = resolve_declared_sort(data, &var.sort)?;
        scope.variables.push((var.identifier.as_str(), sort));
    }
    Ok(variables.len())
}

/// Resolves `inst.identifier` against the equation table (`UndeclaredPropositionalVariable` if
/// missing), checks its argument count against the declared parameter count (`ArityMismatch`), and
/// checks each argument against its parameter's sort.
fn check_prop_var_inst(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &Scope<'_>,
    inst: &PropVarInst,
    typing: &mut TypingInfo,
) -> Result<(), PbesError> {
    let Some(&index) = tables.equations_by_name.get(&inst.identifier) else {
        return Err(PbesError::UndeclaredPropositionalVariable {
            name: inst.identifier.clone(),
            span: inst.span.clone(),
        });
    };

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
