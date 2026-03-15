use merc_collections::{IndexedSet, SetIndex};
use merc_lts::{LTS, LtsBuilder, LtsMultiAction, StateIndex, TransitionLabel};
use merc_syntax::{CommExpr, MultiActionLabel};
use merc_utilities::{MercError, Timing};

/// Computes the parallel composition hide(allow(comm(L1 || ... || Ln))).
/// 
/// The `builder` is used to construct the resulting LTS, which can also be
/// stored immediately in a file.
pub fn combine_lts<L: LTS, B: LtsBuilder<L::Label>>(
    builder: &mut B,
    parallel_composition: Vec<L>,
    hide: &Vec<String>,
    allow: &Vec<MultiActionLabel>,
    comm: &Vec<CommExpr>,
    timing: &Timing,
) -> Result<(), MercError> {

    // Keep track of the discovered states in the combined LTS.
    let mut discovered: IndexedSet<Vec<StateIndex>> = IndexedSet::new();
    let (index, _) = discovered.insert(parallel_composition
        .iter()
        .map(|lts| lts.initial_state_index())
        .collect());

    // Working refers to the state vectors in discovered.
    let mut working: Vec<SetIndex> = vec![index];
    while let Some(current) = working.pop() {

        // Get the current state vector from discovered.
        let current_state_vector = discovered.get(current).unwrap();

        // For each LTS, get the outgoing transitions from the current state.
        for (state, lts) in current_state_vector.iter().zip(parallel_composition.iter()) {

            for transition in lts.outgoing_transitions(*state) {
                
            }
        }
    }

    Ok(())
}

/// Returns true iff the given action is allowed by the allow operator with the
/// given allow set.
/// 
/// # Details
/// 
/// Determines if every action label in the given multi-action is contained in
/// the allow set. If the allow set is empty, all actions are allowed. If action
/// is tau, it is always allowed.
fn allowed(allow: &Vec<MultiActionLabel>, action: &LtsMultiAction) -> bool {
    if allow.is_empty() {
        return true;
    }

    if action.is_tau_label() {
        return true;
    }

    for allowed in allow {
        // let allowed_labels = allowed.iter();
        // let action_labels = action.actions().iter().map(|label| label.label());

    }

    false
}
