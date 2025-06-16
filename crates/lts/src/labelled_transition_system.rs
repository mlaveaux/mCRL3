use std::fmt;

use mcrl3_utilities::TagIndex;

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
#[derive(PartialEq, Eq)]
pub struct LabelledTransitionSystem {
    states: Vec<State>,
    transitions: Vec<(LabelIndex, StateIndex)>,

    labels: Vec<String>,
    hidden_labels: Vec<String>,

    initial_state: StateIndex,

    num_of_transitions: usize,
}

impl LabelledTransitionSystem {
    /// Creates a new a labelled transition system with the given transitions, labels, and hidden labels.
    ///
    /// The initial state is the state with the given index.
    /// num_of_states is the number of states in the LTS, if known. If None then deadlock states without incoming transitions are removed.
    pub fn new<I, F>(
        initial_state: StateIndex,
        num_of_states: Option<usize>,
        transition_iter: F,
        mut labels: Vec<String>,
        hidden_labels: Vec<String>,
    ) -> LabelledTransitionSystem
    where
        F: Fn() -> I,
        I: Iterator<Item = (StateIndex, LabelIndex, StateIndex)>,
    {
        let mut states = Vec::new();
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

            states[*from].outgoing_end += 1;
            num_of_transitions += 1;
        }

        // Track the number of transitions before every state.
        states.iter_mut().fold(0, |count, state| {
            let result = count + state.outgoing_end;
            state.outgoing_start = count;
            state.outgoing_end = count;
            result
        });

        // Place the transitions, and increment the end for every state.
        let mut transitions = vec![(LabelIndex::new(0), StateIndex::new(0)); num_of_transitions];
        for (from, label, to) in transition_iter() {
            transitions[states[*from].outgoing_end] = (label, to);
            states[*from].outgoing_end += 1;
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
        for state in &mut states {
            for (label, _) in &mut transitions[state.outgoing_start..state.outgoing_end] {
                if hidden_indices.binary_search(label).is_ok() {
                    *label = LabelIndex::new(0);
                } else if introduced_tau {
                    // Remap all labels to not be the zero hidden action.
                    *label = LabelIndex::new(**label + 1);
                }
            }
        }

        LabelledTransitionSystem {
            initial_state,
            labels,
            hidden_labels,
            states,
            num_of_transitions: transitions.len(),
            transitions,
        }
    }

    /// Returns the index of the initial state
    pub fn initial_state_index(&self) -> StateIndex {
        self.initial_state
    }

    /// Returns the set of outgoing transitions for the given state.
    pub fn outgoing_transitions(&self, state_index: StateIndex) -> impl Iterator<Item = &(LabelIndex, StateIndex)> {
        let state = &self.states[*state_index];
        self.transitions[state.outgoing_start..state.outgoing_end].iter()
    }

    /// Iterate over all state_index in the labelled transition system
    pub fn iter_states(&self) -> impl Iterator<Item = StateIndex> + use<> {
        (0..self.states.len()).map(|index| StateIndex::new(index))
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
        self.num_of_transitions
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
#[derive(Clone, Default, PartialEq, Eq)]
struct State {
    outgoing_start: usize,
    outgoing_end: usize,
}

impl fmt::Display for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print some information about the LTS.
        writeln!(f, "Number of states: {}", self.states.len())?;
        writeln!(f, "Number of action labels: {}", self.labels.len())?;
        write!(f, "Number of transitions: {}", self.num_of_transitions)
    }
}

impl fmt::Debug for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self)?;
        writeln!(f, "Initial state: {}", self.initial_state)?;
        writeln!(f, "Hidden labels: {:?}", self.hidden_labels)?;

        for state_index in self.iter_states() {
            for &(label, to) in self.outgoing_transitions(state_index) {
                let label_name = &self.labels[label.value()];

                writeln!(f, "{state_index} --[{label_name}]-> {to}")?;
            }
        }

        Ok(())
    }
}
