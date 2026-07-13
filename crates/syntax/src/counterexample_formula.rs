use crate::ActFrm;
use crate::Action;
use crate::FixedPointOperator;
use crate::ModalityOperator;
use crate::MultiAction;
use crate::RegFrm;
use crate::Span;
use crate::StateFrm;
use crate::StateFrmOp;
use crate::StateVarDecl;
use merc_lts::TransitionLabel;
use merc_reduction::DistinguishingFormula;
use merc_refinement::CounterExample;

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
        CounterExample::Divergence(trace) => weaktrace_formula(
            trace,
            // For the divergence we use `nu X. <tau>X` to require an infinite tau path.
            StateFrm::FixedPoint {
                operator: FixedPointOperator::Greatest,
                variable: StateVarDecl {
                    identifier: "X".to_string(),
                    arguments: Vec::new(),
                    span: Span::default(),
                },
                body: Box::new(StateFrm::Modality {
                    operator: ModalityOperator::Diamond,
                    formula: RegFrm::Action(ActFrm::MultAct(MultiAction::tau())),
                    expr: Box::new(StateFrm::Id("X".to_string(), Vec::new())),
                }),
            },
            ModalityOperator::Diamond,
        ),
        CounterExample::StableFailures(trace, refusals) => {
            // Refused actions are characterized by box-false modalities.
            let inner = refusals
                .iter()
                .map(|l| StateFrm::Modality {
                    operator: ModalityOperator::Box,
                    formula: RegFrm::Action(ActFrm::MultAct(label_to_multi_action(l))),
                    expr: Box::new(StateFrm::False),
                })
                .fold(
                    // Stable failures are only observed in stable states, so tau is refused.
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

/// Generates a state formula AST for a minimal-depth strong-bisimulation
/// distinguishing formula (a counter-example produced by
/// [`merc_reduction::strong_bisim_distinguishing_formula`]).
///
/// The resulting formula holds in one of the two compared states but not the
/// other, witnessing that they are not strongly bisimilar.
pub fn generate_distinguishing_formula<L: TransitionLabel>(formula: &DistinguishingFormula<L>) -> StateFrm {
    distinguishing_to_statefrm(formula, false)
}

/// Converts a [`DistinguishingFormula`] to a [`StateFrm`], treating it as
/// negated when `negated` is set.
///
/// Negation is pushed inward through the modalities via the modal De Morgan
/// laws (`!<a>phi == [a]!phi`, dually for `[a]`, distributing over the
/// conjuncts), rather than emitted as a bare [`StateFrm::Unary`] negation:
/// consumers such as `merc_vpg`'s parity-game translation only accept
/// formulas without free negation.
fn distinguishing_to_statefrm<L: TransitionLabel>(formula: &DistinguishingFormula<L>, negated: bool) -> StateFrm {
    match formula {
        DistinguishingFormula::Negate(inner) => distinguishing_to_statefrm(inner, !negated),
        DistinguishingFormula::Diamond { label, conjuncts } => {
            let (operator, op, unit) = if negated {
                (ModalityOperator::Box, StateFrmOp::Disjunction, StateFrm::False)
            } else {
                (ModalityOperator::Diamond, StateFrmOp::Conjunction, StateFrm::True)
            };

            let expr = conjuncts
                .iter()
                .map(|conjunct| distinguishing_to_statefrm(conjunct, negated))
                .reduce(|lhs, rhs| StateFrm::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
                .unwrap_or(unit);

            StateFrm::Modality {
                operator,
                formula: RegFrm::Action(ActFrm::MultAct(label_to_multi_action(label))),
                expr: Box::new(expr),
            }
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
    let tau_star = RegFrm::Iteration(Box::new(RegFrm::Action(ActFrm::MultAct(MultiAction::tau()))));

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
