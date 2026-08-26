//! The scoped walk over every process body (`proc` declarations and `init`), checking each
//! `DataExpr` it contains — action arguments, process-instantiation arguments, conditions, time
//! bounds, `dist` weights — against its own expected sort and its own enclosing variable scope.
//!
//! There is deliberately no generic "visit every `DataExpr` root in the specification" helper
//! here (and no attempt to extend [`crate::Traverse`] to a heterogeneous container like
//! `UntypedProcessSpecification`, which its own doc comment already rules out): a generic walker
//! would hand back roots stripped of exactly what checking a root needs — the expected sort and
//! the variable scope at that point. This module *is* that walk, hand-written once, over
//! [`ProcessExprKind`] directly.

use merc_syntax::Assignment;
use merc_syntax::DataExpr;
use merc_syntax::DataExprBinaryOp;
use merc_syntax::DataExprKind;
use merc_syntax::IdDecl;
use merc_syntax::ProcessExpr;
use merc_syntax::ProcessExprKind;
use merc_syntax::Span;
use merc_syntax::UntypedProcessSpecification;

use crate::DataSpecification;
use crate::ResolvedSortId;
use crate::infer_expression_in_scope;
use crate::lower_data_expr;

use super::ProcessError;
use super::process_specification::DeclarationTables;
use super::process_specification::resolve_declared_sort;

/// A shadowing stack of in-scope variable sorts: global variables, then (for a `proc` body) that
/// process's own parameters, then any `sum`/`dist` binders entered so far.
pub(super) struct Scope<'a> {
    variables: Vec<(&'a str, ResolvedSortId)>,
}

impl<'a> Scope<'a> {
    pub(super) fn new(base: Vec<(&'a str, ResolvedSortId)>) -> Self {
        Scope { variables: base }
    }
}

/// Checks every `proc` body and `init` in `spec` against `tables` (already built by
/// [`super::process_specification::DeclarationTables::build`], which resolved every declared
/// sort — actions' argument sorts, processes' parameter sorts, and global variables').
pub(super) fn check_process_specification(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    spec: &UntypedProcessSpecification,
) -> Result<(), ProcessError> {
    let globals: Vec<(&str, ResolvedSortId)> =
        spec.global_variables.iter().zip(&tables.global_sorts).map(|(decl, &sort)| (decl.identifier.as_str(), sort)).collect();

    for (proc_decl, params) in spec.process_declarations.iter().zip(&tables.process_params) {
        let mut scope = Scope::new(globals.clone());
        // A process's own parameters shadow a global variable of the same name.
        scope.variables.extend(params.iter().map(|(name, sort)| (name.as_str(), *sort)));
        check_process_expr(data, tables, &mut scope, &proc_decl.body)?;
    }

    if let Some(init) = &spec.init {
        let mut scope = Scope::new(globals.clone());
        check_process_expr(data, tables, &mut scope, init)?;
    }

    Ok(())
}

fn check_process_expr<'a>(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &mut Scope<'a>,
    expr: &'a ProcessExpr,
) -> Result<(), ProcessError> {
    match &expr.node {
        ProcessExprKind::Delta | ProcessExprKind::Tau => Ok(()),

        ProcessExprKind::Action(name, args) => check_action_or_process(data, tables, scope, name, args, &expr.span),
        ProcessExprKind::Id(name, assignments) => check_instantiation(data, tables, scope, name, assignments, &expr.span),

        ProcessExprKind::Sum { variables, operand } => {
            let pushed = push_binders(data, scope, variables)?;
            let result = check_process_expr(data, tables, scope, operand);
            scope.variables.truncate(scope.variables.len() - pushed);
            result
        }
        ProcessExprKind::Dist { variables, expr: weight, operand } => {
            let pushed = push_binders(data, scope, variables)?;
            // Checked against `Real` *with* the bound variables already in scope: `dist`'s weight
            // is the distribution's density over them.
            let real_sort = data.context().sorts.real_sort();
            let result = check_expression_against(data, scope, weight, real_sort)
                .and_then(|()| check_process_expr(data, tables, scope, operand));
            scope.variables.truncate(scope.variables.len() - pushed);
            result
        }

        ProcessExprKind::Binary { lhs, rhs, .. } => {
            check_process_expr(data, tables, scope, lhs)?;
            check_process_expr(data, tables, scope, rhs)
        }

        ProcessExprKind::Condition { condition, then, else_ } => {
            let bool_sort = data.context().sorts.bool_sort();
            check_condition(data, tables, scope, condition, bool_sort)?;
            check_process_expr(data, tables, scope, then)?;
            if let Some(else_) = else_ {
                check_process_expr(data, tables, scope, else_)?;
            }
            Ok(())
        }

        ProcessExprKind::At { expr: inner, operand: time } => {
            let real_sort = data.context().sorts.real_sort();
            check_expression_against(data, scope, time, real_sort)?;
            check_process_expr(data, tables, scope, inner)
        }

        ProcessExprKind::Hide { actions, operand } => {
            check_action_names(tables, actions, &expr.span)?;
            check_process_expr(data, tables, scope, operand)
        }
        ProcessExprKind::Block { actions, operand } => {
            check_action_names(tables, actions, &expr.span)?;
            check_process_expr(data, tables, scope, operand)
        }
        ProcessExprKind::Allow { actions, operand } => {
            for label in actions {
                check_action_names(tables, &label.actions, &expr.span)?;
            }
            check_process_expr(data, tables, scope, operand)
        }
        ProcessExprKind::Comm { comm, operand } => {
            for c in comm {
                check_action_names(tables, &c.from.actions, &expr.span)?;
                check_action_names(tables, std::slice::from_ref(&c.to), &expr.span)?;
            }
            check_process_expr(data, tables, scope, operand)
        }
        ProcessExprKind::Rename { renames, operand } => {
            for r in renames {
                check_action_names(tables, std::slice::from_ref(&r.from), &expr.span)?;
                check_action_names(tables, std::slice::from_ref(&r.to), &expr.span)?;
            }
            check_process_expr(data, tables, scope, operand)
        }
    }
}

