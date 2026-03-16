use merc_lts::LTS;
use merc_lts::StateIndex;

use crate::CounterExampleConstructor;
use crate::CounterExampleTree;
use crate::ExplorationStrategy;
use crate::is_refinement_generic;
use crate::is_stable;

/// Checks for the various stable failures refinement relations.
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
            if is_stable(lts, impl_state) {
                // We can skip unstable states as an optimisation.
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

/// Checks for the various stable failures refinement relations.
fn is_weak_trace_refinement<L: LTS, CE: CounterExampleTree>(
    lts: &L,
    impl_state: StateIndex,
    spec_state: StateIndex,
    strategy: ExplorationStrategy,
    counter_example: &mut CE,
) -> (bool, Option<CE::Index>) {
    // The counter example from check is trivial.
    let (result, counter_example, _) = is_refinement_generic(
        strategy,
        lts,
        impl_state,
        spec_state,
        |_, _| Option::<()>::None,
        true,
        counter_example,
    );

    (result, counter_example)
}
