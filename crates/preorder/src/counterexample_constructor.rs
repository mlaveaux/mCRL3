use std::collections::VecDeque;

use merc_lts::{LabelIndex, StateIndex};
use merc_utilities::TagIndex;

/// A unique type for vertices in the counterexample tree.
pub struct CounterTag {}

/// The index type for vertices in the counterexample tree.
pub type CounterIndex = TagIndex<usize, CounterTag>;

/// A class that can be used to store a counter example tree from which a
/// counter example trace can be extracted.
pub trait CounterExampleTree {
    type Index: Clone + Copy;

    /// Creates a new counterexample tree.
    fn new() -> Self;

    /// Returns the index of the root of the counterexample tree.
    fn root_index(&self) -> Self::Index;

    /// Adds a edge to the counterexample tree.
    fn add_edge(&mut self, label: LabelIndex, to: Self::Index) -> Self::Index;
}

pub struct CounterExampleConstructor {
    /// The backward three is stored in a deque. 
    backward_tree: VecDeque<(LabelIndex, CounterIndex)>,
}

impl CounterExampleTree for CounterExampleConstructor { 
    type Index = CounterIndex;

    fn add_edge(&mut self, label: LabelIndex, to: Self::Index) -> Self::Index {
        self.backward_tree.push_back((label, to));
        TagIndex::new(self.backward_tree.len() - 1)
    }
    
    fn new() -> Self {
        Self {
            backward_tree: VecDeque::new(),
        }
    }
    
    fn root_index(&self) -> Self::Index {
        TagIndex::new(0)
    }   
}

impl CounterExampleTree for () {
    type Index = ();

    fn add_edge(&mut self, _label: LabelIndex, _to: Self::Index) -> Self::Index {
        // Do nothing
    }
    
    fn new() -> Self {
        ()
    }
    
    fn root_index(&self) -> Self::Index {
        ()
    }
}