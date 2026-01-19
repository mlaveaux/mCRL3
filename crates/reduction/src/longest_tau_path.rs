use merc_lts::LTS;

/// Computes the length of the longest path consisting solely of tau (hidden) transitions in the given LTS.
///
/// # Details
///
/// Assumes that the LTS does not contain any tau-cycles.
pub fn longest_tau_path(lts: &impl LTS) -> usize {
    let mut length = vec![0usize; lts.num_of_states()];

    loop {
        // For topologically sorted states, a single pass is sufficient, but this generalises to any order.
        let mut changed = false;

        for state in lts.iter_states() {
            for transition in lts
                .outgoing_transitions(state)
                .filter(|transition| lts.is_hidden_label(transition.label))
            {
                if length[transition.to] > length[state] {
                    changed = true;
                }

                length[state] = length[state].max(length[transition.to] + 1);
            }
        }

        if !changed {
            break;
        }
    }

    *length.iter().max().unwrap_or(&0)
}
