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
        let mut state2incoming = bytevec![0; game.num_of_vertices()];

        // Count the number of incoming transitions for each state
        for state_index in game.iter_vertices() {
            for edge in game.outgoing_edges(state_index) {
                state2incoming.update(edge.to().value(), |start| *start += 1);
            }
        }

        // Compute the start offsets (prefix sum)
        state2incoming.fold(0, |offset, start| {
            let new_offset = offset + *start;
            *start = offset;
            new_offset
        });

        // Place the transitions
        for state_index in game.iter_vertices() {
            for edge in game.outgoing_edges(state_index) {
                state2incoming.update(edge.to().value(), |start| {
                    edges_from.set(*start, state_index);
                    *start += 1;
                });
            }
        }

        state2incoming.fold(0, |previous, start| {
            let result = *start;
            *start = previous;
            result
        });

        // Add sentinel state
        state2incoming.push(edges_from.len());

        Self {
            edges_from,
            vertex_to_predecessors: state2incoming,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over the predecessors the given vertex.
    pub fn predecessors(&self, state_index: VertexIndex) -> impl Iterator<Item = VertexIndex> + '_ {
        let start = self.vertex_to_predecessors.index(state_index.value());
        let end = self.vertex_to_predecessors.index(state_index.value() + 1);
        (start..end).map(move |i| self.edges_from.index(i))
    }
}
