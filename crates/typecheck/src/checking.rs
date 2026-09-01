//! Shared building blocks for a `TypingInfo`-accumulating walk over an expression tree that lives
//! *outside* the data specification proper — a process body ([`crate::process::check`]) or a PBES
//! equation ([`crate::pbes::check`]). Crate-private: both callers reach this via
//! `crate::checking::...`.
//!
//! Neither caller threads its own name-shadowing scope through this walk: every variable
//! occurrence in the expression being checked is already a `Resolved` node carrying its own
//! declaration's span (`crate::resolve_process_variables`/`crate::resolve_pbes_variables`, see
//! `docs/name_resolution.md`), so a flat [`Scope`] — every declaration reachable from the current
//! `proc` body/PBES equation, keyed by its own span — is enough; there is no shadowing to resolve
//! here, since two different declarations never share a span.

use merc_syntax::DataExpr;
use merc_syntax::IdDecl;
use merc_syntax::SortExpression;
use merc_syntax::Span;

use crate::DataSpecification;
use crate::InferenceError;
use crate::ResolvedSortId;
use crate::TypingInfo;
use crate::WellTypedError;
use crate::infer_expression_in_scope;
use crate::lower_data_expr;
use crate::typing_info;

/// Every declaration reachable from the `proc` body/PBES equation currently being checked —
/// global variables, that declaration's own parameters, and every `sum`/`dist`/quantifier binder
/// anywhere in it — keyed by each declaration's own span.
pub(crate) type Scope = [(Span, ResolvedSortId)];

/// Prepares a raw expression for inference: resolves its embedded binder sorts (see
/// [`DataSpecification::resolve_expression_binder_sorts`]) and lowers it, exactly as
/// [`DataSpecification::typecheck_expression`] does for a standalone expression.
pub(crate) fn prepare_expression<E>(data: &mut DataSpecification, expr: &DataExpr) -> Result<DataExpr, E>
where
    E: From<WellTypedError>,
{
    let mut expr = expr.clone();
    data.resolve_expression_binder_sorts(&mut expr)?;
    Ok(lower_data_expr(expr))
}

/// Checks `expr` (a `sum`/`dist` condition or time bound, an assignment-form instantiation
/// argument, a `PropVarInst` argument, …) against `expected`, merging its `TypingInfo` into
/// `typing` on success.
///
/// Generic over the caller's own error type `E` (`ProcessError`, `PbesError`, …) rather than
/// returning a `checking`-local error: both callers already wrap [`WellTypedError`]/
/// [`InferenceError`] via `#[from]`, so there is nothing this module needs of its own.
pub(crate) fn check_expression_against<E>(
    data: &mut DataSpecification,
    scope: &Scope,
    expr: &DataExpr,
    expected: ResolvedSortId,
    typing: &mut TypingInfo,
) -> Result<(), E>
where
    E: From<WellTypedError> + From<InferenceError>,
{
    let lowered = prepare_expression::<E>(data, expr)?;
    let (ctx, spec, system) = data.context_and_specs_mut();
    let equation_typing = infer_expression_in_scope(ctx, spec, system, &lowered, scope, Some(expected))?;
    typing.merge(typing_info::build(data, &equation_typing));
    Ok(())
}

/// Resolves each of `variables`' declared sorts, extending `scope` with `(declaration span,
/// resolved sort)` for each — the shared leaf [`crate::process::check`]'s `Sum`/`Dist` and
/// [`crate::pbes::check`]'s `Quantifier` scope collection calls into.
pub(crate) fn collect_binder_sorts<E>(
    data: &mut DataSpecification,
    scope: &mut Vec<(Span, ResolvedSortId)>,
    variables: &[IdDecl],
    mut resolve: impl FnMut(&mut DataSpecification, &SortExpression) -> Result<ResolvedSortId, E>,
) -> Result<(), E> {
    for var in variables {
        let sort = resolve(data, &var.sort)?;
        scope.push((var.span.clone(), sort));
    }
    Ok(())
}
