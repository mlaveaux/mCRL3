//! The scoped walk over every process body (`proc` declarations and `init`), checking each
//! `DataExpr` it contains — action arguments, process-instantiation arguments, conditions, time
//! bounds, `dist` weights — against its own expected sort and its own enclosing variable scope,
//! and accumulating each checked expression's [`TypingInfo`].
//!
//! By the time this runs, [`super::reparse`] has already fixed up every `Condition` the grammar
//! misparsed because of mCRL2's `.`/`+` ambiguity (see its module doc comment and the crate
//! README); this walk assumes every `Condition` it sees is already correctly shaped.

use merc_syntax::ActionName;
use merc_syntax::Assignment;
use merc_syntax::DataExpr;
use merc_syntax::IdDecl;
use merc_syntax::ProcessExpr;
use merc_syntax::ProcessExprKind;
use merc_syntax::Span;
use merc_syntax::UntypedProcessSpecification;

use crate::DataSpecification;
use crate::ResolvedSortId;
use crate::TypingInfo;
use crate::checking::Scope;
use crate::checking::check_expression_against;

use super::ProcessError;
use super::process_specification::DeclarationTables;
use super::process_specification::resolve_declared_sort;

/// Checks every `proc` body and `init` in `spec` against `tables` (already built by
/// [`super::process_specification::DeclarationTables::build`], which resolved every declared
/// sort — actions' argument sorts, processes' parameter sorts, and global variables'), returning
/// every checked expression's merged [`TypingInfo`] in declaration order.
pub(super) fn check_process_specification(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    spec: &UntypedProcessSpecification,
) -> Result<TypingInfo, ProcessError> {
    let mut typing = TypingInfo::default();

    let globals: Vec<(&str, ResolvedSortId)> = spec
        .global_variables
        .iter()
        .zip(&tables.global_sorts)
        .map(|(decl, &sort)| (decl.identifier.as_str(), sort))
        .collect();

    for (proc_decl, params) in spec.process_declarations.iter().zip(&tables.process_params) {
        let mut scope = Scope::new(globals.clone());
        // A process's own parameters shadow a global variable of the same name.
        scope
            .variables
            .extend(params.iter().map(|(name, sort)| (name.as_str(), *sort)));
        check_process_expr(data, tables, &mut scope, &proc_decl.body, &mut typing)?;
    }

    if let Some(init) = &spec.init {
        let mut scope = Scope::new(globals.clone());
        check_process_expr(data, tables, &mut scope, init, &mut typing)?;
    }

    Ok(typing)
}

