//! A generic control flow graph analysis over "positional" summand-like
//! structures.
//!
//! The analysis only needs a shared parameter vector and, per summand, a guard
//! (condition) and the list of parameters it assigns a new value (its
//! non-identity write assignments) — see [`CfgSummand`]. Both an LPS summand
//! (`merc_lps::ExplicitSummand`, which assigns the process parameters directly)
//! and a PBES-in-SRF summand (`merc_pbes::PbesSrfSummand`, which assigns the
//! unified parameter vector of its target equation) fit this shape, so both
//! crates share this implementation instead of each re-deriving it.

use std::collections::HashMap;

use crate::ATerm;
use crate::ATermList;
use crate::DataExpression;
use crate::DataExpressionRef;
use crate::DataVariable;
use crate::LearnSuccessorsContext;
use crate::free_variables_data_expression;
use crate::is_application;
use crate::is_variable;

/// A summand-like structure that [`ControlFlowAnalysis`] can analyse.
pub trait CfgSummand {
    /// The summand's guard.
    fn condition(&self) -> &DataExpression;

    /// The non-identity parameter assignments performed by the summand, as a
    /// list of `data::assignment` terms (`lhs := rhs`, read with `.arg(0)` /
    /// `.arg(1)`). A parameter absent here is left unchanged by the summand.
    fn write_assignments(&self) -> &ATermList<ATerm>;
}

/// The outcome of a control flow graph analysis.
///
/// A parameter `d` is a *control flow parameter* (CFP) when, in every summand,
/// it behaves like a program counter rather than a data variable:
///
/// - whenever a summand changes `d`, it assigns it a closed (constant) value, and
/// - whenever a summand reads `d` to decide its guard, it does so through a
///   conjunct `d == c` with `c` a closed value (its *source value*), and
/// - a summand that does not constrain `d` leaves it unchanged.
///
/// Under these conditions the value of a CFP in any reachable state is one of a
/// statically known set of constants, so a summand whose guard requires
/// `d == c` can only fire from states where `d` already equals `c`.
pub struct ControlFlowAnalysis {
    /// The indices into `parameters` identified as control flow parameters.
    pub control_flow_parameters: Vec<usize>,

    /// For each summand, the source-value constraints on the control flow
    /// parameters: each `(position, value)` requires the state's value at
    /// `control_flow_parameters[position]` (offset by however the caller lays
    /// its state vectors out) to equal `value`.
    pub source_constraints: Vec<Vec<(usize, usize)>>,
}

