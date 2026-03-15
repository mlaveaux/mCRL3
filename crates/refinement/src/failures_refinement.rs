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

use itertools::Itertools;
use log::trace;

use merc_collections::VecSet;
use merc_lts::LabelIndex;
use merc_lts::StateIndex;
use merc_lts::LTS;

use crate::Antichain;
use crate::CounterExampleTree;
use crate::ExplorationStrategy;
use crate::RefinementType;

/// Checks for the various stable failures refinement relations.
pub fn is_failures_refinement<L: LTS, CE: CounterExampleTree>(
    lts: &L,
    initial_spec: StateIndex,
    refinement: RefinementType,
    strategy: ExplorationStrategy,
    counter_example: &mut CE,
) -> (bool, Option<CE::Index>, Option<LabelIndex>) {
    match refinement {
        RefinementType::Trace => is_refinement_generic(
            strategy,
            lts,
            lts.initial_state_index(),
            initial_spec,
            |_, _| None,
            false,
            counter_example,
        ),
        RefinementType::Weaktrace => is_refinement_generic(
            strategy,
            lts,
            lts.initial_state_index(),
            initial_spec,
            |_, _| None,
            true,
            counter_example,
        ),
        RefinementType::StableFailures => is_refinement_generic(
            strategy,
            lts,
            lts.initial_state_index(),
            initial_spec,
            |impl_state, spec_states| refusals_contained_in(lts, impl_state, spec_states),
            true,
            counter_example,
        ),
        _ => unreachable!("This refinement variant {refinement:?} can not be checked by is_failures_refinement"),
    }
}

