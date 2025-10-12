use std::fmt;

use mcrl3_utilities::ByteCompressedVec;
use mcrl3_utilities::CompressedEntry;
use mcrl3_utilities::TagIndex;
use mcrl3_utilities::bytevec;

/// A unique type for the labels.
pub struct LabelTag;

/// A unique type for the labels.
pub struct StateTag;

/// The index type for a label.
pub type LabelIndex = TagIndex<usize, LabelTag>;

/// The index for a state.
pub type StateIndex = TagIndex<usize, StateTag>;

/// Represents a labelled transition system consisting of states with directed
/// labelled edges.
#[derive(PartialEq, Eq, Clone)]
pub struct LabelledTransitionSystem {
    /// Encodes the states and their outgoing transitions.
    states: ByteCompressedVec<State>,
    transitions: ByteCompressedVec<Transition>,

    /// Keeps track of the labels for every index, and which of them are hidden.
    labels: Vec<String>,
    hidden_labels: Vec<String>,

    /// The index of the initial state.
    initial_state: StateIndex,
}

impl LabelledTransitionSystem {
    /// Creates a new a labelled transition system with the given transitions, labels, and hidden labels.
    ///
    /// The initial state is the state with the given index.
    /// num_of_states is the number of states in the LTS, if known. If None then deadlock states without incoming transitions are removed.
    pub fn new<I, F>(
        initial_state: StateIndex,
        num_of_states: Option<usize>,
        mut transition_iter: F,
        mut labels: Vec<String>,
        hidden_labels: Vec<String>,
    ) -> LabelledTransitionSystem
    where
        F: FnMut() -> I,
        I: Iterator<Item = (StateIndex, LabelIndex, StateIndex)>,
    {
        let mut states = ByteCompressedVec::new();
        if let Some(num_of_states) = num_of_states {
            states.resize_with(num_of_states, Default::default);
        }

        // Count the number of transitions for every state
        let mut num_of_transitions = 0;
        for (from, _, to) in transition_iter() {
            // Ensure that the states vector is large enough.
            while states.len() <= *from.max(to) {
                states.push(State::default());
            }

            states.update(*from, |entry| entry.outgoing_end += 1);
            num_of_transitions += 1;
        }

        println!("What");

        // Track the number of transitions before every state.
        states.fold(0, |count, state| {
            let result = count + state.outgoing_end;
            *state = State::new(count, count);
            result
        });

        // Place the transitions, and increment the end for every state.
        let mut transitions = bytevec![Transition::new(LabelIndex::new(0), StateIndex::new(0)); num_of_transitions];
        for (from, label, to) in transition_iter() {
            transitions.set(states.index(*from).outgoing_end, Transition::new(label, to));
            states.update(*from, |entry| entry.outgoing_end += 1);
        }

        // Keep track of which label indexes are hidden labels.
        let mut hidden_indices: Vec<usize> = Vec::new();
        for label in &hidden_labels {
            if let Some(index) = labels.iter().position(|other| other == label) {
                hidden_indices.push(index);
            }
        }
        hidden_indices.sort();

        // Make an implicit tau label the first label.
        let introduced_tau = if hidden_indices.contains(&0) {
            labels[0] = "tau".to_string();
            false
        } else {
            labels.insert(0, "tau".to_string());
            true
        };

        // Remap all hidden actions to zero.
        transitions.map(|trans| {
            if hidden_indices.binary_search(&trans.label.value()).is_ok() {
                trans.label = LabelIndex::new(0);
            } else if introduced_tau {
                // Remap all labels to not be the zero hidden action.
                trans.label = LabelIndex::new(*trans.label + 1);
            }
        });

        println!("States {}", states.metrics());
        println!("Transitions {}", transitions.metrics());

        LabelledTransitionSystem {
            initial_state,
            labels,
            hidden_labels,
            states,
            transitions,
        }
    }

    /// Creates a labelled transition system from another one, given the permutation of state indices
    ///
    pub fn new_from_permutation<P>(lts: LabelledTransitionSystem, permutation: P) -> Self
    where
        P: Fn(StateIndex) -> StateIndex + Copy,
    {
        let mut states = bytevec![State::default(); lts.num_of_states()];

        for state_index in lts.iter_states() {
            // Keep the transitions the same, but
            let new_state_index = permutation(state_index);
            let state = lts.states.index(*state_index);
            states.update(*new_state_index, |entry| {
                entry.outgoing_start = state.outgoing_start;
                entry.outgoing_end = state.outgoing_end;
            });
        }

        LabelledTransitionSystem {
            initial_state: permutation(lts.initial_state),
            labels: lts.labels,
            hidden_labels: lts.hidden_labels,
            states,
            transitions: lts.transitions,
        }
    }

    /// Returns the index of the initial state
    pub fn initial_state_index(&self) -> StateIndex {
        self.initial_state
    }