impl ControlFlowAnalysis {
    /// Runs the control flow graph analysis over `summands`, whose guards and
    /// assignments are expressed in terms of `parameters`.
    ///
    /// `live` marks which summands actually participate in the reachable
    /// automaton: only a live summand's write behaviour can disqualify a
    /// parameter from being a control flow parameter (see
    /// [`is_control_flow_parameter`]), so that a summand belonging to dead code
    /// cannot spuriously defeat the analysis merely by existing. A caller with no
    /// such notion (every summand is live) can pass `|_| true`.
    ///
    /// `context` rewrites a term to normal form (and, for callers that seed it
    /// with a substitution, resolves that too) before it is tested for
    /// closedness. `intern` interns a normalised source value into the caller's
    /// own value mapping, returning the index that its state vectors use for
    /// it, so [`ControlFlowAnalysis::source_constraints`] can be compared
    /// directly against explored states.
    pub fn new<S: CfgSummand>(
        parameters: &[DataVariable],
        summands: &[S],
        live: impl Fn(&S) -> bool,
        context: &LearnSuccessorsContext,
        mut intern: impl FnMut(&DataExpressionRef) -> usize,
    ) -> Self {
        // Analyse every summand once: which parameters it constrains to a source
        // value and which parameters it changes (and whether to a constant).
        let analyses: Vec<SummandAnalysis> = summands
            .iter()
            .map(|summand| analyse_summand(summand, parameters, context))
            .collect();
        let liveness: Vec<bool> = summands.iter().map(&live).collect();

        // A parameter is a control flow parameter iff every live summand treats
        // it as such, and at least one live summand actually constrains it.
        let control_flow_parameters: Vec<usize> = (0..parameters.len())
            .filter(|&j| is_control_flow_parameter(j, &analyses, &liveness))
            .collect();

        // Intern the source values into the caller's value mapping so they can be
        // compared directly against the entries of explored state vectors.
        let source_constraints: Vec<Vec<(usize, usize)>> = analyses
            .iter()
            .map(|analysis| {
                control_flow_parameters
                    .iter()
                    .enumerate()
                    .filter_map(|(position, &j)| {
                        let term = analysis.source.get(&j)?;
                        let value: DataExpressionRef<'_> = term.copy().into();
                        Some((position, intern(&value)))
                    })
                    .collect()
            })
            .collect();

        log::debug!(
            "Control flow parameters: {:?}",
            control_flow_parameters
                .iter()
                .map(|&j| parameters[j].name().to_string())
                .collect::<Vec<_>>()
        );

        // Describe the identified control flow graph: per parameter the distinct
        // source values (the locations it can be in), and per summand the source
        // values it requires (the edges between those locations).
        if log::log_enabled!(log::Level::Debug) {
            for &j in &control_flow_parameters {
                let mut values: Vec<String> = analyses
                    .iter()
                    .filter_map(|analysis| analysis.source.get(&j))
                    .map(|value| value.to_string())
                    .collect();
                values.sort();
                values.dedup();
                log::debug!(
                    "Control flow graph for {}: locations {:?}",
                    parameters[j].name(),
                    values
                );
            }

            for (index, analysis) in analyses.iter().enumerate() {
                let constraints: Vec<String> = control_flow_parameters
                    .iter()
                    .filter_map(|&j| {
                        analysis
                            .source
                            .get(&j)
                            .map(|value| format!("{} == {}", parameters[j].name(), value))
                    })
                    .collect();
                if !constraints.is_empty() {
                    log::debug!("Summand {index} requires {constraints:?}");
                }
            }
        }

        Self {
            control_flow_parameters,
            source_constraints,
        }
    }
}

/// The result of analysing a single summand for the control flow graph.
struct SummandAnalysis {
    /// Maps a parameter index to its required source value (closed under
    /// `context`) when the summand's guard contains a conjunct `d == c`.
    source: HashMap<usize, ATerm>,

    /// Maps each parameter index changed by the summand to whether it is
    /// assigned a constant value (closed under `context`). Parameters absent
    /// from the map are left unchanged by the summand.
    changed: HashMap<usize, bool>,
}

/// Analyses a single summand, extracting its source values and changed
/// parameters with respect to `parameters`.
fn analyse_summand<S: CfgSummand>(
    summand: &S,
    parameters: &[DataVariable],
    context: &LearnSuccessorsContext,
) -> SummandAnalysis {
    // Collect the top-level conjuncts of the guard and look for `d == c`
    // constraints on the parameters.
    let mut conjuncts = Vec::new();
    collect_conjuncts(summand.condition(), &mut conjuncts);

    let mut source = HashMap::new();
    for conjunct in &conjuncts {
        if let Some((index, value)) = as_parameter_equality(conjunct, parameters, context) {
            // A parameter cannot consistently be constrained to two different
            // closed values in the same (satisfiable) guard, so the first wins.
            source.entry(index).or_insert(value);
        }
    }

    // `write_assignments` contains only non-identity assignments, so every
    // parameter that appears here is genuinely changed by the summand.
    let mut changed = HashMap::new();
    for assignment in summand.write_assignments().iter() {
        let lhs = DataVariable::from(assignment.arg(0).protect());
        if let Some(index) = parameters.iter().position(|param| *param == lhs) {
            let rhs = assignment.arg(1).protect();
            changed.insert(index, is_closed(context, &rhs));
        }
    }

    SummandAnalysis { source, changed }
}