fn check_process_expr<'a>(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &mut Scope<'a>,
    expr: &'a ProcessExpr,
    typing: &mut TypingInfo,
) -> Result<(), ProcessError> {
    match &expr.node {
        ProcessExprKind::Delta | ProcessExprKind::Tau => Ok(()),

        ProcessExprKind::Action(name, args) => {
            check_action_or_process(data, tables, scope, name, args, &expr.span, typing)
        }
        ProcessExprKind::Id(name, assignments) => {
            check_instantiation(data, tables, scope, name, assignments, &expr.span, typing)
        }

        ProcessExprKind::Sum { variables, operand } => {
            let pushed = push_binders(data, scope, variables)?;
            let result = check_process_expr(data, tables, scope, operand, typing);
            scope.variables.truncate(scope.variables.len() - pushed);
            result
        }
        ProcessExprKind::Dist {
            variables,
            expr: weight,
            operand,
        } => {
            let pushed = push_binders(data, scope, variables)?;
            // Checked against `Real` *with* the bound variables already in scope: `dist`'s weight
            // is the distribution's density over them.
            let real_sort = data.context().sorts.real_sort();
            let result = check_expression_against::<ProcessError>(data, scope, weight, real_sort, typing)
                .and_then(|()| check_process_expr(data, tables, scope, operand, typing));
            scope.variables.truncate(scope.variables.len() - pushed);
            result
        }

        ProcessExprKind::Binary { lhs, rhs, .. } => {
            check_process_expr(data, tables, scope, lhs, typing)?;
            check_process_expr(data, tables, scope, rhs, typing)
        }

        ProcessExprKind::Condition { condition, then, else_ } => {
            let bool_sort = data.context().sorts.bool_sort();
            check_expression_against::<ProcessError>(data, scope, condition, bool_sort, typing)?;
            check_process_expr(data, tables, scope, then, typing)?;
            if let Some(else_) = else_ {
                check_process_expr(data, tables, scope, else_, typing)?;
            }
            Ok(())
        }

        ProcessExprKind::At {
            expr: inner,
            operand: time,
        } => {
            let real_sort = data.context().sorts.real_sort();
            check_expression_against::<ProcessError>(data, scope, time, real_sort, typing)?;
            check_process_expr(data, tables, scope, inner, typing)
        }

        ProcessExprKind::Hide { actions, operand } => {
            check_action_names(tables, actions)?;
            check_process_expr(data, tables, scope, operand, typing)
        }
        ProcessExprKind::Block { actions, operand } => {
            check_action_names(tables, actions)?;
            check_process_expr(data, tables, scope, operand, typing)
        }
        ProcessExprKind::Allow { actions, operand } => {
            for label in actions {
                check_action_names(tables, &label.actions)?;
            }
            check_process_expr(data, tables, scope, operand, typing)
        }
        ProcessExprKind::Comm { comm, operand } => {
            for c in comm {
                check_action_names(tables, &c.from.actions)?;
                check_action_names(tables, std::slice::from_ref(&c.to))?;
            }
            check_process_expr(data, tables, scope, operand, typing)
        }
        ProcessExprKind::Rename { renames, operand } => {
            for r in renames {
                check_action_names(tables, std::slice::from_ref(&r.from))?;
                check_action_names(tables, std::slice::from_ref(&r.to))?;
            }
            check_process_expr(data, tables, scope, operand, typing)
        }
    }
}

/// Resolves each of `variables`' declared sorts and pushes them onto `scope`, returning how many
/// were pushed so the caller can `truncate` them back off once it's done with them — `Sum`/
/// `Dist`'s shared shadowing step.
fn push_binders<'a>(
    data: &mut DataSpecification,
    scope: &mut Scope<'a>,
    variables: &'a [IdDecl],
) -> Result<usize, ProcessError> {
    for var in variables {
        let sort = resolve_declared_sort(data, &var.sort)?;
        scope.variables.push((var.identifier.as_str(), sort));
    }
    Ok(variables.len())
}

/// Resolves `name(args)` (an action instance or a positional process instantiation — the grammar
/// makes these ambiguous, see the crate README) against both declaration tables, trying every
/// candidate of the right arity and requiring exactly one to succeed.
///
/// Each candidate is checked against its own scratch `TypingInfo`, merged into `typing` only once
/// the single successful candidate is known — a failed or ultimately-ambiguous candidate's typing
/// must never reach `typing`, since it would otherwise misreport a sort for the wrong overload at
/// the same span.
fn check_action_or_process(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &Scope<'_>,
    name: &str,
    args: &[DataExpr],
    span: &Span,
    typing: &mut TypingInfo,
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
        return Err(ProcessError::UndeclaredActionOrProcess {
            name: name.to_string(),
            arity: args.len(),
            span: span.clone(),
        });
    }

    let mut successes = 0usize;
    let mut first_error = None;
    let mut matched_typing = TypingInfo::default();
    for expected in &candidates {
        let mut candidate_typing = TypingInfo::default();
        match check_arguments(data, scope, args, expected, &mut candidate_typing) {
            Ok(()) => {
                successes += 1;
                matched_typing = candidate_typing;
            }
            Err(error) => drop(first_error.get_or_insert(error)),
        }
    }

    match successes {
        0 => Err(ProcessError::NoMatchingOverload {
            name: name.to_string(),
            span: span.clone(),
            cause: Box::new(
                first_error.expect("at least one candidate, so at least one recorded error when none succeed"),
            ),
        }),
        1 => {
            typing.merge(matched_typing);
            Ok(())
        }
        count => Err(ProcessError::AmbiguousActionOrProcess {
            name: name.to_string(),
            count,
            span: span.clone(),
        }),
    }
}

