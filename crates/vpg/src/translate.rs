use std::ops::ControlFlow;

use log::debug;
use log::info;
use log::trace;

use log::warn;
use merc_collections::IndexedSet;
use merc_io::TimeProgress;
use merc_lts::LabelledTransitionSystem;
use merc_lts::StateIndex;
use merc_lts::LTS;
use merc_syntax::ActFrm;
use merc_syntax::ActFrmBinaryOp;
use merc_syntax::FixedPointOperator;
use merc_syntax::ModalityOperator;
use merc_syntax::MultiAction;
use merc_syntax::RegFrm;
use merc_syntax::StateFrm;
use merc_syntax::StateFrmOp;
use merc_syntax::StateVarDecl;
use merc_syntax::apply_statefrm;
use merc_syntax::visit_action_formula;
use merc_syntax::visit_regular_formula;
use merc_syntax::visit_statefrm;
use merc_utilities::MercError;

use crate::FreshStateVarGenerator;
use crate::compute_reachable;
use crate::ModalEquationSystem;
use crate::ParityGame;
use crate::Player;
use crate::Priority;
use crate::VertexIndex;

/// Translates a labelled transition system into a variability parity game.
pub fn translate(lts: &LabelledTransitionSystem<String>, formula: &StateFrm) -> Result<ParityGame, MercError> {
    // Parses all labels into MultiAction once
    let parsed_labels: Result<Vec<MultiAction>, MercError> =
        lts.labels().iter().map(|label| MultiAction::parse(label)).collect();
    let labels = parsed_labels?;

    // Warn about any labels that are used in the formula but do not correspond to any label in the LTS. 
    warn_unknown_action_labels(formula, &labels);

    let equation_system = ModalEquationSystem::new(formula);
    debug!("{}", equation_system);

    let mut algorithm: Translation<'_, _, ()> = Translation::new(lts, &labels, &equation_system);
    algorithm.translate(lts.initial_state_index(), 0, |_| ())?;

    // Construct the parity game from the collected vertices and edges, where the `()` edge label is ignored.
    let result = ParityGame::from_edges(
        VertexIndex::new(0),
        algorithm.vertices.iter().map(|(p, _)| p).cloned().collect(),
        algorithm.vertices.into_iter().map(|(_, pr)| pr).collect(),
        true,
        || algorithm.edges.iter().map(|(s, _, t)| (*s, *t)),
    );

    // Check that all vertices are reachable from the initial vertex. After
    // totality it could be that the true or false nodes are not reachable.
    if cfg!(debug_assertions) {
        let (_, reachable_vertices) = compute_reachable(&result);
        debug_assert!(
            reachable_vertices.iter().all(|v| v.is_some()),
            "Not all vertices are reachable from the initial vertex"
        );
    }

    Ok(result)
}

/// Produces a warning for each label that is used in the formula but does not correspond to any label in the LTS.
pub fn warn_unknown_action_labels(formula: &StateFrm, labels: &Vec<MultiAction>) {
    visit_statefrm::<(), _>(formula, |statefrm| {
        if let StateFrm::Modality { formula, .. } = statefrm {
            visit_regular_formula::<(), _>(formula, |regfrm| {

                if let RegFrm::Action(act_frm) = regfrm {
                    visit_action_formula::<(), _>(act_frm, |act_frm| {
                        if let ActFrm::MultAct(action) = act_frm {
                            if !labels.contains(action) {
                                warn!("Label {} in formula does not correspond to any label in the LTS", action);
                            }
                        }

                        Ok(ControlFlow::Continue(()))
                    })?;
                }

                Ok(ControlFlow::Continue(()))
            })?;           
        }

        Ok(ControlFlow::Continue(()))
    }).expect("Failed to visit state formula");
}

