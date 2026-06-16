//! Control flow graph analysis for linear process specifications.
//!
//! The analysis identifies the *control flow parameters* (CFPs) of an LPS and,
//! for every summand, the source values these parameters must have for the
//! summand's guard to hold. [`crate::cfg_lps`] uses the result to prune summands
//! before enumeration.

use std::collections::HashMap;

use log::debug;
use mcrl2::ATerm;
use mcrl2::DataExpression;
use mcrl2::DataExpressionRef;
use mcrl2::DataVariable;
use mcrl2::LearnSuccessorsContext;
use mcrl2::free_variables_data_expression;
use mcrl2::is_application;
use mcrl2::is_variable;
use merc_explore::LPS;

use crate::explore_explicit::ExplicitLinearProcessSpecification;
use crate::explore_explicit::ExplicitSummand;

/// The outcome of the control flow graph analysis of an LPS.
///
/// A process parameter `d` is a *control flow parameter* (CFP) when, in every
/// summand, it behaves like a program counter rather than a data variable:
///
/// - whenever a summand changes `d`, it assigns it a closed (constant) value, and
/// - whenever a summand reads `d` to decide its guard, it does so through a
///   conjunct `d == c` with `c` a closed value (its *source value*), and
/// - a summand that does not constrain `d` leaves it unchanged.
///
/// Under these conditions the value of a CFP in any reachable state is one of a
/// statically known set of constants, so a summand whose guard requires
/// `d == c` can only fire from states where `d` already equals `c`.
pub(crate) struct ControlFlowAnalysis {
    /// The process parameter indices identified as control flow parameters.
    pub(crate) control_flow_parameters: Vec<usize>,

    /// For each summand, the source-value constraints on the control flow
    /// parameters: each `(position, value)` requires the state's value at
    /// `control_flow_parameters[position]` to equal the interned `value`.
    pub(crate) source_constraints: Vec<Vec<(usize, usize)>>,
}

impl ControlFlowAnalysis {
    /// Runs the control flow graph analysis on top of an explicit LPS.
    ///
    /// The source values are interned into the same value mapping (and under the
    /// same normalisation) that `lps` uses for its state vectors, so the indices
    /// can be compared directly against state entries.
    pub(crate) fn new(lps: &ExplicitLinearProcessSpecification) -> Self {
        let parameters = lps.parameters();

        // Analyse every summand once: which parameters it constrains to a source
        // value and which parameters it changes (and whether to a constant).
        let analyses: Vec<SummandAnalysis> = lps
            .summands()
            .iter()
            .map(|summand| analyse_summand(summand, &parameters))
            .collect();

        // A parameter is a control flow parameter iff every summand treats it as
        // such, and at least one summand actually constrains it.
        let control_flow_parameters: Vec<usize> = (0..parameters.len())
            .filter(|&j| is_control_flow_parameter(j, &analyses))
            .collect();

        // Intern the source values into the LPS's value mapping so they can be
        // compared directly against the entries of explored state vectors.
        let context = LearnSuccessorsContext::new(lps.lps());
        let source_constraints: Vec<Vec<(usize, usize)>> = analyses
            .iter()
            .map(|analysis| {
                control_flow_parameters
                    .iter()
                    .enumerate()
                    .filter_map(|(position, &j)| {
                        let term = analysis.source.get(&j)?;
                        let value: DataExpressionRef<'_> = term.copy().into();
                        Some((position, lps.intern_normal_form(&context, &value)))
                    })
                    .collect()
            })
            .collect();

        debug!(
            "Control flow parameters: {:?}",
            control_flow_parameters
                .iter()
                .map(|&j| parameters[j].name().to_string())
                .collect::<Vec<_>>()
        );

        Self {
            control_flow_parameters,
            source_constraints,
        }
    }
}

/// The result of analysing a single summand for the control flow graph.
struct SummandAnalysis {
    /// Maps a parameter index to its required source value (a closed data
    /// expression) when the summand's guard contains a conjunct `d == c`.
    source: HashMap<usize, ATerm>,

    /// Maps each parameter index changed by the summand to whether it is
    /// assigned a closed (constant) value. Parameters absent from the map are
    /// left unchanged by the summand.
    changed: HashMap<usize, bool>,
}

/// Analyses a single summand, extracting its source values and changed
/// parameters with respect to the process `parameters`.
fn analyse_summand(summand: &ExplicitSummand, parameters: &[DataVariable]) -> SummandAnalysis {
    // Collect the top-level conjuncts of the guard and look for `d == c`
    // constraints on the process parameters.
    let mut conjuncts = Vec::new();
    collect_conjuncts(summand.condition(), &mut conjuncts);

    let mut source = HashMap::new();
    for conjunct in &conjuncts {
        if let Some((index, value)) = as_parameter_equality(conjunct, parameters) {
            // A parameter cannot consistently be constrained to two different
            // closed values in the same (satisfiable) guard, so the first wins.
            source.entry(index).or_insert(value);
        }
    }

    // The explicit summand stores only its non-identity assignments, so every
    // parameter that appears here is genuinely changed by the summand.
    let mut changed = HashMap::new();
    for assignment in summand.write_assignments().iter() {
        let lhs = DataVariable::from(assignment.arg(0).protect());
        if let Some(index) = parameters.iter().position(|param| *param == lhs) {
            let rhs: DataExpressionRef<'_> = assignment.arg(1).into();
            changed.insert(index, free_variables_data_expression(&rhs).is_empty());
        }
    }

    SummandAnalysis { source, changed }
}

/// Returns whether parameter `j` is a control flow parameter across all
/// `analyses`, i.e. every summand either leaves it unchanged or constrains it to
/// a source value and changes it only to a constant, and at least one summand
/// constrains it.
fn is_control_flow_parameter(j: usize, analyses: &[SummandAnalysis]) -> bool {
    let mut constrained_somewhere = false;

    for analysis in analyses {
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

/// If `expr` is an equality `d == c` (in either order) between a process
/// parameter `d` and a closed data expression `c`, returns the parameter index
/// and the constant term `c`.
fn as_parameter_equality(expr: &DataExpression, parameters: &[DataVariable]) -> Option<(usize, ATerm)> {
    if !is_application(expr) || expr.data_function_symbol().name() != "==" {
        return None;
    }

    let arguments: Vec<ATerm> = expr.data_arguments().map(|argument| argument.protect()).collect();
    if arguments.len() != 2 {
        return None;
    }

    match_parameter_constant(&arguments[0], &arguments[1], parameters)
        .or_else(|| match_parameter_constant(&arguments[1], &arguments[0], parameters))
}

/// Returns the parameter index and constant term when `variable` is a process
/// parameter and `constant` is closed (contains no free variables).
fn match_parameter_constant(variable: &ATerm, constant: &ATerm, parameters: &[DataVariable]) -> Option<(usize, ATerm)> {
    if !is_variable(variable) {
        return None;
    }

    let variable = DataVariable::from(variable.clone());
    let index = parameters.iter().position(|param| *param == variable)?;

    let constant_ref: DataExpressionRef<'_> = constant.copy().into();
    if free_variables_data_expression(&constant_ref).is_empty() {
        Some((index, constant.clone()))
    } else {
        None
    }
}
