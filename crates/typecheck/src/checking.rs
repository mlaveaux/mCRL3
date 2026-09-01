//! Shared building blocks for a scoped, `TypingInfo`-accumulating walk over an expression tree
//! that lives *outside* the data specification proper — a process body ([`crate::process::check`])
//! or a PBES equation ([`crate::pbes::check`]) — each with its own variable scope. Crate-private:
//! both callers reach this via `crate::checking::...`.

use merc_syntax::DataExpr;

use crate::DataSpecification;
use crate::InferenceError;
use crate::ResolvedSortId;
use crate::TypingInfo;
use crate::WellTypedError;
use crate::infer_expression_in_scope;
use crate::lower_data_expr;
use crate::typing_info;

/// A shadowing stack of in-scope variable sorts: global variables, then (for a `proc` body, a
/// PBES equation, …) that declaration's own parameters, then any binder pushed so far
/// (`sum`/`dist`, a quantifier).
pub(crate) struct Scope<'a> {
    pub(crate) variables: Vec<(&'a str, ResolvedSortId)>,
}

impl<'a> Scope<'a> {
    pub(crate) fn new(base: Vec<(&'a str, ResolvedSortId)>) -> Self {
        Scope { variables: base }
    }
}

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
    scope: &Scope<'_>,
    expr: &DataExpr,
    expected: ResolvedSortId,
    typing: &mut TypingInfo,
) -> Result<(), E>
where
    E: From<WellTypedError> + From<InferenceError>,
{
    let lowered = prepare_expression::<E>(data, expr)?;
    let (ctx, spec, system) = data.context_and_specs_mut();
    let equation_typing = infer_expression_in_scope(ctx, spec, system, &lowered, &scope.variables, Some(expected))?;
    typing.merge(typing_info::build(data, &equation_typing));
    Ok(())
}