/// Translates regular formulas in modalities to fixpoint equations.
///
/// [a*]phi = nu X. [a]X && phi
/// <a*>phi = mu X. <a>X || phi
pub fn translate_regular_formulas(formula: StateFrm, identifier_generator: &mut FreshStateVarGenerator) -> StateFrm {
    apply_statefrm(formula, |subformula| {
        if let StateFrm::Modality {
            operator,
            formula,
            expr,
        } = subformula
        {
            return match formula {
                merc_syntax::RegFrm::Action(_action_frm) => Ok(None),
                merc_syntax::RegFrm::Iteration(reg_frm) => {
                    let iteration_var = identifier_generator.generate("I");
                    Ok(Some(StateFrm::FixedPoint {
                        operator: FixedPointOperator::Greatest,
                        variable: StateVarDecl::new(iteration_var.clone(), Vec::new()),
                        body: Box::new(StateFrm::Binary {
                            op: StateFrmOp::Conjunction,
                            lhs: Box::new(StateFrm::Modality {
                                operator: operator.clone(),
                                formula: *reg_frm.clone(),
                                expr: Box::new(StateFrm::Id(iteration_var, Vec::new())),
                            }),
                            rhs: expr.clone(),
                        }),
                    }))
                }
                merc_syntax::RegFrm::Plus(reg_frm) => todo!(),
                merc_syntax::RegFrm::Sequence { lhs, rhs } => todo!(),
                merc_syntax::RegFrm::Choice { lhs, rhs } => todo!(),
            }
        }

        Ok(None)
    })
    .expect("The apply does not fail")
}

/// Is used to distinguish between StateFrm and Equation vertices in the vertex map.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Formula<'a> {
    StateFrm(&'a StateFrm),
    Equation(usize),
}

