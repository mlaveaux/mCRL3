use merc_lts::StateIndex;
use merc_lts::LTS;

/// Computes the length of the longest path consisting solely of tau (hidden) transitions in the given LTS.
pub fn longest_tau_path(lts: &impl LTS) -> usize {
    let mut result = 0;
    for state in lts.iter_states() {
        let length = longest_tau_path_state(lts, state);
        result = result.max(length);
    }
    result
}

/// Computes the length of the longest path consisting solely of tau (hidden) transitions from the given state.
pub fn longest_tau_path_state(lts: &impl LTS, start: StateIndex) -> usize {
    let mut max_length = 0;

    let mut stack = vec![(start, 0usize)];
    let mut visited = std::collections::HashSet::new();

    while let Some((state, length)) = stack.pop() {
        if !visited.insert(state) {
            continue; // Already visited this state in the current path
        }

        max_length = max_length.max(length);
        for transition in lts.outgoing_transitions(state) {
            if lts.is_hidden_label(transition.label) {
                stack.push((transition.to, length + 1));
            }
        }
        visited.remove(&state);
    }

    max_length
}
