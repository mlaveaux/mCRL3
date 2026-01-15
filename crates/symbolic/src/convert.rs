use log::debug;
use log::info;
use merc_ldd::height;
use rustc_hash::FxBuildHasher;

use merc_collections::IndexedSet;
use merc_io::LargeFormatter;
use merc_io::TimeProgress;
use merc_ldd::Storage;
use merc_ldd::iterators::iter;
use merc_ldd::len;
use merc_lts::LabelledTransitionSystem;
use merc_lts::LtsBuilder;
use merc_lts::StateIndex;
use merc_utilities::MercError;

use crate::SymbolicLTS;
use crate::TransitionGroup;

/// Converts a symbolic LTS to an explicit LTS.
///
/// # Details
///
/// This basically applies the symbolic transitions to every state in the state
/// space, and constructs the explicit LTS.
pub fn convert_symbolic_lts(
    storage: &mut Storage,
    lts: &impl SymbolicLTS,
) -> Result<LabelledTransitionSystem<String>, MercError> {
    for group in lts.transition_groups() {
        if group.summand_count() > 1 {
            return Err("Cannot convert a symbolic LTS with non-trivial transition groups".into());
        }
        if group.action_label_index().is_none() {
            return Err("Cannot convert a symbolic LTS with transition groups without action labels".into());
        }
    }

    // Total number of states for progress reporting.
    let total_number_of_states = len(storage, lts.states());

    let mut builder = LtsBuilder::new(lts.action_labels().to_vec(), Vec::new());
    let mut discovered: IndexedSet<Vec<u32>, FxBuildHasher> = IndexedSet::new();
    discovered.insert(
        iter(storage, lts.initial_state())
            .next()
            .ok_or("The initial state should contain exactly one cube")?,
    );

    // Compute for every read and write index its position in the transition vector.
    let mut read_positions = Vec::new();
    let mut write_positions = Vec::new();
    for transition in lts.transition_groups() {
        // Concatenate all the indices, sort them, and find their position in the sorted positions
        let mut indices: Vec<u32> = transition
            .read_indices()
            .iter()
            .chain(transition.write_indices().iter())
            .cloned()
            .collect();
        indices.sort_unstable();
        indices.dedup();

        // The resulting vectors map from the index in the transition group to the position in the
        // transition vector.
        read_positions.push(
            transition
                .read_indices()
                .iter()
                .map(|i| indices.iter().position(|x| x == i).unwrap())
                .collect::<Vec<usize>>(),
        );
        write_positions.push(
            transition
                .write_indices()
                .iter()
                .map(|i| indices.iter().position(|x| x == i).unwrap())
                .collect::<Vec<usize>>(),
        );
    }

    for (group_index, group) in lts.transition_groups().iter().enumerate() {
        debug!("Transition group {}: {:?}", group_index, group,);

        debug!("  Read indices: {:?}", read_positions[group_index],);
        debug!("  Write indices: {:?}", write_positions[group_index],);
    }

    let progress = TimeProgress::new(
        move |(number_of_states, number_of_transitions)| {
            info!(
                "Explored {} states, and {} transitions ({}%)",
                LargeFormatter(number_of_states),
                LargeFormatter(number_of_transitions),
                number_of_states * 100 / total_number_of_states
            );
        },
        1,
    );

    info!(
        "Converting symbolic LTS to explicit LTS with {} states",
        LargeFormatter(total_number_of_states)
    );

    // TODO: We assume that the states are fully explored.

    let mut target = vec![0u32; height(storage, lts.states())];
    for state in iter(storage, lts.states()) {
        // Insert the state if necessary, this avoids cloning when already present.
        let state_index = if let Some(index) = discovered.index(&state) {
            index
        } else {
            discovered.insert(state.clone()).0
        };

        // Apply every transition group to this state.
        for (group_index, group) in lts.transition_groups().iter().enumerate() {
            'skip: for transition in iter(storage, group.relation()) {
                // Try to match the read parameters of this vector.
                for (index, i) in group.read_indices().iter().enumerate() {
                    if state[*i as usize] != transition[read_positions[group_index][index]] {
                        // If they do not match, skip this transition.
                        continue 'skip;
                    }
                }

                // Apply the transition writes to the state vector.
                target.clone_from_slice(&state);
                for (index, i) in group.write_indices().iter().enumerate() {
                    target[*i as usize] = transition[write_positions[group_index][index]];
                }

                // Find the action label.
                let label = &lts.action_labels()[transition[group
                    .action_label_index()
                    .ok_or("Transition vector should at least have the action label")?]
                    as usize];

                // Insert the target state if necessary, this avoids cloning when already present.
                let target_index = if let Some(index) = discovered.index(&target) {
                    index
                } else {
                    discovered.insert(target.clone()).0
                };

                builder.add_transition(StateIndex::new(*state_index), label, StateIndex::new(*target_index));
            }
        }

        progress.print((discovered.len(), builder.num_of_transitions()));
    }

    Ok(builder.finish(StateIndex::new(0)))
}
