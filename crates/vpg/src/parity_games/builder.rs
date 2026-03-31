#![forbid(unsafe_code)]

use oxidd::BooleanFunction;
use oxidd::bdd::BDDFunction;
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
    owners: Vec<Player>,

    /// The priority of each vertex, indexed by vertex index.
    priorities: Vec<Priority>,

    /// The initial vertex of the game.
    initial_vertex: VertexIndex,

    /// The number of vertices discovered so far.
    num_of_vertices: usize,
}

impl ParityGameBuilder {
    /// Initializes a new empty builder with the given initial vertex.
    pub fn new(initial_vertex: VertexIndex) -> Self {
        Self::with_capacity(initial_vertex, 0)
    }

    /// Initializes the builder with pre-allocated capacity for edges.
    pub fn with_capacity(initial_vertex: VertexIndex, num_of_edges: usize) -> Self {
        let num_of_vertices = initial_vertex.value() + 1;
        Self {
            edges: Vec::with_capacity(num_of_edges),
            owners: vec![Player::Even; num_of_vertices],
            priorities: vec![Priority::new(0); num_of_vertices],
            initial_vertex,
            num_of_vertices,
        }
    }

    /// Adds a vertex to the builder with its owner and priority.
    pub fn add_vertex(&mut self, vertex: VertexIndex, owner: Player, priority: Priority) {
        let num_of_vertices = vertex.value() + 1;
        self.ensure_vertex_capacity(num_of_vertices);
        self.owners[vertex.value()] = owner;
        self.priorities[vertex.value()] = priority;
        self.num_of_vertices = self.num_of_vertices.max(num_of_vertices);
    }

    /// Adds an edge to the builder.
    pub fn add_edge(&mut self, from: VertexIndex, to: VertexIndex) {
        self.edges.push((from, to));
        let num_of_vertices = self.num_of_vertices.max(from.value() + 1).max(to.value() + 1);
        self.ensure_vertex_capacity(num_of_vertices);
        self.num_of_vertices = num_of_vertices;
    }

    /// Returns the number of edges added to the builder.
    pub fn num_of_edges(&self) -> usize {
        self.edges.len()
    }

    /// Returns the number of vertices that the builder currently found.
    pub fn num_of_vertices(&self) -> usize {
        self.num_of_vertices
    }

    /// Finalizes the builder and returns the constructed parity game.
    pub fn finish(mut self, make_total: bool, remove_duplicates: bool) -> ParityGame {
        if remove_duplicates {
            self.remove_duplicates();
        }

        self.ensure_vertex_capacity(self.num_of_vertices);

        // Build the parity game using the from_edges method
        let edges = self.edges.clone();
        ParityGame::from_edges(
            self.initial_vertex,
            self.owners,
            self.priorities,
            make_total,
            || edges.iter().cloned(),
        )
    }

    /// Ensures that the owners and priorities vectors have enough capacity for the given number of vertices.
    fn ensure_vertex_capacity(&mut self, num_of_vertices: usize) {
        if self.owners.len() < num_of_vertices {
            self.owners.resize(num_of_vertices, Player::Even);
        }
        if self.priorities.len() < num_of_vertices {
            self.priorities.resize(num_of_vertices, Priority::new(0));
        }
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
    owners: Vec<Player>,

    /// The priority of each vertex, indexed by vertex index.
    priorities: Vec<Priority>,

    /// The initial vertex of the game.
    initial_vertex: VertexIndex,

    /// The number of vertices discovered so far.
    num_of_vertices: usize,
}

impl VariabilityParityGameBuilder {
    /// Initializes a new empty builder with the given initial vertex.
    pub fn new(initial_vertex: VertexIndex) -> Self {
        Self::with_capacity(initial_vertex, 0)
    }

    /// Initializes the builder with pre-allocated capacity for edges.
    pub fn with_capacity(initial_vertex: VertexIndex, num_of_edges: usize) -> Self {
        let num_of_vertices = initial_vertex.value() + 1;
        Self {
            edges: Vec::with_capacity(num_of_edges),
            owners: vec![Player::Even; num_of_vertices],
            priorities: vec![Priority::new(0); num_of_vertices],
            initial_vertex,
            num_of_vertices,
        }
    }

    /// Adds a vertex to the builder with its owner and priority.
    pub fn add_vertex(&mut self, vertex: VertexIndex, owner: Player, priority: Priority) {
        let num_of_vertices = vertex.value() + 1;
        self.ensure_vertex_capacity(num_of_vertices);
        self.owners[vertex.value()] = owner;
        self.priorities[vertex.value()] = priority;
        self.num_of_vertices = self.num_of_vertices.max(num_of_vertices);
    }

    /// Adds an edge to the builder with its configuration.
    pub fn add_edge(&mut self, from: VertexIndex, configuration: oxidd::bdd::BDDFunction, to: VertexIndex) {
        self.edges.push((from, configuration, to));
        let num_of_vertices = self.num_of_vertices.max(from.value() + 1).max(to.value() + 1);
        self.ensure_vertex_capacity(num_of_vertices);
        self.num_of_vertices = num_of_vertices;
    }

    /// Returns the number of edges added to the builder.
    pub fn num_of_edges(&self) -> usize {
        self.edges.len()
    }

    /// Returns the number of vertices that the builder currently found.
    pub fn num_of_vertices(&self) -> usize {
        self.num_of_vertices
    }

    /// Consumes the builder and returns the constructed variability parity game.
    pub fn finish(
        mut self,
        manager_ref: &BDDManagerRef,
        configuration: BDDFunction,
        variables: Vec<BDDFunction>,
        remove_duplicates: bool,
    ) -> VariabilityParityGame {
        if remove_duplicates {
            self.remove_duplicates();
        }

        self.ensure_vertex_capacity(self.num_of_vertices);

        // Build the variability parity game using the from_edges method
        let edges = self.edges.clone();
        VariabilityParityGame::from_edges(
            manager_ref,
            self.initial_vertex,
            self.owners,
            self.priorities,
            configuration,
            variables,
            || edges.iter().cloned(),
        )
    }

    /// Ensures that the owners and priorities vectors have enough capacity for the given number of vertices.
    fn ensure_vertex_capacity(&mut self, num_of_vertices: usize) {
        if self.owners.len() < num_of_vertices {
            self.owners.resize(num_of_vertices, Player::Even);
        }
        if self.priorities.len() < num_of_vertices {
            self.priorities.resize(num_of_vertices, Priority::new(0));
        }
    }

    /// Removes duplicated edges from the added edges.
    fn remove_duplicates(&mut self) {
        self.edges.sort_by_key(|(from, _, to)| (*from, *to));

        let mut merged: Vec<(VertexIndex, BDDFunction, VertexIndex)> = Vec::with_capacity(self.edges.len());
        for (from, configuration, to) in self.edges.drain(..) {
            if let Some((last_from, last_configuration, last_to)) = merged.last_mut()
                && *last_from == from
                && *last_to == to
            {
                *last_configuration = last_configuration
                    .or(&configuration)
                    .expect("Duplicate edges should have compatible BDD managers");
                continue;
            }

            merged.push((from, configuration, to));
        }

        self.edges = merged;
    }
}
