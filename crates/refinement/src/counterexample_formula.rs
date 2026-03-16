use merc_lts::TransitionLabel;
use merc_syntax::ActFrm;
use merc_syntax::Action;
use merc_syntax::ModalityOperator;
use merc_syntax::MultiAction;
use merc_syntax::RegFrm;
use merc_syntax::StateFrm;
use merc_syntax::StateFrmOp;

/// Represents a counter example.
pub enum CounterExample<L: TransitionLabel> {
    /// Represents a simple trace formula `<a0><a1>..<a_n>true`.
    Trace(Vec<L>),
    /// Represents a weak trace formula `<tau*><a0><tau*><a1>...<a_n>true`.
    WeakTrace(Vec<L>),
    /// Represents a stable failures formula `<tau*><a0><tau*><a1>...<a_n>(<enabled_0>true && ... <enabled_k>true).
    StableFailures(Vec<L>, Vec<L>),
    /// Represents an impossible futures formula `<tau*><a0><tau*><a1>...<a_n>(<future_0>false || ... <future_k>false)`.
    ImpossibleFutures(Vec<L>, Vec<Vec<L>>),
}

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
        CounterExample::StableFailures(trace, enabled) => {
            // The formula at the end.
            let inner = enabled.iter().map(|l| {
                StateFrm::Modality {
                    operator: ModalityOperator::Diamond,
                    formula: RegFrm::Action(ActFrm::MultAct(label_to_multi_action(l))),
                    expr: Box::new(StateFrm::True),
                }
            }).fold(StateFrm::True, |acc, expr| {
                StateFrm::Binary { op: StateFrmOp::Conjunction, lhs: Box::new(acc), rhs: Box::new(expr) }
            });

            weaktrace_formula(
                trace,
                inner,
                ModalityOperator::Diamond
            )
        },
        CounterExample::ImpossibleFutures(trace, futures) => {
            let expressions = futures.iter().map(|future| {
                weaktrace_formula(future, StateFrm::False, ModalityOperator::Box)
            }).collect::<Vec<_>>();

            // Generate a conjunction of the expressions for each future.
            let expr = expressions.into_iter().fold(StateFrm::True, |acc, expr| {
                StateFrm::Binary { op: StateFrmOp::Conjunction, lhs: Box::new(acc), rhs: Box::new(expr) }
            });

            weaktrace_formula(trace, expr, ModalityOperator::Diamond)
        },
    }
}

/// Generates a formula [tau* . label1 . tau* . label2 ...]expr that characterizes the weak trace counter example.
fn weaktrace_formula<L: TransitionLabel>(trace: &Vec<L>, expr: StateFrm, modality: ModalityOperator) -> StateFrm {
    // Build the formula tau*
    let tau_star = RegFrm::Iteration(Box::new(RegFrm::Action(ActFrm::MultAct(MultiAction::new(vec![])))));

    // We build the formula bottom up: tau* . label ...
    let mut result = expr;
    for label in trace.iter().rev() {
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