use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::StateIndex;

/// Stores the incoming transitions for a given labelled transition system.
pub struct IncomingTransitions {
    incoming_transitions: Vec<(LabelIndex, StateIndex)>,
    state2incoming: Vec<TransitionIndex>,
}

/// Stores the offsets at which the transitions for a state can be found.
///
/// The offsets [begin, end] contain all incoming transitions, and [begin, silent] contain only the silent transitions.
#[derive(Default, Clone)]
struct TransitionIndex {
    start: usize,
    end: usize,
}

impl IncomingTransitions {
    pub fn new(lts: &LabelledTransitionSystem) -> IncomingTransitions {
        let mut incoming_transitions = vec![(LabelIndex::new(0), StateIndex::new(0)); lts.num_of_transitions()];
        let mut state2incoming: Vec<TransitionIndex> = vec![TransitionIndex::default(); lts.num_of_states()];

        // Compute the number of incoming (silent) transitions for each state.
        for state_index in lts.iter_states() {
            for (_label_index, to) in lts.outgoing_transitions(state_index) {
                state2incoming[to.value()].end += 1;
            }
        }

        // Fold the counts in state2incoming. Temporarily mixing up the data
        // structure such that after placing the transitions below the counts
        // will be correct.
        state2incoming.iter_mut().fold(0, |count, index| {
            let end = count + index.end;
            index.start = end;
            index.end = end;
            end
        });

        for state_index in lts.iter_states() {
            for (label_index, to) in lts.outgoing_transitions(state_index) {
                let index = &mut state2incoming[to.value()];
                index.start -= 1;
                incoming_transitions[index.start] = (*label_index, state_index);
            }
        }

        for state_index in lts.iter_states() {
            // Sort the incoming transitions such that silent transitions come first.
            let slice = &mut incoming_transitions[state2incoming[state_index].start..state2incoming[state_index].end];
            slice.sort_unstable();
        }

        IncomingTransitions {
            incoming_transitions,
            state2incoming,
        }
    }

    /// Returns an iterator over the incoming transitions for the given state.
    pub fn incoming_transitions(&self, state_index: StateIndex) -> impl Iterator<Item = &(LabelIndex, StateIndex)> {
        self.incoming_transitions[self.state2incoming[state_index].start..self.state2incoming[state_index].end].iter()
    }

    // Return an iterator over the incoming silent transitions for the given state.
    pub fn incoming_silent_transitions(
        &self,
        state_index: StateIndex,
    ) -> impl Iterator<Item = &(LabelIndex, StateIndex)> {
        // Check for hidden label.
        self.incoming_transitions[self.state2incoming[state_index].start..self.state2incoming[state_index].end]
            .iter()
            .take_while(|(label, _)| *label == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mcrl3_utilities::random_test;

    use crate::random_lts;

    #[test]
    fn test_random_incoming_transitions() {
        random_test(100, |rng| {
            let lts = random_lts(rng, 10, 3, 3);
            let _ = IncomingTransitions::new(&lts);
        });
    }
}