fn check_arguments(
    data: &mut DataSpecification,
    scope: &Scope<'_>,
    args: &[DataExpr],
    expected: &[ResolvedSortId],
    typing: &mut TypingInfo,
) -> Result<(), ProcessError> {
    for (arg, &sort) in args.iter().zip(expected) {
        check_expression_against::<ProcessError>(data, scope, arg, sort, typing)?;
    }
    Ok(())
}

/// Resolves `P(x = e, ...)` (the assignment form of process instantiation) against every
/// same-named process declaration whose parameters cover every assigned identifier, requiring
/// exactly one to succeed. Unlike [`check_action_or_process`], this is not arity-filtered: an
/// assignment list may legally leave some parameters unassigned.
///
/// Merges only the single successful candidate's `TypingInfo` into `typing` — see
/// [`check_action_or_process`]'s doc comment.
fn check_instantiation(
    data: &mut DataSpecification,
    tables: &DeclarationTables,
    scope: &Scope<'_>,
    name: &str,
    assignments: &[Assignment],
    span: &Span,
    typing: &mut TypingInfo,
) -> Result<(), ProcessError> {
    let indices: &[usize] = tables.processes_by_name.get(name).map_or(&[], Vec::as_slice);
    if indices.is_empty() {
        return Err(ProcessError::UndeclaredActionOrProcess {
            name: name.to_string(),
            arity: assignments.len(),
            span: span.clone(),
        });
    }

    let mut successes = 0usize;
    let mut first_error = None;
    let mut matched_typing = TypingInfo::default();
    for &index in indices {
        let mut candidate_typing = TypingInfo::default();
        match check_one_instantiation(
            data,
            scope,
            &tables.process_params[index],
            assignments,
            name,
            &mut candidate_typing,
        ) {
            Ok(()) => {
                successes += 1;
                matched_typing = candidate_typing;
            }
            Err(error) => drop(first_error.get_or_insert(error)),
        }
    }

    match successes {
        0 => Err(first_error.expect("at least one candidate, so at least one recorded error when none succeed")),
        1 => {
            typing.merge(matched_typing);
            Ok(())
        }
        count => Err(ProcessError::AmbiguousActionOrProcess {
            name: name.to_string(),
            count,
            span: span.clone(),
        }),
    }
}

fn check_one_instantiation(
    data: &mut DataSpecification,
    scope: &Scope<'_>,
    params: &[(String, ResolvedSortId)],
    assignments: &[Assignment],
    process: &str,
    typing: &mut TypingInfo,
) -> Result<(), ProcessError> {
    for assignment in assignments {
        let Some(&(_, sort)) = params.iter().find(|(param, _)| *param == assignment.identifier) else {
            return Err(ProcessError::UnknownProcessParameter {
                process: process.to_string(),
                name: assignment.identifier.clone(),
                span: assignment.span.clone(),
            });
        };
        check_expression_against::<ProcessError>(data, scope, &assignment.expr, sort, typing)?;
    }
    Ok(())
}

/// Checks that every `names` entry is a declared action, reporting the offending name's own
/// [Span] (rather than the enclosing expression's) since each [ActionName] now carries one.
fn check_action_names(tables: &DeclarationTables, names: &[ActionName]) -> Result<(), ProcessError> {
    for name in names {
        if !tables.actions_by_name.contains_key(name.as_str()) {
            return Err(ProcessError::UndeclaredAction {
                name: name.node.clone(),
                span: name.span.clone(),
            });
        }
    }
    Ok(())
}