/// Resolves each of `variables`' declared sorts and pushes them onto `scope`, returning how many
/// were pushed so the caller can `truncate` them back off once it's done with them — `Sum`/
/// `Dist`'s shared shadowing step.
fn push_binders<'a>(data: &mut DataSpecification, scope: &mut Scope<'a>, variables: &'a [IdDecl]) -> Result<usize, ProcessError> {
    for var in variables {
        let sort = resolve_declared_sort(data, &var.sort)?;
        scope.variables.push((var.identifier.as_str(), sort));
    }
    Ok(variables.len())
}

/// Prepares a raw process-body expression for inference: resolves its embedded binder sorts
/// (see [`DataSpecification::resolve_expression_binder_sorts`] — an equation's binder sorts are
/// resolved once, up front, over the whole data specification, but a process-body expression
/// never goes through that pass) and lowers it, exactly as
/// [`crate::DataSpecification::typecheck_expression`] does for a standalone expression.
fn prepare_expression(data: &mut DataSpecification, expr: &DataExpr) -> Result<DataExpr, ProcessError> {
    let mut expr = expr.clone();
    data.resolve_expression_binder_sorts(&mut expr)?;
    Ok(lower_data_expr(expr))
}

/// Checks `expr` (a `sum`/`dist` condition or time bound, an assignment-form instantiation
/// argument, …) against `expected`.
fn check_expression_against(
    data: &mut DataSpecification,
    scope: &Scope<'_>,
    expr: &DataExpr,
    expected: ResolvedSortId,
) -> Result<(), ProcessError> {
    let lowered = prepare_expression(data, expr)?;
    let (ctx, spec, system) = data.context_and_specs_mut();
    infer_expression_in_scope(ctx, spec, system, &lowered, &scope.variables, Some(expected))?;
    Ok(())
}

/// Checks `condition` against `Bool`, recovering from mCRL2's well-known ambiguity between
/// process sequential composition and the data "at" (function/list indexing) operator, which
/// share the `.` token (see the crate README): `act(args) . cond -> then <> else` parses its
/// whole `act(args) . cond` as *one* `DataExprAt` chain, rather than a process step followed by
/// the real trailing condition, because the (context-free) grammar's `->`-prefix rule always
/// tries the greedy DataExpr reading first and only semantic information — which name is
/// declared as what, gathered by [`super::process_specification::DeclarationTables::build`] —
/// can actually disambiguate the two readings.
///
/// If `condition` type checks as `Bool` outright, this is a no-op (the overwhelmingly common
/// case — no misparse to recover from). Otherwise, flattens `condition`'s left-associative `.`
/// chain and, if its *last* piece alone type checks as `Bool`, treats every earlier piece as an
/// action or (positional) process-instantiation step to check and run first. Any failure along
/// the way — the last piece still doesn't type check alone, or an earlier piece isn't shaped
/// like a plain action/process call — falls back to reporting the *original* whole-condition
/// error, which is more informative than one from a speculative reinterpretation that was itself
/// on the wrong track.
fn check_condition(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &mut Scope<'_>,
    condition: &DataExpr,
    bool_sort: ResolvedSortId,
) -> Result<(), ProcessError> {
    let Err(original_error) = check_expression_against(data, scope, condition, bool_sort) else {
        return Ok(());
    };

    let parts = flatten_at_chain(condition);
    let (last, prefix) = parts.split_last().expect("flatten_at_chain always returns at least one part");
    if prefix.is_empty() || check_expression_against(data, scope, last, bool_sort).is_err() {
        return Err(original_error);
    }

    for piece in prefix {
        let DataExprKind::Application { function, arguments } = &piece.node else {
            return Err(original_error);
        };
        let DataExprKind::Id(name) = &function.node else {
            return Err(original_error);
        };
        check_action_or_process(data, tables, scope, name, arguments, &piece.span)?;
    }
    Ok(())
}

