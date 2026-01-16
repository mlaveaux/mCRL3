//! Authors: Jan Friso Groote, Maurice Laveaux, Wieger Wesselink and Tim A.C.
//! Willemse
//!
//! > M. Laveaux, J.F. Groote and T.A.C. Willemse. Correct and Efficient
//! > Antichain Algorithms for Refinement Checking. Logical Methods in Computer
//! > Science 17(1) 2021
//!
//! There are six algorithms. One for trace inclusion, one for failures
//! inclusion and one for failures-divergence inclusion. All algorithms come in
//! a variant with and without internal steps. It is possible to generate a
//! counter transition system in case the inclusion is answered by no.

use std::collections::HashSet;
use std::collections::VecDeque;

use log::trace;
use merc_collections::VecSet;
use merc_lts::StateIndex;
use merc_lts::LTS;
use merc_reduction::quotient_lts_block;
use merc_reduction::reduce_lts;
use merc_reduction::strong_bisim_sigref;
use merc_reduction::Equivalence;
use merc_reduction::Partition;
use merc_utilities::Timing;

use crate::Antichain;
use crate::CounterExampleConstructor;
use crate::CounterExampleTree;
use crate::RefinementType;

/// Sets the exploration strategy for the failures refinement algorithm.
pub enum ExplorationStrategy {
    BFS,
    DFS,
}

/// This function checks using algorithms in the paper mentioned above whether
/// transition system l1 is included in transition system l2.
///
/// # Details
///
/// The `refinement_type` determines (weak) trace inclusions, failures inclusion
/// and divergence failures inclusion etc.
///
/// The `strategy` parameter determines whether a breadth-first search
/// or depth-first search is used to explore the state space. Brreadth-first search
/// is often better suited for finding short counter examples, while depth-first
/// search often uses less memory.
///
/// The `preprocess` flag indicates whether preprocessing should be applied to
/// the LTSs. The refinement checks often involve product constructions, which
/// reducing the state space beforehand can lead to significant performance
/// improvements. However, for quick failing checks the preprocessing could cause
/// unnecessary overhead.
pub fn is_failures_refinement<L: LTS>(
    impl_lts: L,
    spec_lts: L,
    refinement: RefinementType,
    strategy: ExplorationStrategy,
    preprocess: bool,
    counter_example: bool,
    timing: &mut Timing,
) -> (bool, Option<Vec<L::Label>>) {
    let reduction = match refinement {
        RefinementType::Trace => Equivalence::StrongBisim,
        RefinementType::Weaktrace => Equivalence::BranchingBisim,
    };

    // For the preprocessing/quotienting step it makes sense to merge both LTSs
    // together in case that some states are equivalent. So we do this in all branches.
    let (merged_lts, initial_spec) = if preprocess {
        if counter_example {
            // If a counter example is to be generated, we only reduce the
            // specification LTS such that the resulting counter example remains valid.
            let reduced_spec = reduce_lts(spec_lts, reduction, timing);
            impl_lts.merge_disjoint(&reduced_spec)
        } else {
            let (merged_lts, initial_spec) = impl_lts.merge_disjoint(&spec_lts);

            // Reduce all states in the merged LTS.
            match reduction {
                Equivalence::StrongBisim => {
                    let (preprocess_lts, partition) = strong_bisim_sigref(merged_lts, timing);

                    let initial_spec = partition.block_number(initial_spec);
                    let reduced_lts = quotient_lts_block::<_, false>(&preprocess_lts, &partition);

                    // After partitioning the block becomes the state in the reduced_lts.
                    (reduced_lts, StateIndex::new(*initial_spec))
                }
                _ => unimplemented!(),
            }
        }
    } else {
        impl_lts.merge_disjoint(&spec_lts)
    };

    let mut refine_time = timing.start("refinement");
    let (result, ce) = if counter_example {
        // Construct a counter example tree, and return a trace.
        let mut ce_constructor = CounterExampleConstructor::new();
        let (result, state) =
            is_refinement_internal(strategy, refinement, &merged_lts, initial_spec, &mut ce_constructor);
        trace!("Counter example tree: {:?}", ce_constructor);

        if let Some(state) = state {
            // Reconstruct a trace from the counter example tree, relabelling the indices to their actual labels.
            (
                result,
                Some(
                    ce_constructor
                        .reconstruct_trace(state)
                        .iter()
                        .map(|l| merged_lts.labels()[*l].clone())
                        .collect(),
                ),
            )
        } else {
            (result, None)
        }
    } else {
        // Run without constructing a counter example.
        let (result, _) = is_refinement_internal::<_, ()>(strategy, refinement, &merged_lts, initial_spec, &mut ());
        (result, None)
    };

    refine_time.finish();
    (result, ce)
}

