//! Generic explicit-state space exploration for an [`LPS`].

use log::info;
use merc_io::TimeProgress;
use merc_lts::LtsBuilder;
use merc_lts::StateIndex;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::DiscoveredSet;
use crate::LPS;
use crate::StateRef;
use crate::Summand;

/// Explores the state space of `lps` and feeds the discovered transitions to
/// `builder`.
///
/// # Details
///
/// The `builder` is finalised before returning so the resulting LTS — whether
/// in memory or streamed to disk — can be obtained from the builder by the
/// caller.
pub fn explore<P, B>(builder: &mut B, lps: &P, timing: &Timing) -> Result<(), MercError>
where
    P: LPS,
    B: LtsBuilder<P::Label>,
{
    // Map from discovered state vectors to their `StateRef`. The discovered set
    // owns the compact (maximally shared) representation of every state vector.
    let mut discovered: DiscoveredSet<P::Value> = DiscoveredSet::new();
    let (initial_ref, _) = discovered.insert(&lps.initial_state());

    let progress = TimeProgress::new(
        |(states, transitions): (usize, usize)| {
            info!("Explored {states} states, {transitions} transitions...");
        },
        1,
    );

    let mut working: Vec<StateRef> = vec![initial_ref];
    // Reusable buffer holding the current state vector reconstructed from the
    // discovered set, avoiding an allocation per explored state.
    let mut current_state: Vec<P::Value> = Vec::new();
    timing.measure("explore", || -> Result<(), MercError> {
        while let Some(current) = working.pop() {
            // Reconstruct the state vector into the reusable buffer so
            // `discovered` can be mutated by inserts inside the summand callback
            // below.
            discovered.get_into(current, &mut current_state);
            let from = StateIndex::new(current.index());

            for summand in lps.summands() {
                summand.enumerate(&current_state, |label, next_state| {
                    // The discovered set deduplicates and only the `is_new` flag
                    // tells us whether the state vector still needs exploring.
                    let (target_ref, is_new) = discovered.insert(next_state);
                    let to = StateIndex::new(target_ref.index());
                    builder.add_transition(from, label, to)?;
                    if is_new {
                        working.push(target_ref);
                    }
                    Ok(())
                })?;
            }

            progress.print((discovered.len(), builder.num_of_transitions()));
        }

        Ok(())
    })?;

    info!(
        "Exploration complete: {} states, {} transitions",
        discovered.len(),
        builder.num_of_transitions()
    );

    builder.finish(StateIndex::new(initial_ref.index()))?;
    Ok(())
}
