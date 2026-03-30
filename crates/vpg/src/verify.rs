use std::collections::HashSet;
use std::fmt::Debug;

use bitvec::bitvec;
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use delegate::delegate;

use itertools::Itertools;
use log::trace;
use merc_collections::scc_decomposition;
use merc_collections::BlockIndex;
use merc_collections::BlockPartition;
use merc_utilities::MercIndex;

use crate::check_partition;
use crate::AsGraph;
use crate::Edge;
use crate::Player;
use crate::Predecessors;
use crate::Priority;
use crate::Set;
use crate::Strategy;
use crate::VertexIndex;
use crate::PG;

/// Verifies that a proposed solution is valid for the given parity game and
/// strategies.
///
/// # Details
///
/// This is done by restricting the `pg` according to the strategy for each
/// player, and computing the solution for the induced solitaire game.
pub fn verify_solution<G: PG>(pg: &G, solution: &[Set; 2], strategy: &[Strategy; 2]) {
    debug_assert!(pg.is_total(), "Verifying requires a total parity game");

    // The set of all vertices in the game.
    let vertices = bitvec![usize, Lsb0; 1; pg.num_of_vertices()];

    // Check that the input solution is a proper partitioning
    check_partition(&solution[0], &solution[1], &vertices);

    // We check both players' strategies
    for player in [Player::Even, Player::Odd] {
        // Check that the strategies are consistent with the ownership of the vertices
        strategy[player.to_index()].check_consistent(pg, player);

        // Restricted game according to the strategy for the current player
        let restricted = Restricted::new(pg, player, &strategy[player.to_index()]);

        // The opponent is the solitaire player.
        if solve_solitaire_game(&restricted, player.opponent()) != solution[player.opponent().to_index()] {
            panic!("The proposed winning set for player {} is incorrect", player);
        }
    }
}

/// Solves a solitaire game for the given player.
///
/// # Details
///
/// This is done by considering all subgames Gi restricted to priority `i`
/// belonging to `player`, and solving the simple solitaire game on each of these subgames.
fn solve_solitaire_game<G: PG>(pg: &G, player: Player) -> BitVec {
    debug_assert!(
        pg.iter_vertices().all(|vertex| pg.owner(vertex) == Player::Even)
            || pg.iter_vertices().all(|vertex| pg.owner(vertex) == Player::Odd),
        "solve_solitair_game requires a solitaire game"
    );

    let mut winning_vertices = bitvec![usize, Lsb0; 0; pg.num_of_vertices()];

    for priority in 0..=pg.highest_priority().value() {
        if Player::from_priority(&(Priority::new(priority))) != player {
            // Skip priorities that do not belong to the current player
            continue;
        }

        // Restrict the game to the current priority.
        trace!("Solving subgame for max-priority {}", priority);
        let prio_subgame = PrioSubgame::new(pg, Priority::new(priority));

        let subgame_solution = solve_solitaire_simple(&prio_subgame, player);
        for vertex in prio_subgame.iter_vertices() {
            if subgame_solution[*vertex] {
                winning_vertices.set(*vertex, true)
            }
        }
    }

    let predecessors = Predecessors::new(pg);
    backward_reachability(&predecessors, winning_vertices)
}

/// Solves a solitaire game that only contains two priorities, where `player`
/// should be the player that makes decisions.
///
/// # Details
///
/// For a solitaire game with only two priorities, the winning set for the player
/// is exactly those vertices that can reach a strongly connected component that
/// contains the highest priority. Note that the highest priority should belong
/// to the player.
fn solve_solitaire_simple<G: PG>(pg: &G, player: Player) -> BitVec {
    trace!("Subgame {{ {:?} }}, player {}", pg.iter_vertices().format(", "), player);

    let scc_partition = scc_decomposition(&AsGraph(pg), |_, _, _| true);

    // Determine vertices that are winning for the player in the restricted game, which are those that can reach a vertex with the current priority.
    let mut winning_vertices = bitvec![usize, Lsb0; 0; pg.num_of_vertices()];

    let subgame_vertices: HashSet<VertexIndex> = HashSet::from_iter(pg.iter_vertices());

    // Convert to block partition to compute reachability on the SCCs
    let block_partition = BlockPartition::<()>::from_indexed_partition(&scc_partition);
    for scc in (0..scc_partition.num_of_blocks()).map(BlockIndex::new) {
        if is_trivial_scc(pg, &block_partition, scc, &subgame_vertices) {
            trace!("SCC {} is trivial, skipping", scc);
            continue;
        }

        if block_partition
            .iter_block(scc)
            // TODO: This assumes that this is the highest priority, so priorities (0,1) for odd and (1,2) for even.
            .any(|i| {
                subgame_vertices.contains(&VertexIndex::new(i))
                    && Player::from_priority(&pg.priority(VertexIndex::new(i))) == player
            })
        {
            for vertex in block_partition.iter_block(scc) {
                if subgame_vertices.contains(&VertexIndex::new(vertex)) {
                    winning_vertices.set(vertex, true);
                    trace!(
                        "Player {} wins {} in SCC {}",
                        player,
                        vertex,
                        scc
                    );
                }
            }
        }
    }

    let predecessors = Predecessors::new(pg);
    backward_reachability(&predecessors, winning_vertices)
}