/// A generic implementation for antichain based refinement algorithm.
///
/// # Details
///
/// Given the (initial_impl, initial_spec) pair the algorithm explores the
/// product state space of the implementation and the normalised specification
/// LTSs.
///
/// If `weak_transition` is true then the algorithm considers weak transitions
/// for the specification LTS, otherwise it only considers the immediate
/// transitions.
///
/// The `check` function is applied to every pair (impl, spec) that is explored
/// during the algorithm, it should return `None` if the pair is valid, and
/// `Some(counter_example)` if the pair is invalid.
///
/// The `CE` type parameter indicates the type of counter example tree that is
/// used to construct counter examples. If no counter examples are required,
/// this can be set to `()`. Avoiding the cost for keeping track of counter
/// example information.
pub fn is_refinement_generic<L: LTS, CE: CounterExampleTree, F, CC>(
    strategy: ExplorationStrategy,
    merged_lts: &L,
    initial_impl: StateIndex,
    initial_spec: StateIndex,
    check: F,
    weak_transition: bool,
    counter_example: &mut CE,
) -> (bool, Option<CE::Index>, Option<CC>)
where
    F: Fn(StateIndex, &VecSet<StateIndex>) -> Option<CC>,
{
    // The antichain data structure is used for storing explored states. However, as opposed to a discovered set it
    // allows for pruning additional pairs based on the `antichain` property.
    let mut antichain = Antichain::new();

    // A local cache used for the tau closure computations.
    let mut closure_cache = ClosureCache::new();

    let mut working = VecDeque::from([(
        initial_impl,
        if weak_transition {
            VecSet::from_vec(tau_closure(merged_lts, vec![initial_spec], &mut closure_cache))
        } else {
            VecSet::singleton(initial_spec)
        },
        counter_example.root_index(),
    )]);

    while let Some((impl_state, spec, ce)) = working.pop_front() {
        trace!("Checking ({:?}, {:?})", impl_state, spec);
        // pop (impl,spec) from working;

        if let Some(counter_example) = check(impl_state, &spec) {
            // if not check(impl,spec) then
            return (false, Some(ce), Some(counter_example));
        }

        // for all impl-e->impl' do
        for impl_transition in merged_lts.outgoing_transitions(impl_state) {
            let new_edge = counter_example.add_edge(impl_transition.label, ce);

            let spec_prime = if weak_transition && merged_lts.is_hidden_label(impl_transition.label) {
                // spec' := spec if e == tau
                spec.clone()
            } else {
                // spec' := {s' | exists s in spec. s-e->s'};
                let mut spec_prime = VecSet::new();

                if weak_transition {
                    // For weak trace refinement we need to consider tau-closures s -->* s1 -e-> s2 -*-> s'
                    let closure = tau_closure(merged_lts, spec.clone().to_vec(), &mut closure_cache);

                    for s in &closure {
                        for spec_transition in merged_lts.outgoing_transitions(*s) {
                            if impl_transition.label == spec_transition.label {
                                spec_prime.insert(spec_transition.to);
                            }
                        }
                    }

                    spec_prime =
                        VecSet::from_vec(tau_closure(merged_lts, spec_prime.to_vec(), &mut closure_cache));
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
                return (false, Some(new_edge), None);
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

    (true, None, None)
}

/// This function checks that the refusals(impl) are contained in the refusals
/// of spec, it returns Some(action) iff the inclusion fails.
/// 
/// See [refusals_contained_in_naive] for its definition.
fn refusals_contained_in<L: LTS>(
    lts: &L,
    impl_state: StateIndex,
    spec_states: &VecSet<StateIndex>,
) -> Option<LabelIndex> {
    if !is_stable(lts, impl_state) {
        // If the implementation state is not stable, then it cannot have any refusals.
        return None;
    }

    // refusals(impl) ⊆ refusals(spec) iff there exists a stable spec state s with enabled(s) ⊆ enabled(impl).
    // Violation: no such witness exists.
    let mut violation_label = None;
    'outer: for s in spec_states.iter() {
        if !is_stable(lts, *s) {
            // Unstable spec states do not contribute to the refusals of the specification, so we can ignore them.
            continue;
        }

        for transition_spec in lts.outgoing_transitions(*s) {
            if lts
                .outgoing_transitions(impl_state)
                .any(|transition_impl| transition_impl.label == transition_spec.label)
            {
                // Label is enabled in spec, but also in impl, so it cannot be part of the refusal set of impl.
                continue;
            }

            // s has an action impl cannot do, so s is not a witness (enabled(s) ⊄ enabled(impl)).
            violation_label = Some(transition_spec.label);
            continue 'outer;
        }

        // All of s's enabled actions are also enabled in impl: s is a witness.
        // Therefore refusals(impl) ⊆ refusals(s) ⊆ refusals_set(spec).
        debug_assert!(refusals_contained_in_naive(lts, impl_state, spec_states));
        return None;
    }

    if let Some(label) = violation_label {
        // No witness found: the refusal set of impl is not contained in the refusal set of spec.
        debug_assert!(!refusals_contained_in_naive(lts, impl_state, spec_states));
        return Some(label);
    }

    debug_assert!(refusals_contained_in_naive(lts, impl_state, spec_states));
    None
}

/// A naive implementation for checking that the refusals of an implementation state are contained in the refusals of a set of specification states.
fn refusals_contained_in_naive<L: LTS>(lts: &L, impl_state: StateIndex, spec_states: &VecSet<StateIndex>) -> bool {
    if !is_stable(lts, impl_state) {
        // If the implementation state is not stable, then it cannot have any refusals.
        return true;
    }

    let impl_refusals = refusals(lts, impl_state);
    let spec_refusals = refusals_set(lts, spec_states);
    impl_refusals.is_subset(&spec_refusals)
}

/// Naive implementation for the refusals of a set of states spec:
/// 
/// > refusals(spec) = { r | exists s in spec. r in refusals(s) and stable(r) }
fn refusals_set<L: LTS>(lts: &L, spec_states: &VecSet<StateIndex>) -> VecSet<VecSet<LabelIndex>> {
    let mut result = VecSet::new();

    for s in spec_states.iter() {
        if is_stable(lts, *s) {
            result.extend(refusals(lts, *s).iter());
        }
    }

    result
}

/// Naive implementation of refusals of a state s:
///
/// A state s is stable, denoted by stable(s) iff tau \not\in enabled(s), and
/// refusals are defined for stable states s by:
///
/// > refusals(s) = { r | r \subseteq (Act \setminus enabled(s)) }.
fn refusals<L: LTS>(lts: &L, state: StateIndex) -> VecSet<VecSet<LabelIndex>> {
    if !is_stable(lts, state) {
        return VecSet::new();
    }

    // The set of actions enabled in the given state.
    let enabled_labels: VecSet<LabelIndex> = VecSet::from_vec(
        lts.outgoing_transitions(state)
            .map(|t| t.label)
            .collect(),
    );

    // The refusal set of a stable state includes all subsets of the set of labels that are not enabled.
    let all_labels: VecSet<LabelIndex> = VecSet::from_vec(
        lts.labels()
            .iter()
            .enumerate()
            .filter(|(i, _)| !lts.is_hidden_label(LabelIndex::new(*i)))
            .map(|(i, _)| LabelIndex::new(i))
            .collect(),
    );

    VecSet::from_iter(
        all_labels
            .difference(&enabled_labels)
            .cloned()
            .powerset()
            .map(|subset| VecSet::from_iter(subset.into_iter())),
    )
}

/// Returns true iff the given state is stable, i.e., it has no outgoing tau transitions.
pub fn is_stable<L: LTS>(lts: &L, state: StateIndex) -> bool {
    lts.outgoing_transitions(state).all(|t| !lts.is_hidden_label(t.label))
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

impl Default for ClosureCache {
    fn default() -> Self {
        Self::new()
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
pub fn tau_closure<L: LTS>(
    lts: &L,
    mut states: Vec<StateIndex>,
    cache: &mut ClosureCache,
) -> Vec<StateIndex> {
    debug_assert!(cache.working.is_empty(), "Closure cache not cleared before use.");
    debug_assert!(cache.visited.is_empty(), "Closure cache not cleared before use.");

    // Initialize the working set with the initial states, note that states is
    // kept in tact. As such the original states are also returned.
    cache.working.extend(states.iter().cloned());

    // Keep track of states that are already in the closure.
    for s in &states {
        cache.visited.insert(*s);
    }

    while let Some(s) = cache.working.pop() {
        for t in lts.outgoing_transitions(s) {
            if lts.is_hidden_label(t.label) && !cache.visited.contains(&t.to) {
                cache.working.push(t.to);
                cache.visited.insert(t.to);
                states.push(t.to);
            }
        }
    }

    // Clear the cache for the next use, the working set is empty by now.
    cache.visited.clear();

    states
}

#[cfg(test)]
mod tests {
    use merc_io::DumpFiles;
    use merc_lts::LTS;
    use merc_lts::random_lts;
    use merc_lts::read_aut;
    use merc_lts::write_aut;
    use merc_utilities::random_test;
    use merc_utilities::Timing;
    use merc_utilities::test_logger;
    use merc_vpg::solve_zielonka;
    use merc_vpg::translate;
    use rand::rngs::StdRng;

    use crate::generate_formula;
    use crate::refines;
    use crate::ExplorationStrategy;
    use crate::RefinementType;

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_trace_refinement() {
        random_test(100, |rng| {
            is_refinment_test(
                "test_random_trace_refinement",
                rng,
                RefinementType::Trace,
                ExplorationStrategy::BFS,
                false,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_weak_trace_refinement() {
        random_test(100, |rng| {
            is_refinment_test(
                "test_random_weak_trace_refinement",
                rng,
                RefinementType::Weaktrace,
                ExplorationStrategy::BFS,
                false,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_stable_failures_refinement() {
        random_test(100, |rng| {
            is_refinment_test(
                "test_random_stable_failures_refinement",
                rng,
                RefinementType::StableFailures,
                ExplorationStrategy::BFS,
                false,
            );
        });
    }

    #[test]
    fn test_example_2_12() {
        let _ = test_logger();

        let s0 = r#"des(0, 6, 5)
            (0, "req", 1)
            (1, "i", 2)
            (2, "10", 3)
            (3, "10", 0)
            (1, "i", 5)
            (5, "20", 0)"#;

        let t0 = r#"des(0, 3, 2)
            (0, "req", 1)
            (1, "20", 2)"#;

        let u0 = r#"des(0, 4, 3)
            (0, "req", 1)
            (1, "i", 1)
            (1, "20", 2)
            (2, "i", 0)"#;

        let s0 = read_aut(s0.as_bytes(), Vec::new()).unwrap();
        let t0 = read_aut(t0.as_bytes(), Vec::new()).unwrap();
        let _u0 = read_aut(u0.as_bytes(), Vec::new()).unwrap();

        let mut timing = Timing::new();
        assert!(
            refines(
                t0.clone(),
                s0.clone(),
                RefinementType::Weaktrace,
                ExplorationStrategy::BFS,
                false,
                false,
                &mut timing
            )
            .0
        );
        assert!(
            !refines(
                t0,
                s0,
                RefinementType::StableFailures,
                ExplorationStrategy::BFS,
                false,
                false,
                &mut timing
            )
            .0
        );
    }

    /// Helper function to define a refinement test that can be instantiated for
    /// the various types.
    ///
    /// # Details
    ///
    /// Internally requests a counter example to be generated, and checks that
    /// the counter example is indeed a valid witness for the failure of the
    /// refinement check.
    fn is_refinment_test(
        dump_name: &str,
        rng: &mut StdRng,
        refinement: RefinementType,
        strategy: ExplorationStrategy,
        preprocess: bool,
    ) {
        let mut files = DumpFiles::new(dump_name);

        let spec_lts = random_lts(rng, 10, 10, 3);
        let impl_lts = random_lts(rng, 10, 10, 3);

        files.dump("spec.aut", |w| write_aut(w, &spec_lts)).unwrap();
        files.dump("impl.aut", |w| write_aut(w, &impl_lts)).unwrap();

        let mut timing = Timing::default();
        let (result, counter_example) = refines(
            impl_lts.clone(),
            spec_lts.clone(),
            refinement,
            strategy,
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

                assert!(
                    impl_solution[impl_lts.initial_state_index()] != spec_solution[spec_lts.initial_state_index()],
                    "Refinement returned false, but the counter example is not distinguishing."
                );
            } else {
                panic!("Expected a counter example.");
            }
        }
    }
}
