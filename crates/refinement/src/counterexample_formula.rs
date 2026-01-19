use merc_lts::TransitionLabel;
use merc_syntax::ActFrm;
use merc_syntax::Action;
use merc_syntax::ModalityOperator;
use merc_syntax::MultiAction;
use merc_syntax::RegFrm;
use merc_syntax::StateFrm;

/// Represents a counter example.
pub enum CounterExample<L: TransitionLabel> {
    Trace(Vec<L>),
    WeakTrace(Vec<L>),
}

/// Generates a formula that characterizes the counter example trace.
pub fn generate_formula<L: TransitionLabel>(counter_example: &Vec<L>) -> StateFrm {
    let mut expr = StateFrm::True;

    // We build the formula bottom up.
    for label in counter_example.iter().rev() {
        expr = StateFrm::Modality {
            operator: ModalityOperator::Diamond,
            formula: RegFrm::Action(ActFrm::MultAct(MultiAction::new(vec![Action::new(
                label.to_string(),
                Vec::new(),
            )]))),
            expr: Box::new(expr),
        }
    }

    expr
}