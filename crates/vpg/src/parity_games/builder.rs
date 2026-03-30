#![forbid(unsafe_code)]

use std::collections::HashMap;

use oxidd::bdd::BDDManagerRef;

use crate::ParityGame;
use crate::Player;
use crate::Priority;
use crate::VariabilityParityGame;
use crate::VertexIndex;

/// A builder for parity games that accepts edges one by one and can remove
/// duplicates.
pub struct ParityGameBuilder {
    /// The edges of the parity game.
    edges: Vec<(VertexIndex, VertexIndex)>,

    /// The owner of each vertex, indexed by vertex index.
    owners: HashMap<usize, Player>,

    /// The priority of each vertex, indexed by vertex index.
    priorities: HashMap<usize, Priority>,

    /// The initial vertex of the game.
    initial_vertex: VertexIndex,

    /// The number of states discovered so far.
    num_of_states: usize,
}

impl ParityGameBuilder {
    /// Initializes a new empty builder with the given initial vertex.
    pub fn new(initial_vertex: VertexIndex) -> Self {
        Self::with_capacity(initial_vertex, 0)
    }

    /// Initializes the builder with pre-allocated capacity for edges.
    pub fn with_capacity(initial_vertex: VertexIndex, num_of_edges: usize) -> Self {
        Self {
            edges: Vec::with_capacity(num_of_edges),
            owners: HashMap::new(),
            priorities: HashMap::new(),
            initial_vertex,
            num_of_states: initial_vertex.value() + 1,
        }
    }

    /// Adds a vertex to the builder with its owner and priority.
    pub fn add_vertex(&mut self, vertex: VertexIndex, owner: Player, priority: Priority) {
        self.owners.insert(vertex.value(), owner);
        self.priorities.insert(vertex.value(), priority);
        self.num_of_states = self.num_of_states.max(vertex.value() + 1);
    }

    /// Adds an edge to the builder.
    pub fn add_edge(&mut self, from: VertexIndex, to: VertexIndex) {
        self.edges.push((from, to));
        self.num_of_states = self.num_of_states.max(from.value() + 1).max(to.value() + 1);
    }

    /// Returns the number of edges added to the builder.
    pub fn num_of_edges(&self) -> usize {
        self.edges.len()
    }

    /// Returns the number of states that the builder currently found.
    pub fn num_of_states(&self) -> usize {
        self.num_of_states
    }

    /// Finalizes the builder and returns the constructed parity game.
    pub fn finish(&mut self, make_total: bool, remove_duplicates: bool) -> ParityGame {
        if remove_duplicates {
            self.remove_duplicates();
        }

        // Initialize vertices and priorities with defaults
        let mut owner = vec![Player::Even; self.num_of_states];
        let mut priority = vec![Priority::new(0); self.num_of_states];

        // Set the owners and priorities from the map
        for (vertex_idx, player) in &self.owners {
            if *vertex_idx < self.num_of_states {
                owner[*vertex_idx] = *player;
            }
        }

        for (vertex_idx, prio) in &self.priorities {
            if *vertex_idx < self.num_of_states {
                priority[*vertex_idx] = *prio;
            }
        }

        // Build the parity game using the from_edges method
        let edges = self.edges.clone();
        ParityGame::from_edges(self.initial_vertex, owner, priority, make_total, || {
            edges.iter().cloned()
        })
    }
    
    /// Removes duplicated edges from the added edges.
    fn remove_duplicates(&mut self) {
        self.edges.sort();
        self.edges.dedup();
    }
}

/// A builder for variability parity games that accepts edges with BDD configurations
/// one by one and can remove duplicates.
pub struct VariabilityParityGameBuilder {
    /// The edges of the variability parity game with their configurations.
    edges: Vec<(VertexIndex, oxidd::bdd::BDDFunction, VertexIndex)>,

    /// The owner of each vertex, indexed by vertex index.
    owners: HashMap<usize, Player>,

    /// The priority of each vertex, indexed by vertex index.
    priorities: HashMap<usize, Priority>,

    /// The initial vertex of the game.
    initial_vertex: VertexIndex,

    /// The number of states discovered so far.
    num_of_states: usize,
}

impl VariabilityParityGameBuilder {
    /// Initializes a new empty builder with the given initial vertex.
    pub fn new(initial_vertex: VertexIndex) -> Self {
        Self::with_capacity(initial_vertex, 0)
    }

    /// Initializes the builder with pre-allocated capacity for edges.
    pub fn with_capacity(initial_vertex: VertexIndex, num_of_edges: usize) -> Self {
        Self {
            edges: Vec::with_capacity(num_of_edges),
            owners: HashMap::new(),
            priorities: HashMap::new(),
            initial_vertex,
            num_of_states: initial_vertex.value() + 1,
        }
    }

    /// Adds a vertex to the builder with its owner and priority.
    pub fn add_vertex(&mut self, vertex: VertexIndex, owner: Player, priority: Priority) {
        self.owners.insert(vertex.value(), owner);
        self.priorities.insert(vertex.value(), priority);
        self.num_of_states = self.num_of_states.max(vertex.value() + 1);
    }

    /// Adds an edge to the builder with its configuration.
    pub fn add_edge(&mut self, from: VertexIndex, configuration: oxidd::bdd::BDDFunction, to: VertexIndex) {
        self.edges.push((from, configuration, to));
        self.num_of_states = self.num_of_states.max(from.value() + 1).max(to.value() + 1);
    }

    /// Returns the number of edges added to the builder.
    pub fn num_of_edges(&self) -> usize {
        self.edges.len()
    }

    /// Returns the number of states that the builder currently found.
    pub fn num_of_states(&self) -> usize {
        self.num_of_states
    }

    /// Finalizes the builder and returns the constructed variability parity game.
    pub fn finish(
        &mut self,
        manager_ref: &BDDManagerRef,
        configuration: oxidd::bdd::BDDFunction,
        variables: Vec<oxidd::bdd::BDDFunction>,
        remove_duplicates: bool,
    ) -> VariabilityParityGame {
        if remove_duplicates {
            self.remove_duplicates();
        }

        // Initialize vertices and priorities with defaults
        let mut owner = vec![Player::Even; self.num_of_states];
        let mut priority = vec![Priority::new(0); self.num_of_states];

        // Set the owners and priorities from the map
        for (vertex_idx, player) in &self.owners {
            if *vertex_idx < self.num_of_states {
                owner[*vertex_idx] = *player;
            }
        }

        for (vertex_idx, prio) in &self.priorities {
            if *vertex_idx < self.num_of_states {
                priority[*vertex_idx] = *prio;
            }
        }

        // Build the variability parity game using the from_edges method
        let edges = self.edges.clone();
        VariabilityParityGame::from_edges(
            manager_ref,
            self.initial_vertex,
            owner,
            priority,
            configuration,
            variables,
            || edges.iter().cloned(),
        )
    }
    
    /// Removes duplicated edges from the added edges.
    ///
    /// Note: This requires comparing BDD functions, which might be expensive.
    fn remove_duplicates(&mut self) {
        self.edges.sort_by_key(|(from, _, to)| (*from, *to));
        self.edges.dedup_by_key(|(from, _, to)| (*from, *to));
    }
}