/// Returns true if the given SCC is trivial, i.e., it does not contain any
/// cycles. This is the case if the SCC contains only one vertex and that vertex
/// does not have a self-loop.
///
/// Only vertices contained in `subgame_vertices` are considered part of the SCC.
fn is_trivial_scc<G: PG, T: Clone + Debug + Default>(
    pg: &G,
    partition: &BlockPartition<T>,
    block: BlockIndex,
    subgame_vertices: &HashSet<VertexIndex>,
) -> bool {
    let vertices_in_subgame: Vec<usize> = partition
        .iter_block(block)
        .filter(|&i| subgame_vertices.contains(&VertexIndex::new(i)))
        .collect();

    if vertices_in_subgame.len() != 1 {
        return false;
    }

    let vertex = VertexIndex::new(vertices_in_subgame[0]);
    !pg.outgoing_edges(vertex).any(|edge| edge.to() == vertex)
}

/// Computes the set of vertices reachable from the given initial vertices in
/// the given graph.
fn backward_reachability(predecessors: &Predecessors, mut initial: BitVec) -> BitVec {
    // The set of vertices that are already visited.
    let visited = &mut initial;
    let mut queue: Vec<VertexIndex> = Vec::new();

    for v in visited.iter_ones() {
        queue.push(VertexIndex::new(v));
    }

    while let Some(v) = queue.pop() {
        for w in predecessors.predecessors(v) {
            if !visited.get(w.index()).expect("Vertex must be in the reachable vector") {
                trace!("Reached vertex {} from vertex {}", w, v);
                visited.set(w.index(), true);
                queue.push(w);
            }
        }
    }

    initial
}

/// A subgame of a parity game that is induced by taking the strategy for the
/// given player into account.
struct Restricted<'a, G: PG> {
    /// The game that is being restricted.
    game: &'a G,

    /// The strategy that is applied to the game.
    strategy: &'a Strategy,

    /// The player that owns the strategy.
    player: Player,
}

impl<G: PG> Restricted<'_, G> {
    /// Create a new sub-game induced by the given strategy on the given game.
    pub fn new<'a>(game: &'a G, player: Player, strategy: &'a Strategy) -> Restricted<'a, G> {
        Restricted { game, player, strategy }
    }
}

impl<G: PG> PG for Restricted<'_, G> {
    type Label = G::Label;

    fn outgoing_edges<'a>(&'a self, vertex_index: VertexIndex) -> impl Iterator<Item = Edge<'a, G::Label>> + 'a {
        self.game.outgoing_edges(vertex_index).filter(move |edge| {
            // Only consider edges that follow the strategy.
            self.game.owner(vertex_index) != self.player || self.strategy.get(vertex_index) == Some(&edge.to())
        })
    }

    fn is_total(&self) -> bool {
        // The sub-game is total if every vertex in the restricted set has at least one outgoing edge.
        self.iter_vertices().all(|v| self.outgoing_edges(v).next().is_some())
    }

    delegate! {
        to self.game {
            fn initial_vertex(&self) -> VertexIndex;
            fn num_of_vertices(&self) -> usize;
            fn num_of_edges(&self) -> usize;
            fn owner(&self, vertex: VertexIndex) -> Player;
            fn iter_vertices(&self) -> impl Iterator<Item = VertexIndex> + '_;
            fn priority(&self, vertex: VertexIndex) -> Priority;
            fn highest_priority(&self) -> Priority;
        }
    }
}

/// A subgame Gi induced by mapping all priorities equal to `max_priority` to
/// the player's priority, and all other priorities to the opponent's priority.
struct PrioSubgame<'a, G: PG> {
    restricted: Set,

    /// The game that is being restricted.
    game: &'a G,

    /// The maximum priority in the sub-game.
    max_priority: Priority,
}