/// Returns whether parameter `j` is a control flow parameter across all `live`
/// `analyses`, i.e. every live summand either leaves it unchanged or constrains
/// it to a source value and changes it only to a constant, and at least one live
/// summand constrains it.
///
/// A summand for which `live[index]` is `false` is skipped entirely: its write
/// behaviour cannot disqualify `j`, and its source constraint (if any) does not
/// count towards `j` being constrained anywhere. This keeps dead code (a summand
/// belonging to an equation the caller knows to be unreachable) from spuriously
/// defeating the analysis merely by existing.
fn is_control_flow_parameter(j: usize, analyses: &[SummandAnalysis], live: &[bool]) -> bool {
    let mut constrained_somewhere = false;

    for (analysis, &is_live) in analyses.iter().zip(live) {
        if !is_live {
            continue;
        }

        let has_source = analysis.source.contains_key(&j);
        constrained_somewhere |= has_source;

        if let Some(&changed_to_constant) = analysis.changed.get(&j) {
            // The summand changes the parameter. For it to remain a control flow
            // parameter the new value must be a constant and the summand must
            // also pin down the source value it transitions from.
            if !has_source || !changed_to_constant {
                return false;
            }
        }
    }

    constrained_somewhere
}

/// Collects the top-level conjuncts of `expr`, flattening nested `&&`.
fn collect_conjuncts(expr: &DataExpression, out: &mut Vec<DataExpression>) {
    if is_application(expr) && expr.data_function_symbol().name() == "&&" {
        for argument in expr.data_arguments() {
            collect_conjuncts(&DataExpression::new(argument.protect()), out);
        }
    } else {
        out.push(expr.clone());
    }
}

/// If `expr` is an equality `d == c` (in either order) between a parameter `d`
/// and a closed data expression `c`, returns the parameter index and the term
/// `c`. Closedness of `c` is tested under `context` (see [`is_closed`]).
fn as_parameter_equality(
    expr: &DataExpression,
    parameters: &[DataVariable],
    context: &LearnSuccessorsContext,
) -> Option<(usize, ATerm)> {
    if !is_application(expr) || expr.data_function_symbol().name() != "==" {
        return None;
    }

    let arguments: Vec<ATerm> = expr.data_arguments().map(|argument| argument.protect()).collect();
    if arguments.len() != 2 {
        return None;
    }

    match_parameter_constant(&arguments[0], &arguments[1], parameters, context)
        .or_else(|| match_parameter_constant(&arguments[1], &arguments[0], parameters, context))
}

/// Returns the parameter index and constant term when `variable` is a
/// parameter and `constant` is closed under `context` (resolves to a
/// variable-free value).
fn match_parameter_constant(
    variable: &ATerm,
    constant: &ATerm,
    parameters: &[DataVariable],
    context: &LearnSuccessorsContext,
) -> Option<(usize, ATerm)> {
    if !is_variable(variable) {
        return None;
    }

    let variable = DataVariable::from(variable.clone());
    let index = parameters.iter().position(|param| *param == variable)?;

    if is_closed(context, constant) {
        Some((index, constant.clone()))
    } else {
        None
    }
}

/// Returns whether `term`, rewritten under `context`'s substitution, is a
/// closed (variable-free) data expression.
///
/// A caller that seeds `context` with a substitution (as `merc_lps` does for
/// the `@rewr_var` placeholders `replace_constants_by_variables` introduces)
/// gets those resolved to the constants they stand for; a caller whose context
/// carries no substitution (as `merc_pbes` does — SRF conversion has no
/// equivalent preprocessing step) only gets `term` normalised.
fn is_closed(context: &LearnSuccessorsContext, term: &ATerm) -> bool {
    let expr: DataExpressionRef<'_> = term.copy().into();
    let rewritten = context.rewrite_under_sigma(&expr);
    let rewritten_ref: DataExpressionRef<'_> = rewritten.copy().into();
    free_variables_data_expression(&rewritten_ref).is_empty()
}
