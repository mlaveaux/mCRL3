use std::collections::VecDeque;

use log::info;

use merc_io::TimeProgress;
use merc_lts::LtsBuilder;
use merc_lts::StateIndex;
use merc_lts::TransitionLabel;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::DiscoveredSet;
use crate::LPS;
use crate::StateRef;
use crate::Summand;

/// Order in which discovered states are explored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Default)]
pub enum ExplorationStrategy {
    /// Depth-first exploration.
    #[default]
    Dfs,
    /// Breadth-first exploration.
    Bfs,
}

/// Explores the state space of `lps`, invoking caller-supplied closures for
/// each discovered state and transition.
///
/// # Closures
///
/// The mutable caller context `ctx` (typically a builder) is passed into
/// every closure invocation so the closures themselves can stay
/// non-capturing of the outer mutable state.
///
/// - `on_state(ctx, state_index, &state_info)` is called exactly once per
///   discovered state, in the order the state was popped from the working set,
///   after the implementation has been prepared for that state.
/// - `on_transition(ctx, from, &label, to)` is called for every transition
///   produced by the summands.
///
/// Returns the [`StateIndex`] assigned to the initial state of `lps`. The
/// caller is responsible for finalising any builder it owns.
pub fn explore<P, Ctx, OnState, OnTransition>(
    lps: &P,
    strategy: ExplorationStrategy,
    timing: &Timing,
    ctx: &mut Ctx,
    mut on_state: OnState,
    mut on_transition: OnTransition,
) -> Result<StateIndex, MercError>
where
    P: LPS,
    OnState: FnMut(&mut Ctx, StateIndex, &P::StateInfo) -> Result<(), MercError>,
    OnTransition: FnMut(&mut Ctx, StateIndex, &P::Label, StateIndex) -> Result<(), MercError>,
{
    let mut discovered: DiscoveredSet<P::Value> = DiscoveredSet::new();
    let (initial_ref, _) = discovered.insert(&lps.initial_state());

    let mut num_transitions = 0usize;
    let progress = TimeProgress::new(
        |(states, transitions): (usize, usize)| {
            info!("Explored {states} states, {transitions} transitions...");
        },
        1,
    );

    let mut working: VecDeque<StateRef> = VecDeque::from([initial_ref]);
    // Reusable buffer holding the current state vector reconstructed from the
    // discovered set, avoiding an allocation per explored state.
    let mut current_state: Vec<P::Value> = Vec::new();

    timing.measure("explore", || -> Result<(), MercError> {
        loop {
            let current = match strategy {
                ExplorationStrategy::Dfs => working.pop_back(),
                ExplorationStrategy::Bfs => working.pop_front(),
            };
            let Some(current) = current else { break };

            // Reconstruct the state vector into the reusable buffer so
            // `discovered` can be mutated by inserts inside the summand callback
            // below.
            let found = discovered.get_into(current, &mut current_state);
            debug_assert!(found, "StateRef from working queue must be valid");
            let from = StateIndex::new(current.index());
            lps.prepare(&current_state);

            let info = lps.state_info(&current_state);
            on_state(ctx, from, &info)?;

            for summand in lps.summands() {
                summand.enumerate(&current_state, |label, next_state| {
                    let (target_ref, is_new) = discovered.insert(next_state);
                    let to = StateIndex::new(target_ref.index());
                    on_transition(ctx, from, label, to)?;
                    num_transitions += 1;
                    if is_new {
                        working.push_back(target_ref);
                    }
                    Ok(())
                })?;
            }

            progress.print((discovered.len(), num_transitions));
        }

        Ok(())
    })?;

    info!(
        "Exploration complete: {} states, {} transitions",
        discovered.len(),
        num_transitions,
    );

    Ok(StateIndex::new(initial_ref.index()))
}

/// Explores the state space of `lps` and feeds the discovered transitions to
/// the [`LtsBuilder`] `builder`.
pub fn explore_to_lts<P, B>(
    builder: &mut B,
    lps: &P,
    strategy: ExplorationStrategy,
    timing: &Timing,
) -> Result<(), MercError>
where
    P: LPS,
    P::Label: TransitionLabel,
    B: LtsBuilder<P::Label>,
{
    let initial = explore(
        lps,
        strategy,
        timing,
        builder,
        |_, _, _| Ok(()),
        |b, from, label, to| b.add_transition(from, label, to),
    )?;
    builder.finish(initial)?;
    Ok(())
}