impl<G: PG> PrioSubgame<'_, G> {
    /// Create a new sub-game induced by the given strategy on the given game.
    pub fn new<'a>(game: &'a G, max_priority: Priority) -> PrioSubgame<'a, G> {
        let mut restricted = bitvec![usize, Lsb0; 0; game.num_of_vertices()];
        for vertex in game.iter_vertices() {
            if game.priority(vertex) <= max_priority {
                restricted.set(vertex.value(), true);
            }
        }

        PrioSubgame {
            game,
            restricted,
            max_priority,
        }
    }
}

impl<G: PG> PG for PrioSubgame<'_, G> {
    type Label = G::Label;

    fn iter_vertices(&self) -> impl Iterator<Item = VertexIndex> + '_ {
        // Only consider vertices that are below the maximum priority.
        self.restricted.iter_ones().map(|index| VertexIndex::new(index))
    }

    fn outgoing_edges<'a>(&'a self, vertex_index: VertexIndex) -> impl Iterator<Item = Edge<'a, G::Label>> + 'a {
        debug_assert!(
            self.restricted
                .get(vertex_index.value())
                .expect("Vertex must be in the restricted set"),
            "Vertex must be in the restricted set"
        );

        self.game.outgoing_edges(vertex_index).filter(move |edge| {
            // Only consider edges to vertices that are in the restricted set and follow the strategy.
            self.restricted
                .get(*edge.to())
                .expect("Vertex must be in the restricted set")
                == true
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
        debug_assert!(
            self.restricted
                .get(vertex.value())
                .expect("Vertex must be in the restricted set"),
            "Vertex must be in the restricted set"
        );

        if self.game.priority(vertex) == self.max_priority {
            // Return the priority for the opponent.
            if self.max_priority.is_multiple_of(2) {
                Priority::new(2)
            } else {
                Priority::new(1)
            }
        } else {
            // Return the priority for the opponent.
            if self.max_priority.is_multiple_of(2) {
                Priority::new(1)
            } else {
                Priority::new(0)
            }
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

/// A parity game where every player is now owned by given player, making this a
/// solitaire game.
#[cfg(test)]
struct SolitaireGame<'a, G: PG> {
    game: &'a G,

    player: Player,
}

#[cfg(test)]
impl<G: PG> SolitaireGame<'_, G> {
    /// Create a new solitaire game induced by the given strategy on the given game.
    fn new<'a>(game: &'a G, player: Player) -> SolitaireGame<'a, G> {
        SolitaireGame { game, player }
    }
}

#[cfg(test)]
impl<G: PG> PG for SolitaireGame<'_, G> {
    type Label = G::Label;

    fn owner(&self, _vertex: VertexIndex) -> Player {
        // All vertices are owned by the given player, making it a solitaire game.
        self.player
    }

    delegate! {
        to self.game {
            fn initial_vertex(&self) -> VertexIndex;
            fn num_of_vertices(&self) -> usize;
            fn num_of_edges(&self) -> usize;
            fn iter_vertices(&self) -> impl Iterator<Item = VertexIndex> + '_;
            fn outgoing_edges<'a>(&'a self, vertex_index: VertexIndex) -> impl Iterator<Item = Edge<'a, G::Label>> + 'a;
            fn priority(&self, vertex: VertexIndex) -> Priority;
            fn is_total(&self) -> bool;
            fn highest_priority(&self) -> Priority;
        }
    }
}

#[cfg(test)]
mod tests {
    use merc_io::DumpFiles;
    use merc_utilities::random_test;

    use crate::random_parity_game;
    use crate::solve_zielonka;
    use crate::verify::solve_solitaire_game;
    use crate::verify::SolitaireGame;
    use crate::write_pg;
    use crate::Player;

    #[test]
    fn test_random_solitaire_game() {
        random_test(100, |rng| {
            let mut files = DumpFiles::new("test_random_solitaire_game");
            let pg = random_parity_game(rng, true, 5, 3, 3);
            let solitaire = SolitaireGame::new(&pg, Player::Even);
            files.dump("input.pg", |writer| write_pg(writer, &solitaire)).unwrap();

            let solution = solve_solitaire_game(&solitaire, Player::Even);
            let (expected_solution, _expected_strategy) = solve_zielonka(&solitaire);

            assert_eq!(
                solution, expected_solution[0],
                "The solution for the solitaire solver should match the zielonka solution"
            );
        })
    }
}