/// The inner loop for checking refinement.
///
/// # Details
///
/// This is mostly used internally. The `CE` type parameter indicates the type
/// of counter example tree that is used to construct counter examples. If no
/// counter examples are required, this can be set to `()`. Avoiding the cost
/// for keeping track of counter example information.
fn is_refinement_internal<L: LTS, CE: CounterExampleTree>(
    strategy: ExplorationStrategy,
    refinement: RefinementType,
    merged_lts: &L,
    initial_spec: StateIndex,
    counter_example: &mut CE,
) -> (bool, Option<CE::Index>) {
    let mut working = VecDeque::from([(
        merged_lts.initial_state_index(),
        VecSet::singleton(initial_spec),
        counter_example.root_index(),
    )]);

    // The antichain data structure is used for storing explored states. However, as opposed to a discovered set it
    // allows for pruning additional pairs based on the `antichain` property.
    let mut antichain = Antichain::new();

    // Cache used for the tau-closure computations.
    let mut closure_cache = ClosureCache::new();

    while let Some((impl_state, spec, ce)) = working.pop_front() {
        trace!("Checking ({:?}, {:?})", impl_state, spec);
        // pop (impl,spec) from working;

        // for all impl-e->impl' do
        for impl_transition in merged_lts.outgoing_transitions(impl_state) {
            let new_edge = counter_example.add_edge(impl_transition.label, ce);

            let spec_prime =
                if refinement == RefinementType::Weaktrace && merged_lts.is_hidden_label(impl_transition.label) {
                    // spec' := spec if e == tau
                    spec.clone()
                } else {
                    // spec' := {s' | exists s in spec. s-e->s'};
                    let mut spec_prime = VecSet::new();

                    if refinement == RefinementType::Weaktrace {
                        // For weak trace refinement we need to consider tau-closures s -->* s1 -e-> s2 -*-> s'
                        let closure = tau_closure(merged_lts, spec.clone().to_vec(), true, &mut closure_cache);

                        for s in &closure {
                            for spec_transition in merged_lts.outgoing_transitions(*s) {
                                if impl_transition.label == spec_transition.label {
                                    spec_prime.insert(spec_transition.to);
                                }
                            }
                        }

                        spec_prime =
                            VecSet::from_vec(tau_closure(merged_lts, spec_prime.to_vec(), false, &mut closure_cache));
                    } else {
                        // Otherwise, simply consider direct transitions.
                        for s in &spec {
                            for spec_transition in merged_lts.outgoing_transitions(*s) {
                                if impl_transition.label == spec_transition.label {
                                    spec_prime.insert(spec_transition.to);
                                }
                            }
                        }
                    }

                    spec_prime
                };

            trace!("spec' = {:?}", spec_prime);
            if spec_prime.is_empty() {
                // if spec' = {} then
                return (false, Some(new_edge));
            }

            if antichain.insert(impl_transition.to, spec_prime.clone()) {
                // if antichain_insert(impl,spec') then
                match strategy {
                    ExplorationStrategy::BFS => working.push_back((impl_transition.to, spec_prime, new_edge)),
                    ExplorationStrategy::DFS => working.push_front((impl_transition.to, spec_prime, new_edge)),
                }
            }
        }
    }

    (true, None)
}