/// Flattens a left-associative `Binary { op: At, lhs, rhs }` chain (`a . b . c` parses as
/// `(a . b) . c`) back into its original left-to-right pieces `[a, b, c]`. A `condition` whose
/// top-level node isn't `At` at all flattens to the single piece `[condition]`.
fn flatten_at_chain(expr: &DataExpr) -> Vec<&DataExpr> {
    let mut parts = Vec::new();
    let mut current = expr;
    while let DataExprKind::Binary { op: DataExprBinaryOp::At, lhs, rhs } = &current.node {
        parts.push(rhs.as_ref());
        current = lhs;
    }
    parts.push(current);
    parts.reverse();
    parts
}

/// Resolves `name(args)` (an action instance or a positional process instantiation — the grammar
/// makes these ambiguous, see the crate README) against both declaration tables, trying every
/// candidate of the right arity and requiring exactly one to succeed.
fn check_action_or_process(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &Scope<'_>,
    name: &str,
    args: &[DataExpr],
    span: &Span,
) -> Result<(), ProcessError> {
    let candidates: Vec<Vec<ResolvedSortId>> = tables
        .actions_by_name
        .get(name)
        .into_iter()
        .flatten()
        .map(|&index| tables.action_domains[index].clone())
        .chain(
            tables
                .processes_by_name
                .get(name)
                .into_iter()
                .flatten()
                .map(|&index| tables.process_params[index].iter().map(|&(_, sort)| sort).collect()),
        )
        .filter(|domain| domain.len() == args.len())
        .collect();

    if candidates.is_empty() {
        return Err(ProcessError::UndeclaredActionOrProcess { name: name.to_string(), arity: args.len(), span: span.clone() });
    }

    let mut successes = 0usize;
    let mut first_error = None;
    for expected in &candidates {
        match check_arguments(data, scope, args, expected) {
            Ok(()) => successes += 1,
            Err(error) => drop(first_error.get_or_insert(error)),
        }
    }

    match successes {
        0 => Err(ProcessError::NoMatchingOverload {
            name: name.to_string(),
            span: span.clone(),
            cause: Box::new(first_error.expect("at least one candidate, so at least one recorded error when none succeed")),
        }),
        1 => Ok(()),
        count => Err(ProcessError::AmbiguousActionOrProcess { name: name.to_string(), count, span: span.clone() }),
    }
}

fn check_arguments(
    data: &mut DataSpecification,
    scope: &Scope<'_>,
    args: &[DataExpr],
    expected: &[ResolvedSortId],
) -> Result<(), ProcessError> {
    for (arg, &sort) in args.iter().zip(expected) {
        check_expression_against(data, scope, arg, sort)?;
    }
    Ok(())
}

/// Resolves `P(x = e, ...)` (the assignment form of process instantiation) against every
/// same-named process declaration whose parameters cover every assigned identifier, requiring
/// exactly one to succeed. Unlike [`check_action_or_process`], this is not arity-filtered: an
/// assignment list may legally leave some parameters unassigned.
fn check_instantiation(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &Scope<'_>,
    name: &str,
    assignments: &[Assignment],
    span: &Span,
) -> Result<(), ProcessError> {
    let indices: &[usize] = tables.processes_by_name.get(name).map_or(&[], Vec::as_slice);
    if indices.is_empty() {
        return Err(ProcessError::UndeclaredActionOrProcess { name: name.to_string(), arity: assignments.len(), span: span.clone() });
    }

    let mut successes = 0usize;
    let mut first_error = None;
    for &index in indices {
        match check_one_instantiation(data, scope, &tables.process_params[index], assignments, name, span) {
            Ok(()) => successes += 1,
            Err(error) => drop(first_error.get_or_insert(error)),
        }
    }

    match successes {
        0 => Err(first_error.expect("at least one candidate, so at least one recorded error when none succeed")),
        1 => Ok(()),
        count => Err(ProcessError::AmbiguousActionOrProcess { name: name.to_string(), count, span: span.clone() }),
    }
}

fn check_one_instantiation(
    data: &mut DataSpecification,
    scope: &Scope<'_>,
    params: &[(String, ResolvedSortId)],
    assignments: &[Assignment],
    process: &str,
    // `Assignment` carries no span of its own (see the crate README); an error about one is
    // located at the enclosing instantiation instead.
    span: &Span,
) -> Result<(), ProcessError> {
    for assignment in assignments {
        let Some(&(_, sort)) = params.iter().find(|(param, _)| *param == assignment.identifier) else {
            return Err(ProcessError::UnknownProcessParameter {
                process: process.to_string(),
                name: assignment.identifier.clone(),
                span: span.clone(),
            });
        };
        check_expression_against(data, scope, &assignment.expr, sort)?;
    }
    Ok(())
}

fn check_action_names(tables: &DeclarationTables, names: &[String], span: &Span) -> Result<(), ProcessError> {
    for name in names {
        if !tables.actions_by_name.contains_key(name) {
            return Err(ProcessError::UndeclaredAction { name: name.clone(), span: span.clone() });
        }
    }
    Ok(())
}
