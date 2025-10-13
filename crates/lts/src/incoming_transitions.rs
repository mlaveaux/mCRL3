use mcrl3_utilities::bytevec;
use mcrl3_utilities::ByteCompressedVec;
use mcrl3_utilities::CompressedEntry;

use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::StateIndex;
use crate::Transition;

/// Stores the incoming transitions for a given labelled transition system.
pub struct IncomingTransitions {
    incoming_transitions: ByteCompressedVec<Transition>,
    state2incoming: ByteCompressedVec<TransitionIndex>,
}

/// Stores the offsets at which the transitions for a state can be found.
///
/// The offsets [begin, end] contain all incoming transitions, and [begin, silent] contain only the silent transitions.
#[derive(Default, Clone, Debug)]
struct TransitionIndex {
    offset: u8,
    start: usize,
    end: usize,
}

impl TransitionIndex {
    fn new(start: usize, end: usize) -> TransitionIndex {
        let offset = start.bytes_required() as u8;
        TransitionIndex { offset, start, end }
    }
}

impl IncomingTransitions {
    pub fn new(lts: &LabelledTransitionSystem) -> IncomingTransitions {
        let mut incoming_transitions = bytevec![Transition::new(LabelIndex::new(0), StateIndex::new(0)); lts.num_of_transitions()];
        let mut state2incoming = bytevec![TransitionIndex::default(); lts.num_of_states()];

        // Compute the number of incoming (silent) transitions for each state.
        for state_index in lts.iter_states() {
            for transition in lts.outgoing_transitions(state_index) {
                // Updating end does not require shifting the offset.
                state2incoming.update(transition.to.value(), |incoming| incoming.end += 1);
            }
        }

        // Fold the counts in state2incoming. Temporarily mixing up the data
        // structure such that after placing the transitions below the counts
        // will be correct.
        state2incoming.fold(0, |offset, incoming| {
            let new_offset = offset + incoming.end;
            *incoming = TransitionIndex::new(offset, offset);
            new_offset
        });

        for state_index in lts.iter_states() {
            for transition in lts.outgoing_transitions(state_index) {
                state2incoming.update(transition.to.value(), |incoming| {
                    incoming_transitions.set(incoming.end, Transition::new(transition.label, state_index));
                    incoming.end += 1;
                });
            }
        }

        for state_index in lts.iter_states() {
            // Sort the incoming transitions such that silent transitions come first.
            let state = state2incoming.index(state_index.value());
            incoming_transitions.sort_unstable_range(state.start, state.end);
        }

        IncomingTransitions {
            incoming_transitions,
            state2incoming,
        }
    }

    /// Returns an iterator over the incoming transitions for the given state.
    pub fn incoming_transitions(&self, state_index: StateIndex) -> impl Iterator<Item = Transition> {
        let state = self.state2incoming.index(state_index.value());
        self.incoming_transitions.iter_range(state.start,state.end)
    }

    // Return an iterator over the incoming silent transitions for the given state.
    pub fn incoming_silent_transitions(
        &self,
        state_index: StateIndex,
    ) -> impl Iterator<Item = Transition> {
        // Check for hidden label.
        let state = self.state2incoming.index(state_index.value());
        self.incoming_transitions.iter_range(state.start,state.end)
            .take_while(|transition| transition.label == 0)
    }
}

impl CompressedEntry for TransitionIndex {
    fn to_bytes(&self, bytes: &mut [u8]) {
        bytes[0] = self.offset;
        self.start.to_bytes(&mut bytes[1..self.offset as usize + 1]);
        self.end.to_bytes(&mut bytes[self.offset as usize + 1..]);
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let offset = bytes[0];

        Self {
            offset,
            start: usize::from_bytes(&bytes[1..offset as usize + 1]),
            end: usize::from_bytes(&bytes[offset as usize + 1..]),
        }
    }

    fn bytes_required(&self) -> usize {
        // One for the first offset.
        1 + self.start.bytes_required() + self.end.bytes_required()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use log::trace;
    use mcrl3_utilities::random_test;

    use crate::random_lts;

    #[test]
    fn test_random_incoming_transitions() {
        random_test(100, |rng| {
            let lts = random_lts(rng, 10, 3, 3);
            trace!("{:?}", lts);
            let incoming = IncomingTransitions::new(&lts);

            // Check that for every outgoing transition there is an incoming transition.
            for state_index in lts.iter_states() {
                for transition in lts.outgoing_transitions(state_index) {
                    let found = incoming.incoming_transitions(transition.to)
                        .any(|incoming| incoming.label == transition.label && incoming.to == state_index);
                    assert!(found, "Outgoing transition ({state_index}, {transition:?}) should have an incoming transition");
                }
            }
        });
    }
}