    /// Returns the set of outgoing transitions for the given state.
    pub fn outgoing_transitions(&self, state_index: StateIndex) -> impl Iterator<Item = Transition> {
        let state = &self.states.index(*state_index);
        self.transitions.iter_range(state.outgoing_start, state.outgoing_end)
    }

    /// Iterate over all state_index in the labelled transition system
    pub fn iter_states(&self) -> impl Iterator<Item = StateIndex> + use<> {
        (0..self.states.len()).map(StateIndex::new)
    }

    /// Returns the number of states.
    pub fn num_of_states(&self) -> usize {
        self.states.len()
    }

    /// Returns the number of labels.
    pub fn num_of_labels(&self) -> usize {
        self.labels.len()
    }

    /// Returns the number of transitions.
    pub fn num_of_transitions(&self) -> usize {
        self.transitions.len()
    }

    /// Returns the list of labels.
    pub fn labels(&self) -> &[String] {
        &self.labels[0..]
    }

    /// Returns the list of hidden labels.
    pub fn hidden_labels(&self) -> &[String] {
        &self.hidden_labels[0..]
    }

    /// Returns true iff the given label index is a hidden label.
    pub fn is_hidden_label(&self, label_index: LabelIndex) -> bool {
        label_index.value() == 0
    }
}

/// A single state in the LTS, containing a vector of outgoing edges.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
struct State {
    offset: u8,
    outgoing_start: usize,
    outgoing_end: usize,
}

impl State {
    /// Constructs a new state
    fn new(outgoing_start: usize, outgoing_end: usize) -> Self {
        Self { offset: outgoing_start.bytes_required() as u8, outgoing_start, outgoing_end }
    }
}

impl CompressedEntry for State {
    fn to_bytes(&self, bytes: &mut [u8]) {
        bytes[0] = self.offset;
        self.outgoing_start.to_bytes(&mut bytes[1..self.offset as usize + 1]);
        self.outgoing_end.to_bytes(&mut bytes[self.offset as usize + 1..]);
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let offset = bytes[0];

        Self {
            offset,
            outgoing_start: usize::from_bytes(&bytes[1..offset as usize + 1]),
            outgoing_end: usize::from_bytes(&bytes[offset as usize + 1..]),
        }
    }

    fn bytes_required(&self) -> usize {
        // One for the first offset.
        1 + self.outgoing_start.bytes_required() + self.outgoing_end.bytes_required()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Transition {
    offset: u8,
    pub label: LabelIndex,
    pub to: StateIndex,
}

impl Transition {
    /// Constructs a new transition.
    pub fn new(label: LabelIndex, to: StateIndex) -> Self {
        Self { offset: label.bytes_required() as u8, label, to }
    }
}

impl CompressedEntry for Transition {
    fn to_bytes(&self, bytes: &mut [u8]) {
        bytes[0] = self.offset;
        self.label.to_bytes(&mut bytes[1..self.offset as usize + 1]);
        self.to.to_bytes(&mut bytes[self.offset as usize + 1..]);
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let offset = bytes[0];

        Self {
            offset,
            label: LabelIndex::from_bytes(&bytes[1..offset as usize + 1]),
            to: StateIndex::from_bytes(&bytes[offset as usize + 1..]),
        }
    }

    fn bytes_required(&self) -> usize {
        // One for the first offset.
        1 + self.label.bytes_required() + self.to.bytes_required()
    }
}


impl fmt::Display for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print some information about the LTS.
        writeln!(f, "Number of states: {}", self.states.len())?;
        writeln!(f, "Number of action labels: {}", self.labels.len())?;
        write!(f, "Number of transitions: {}", self.num_of_transitions())
    }
}

impl fmt::Debug for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{self}")?;
        writeln!(f, "Initial state: {}", self.initial_state)?;
        writeln!(f, "Hidden labels: {:?}", self.hidden_labels)?;

        for state_index in self.iter_states() {
            for transition in self.outgoing_transitions(state_index) {
                let label_name = &self.labels[transition.label.value()];

                writeln!(f, "{state_index} --[{label_name}]-> {}", transition.to)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mcrl3_utilities::random_test;
    use rand::Rng;

    #[test]
    fn random_state_entry() {
        random_test(100, |rng| {
            let begin: usize = rng.random::<u64>() as usize;
            let end: usize = rng.random::<u64>() as usize;

            let state = State::new(begin, end);

            let mut bytes: Vec<u8> = vec![0; state.bytes_required()];
            state.to_bytes(&mut bytes[..]);
            assert_eq!(State::from_bytes(&bytes[..]), state);
        });
    }

    #[test]
    fn random_transition_entry() {
        random_test(100, |rng| {
            let label: usize = rng.random::<u64>() as usize;
            let to: usize = rng.random::<u64>() as usize;

            let state = Transition::new(LabelIndex::new(label), StateIndex::new(to));

            let mut bytes: Vec<u8> = vec![0; state.bytes_required()];
            state.to_bytes(&mut bytes[..]);
            assert_eq!(Transition::from_bytes(&bytes[..]), state);
        });
    }
}

