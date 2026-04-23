use merc_lts::TransitionLabel;
use merc_refinement::CounterExample;
use crate::ActFrm;
use crate::Action;
use crate::ModalityOperator;
use crate::MultiAction;
use crate::RegFrm;
use crate::StateFrm;
use crate::StateFrmOp;

/// Generates a formula that characterizes the counter example trace.
pub fn generate_formula<L: TransitionLabel>(counter_example: &CounterExample<L>) -> StateFrm {
    match counter_example {
        CounterExample::Trace(trace) => {
            let mut expr = StateFrm::True;

            // We build the formula bottom up.
            for label in trace.iter().rev() {
                expr = StateFrm::Modality {
                    operator: ModalityOperator::Diamond,
                    formula: RegFrm::Action(ActFrm::MultAct(label_to_multi_action(label))),
                    expr: Box::new(expr),
                }
            }

            expr
        }
        CounterExample::WeakTrace(trace) => weaktrace_formula(trace, StateFrm::True, ModalityOperator::Diamond),
        CounterExample::StableFailures(trace, refusals) => {
            // Refused actions are characterized by box-false modalities.
            let inner = refusals
                .iter()
                .map(|l| {
                    StateFrm::Modality {
                        operator: ModalityOperator::Box,
                        formula: RegFrm::Action(ActFrm::MultAct(label_to_multi_action(l))),
                        expr: Box::new(StateFrm::False),
                    }
                    // Stable failures are only observed in stable states, so make this the base case.
                })
                .fold(
                    StateFrm::Modality {
                        operator: ModalityOperator::Box,
                        formula: RegFrm::Action(ActFrm::MultAct(MultiAction::tau())),
                        expr: Box::new(StateFrm::False),
                    },
                    |acc, expr| StateFrm::Binary {
                        op: StateFrmOp::Conjunction,
                        lhs: Box::new(acc),
                        rhs: Box::new(expr),
                    },
                );

            weaktrace_formula(trace, inner, ModalityOperator::Diamond)
        }
        CounterExample::ImpossibleFutures(trace, futures) => {
            let expressions = futures
                .iter()
                .map(|future| weaktrace_formula(future, StateFrm::False, ModalityOperator::Box))
                .collect::<Vec<_>>();

            // Generate a conjunction of the expressions for each future.
            let expr = expressions
                .into_iter()
                .fold(StateFrm::True, |acc, expr| StateFrm::Binary {
                    op: StateFrmOp::Conjunction,
                    lhs: Box::new(acc),
                    rhs: Box::new(expr),
                });

            weaktrace_formula(trace, expr, ModalityOperator::Diamond)
        }
    }
}

/// Generates a formula `[tau* . label1 . tau* . label2. ... . tau*]expr`, or
/// diamond based on `modality`, that characterizes the weak trace.
///
/// Note that every hidden label in the given `trace` is ignored, to ensure that
/// it is a valid weaktrace formula.
fn weaktrace_formula<L: TransitionLabel>(trace: &[L], expr: StateFrm, modality: ModalityOperator) -> StateFrm {
    // Build the formula tau*
    let tau_star = RegFrm::Iteration(Box::new(RegFrm::Action(ActFrm::MultAct(MultiAction::new(vec![])))));

    // We build the formula bottom up: tau* . label . ... . tau*
    let mut result = StateFrm::Modality {
        operator: modality,
        formula: tau_star.clone(),
        expr: Box::new(expr),
    };

    for label in trace.iter().rev().filter(|l| !l.is_tau_label()) {
        result = StateFrm::Modality {
            operator: modality,
            formula: tau_star.clone(),
            expr: Box::new(StateFrm::Modality {
                operator: modality,
                formula: RegFrm::Action(ActFrm::MultAct(label_to_multi_action(label))),
                expr: Box::new(result),
            }),
        }
    }

    result
}

/// Converts a label to a multi-action, where tau labels are converted to the empty multi-action.
fn label_to_multi_action<L: TransitionLabel>(label: &L) -> MultiAction {
    if label.is_tau_label() {
        MultiAction::tau()
    } else {
        MultiAction::new(vec![Action::new(label.to_string(), Vec::new())])
    }
}
