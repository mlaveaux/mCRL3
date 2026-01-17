use std::{collections::VecDeque, fmt};

use merc_lts::LabelIndex;
use merc_utilities::TagIndex;

/// A unique type for vertices in the counterexample tree.
pub struct CounterTag {}

/// The index type for vertices in the counterexample tree.
pub type CounterIndex = TagIndex<usize, CounterTag>;

/// A generic trait for counterexample tree structures.
/// 
/// # Details
/// 
/// This is useful such that we can both provide a concrete implementation that
/// constructs a counterexample tree, but also a placeholder implementation that
/// does nothing when no counterexample is needed.
pub trait CounterExampleTree {
    type Index: Clone + Copy;

    /// Returns the index of the root of the counterexample tree.
    fn root_index(&self) -> Self::Index;

    /// Adds an edge to the counterexample tree.
    fn add_edge(&mut self, label: LabelIndex, to: Self::Index) -> Self::Index;
}

/// A class that can be used to store a counter example tree from which a
/// counter example trace can be extracted.
pub struct CounterExampleConstructor {
    /// The backward tree is stored in a deque. 
    backward_tree: VecDeque<(LabelIndex, CounterIndex)>,
}

impl CounterExampleConstructor {
    /// Creates a new counterexample constructor.    
    pub fn new() -> Self {
        Self {
            // Add the root such that index 0 is the root.
            backward_tree: VecDeque::from([(LabelIndex::default(), TagIndex::default())]),
        }
    }

    /// Reconstructs the trace from the counterexample tree given the index
    /// of the leaf node.
    pub fn reconstruct_trace(&self, mut index: CounterIndex) -> Vec<LabelIndex> {
        let mut trace = Vec::new();

        while index != self.root_index() {
            let (label, parent) = &self.backward_tree[*index];
            trace.push(*label);
            index = *parent;
        }

        trace.reverse();
        trace
    }
}

impl fmt::Debug for CounterExampleConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CounterExampleConstructor")
            .field("backward_tree", &self.backward_tree)
            .finish()
    }
}

impl CounterExampleTree for CounterExampleConstructor { 
    type Index = CounterIndex;

    fn add_edge(&mut self, label: LabelIndex, to: Self::Index) -> Self::Index {
        self.backward_tree.push_back((label, to));
        TagIndex::new(self.backward_tree.len() - 1)
    }
    
    fn root_index(&self) -> Self::Index {
        TagIndex::new(0)
    }   
}

impl Default for CounterExampleConstructor {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterExampleTree for () {
    type Index = ();

    fn add_edge(&mut self, _label: LabelIndex, _to: Self::Index) -> Self::Index {
        // Do nothing
    }
        
    fn root_index(&self) -> Self::Index {}
}