//! Authors: Maurice Laveaux and Sjef van Loo

use std::marker::PhantomData;

use merc_collections::ByteCompressedVec;
use merc_collections::bytevec;

use crate::PG;
use crate::VertexIndex;

/// Stores the predecessors for a given parity game.
pub struct Predecessors<'a> {
    /// A flat list of all predecessors in the game.
    edges_from: ByteCompressedVec<VertexIndex>,

    /// A mapping from the vertex to the `edges_from` that stores its
    /// predecessors.
    vertex_to_predecessors: ByteCompressedVec<usize>,

    /// Marker to tie the lifetime of the predecessors to the game.
    _marker: PhantomData<&'a ()>,
}

impl<'a> Predecessors<'a> {
    /// Creates the predecessors structure for the given parity game.
    pub fn new<G: PG>(game: &'a G) -> Self {
        let mut edges_from = bytevec![VertexIndex::new(0); game.num_of_edges()];
        let mut vertex_to_predecessors = bytevec![0; game.num_of_vertices()];

        // Count the number of incoming transitions for each vertex
        for vertex_index in game.iter_vertices() {
            for edge in game.outgoing_edges(vertex_index) {
                vertex_to_predecessors.update(edge.to().value(), |start| *start += 1);
            }
        }

        // Compute the start offsets (prefix sum)
        vertex_to_predecessors.fold(0, |offset, start| {
            let new_offset = offset + *start;
            *start = offset;
            new_offset
        });

        // Place the transitions
        for vertex_index in game.iter_vertices() {
            for edge in game.outgoing_edges(vertex_index) {
                vertex_to_predecessors.update(edge.to().value(), |start| {
                    edges_from.set(*start, vertex_index);
                    *start += 1;
                });
            }
        }

        let mut offset = 0usize;
        vertex_to_predecessors.fold(0, |previous, start| {
            let result = *start;
            *start = previous;
            offset = result;
            result
        });

        vertex_to_predecessors.push(offset);

        Self {
            edges_from,
            vertex_to_predecessors,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over the predecessors the given vertex.
    pub fn predecessors(&self, vertex_index: VertexIndex) -> impl Iterator<Item = VertexIndex> + use<'_> {
        let start = self.vertex_to_predecessors.index(vertex_index.value());
        let end = self.vertex_to_predecessors.index(vertex_index.value() + 1);
        (start..end).map(move |i| self.edges_from.index(i))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use merc_utilities::random_test;

    use crate::PG;
    use crate::Predecessors;
    use crate::PrioSubgame;
    use crate::Priority;
    use crate::random_parity_game;

    #[test]
    fn test_random_predecessors() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, true, 50, 10, 5);
            check_predecessors(game);
        })
    }

    #[test]
    fn test_random_predecessors_subgame() {
        random_test(100, |rng| {
            let pg = random_parity_game(rng, true, 50, 10, 5);
            let game = PrioSubgame::new(&pg, Priority::new(0));

            check_predecessors(game);
        })
    }

    /// Checks that the predecessors computed by the `Predecessors` structure
    /// match the expected predecessors computed by iterating over all vertices
    /// and their outgoing edges.
    fn check_predecessors<G: PG>(game: G) {
        let predecessors = Predecessors::new(&game);
        for vertex in game.iter_vertices() {
            // Compute the expected predecessors by iterating over all vertices and their outgoing edges.
            let expected_predecessors: Vec<_> = game
                .iter_vertices()
                .filter(|&v| game.outgoing_edges(v).any(|e| e.to() == vertex))
                .collect();
            let actual_predecessors: Vec<_> = predecessors.predecessors(vertex).collect();
    
            assert_eq!(
                expected_predecessors.into_iter().sorted().collect::<Vec<_>>(),
                actual_predecessors.into_iter().sorted().collect::<Vec<_>>(),
                "Predecessors of vertex {} do not match",
                vertex.value()
            );
        }
    }
}