/// Local struct to keep track of the translation state. The generic bound `E` is the
/// type of the edge labels.
///
/// Implements the translation from (s, Ψ) pairs to VPG vertices and edges.
/// However, to avoid the complication of merging sub-results we immediately
/// store the vertices and edges into mutable vectors. Furthermore, to avoid
/// stack overflows we use a breadth-first search approach with a queue. This
/// means that during queuing we immediately assign a fresh index to each (s, Ψ)
/// pair (if it does not yet exist) and then queue it to assign its actual
/// values later on.
pub(crate) struct Translation<'a, L, E> {
    vertex_map: IndexedSet<(StateIndex, Formula<'a>)>,
    vertices: Vec<(Player, Priority)>,
    edges: Vec<(VertexIndex, E, VertexIndex)>,

    // Used for the breadth first search.
    queue: Vec<(StateIndex, Formula<'a>, VertexIndex)>,

    /// The parsed labels of the LTS.
    parsed_labels: &'a Vec<MultiAction>,

    /// The labelled transition system being translated.
    lts: &'a L,

    /// A reference to the modal equation system being translated.
    equation_system: &'a ModalEquationSystem,

    /// Use to print progress information.
    progress: TimeProgress<usize>,
}

impl<'a, L: LTS, E> Translation<'a, L, E> {
    /// Creates a new translation instance.
    pub fn new(lts: &'a L, parsed_labels: &'a Vec<MultiAction>, equation_system: &'a ModalEquationSystem) -> Self {
        let progress: TimeProgress<usize> = TimeProgress::new(
            |num_of_vertices: usize| {
                info!("Translated {} vertices...", num_of_vertices);
            },
            1,
        );

        Self {
            vertex_map: IndexedSet::new(),
            vertices: Vec::new(),
            edges: Vec::new(),
            queue: Vec::new(),
            lts,
            parsed_labels,
            equation_system,
            progress,
        }
    }

    /// Perform the translation for the given `initial_state` and `initial_equation_index`.
    ///
    /// The `labelling` function is used to compute the edge label for the
    /// outgoing edges of this vertex. The argument is the transition
    /// corresponding to the modality, or None if there is no modality (e.g.,
    /// for conjunctions).
    pub fn translate<F>(
        &mut self,
        initial_state: StateIndex,
        initial_equation_index: usize,
        labelling: F,
    ) -> Result<(), MercError>
    where
        F: Fn(Option<Transition>) -> E,
    {
        // We store (state, formula, N) into the queue, where N is the vertex number assigned to this pair. This means
        // that during the traversal we can assume this N to exist.
        self.queue = vec![(
            initial_state,
            Formula::Equation(initial_equation_index),
            VertexIndex::new(0),
        )];
        self.vertices.push((Player::Odd, Priority::new(0))); // Placeholder for the initial vertex

        while let Some((s, formula, vertex_index)) = self.queue.pop() {
            debug!("Translating vertex {}: (s={}, formula={:?})", vertex_index, s, formula);
            self.progress.print(self.vertices.len());
            match formula {
                Formula::StateFrm(f) => {
                    self.translate_vertex(s, f, vertex_index, &labelling);
                }
                Formula::Equation(i) => {
                    self.translate_equation(s, i, vertex_index, &labelling);
                }
            }
        }

        Ok(())
    }

    /// Returns the collected vertices.
    pub fn vertices(&self) -> &Vec<(Player, Priority)> {
        &self.vertices
    }

    /// Returns the collected edges, where the edge label is ignored.
    pub fn edges(&self) -> &Vec<(VertexIndex, E, VertexIndex)> {
        &self.edges
    }

    /// Translate a single vertex (s, Ψ) into the variability parity game vertex
    /// and its outgoing edges.
    fn translate_vertex<F>(&mut self, s: StateIndex, formula: &'a StateFrm, vertex_index: VertexIndex, labelling: &F)
    where
        F: Fn(Option<Transition>) -> E,
    {
        match formula {
            StateFrm::True => {
                // (s, true) → odd, 0
                self.vertices[vertex_index] = (Player::Odd, Priority::new(0));
            }
            StateFrm::False => {
                // (s, false) → even, 0
                self.vertices[vertex_index] = (Player::Even, Priority::new(0));
            }
            StateFrm::Binary { op, lhs, rhs } => {
                match op {
                    StateFrmOp::Conjunction => {
                        // (s, Ψ_1 ∧ Ψ_2) →_P odd, (s, Ψ_1) and (s, Ψ_2), 0
                        self.vertices[vertex_index] = (Player::Odd, Priority::new(0));
                        let s_psi_1 = self.queue_vertex(s, Formula::StateFrm(lhs));
                        let s_psi_2 = self.queue_vertex(s, Formula::StateFrm(rhs));

                        self.edges.push((vertex_index, labelling(None), s_psi_1));
                        self.edges.push((vertex_index, labelling(None), s_psi_2));
                    }
                    StateFrmOp::Disjunction => {
                        // (s, Ψ_1 ∨ Ψ_2) →_P even, (s, Ψ_1) and (s, Ψ_2), 0
                        self.vertices[vertex_index] = (Player::Even, Priority::new(0));
                        let s_psi_1 = self.queue_vertex(s, Formula::StateFrm(lhs));
                        let s_psi_2 = self.queue_vertex(s, Formula::StateFrm(rhs));

                        self.edges.push((vertex_index, labelling(None), s_psi_1));
                        self.edges.push((vertex_index, labelling(None), s_psi_2));
                    }
                    _ => {
                        unimplemented!("Cannot translate binary operator in {}", formula);
                    }
                }
            }
            StateFrm::Id(identifier, _args) => {
                let (i, _equation) = self
                    .equation_system
                    .find_equation_by_identifier(identifier)
                    .expect("Variable must correspond to an equation");

                self.vertices[vertex_index] = (Player::Odd, Priority::new(0)); // The priority and owner do not matter here
                let equation_vertex = self.queue_vertex(s, Formula::Equation(i));
                self.edges.push((vertex_index, labelling(None), equation_vertex));
            }
            StateFrm::Modality {
                operator,
                formula,
                expr,
            } => {
                match operator {
                    ModalityOperator::Box => {
                        // (s, [a] Ψ) → odd, (s', Ψ) for all s' with s -a-> s', 0
                        self.vertices[vertex_index] = (Player::Odd, Priority::new(0));

                        for transition in self.lts.outgoing_transitions(s) {
                            let action = &self.parsed_labels[*transition.label];

                            trace!("Matching action {} against formula {}", action, formula);

                            if match_regular_formula(formula, action) {
                                let s_prime_psi = self.queue_vertex(transition.to, Formula::StateFrm(expr));

                                self.edges
                                    .push((vertex_index, labelling(Some(transition)), s_prime_psi));
                            }
                        }
                    }
                    ModalityOperator::Diamond => {
                        // (s, <a> Ψ) → even, (s', Ψ) for all s' with s -a-> s', 0
                        self.vertices[vertex_index] = (Player::Even, Priority::new(0));

                        for transition in self.lts.outgoing_transitions(s) {
                            let action = &self.parsed_labels[*transition.label];

                            if match_regular_formula(formula, action) {
                                let s_prime_psi = self.queue_vertex(transition.to, Formula::StateFrm(expr));

                                self.edges
                                    .push((vertex_index, labelling(Some(transition)), s_prime_psi));
                            }
                        }
                    }
                }
            }
            _ => {
                unimplemented!("Cannot translate formula {}", formula);
            }
        }
    }

    /// Applies the translation to the given (s, equation) vertex.
    fn translate_equation<F>(&mut self, s: StateIndex, equation_index: usize, vertex_index: VertexIndex, labelling: &F)
    where
        F: Fn(Option<Transition>) -> E,
    {
        let equation = self.equation_system.equation(equation_index);
        match equation.operator() {
            FixedPointOperator::Least => {
                // (s, μ X. Ψ) →_P odd, (s, Ψ[x := μ X. Ψ]), 2 * floor(AD(Ψ)/2) + 1. In Rust division is already floor.
                self.vertices[vertex_index] = (
                    Player::Odd,
                    Priority::new(2 * (self.equation_system.alternation_depth(equation_index) / 2) + 1),
                );
                let s_psi = self.queue_vertex(s, Formula::StateFrm(equation.body()));
                self.edges.push((vertex_index, labelling(None), s_psi));
            }
            FixedPointOperator::Greatest => {
                // (s, ν X. Ψ) →_P even, (s, Ψ[x := ν X. Ψ]), 2 * (AD(Ψ)/2). In Rust division is already floor.
                self.vertices[vertex_index] = (
                    Player::Even,
                    Priority::new(2 * (self.equation_system.alternation_depth(equation_index) / 2)),
                );
                let s_psi = self.queue_vertex(s, Formula::StateFrm(equation.body()));
                self.edges.push((vertex_index, labelling(None), s_psi));
            }
        }
    }

    /// Queues a new pair to be translated, returning its vertex index.
    fn queue_vertex(&mut self, s: StateIndex, formula: Formula<'a>) -> VertexIndex {
        let (index, inserted) = self.vertex_map.insert((s, formula.clone()));
        let vertex_index = VertexIndex::new(*index);

        if inserted {
            // New vertex, assign placeholder values
            self.vertices.resize(*vertex_index + 1, (Player::Odd, Priority::new(0)));
            self.queue.push((s, formula, vertex_index));
        }

        vertex_index
    }
}

/// Returns true iff the given action matches the regular formula.
fn match_regular_formula(formula: &RegFrm, action: &MultiAction) -> bool {
    match formula {
        RegFrm::Action(action_formula) => match_action_formula(action_formula, action),
        RegFrm::Choice { lhs, rhs } => match_regular_formula(lhs, action) || match_regular_formula(rhs, action),
        _ => {
            unimplemented!("Cannot translate regular formula {}", formula);
        }
    }
}

/// Returns true iff the given action matches the action formula.
fn match_action_formula(formula: &ActFrm, action: &MultiAction) -> bool {
    match formula {
        ActFrm::True => true,
        ActFrm::False => false,
        ActFrm::MultAct(expected_action) => expected_action == action,
        ActFrm::Binary { op, lhs, rhs } => match op {
            ActFrmBinaryOp::Union => match_action_formula(lhs, action) || match_action_formula(rhs, action),
            ActFrmBinaryOp::Intersect => match_action_formula(lhs, action) && match_action_formula(rhs, action),
            _ => {
                unimplemented!("Cannot translate binary operator {}", formula);
            }
        },
        ActFrm::Negation(expr) => !match_action_formula(expr, action),
        _ => {
            unimplemented!("Cannot translate action formula {}", formula);
        }
    }
}

#[cfg(test)]
mod tests {
    use merc_lts::read_aut;
    use merc_macros::merc_test;
    use merc_syntax::UntypedStateFrmSpec;

    use super::*;

    #[merc_test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_running_example() {
        let lts = read_aut(include_bytes!("../../../examples/lts/abp.aut") as &[u8], Vec::new()).unwrap();
        let formula = UntypedStateFrmSpec::parse(include_str!("../../../examples/vpg/running_example.mcf")).unwrap();

        let _pg = translate(&lts, &formula.formula).unwrap();
    }
}
