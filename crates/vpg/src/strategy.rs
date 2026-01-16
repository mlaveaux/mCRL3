use std::{collections::HashMap, fmt};

use crate::VertexIndex;

/// Keeps track of a strategy for a player in a parity game.
/// 
/// # Details
/// 
/// A strategy is a partial function from vertices owned by a player to one of
/// their successors. 
pub struct Strategy {
    mapping: HashMap<VertexIndex, VertexIndex>,
}

impl Strategy {
    /// Creates a new, empty strategy.
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    /// Adds a mapping from `from` to `to` in the strategy.
    pub fn set(&mut self, from: VertexIndex, to: VertexIndex) {
        self.mapping.insert(from, to);
    }

    /// Gets the target vertex for the given source vertex, if it is defined.
    pub fn get(&self, from: VertexIndex) -> Option<&VertexIndex> {
        self.mapping.get(&from)
    }
}

impl fmt::Debug for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Strategy {{")?;
        for (from, to) in &self.mapping {
            writeln!(f, "  v{} -> v{}", from, to)?;
        }
        writeln!(f, "}}")
    }
}