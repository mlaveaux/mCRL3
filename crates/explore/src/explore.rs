//! Generic explicit-state space exploration for an [`LPS`].

use log::info;
use merc_collections::IndexedSet;
use merc_collections::SetIndex;
use merc_io::TimeProgress;
use merc_lts::LtsBuilder;
use merc_lts::StateIndex;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::LPS;
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
    // Map from discovered state vectors to their `StateIndex`.
    let mut discovered: IndexedSet<P::State> = IndexedSet::new();
    let (initial_index, _) = discovered.insert(lps.initial_state());

    let progress = TimeProgress::new(
        |(states, transitions): (usize, usize)| {
            info!("Explored {states} states, {transitions} transitions...");
        },
        1,
    );

    let mut working: Vec<SetIndex> = vec![initial_index];
    timing.measure("explore", || -> Result<(), MercError> {
        while let Some(current) = working.pop() {
            // Clone the state vector so `discovered` can be mutated by inserts
            // inside the summand callback below.
            let current_state = discovered
                .get(current)
                .expect("State must be in the discovered set")
                .clone();
            let from = StateIndex::new(*current);

            for summand in lps.summands() {
                summand.enumerate(&current_state, |label, next_state| {
                    // Only clone the next-state vector when it is genuinely
                    // new; for revisited states we just look up the index.
                    let (target_index, is_new) = match discovered.index(next_state) {
                        Some(idx) => (idx, false),
                        None => discovered.insert(next_state.clone()),
                    };
                    let to = StateIndex::new(*target_index);
                    builder.add_transition(from, label, to)?;
                    if is_new {
                        working.push(target_index);
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

    builder.finish(StateIndex::new(*initial_index))?;
    Ok(())
}
