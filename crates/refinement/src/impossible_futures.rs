use merc_lts::LTS;
use merc_lts::StateIndex;

use crate::CounterExampleConstructor;
use crate::CounterExampleTree;
use crate::ExplorationStrategy;
use crate::is_refinement_generic;
use crate::is_stable;

/// Checks for the impossible futures refinement between the initial state of
/// the `lts` and the `initial_spec` state.
///
/// # Details
///
/// Impossible futures are defined in the following article:
///
/// > Marc Voorhoeve, Sjouke Mauw. Impossible futures and determinism, Inf. Process. Lett. 80, 2001.
pub fn is_impossible_futures_refinement<L: LTS, CE: CounterExampleTree>(
    lts: &L,
    initial_spec: StateIndex,
    strategy: ExplorationStrategy,
    counter_example: &mut CE,
) -> (bool, Option<CE::Index>, Option<Vec<Vec<L::Label>>>) {
    is_refinement_generic(
        strategy,
        lts,
        lts.initial_state_index(),
        initial_spec,
        |impl_state, spec_states| {
            if !is_stable(lts, impl_state) {
                // We can skip unstable states as an optimisation.
                debug_assert!(!diverges(lts, impl_state), "Implementation states should not diverge.");
                return None;
            }

            if !spec_states
                .iter()
                .any(|t| is_weak_trace_refinement(lts, *t, impl_state, strategy, &mut ()).0)
            {
                let mut futures = Vec::new();

                for t in spec_states {
                    // Run the weak trace refinement again with a counter example.
                    let mut ce_constructor = CounterExampleConstructor::new();

                    let (result, ce) = is_weak_trace_refinement(lts, *t, impl_state, strategy, &mut ce_constructor);
                    debug_assert!(
                        !result,
                        "The weak trace refinement should fail according to the previous check."
                    );

                    let trace = ce_constructor
                        .reconstruct_trace(ce.expect("A counter example was requested"))
                        .iter()
                        .map(|l| lts.labels()[*l].clone())
                        .collect();

                    futures.push(trace);
                }

                return Some(futures);
            }

            None
        },
        true,
        counter_example,
    )
}

/// Checks for the various weak trace refinement. This is a helper function for
/// the impossible futures refinement check.
fn is_weak_trace_refinement<L: LTS, CE: CounterExampleTree>(
    lts: &L,
    impl_state: StateIndex,
    spec_state: StateIndex,
    strategy: ExplorationStrategy,
    counter_example: &mut CE,
) -> (bool, Option<CE::Index>) {
    let (result, counter_example, inner_ce) = is_refinement_generic(
        strategy,
        lts,
        impl_state,
        spec_state,
        |_, _| Option::<()>::None,
        true,
        counter_example,
    );

    debug_assert!(inner_ce.is_none(), "The counter example from check is trivial");

    (result, counter_example)
}

/// Returns true iff the given state diverges, i.e., it can perform an infinite
/// sequence of tau transitions.
fn diverges<L: LTS>(lts: &L, state: StateIndex) -> bool {
    let mut visited = vec![false; lts.num_of_states()];
    let mut stack = vec![state];

    while let Some(current) = stack.pop() {
        if visited[current] {
            // We have found a tau loop, so the state diverges.
            return true;
        }

        visited[current] = true;

        for transition in lts.outgoing_transitions(current) {
            if lts.is_hidden_label(transition.label) {
                stack.push(transition.to);
            }
        }
    }

    // We have explored all reachable states via tau transitions without finding a loop, so the state does not diverge.
    false
}