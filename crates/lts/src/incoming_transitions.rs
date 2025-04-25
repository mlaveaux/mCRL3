use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::StateIndex;

/// A struct containg information related to the incoming transitions for every
/// state.
pub struct IncomingTransitions {
    incoming_transitions: Vec<(LabelIndex, StateIndex)>,
    state2incoming: Vec<TransitionIndex>,
}

#[derive(Default, Clone)]
struct TransitionIndex {
    start: usize,
    end: usize,
    silent: usize,
}

impl IncomingTransitions {
    pub fn new(lts: &LabelledTransitionSystem) -> IncomingTransitions {
        let mut incoming_transitions: Vec<(LabelIndex, StateIndex)> = vec![(0, 0); lts.num_of_transitions()];
        let mut state2incoming: Vec<TransitionIndex> = vec![TransitionIndex::default(); lts.num_of_states()];

        // Compute the number of incoming (silent) transitions for each state.
        for state_index in lts.iter_states() {
            for (label_index, to) in lts.outgoing_transitions(state_index) {
                state2incoming[*to].end += 1;
                if lts.is_hidden_label(*label_index) {
                    state2incoming[*to].silent += 1;
                }
            }
        }

        // Fold the counts in state2incoming. Temporarily mixing up the data
        // structure such that after placing the transitions below the counts
        // will be correct.
        state2incoming.iter_mut().fold(0, |count, index| {
            let end = count + index.end;
            index.start = end - index.silent;
            index.end = end;
            index.silent = end;
            end
        });

        for state_index in lts.iter_states() {
            for (label_index, to) in lts.outgoing_transitions(state_index) {
                let index = &mut state2incoming[*to];

                if lts.is_hidden_label(*label_index) {
                    // Place at end of incoming transitions.
                    index.silent -= 1;
                    incoming_transitions[index.silent] = (*label_index, state_index);
                } else {
                    index.start -= 1;
                    incoming_transitions[index.start] = (*label_index, state_index);
                }
            }
        }

        IncomingTransitions {
            incoming_transitions,
            state2incoming,
        }
    }

    /// Returns an iterator over the incoming transitions for the given state.
    pub fn incoming_transitions(&self, state_index: usize) -> impl Iterator<Item = &(LabelIndex, StateIndex)> {
        self.incoming_transitions[self.state2incoming[state_index].start..self.state2incoming[state_index].end].iter()
    }

    // Return an iterator over the incoming silent transitions for the given state.
    pub fn incoming_silent_transitions(&self, state_index: usize) -> impl Iterator<Item = &(LabelIndex, StateIndex)> {
        self.incoming_transitions[self.state2incoming[state_index].silent..self.state2incoming[state_index].end].iter()
    }
}

#[cfg(test)]
mod tests {

    use mcrl3_utilities::{test_logger, test_rng};

    use crate::random_lts;

    use super::*;

    #[test]
    fn test_random_incoming_transitions() {
        let _ = test_logger();
        let mut rng = test_rng();
        
        let lts = random_lts(&mut rng, 10, 3, 3);
        let _ = IncomingTransitions::new(&lts);
    }
}
