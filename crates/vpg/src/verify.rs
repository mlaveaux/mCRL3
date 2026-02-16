use bitvec::bitvec;
use bitvec::order::Lsb0;
use delegate::delegate;

use log::trace;
use merc_collections::Graph;
use merc_collections::scc_decomposition;
use merc_utilities::MercIndex;

use crate::AsGraph;
use crate::Edge;
use crate::PG;
use crate::Player;
use crate::Predecessors;
use crate::Priority;
use crate::Set;
use crate::Strategy;
use crate::VertexIndex;
use crate::check_partition;

/// Verifies that a proposed solution is valid for the given parity game and
/// strategies.
pub fn verify_solution<G: PG>(pg: &G, solution: &[Set; 2], strategy: &[Strategy; 2]) {
    debug_assert!(pg.is_total(), "Verifying requires a total parity game");

    // The set of all vertices in the game.
    let vertices = bitvec![usize, Lsb0; 1; pg.num_of_vertices()];

    // Check that the input solution is a proper partitioning
    check_partition(&solution[0], &solution[1], &vertices);
    let predecessors = Predecessors::new(pg);

    // We check both players' strategies
    for player in [Player::Even, Player::Odd] {
        // Check that the strategies are consistent with the ownership of the vertices
        strategy[player.to_index()].check_consistent(pg, player);

        let mut winning_set = bitvec![usize, Lsb0; 0; pg.num_of_vertices()];

        for priority in 0..pg.highest_priority().value() {
            if Player::from_priority(&(Priority::new(priority))) != player {
                // Skip priorities that do not belong to the current player
                continue;
            }

            // Restrict the game according to the strategy and the current priority.
            trace!("Restricting game for player {} with priority {}", player, priority);
            let restricted_game = SubGame::new(pg, player, &strategy[player.to_index()], Priority::new(priority));


            let scc_partition = scc_decomposition(&AsGraph(&restricted_game), |_, _, _| true);

            // For every strongly connected component, check if it is winning for the player
            for block in 0..scc_partition.num_of_blocks() {
                for element in 0..scc_partition.len() {
                    if scc_partition.partition()[element] == block {
                        let vertex = VertexIndex::new(element);

                        if restricted_game.priority(vertex) == player.to_index() {
                            // This SCC contains a vertex with the current priority, so it is winning for the player
                            winning_set.set(vertex.value(), true);
                        }
                    }
                }
            }
        }

        if winning_set != solution[player.to_index()] {
            panic!("The proposed winning set for player {} is incorrect", player);
        }
    }
}

/// Computes the set of vertices reachable from the given initial vertices in
/// the given graph.
fn _reachability<G>(graph: &G, initial: Vec<G::VertexIndex>) -> Set
where
    G: Graph,
    G::VertexIndex: MercIndex<Target = usize>,
{
    // The set of vertices that are already visited.
    let mut visited = bitvec![usize, Lsb0; 0; graph.num_of_vertices()];
    let mut queue = initial;

    for v in &queue {
        visited.set(v.index(), true);
    }

    while let Some(v) = queue.pop() {
        for (_label, w) in graph.outgoing_edges(v) {
            if !visited.get(w.index()).expect("Vertex must be in the reachable vector") {
                visited.set(w.index(), true);
                queue.push(w);
            }
        }
    }

    visited
}

/// A sub-game induced by a strategy on a parity game.
struct SubGame<'a, G: PG> {
    restricted: Set,

    /// The strategy that is applied to the game.
    strategy: &'a Strategy,

    /// The player that owns the strategy.
    player: Player,

    /// The game that is being restricted.
    game: &'a G,

    /// The minimum priority in the sub-game.
    max_priority: Priority,
}

impl<G: PG> SubGame<'_, G> {
    /// Create a new sub-game induced by the given strategy on the given game.
    pub fn new<'a>(game: &'a G, player: Player, strategy: &'a Strategy, max_priority: Priority) -> SubGame<'a, G> {
        let mut restricted = bitvec![usize, Lsb0; 0; game.num_of_vertices()];
        for vertex in game.iter_vertices() {
            if game.priority(vertex) <= max_priority {
                restricted.set(vertex.value(), true);
            }
        }

        SubGame {
            restricted,
            game,
            player,
            strategy,
            max_priority,
        }
    }
}

impl<G: PG> PG for SubGame<'_, G> {
    type Label = G::Label;

    fn iter_vertices(&self) -> impl Iterator<Item = VertexIndex> + '_ {
        // Only consider vertices that are below the maximum priority.
        self.game
            .iter_vertices()
            .filter(|v| self.restricted.get(**v).expect("Vertex must be in the restricted set") == true)
    }

    fn outgoing_edges<'a>(&'a self, vertex_index: VertexIndex) -> impl Iterator<Item = Edge<'a, G::Label>> + 'a {
        self.game.outgoing_edges(vertex_index).filter(move |edge| {
            // Only consider edges to vertices that are in the restricted set and follow the strategy.
            self.restricted
                .get(*edge.to())
                .expect("Vertex must be in the restricted set")
                == true
                && (self.game.owner(edge.to()) != self.player || self.strategy.get(vertex_index) == Some(&edge.to()))
        })
    }

    fn highest_priority(&self) -> Priority {
        // Determine the highest priority in the restricted set.
        self.iter_vertices()
            .fold(Priority::new(0), |max, p| max.max(self.game.priority(p)))
    }

    fn is_total(&self) -> bool {
        // The sub-game is total if every vertex in the restricted set has at least one outgoing edge.
        self.iter_vertices().all(|v| self.outgoing_edges(v).next().is_some())
    }

    fn priority(&self, vertex: VertexIndex) -> Priority {
        if self.game.priority(vertex) == self.max_priority {
            // Return the priority for the current player.
            Priority::new(self.player.to_index())
        } else {
            // Return the priority for the opponent.
            Priority::new(self.player.opponent().to_index())
        }
    }

    delegate! {
        to self.game {
            fn initial_vertex(&self) -> VertexIndex;
            fn num_of_vertices(&self) -> usize;
            fn num_of_edges(&self) -> usize;
            fn owner(&self, vertex: VertexIndex) -> Player;
        }
    }
}
