use std::collections::HashMap;
use std::fmt;

use crate::PG;
use crate::Player;
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

    /// Combines two strategies, preferring mappings from the second strategy.
    pub fn combine(mut self, extension: Strategy) -> Strategy {
        // Add all mappings from the extension strategy to the base strategy
        for (&from, &to) in extension.iter() {
            self.set(from, to);
        }
        self
    }

    /// Returns an iterator over all (from, to) pairs in the strategy.
    pub fn iter(&self) -> impl Iterator<Item = (&VertexIndex, &VertexIndex)> {
        self.mapping.iter()
    }

    /// Checks that the strategy is only defined for the vertices owned by the
    /// given player.
    pub fn check_consistent(&self, pg: &impl PG, player: Player) {
        for vertex in pg.iter_vertices() {
            if let Some(_) = self.get(vertex) {
                if pg.owner(vertex) != player {
                    panic!(
                        "Strategy is defined for vertex {:?} owned by {:?}, \
                         but should only be defined for vertices owned by {:?}",
                        vertex,
                        pg.owner(vertex),
                        player
                    );
                }
            }
        }
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::new()
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