/// A cache that is used to reuse allocations during tau-closure computations.
pub struct ClosureCache {
    /// States that are still to be explored.
    working: Vec<StateIndex>,

    /// Set of already visited states.
    visited: HashSet<StateIndex>,
}

impl ClosureCache {
    /// Creates a new closure cache.
    pub fn new() -> Self {
        ClosureCache {
            working: Vec::new(),
            visited: HashSet::new(),
        }
    }
}

/// Returns the tau closure for a set of states in the given LTS.
///
/// # Details
///
/// The `states` parameter indicates the initial set of states for which the
/// tau-closure is to be computed. The `extend` parameter indicates whether the
/// closure should include the original states as well. The `cache` parameter is
/// used to avoid repeated allocations.
fn tau_closure<L: LTS>(
    lts: &L,
    mut states: Vec<StateIndex>,
    extend: bool,
    cache: &mut ClosureCache,
) -> Vec<StateIndex> {
    debug_assert_eq!(cache.working.len(), 0, "Closure cache not cleared before use.");

    if extend {
        cache.working.extend(states.iter().cloned());
    } else {
        // Leaves the states empty.
        cache.working.append(&mut states);
    }

    // Keep track of states that are already in the closure.
    for s in &states {
        cache.visited.insert(*s);
    }

    while let Some(s) = cache.working.pop() {
        for t in lts.outgoing_transitions(s) {
            if lts.is_hidden_label(t.label) && !cache.visited.contains(&t.to) {
                cache.working.push(t.to);
                states.push(t.to);
            }
        }
    }

    states
}

#[cfg(test)]
mod tests {
    use merc_io::DumpFiles;
    use merc_lts::random_lts;
    use merc_lts::TransitionLabel;
    use merc_lts::write_aut;
    use merc_syntax::ActFrm;
    use merc_syntax::Action;
    use merc_syntax::ModalityOperator;
    use merc_syntax::MultiAction;
    use merc_syntax::RegFrm;
    use merc_syntax::StateFrm;
    use merc_utilities::random_test;
    use merc_utilities::Timing;
    use merc_vpg::solve_zielonka;
    use merc_vpg::translate;

    use crate::is_failures_refinement;
    use crate::ExplorationStrategy;
    use crate::RefinementType;

    /// Generates a formula that characterises the counter example trace.
    fn generate_formula<L: TransitionLabel>(counter_example: &Vec<L>) -> StateFrm {
        let mut expr = StateFrm::True;

        for label in counter_example {
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

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_trace_refinement() {
        random_test(100, |rng| {
            let mut files = DumpFiles::new("test_random_trace_refinement");

            let spec_lts = random_lts(rng, 10, 10, 3);
            let impl_lts = random_lts(rng, 10, 10, 3);

            files.dump("spec.aut", |w| write_aut(w, &spec_lts)).unwrap();
            files.dump("impl.aut", |w| write_aut(w, &impl_lts)).unwrap();

            for preprocess in [false, true] {
                let mut timing = Timing::default();
                let (result, counter_example) = is_failures_refinement(
                    impl_lts.clone(),
                    spec_lts.clone(),
                    RefinementType::Trace,
                    ExplorationStrategy::BFS,
                    preprocess,
                    true,
                    &mut timing,
                );

                if !result {
                    if let Some(ce) = counter_example {
                        let formula = generate_formula(&ce);
                        println!("Counter example formula: {}", formula);

                        let impl_pg = translate(&impl_lts, &formula).unwrap();
                        let spec_pg = translate(&spec_lts, &formula).unwrap();

                        let (impl_solution, _) = solve_zielonka(&impl_pg);
                        let (spec_solution, _) = solve_zielonka(&spec_pg);

                        assert!(impl_solution[0] != spec_solution[0], "Refinement returned false, but the counter example is not distinguishing.");
                    } else {
                        panic!("Expected a counter example.");
                    }
                }
            }
        });
    }
}
