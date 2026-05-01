//! Authors: Maurice Laveaux
//!
//! Implementation of the priority promotion algorithm introduced in:
//!
//! > Massimo Benerecetti, Daniele Dell'Erba, and Fabio Mogavero. Solving
//! > Parity Games via Priority Promotion, pages 270-290. Springer International
//! > Publishing, Cham, 2016.
//!
//! Given the current triple (region_function, strategy, prio), referred to as
//! state, a region R is extracted by means of the query function. An alpha-region
//! is a set of vertices R and a witness strategy sigma. So that for all plays
//! consistent with sigma, they either stay within R and are winning for alpha.
//! Or they escape R via vertices having the highest priority in the game. The
//! initial state is (priority_function, empty_strategy, min(range(priority_function)))
//!
//! If the region R is a dominion in the whole game, the dominion is removed from
//! the game and the algorithm runs on the remaining game. If the extracted region
//! is open in the subgame G >= prio. Which is the subgame with only vertices
//! having greater or equal priority then prio. The next state becomes
//! (region_function\[R -> prio\], strategy\*, min(range(region_function >= prio))).
//!
//! If the alpha-region is a dominion in the subgame G >= prio, the lowest priority
//! region that the opponent can flee to is determined in [`PriorityPromotionSolver::promote_sub_dominion`].
//! And then the next state becomes (region_function\*\[R -> prio\*\], strategy\*, prio\*), and
//! in region_function\* all regions below prio\* are reset to the original priority.
//!
//! The strategy\* was not presented in the paper, but was partially determined
//! by the follow up paper Improving Priority Promotion for parity games. For
//! the attractor for some region R the strategy is determined by the witness
//! that alpha can reach R. For vertices inside the region R a witness core strategy
//! sigma is given. For all vertices inside R where no strategy is defined yet
//! an arbitrary successor inside R is taken to complete the strategy.

use std::collections::VecDeque;

use bitvec::bitvec;
use bitvec::order::Lsb0;
use log::debug;
use log::trace;

use crate::PG;
use crate::Player;
use crate::Pred;
use crate::Predecessors;
use crate::Priority;
use crate::Strategy;
use crate::VertexIndex;
use crate::zielonka::Set;

/// The sentinel value indicating that a vertex has been solved (its dominion was found).
const COMPUTED_REGION: usize = usize::MAX;

/// Solves the given parity game using the priority promotion algorithm.
///
/// Returns the winning sets for both players and their winning strategies.
pub fn solve_priority_promotion<G: PG>(game: &G) -> ([Set; 2], [Strategy; 2]) {
    debug_assert!(
        game.is_total(),
        "Priority promotion solver requires a total parity game"
    );

    let mut solver = PriorityPromotionSolver::new(game);
    let strategy = solver.solve();

    // Convert region_function into winning sets based on the strategy.
    let mut W0 = bitvec![usize, Lsb0; 0; game.num_of_vertices()];
    let mut W1 = bitvec![usize, Lsb0; 0; game.num_of_vertices()];
    let mut S0 = Strategy::new();
    let mut S1 = Strategy::new();

    for v in game.iter_vertices() {
        let prio = solver.region_function[*v];
        debug_assert_eq!(prio, COMPUTED_REGION, "All vertices should be solved");

        // Determine the winner from the strategy: if a vertex has a strategy
        // entry, the player owning it wins; otherwise check which dominion it
        // belonged to by looking at the final_winner array.
        let winner = solver.final_winner[*v];
        match winner {
            Player::Even => {
                W0.set(*v, true);
                if game.owner(v) == Player::Even {
                    if let Some(&target) = strategy.get(v) {
                        S0.set(v, target);
                    }
                }
            }
            Player::Odd => {
                W1.set(*v, true);
                if game.owner(v) == Player::Odd {
                    if let Some(&target) = strategy.get(v) {
                        S1.set(v, target);
                    }
                }
            }
        }
    }

    ([W0, W1], [S0, S1])
}

/// Internal solver state for the priority promotion algorithm.
struct PriorityPromotionSolver<'a, G: PG> {
    game: &'a G,

    /// Precomputed predecessors for backward iteration.
    predecessors: Predecessors<'a>,

    /// Maps each vertex to its current region priority.
    /// `COMPUTED_REGION` indicates a solved vertex.
    region_function: Vec<usize>,

    /// Stores a list of vertices not yet solved by the algorithm.
    unsolved: Vec<VertexIndex>,

    /// Count the number of vertices per region, to speed up [`Self::next_priority`].
    regions: Vec<usize>,

    /// This is a reused queue with vertices to compute the attractor set from.
    todo: VecDeque<VertexIndex>,

    /// The number of promotions required.
    promotions: usize,

    /// The number of dominions found.
    dominions: usize,

    /// Records the winning player for each vertex (set when a dominion is found).
    final_winner: Vec<Player>,
}

impl<'a, G: PG> PriorityPromotionSolver<'a, G> {
    /// Creates a new priority promotion solver for the given parity game.
    fn new(game: &'a G) -> Self {
        let num_vertices = game.num_of_vertices();

        // The lowest priority in the game (the highest number).
        let mut lowest_region: usize = 0;

        // Set region_function to the original priorities and initialize the mapping.
        let mut region_function = vec![0usize; num_vertices];
        let mut unsolved = Vec::with_capacity(num_vertices);

        for v in game.iter_vertices() {
            let prio = *game.priority(v);
            region_function[*v] = prio;
            unsolved.push(v);
            lowest_region = lowest_region.max(prio);
        }

        // Initialize all regions that have some vertices.
        let mut regions = vec![0usize; lowest_region + 1];
        for &r in &region_function {
            regions[r] += 1;
        }

        PriorityPromotionSolver {
            game,
            predecessors: Predecessors::new(game),
            region_function,
            unsolved,
            regions,
            todo: VecDeque::new(),
            promotions: 0,
            dominions: 0,
            final_winner: vec![Player::Even; num_vertices],
        }
    }

    /// Compute winning strategies by means of priority promotion, follows the
    /// paper as closely as possible.
    ///
    /// # Details
    ///
    /// Important note: instead of actually repeatedly removing dominions from
    /// the game, the game is kept the same but the region_function is used to
    /// determine which vertices still are not solved. This is done because
    /// removing subgames allocates new memory repeatedly and parity games
    /// can be huge.
    fn solve(&mut self) -> Strategy {
        let mut strategy = Strategy::new();

        // Find the lowest priority in the game.
        let mut prio = self.next_priority(0);

        // The algorithm was tail recursive so can also be written as iteration.
        loop {
            self.query(&mut strategy, prio);

            if self.is_open(prio, true) {
                debug!(
                    "Newly computed region is open in the subgame, with p = {}",
                    prio
                );
                self.print_region(prio);

                // Keep the new region_function and substrategy, but go to the next priority.
                prio = self.next_priority(prio + 1);
            } else if !self.is_open(prio, false) {
                // This is a dominion D in the whole game, compute the attractor
                // for this region.
                debug_assert!(self.todo.is_empty());

                for &v in &self.unsolved {
                    if self.region_function[*v] == prio {
                        self.todo.push_back(v);
                    }
                }

                self.compute_attractor(&mut strategy, prio, false);

                // Remove the dominion from the game and keep the unsolved vertices, also reset
                // lower priorities and set region of prio to the COMPUTED_REGION.
                debug!("Found the dominion D, with p = {}", prio);
                self.print_region(prio);

                // Record the winner for this dominion.
                let winner = Player::from_priority(&Priority::new(prio));
                for v in self.game.iter_vertices() {
                    if self.region_function[*v] == prio {
                        self.final_winner[*v] = winner;
                    }
                }

                // Reset the unsolved set and remove all regions, also add one dominion to statistics.
                self.unsolved.clear();
                self.regions.fill(0);
                self.dominions += 1;

                for v in self.game.iter_vertices() {
                    if self.region_function[*v] == prio {
                        // Assign a special region indicating that it's solved.
                        self.region_function[*v] = COMPUTED_REGION;
                    } else if self.region_function[*v] != COMPUTED_REGION {
                        let original_prio = *self.game.priority(v);
                        self.region_function[*v] = original_prio;
                        strategy.remove(v);

                        // Add the not solved vertices to the unsolved set and add vertices to their region.
                        self.unsolved.push(v);
                        self.regions[original_prio] += 1;
                    }
                }

                if self.unsolved.is_empty() {
                    break; // Stop the algorithm, as all the vertices were solved.
                }

                // Reset the game and find the lowest priority in the game.
                prio = self.next_priority(0);
            } else {
                // The game is a dominion, but only in the subgame, so promote its priority.
                debug!("Promoted dominion D, with p = {}", prio);
                prio = self.promote_sub_dominion(&mut strategy, prio);
                debug!(" to {}", prio);
                self.print_region(prio);
            }
        }

        debug!(
            "{} dominions found, and {} promotions required",
            self.dominions, self.promotions
        );

        strategy
    }

    /// From the state (region_function, strategy, prio) compute the new alpha-region
    /// R and update region_function\[R -> p\]. The strategy will be updated in
    /// [`Self::compute_attractor`]. The unsolved set is used to quickly iterate unsolved vertices.
    /// The todo queue is passed to be reused by [`Self::compute_attractor`].
    fn query(&mut self, strategy: &mut Strategy, prio: usize) {
        // Make sure nothing else is stored in the todo.
        debug_assert!(self.todo.is_empty());

        // R* = region_function^-1(prio), this results in the todo for the attractor
        // computation, the initial set essentially.
        for &v in &self.unsolved {
            if self.region_function[*v] == prio {
                self.todo.push_back(v);
            }
        }

        // (region_function[R -> prio], strategy*) <- computeAttractor_G(todo, strategy
        // restricted to todo)
        self.compute_attractor(strategy, prio, true);
    }

    /// Compute the attractor set A for vertices in todo, with alpha being prio mod 2.
    /// `in_subgraph` indicates that only vertices in game >= prio are considered. This
    /// updates region_function\[A -> prio\]. The strategy is changed for alpha for the
    /// attraction witness. The remaining vertices of alpha without a strategy can pick
    /// any vertex inside A as witness.
    fn compute_attractor(&mut self, strategy: &mut Strategy, prio: usize, in_subgraph: bool) {
        let alpha = Player::from_priority(&Priority::new(prio));

        // O(V): Compute the attractor set to the alpha-region.
        while let Some(w) = self.todo.pop_front() {
            // Check all predecessors v of w.
            for v in self.predecessors.predecessors(w) {
                // Skip predecessors that are already in the attractor set, also skip
                // vertices outside the subgame G >= prio. Or vertices that are computed.
                if self.region_function[*v] == prio
                    || self.region_function[*v] == COMPUTED_REGION
                    || (in_subgraph && self.region_function[*v] < prio)
                {
                    continue;
                }

                if self.game.owner(v) == alpha {
                    // sigma(v) = w, a valid strategy for alpha is to pick a successor in A.
                    strategy.set(v, w);
                } else {
                    // Check if all successors (v, x) subset A, thus if they end up in a vertex with prio.
                    let is_subset = self.game.outgoing_edges(v).all(|edge| {
                        let x = edge.to();

                        // Skip vertices that are not considered in the subgraph G >= prio
                        // or that already belong to COMPUTED_REGION.
                        if self.region_function[*x] == prio || self.region_function[*x] == COMPUTED_REGION {
                            return true;
                        }

                        // Either only take vertices in G >= prio or all when in_subgraph is false.
                        if self.region_function[*x] > prio || !in_subgraph {
                            return false;
                        }

                        true
                    });

                    if !is_subset {
                        continue; // not in the attractor set yet!
                    }

                    // For opponent controlled vertices no strategy exists, so
                    // every possible outgoing edge is losing.
                }

                // Add a vertex to their new region and remove from the old one.
                self.regions[self.region_function[*v]] -= 1;
                self.regions[prio] += 1;

                // When this part is reached, all liberties of v are gone or v belongs
                // to alpha, so add vertex v to the attractor set.
                self.region_function[*v] = prio;
                self.todo.push_back(v);
            }
        }

        // R \ domain(tau restricted to R*), essentially vertices in R belonging to
        // alpha where no strategy is defined yet. These can pick an arbitrary
        // successor that can reach R \ R*, these already have an attraction
        // strategy so that is always fine.
        for &v in &self.unsolved {
            if self.region_function[*v] == prio && self.game.owner(v) == alpha && strategy.get(v).is_none() {
                for edge in self.game.outgoing_edges(v) {
                    let w = edge.to();

                    if self.region_function[*w] == prio {
                        // There exists some (v, w) in E such that w belongs to R (has r[w] == prio).
                        strategy.set(v, w);
                        break;
                    }
                }
            }
        }
    }

    /// Determine whether the alpha-region with priority `prio` is open in G, or
    /// in G >= prio (indicated by `in_subgraph`). This means that for all vertices
    /// v with region_function\[v\] equal to prio, this is set R. When v belongs
    /// to alpha, determined by prio mod 2, there is some witness successor in R.
    /// For opponent vertices all successors lead to R, no witness to escape basically.
    fn is_open(&self, prio: usize, in_subgraph: bool) -> bool {
        let alpha = Player::from_priority(&Priority::new(prio));

        // O(V): Loop over unsolved vertices and find vertices belonging to region with prio.
        for &v in &self.unsolved {
            if self.region_function[*v] == prio {
                if self.game.owner(v) != alpha {
                    // For all (v, u) in E, u should belong to R.
                    for edge in self.game.outgoing_edges(v) {
                        let u = edge.to();

                        // There is an edge from opponent to a vertex in the subgraph or in the whole graph.
                        if self.region_function[*u] != COMPUTED_REGION
                            && ((in_subgraph && self.region_function[*u] > prio)
                                || (!in_subgraph && self.region_function[*u] != prio))
                        {
                            return true;
                        }
                    }
                } else {
                    // If there exists a (v, u) to R its closed.
                    let is_open = self
                        .game
                        .outgoing_edges(v)
                        .all(|edge| self.region_function[*edge.to()] != prio);

                    if is_open {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Promote a sub dominion D to the maximum region lower than prio that the
    /// opponent can reach. This updates region_function\[D -> prio\*\] and resets
    /// all priorities lower than prio\* to the original priority function. The
    /// strategy is updated by means of [`Self::compute_attractor`]. And lower strategies
    /// are set to `None` (no strategy known).
    ///
    /// # Details
    ///
    /// This is referred to as r\* = bep(R, r) in the paper (best escape priority).
    /// For every opponent vertex this is the lowest priority (highest value in
    /// min-prio games) that it can reach. The region it can reach belongs to alpha,
    /// otherwise it would be attracted in some earlier state.
    fn promote_sub_dominion(&mut self, strategy: &mut Strategy, prio: usize) -> usize {
        let alpha = Player::from_priority(&Priority::new(prio));

        // O(V): It is only a dominion in the subgraph, determine highest p < prio
        // that opponent can escape to.
        let mut promotion: usize = 0;

        for &v in &self.unsolved {
            if self.region_function[*v] == prio && self.game.owner(v) != alpha {
                // For all (v, u) in E collect the highest priority smaller than prio that opponent can flee to.
                for edge in self.game.outgoing_edges(v) {
                    let u = edge.to();

                    if self.region_function[*u] < prio {
                        promotion = promotion.max(self.region_function[*u]);
                    }
                }
            }
        }

        self.promotions += 1;

        // Here the prio region is promoted to the new priority and all lower positions
        // are reset.
        for &v in &self.unsolved {
            if self.region_function[*v] == prio {
                // Promote the current region to the promotion priority.
                self.regions[self.region_function[*v]] -= 1;
                self.region_function[*v] = promotion;
                self.regions[self.region_function[*v]] += 1;
            } else if self.region_function[*v] > promotion {
                // Reset all vertices lower to the original priorities, remove the strategy.
                self.regions[self.region_function[*v]] -= 1;
                self.region_function[*v] = *self.game.priority(v);
                strategy.remove(v);
                self.regions[self.region_function[*v]] += 1;
            }
        }

        promotion
    }

    /// Print the vertices with region_function\[v\] equal to prio, representing the region.
    ///
    /// This costs O(V) so only enable this in debug.
    fn print_region(&self, prio: usize) {
        if log::log_enabled!(log::Level::Trace) {
            let vertices: Vec<_> = self
                .unsolved
                .iter()
                .filter(|&&v| self.region_function[*v] == prio)
                .map(|v| v.value())
                .collect();
            trace!("alpha-region[{}] = {{ {} }}", prio, vertices.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
        }
    }

    /// Computes min(rng(region_function >= prio)), so the next lower priority,
    /// greater or equal to prio, that some vertex has.
    ///
    /// Starting from the current priority, find the next region that exists.
    /// This should never go out of bounds as the lowest region will always be a dominion.
    fn next_priority(&self, prio: usize) -> usize {
        let mut p = prio;
        while self.regions[p] == 0 {
            p += 1;
            debug_assert!(p < self.regions.len(), "next_priority went out of bounds");
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use merc_io::DumpFiles;
    use merc_utilities::random_test;

    use crate::random_parity_game;
    use crate::solve_zielonka;
    use crate::verify_solution;
    use crate::write_pg;

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow for this test.
    fn test_random_priority_promotion_solver() {
        random_test(100, |rng| {
            let mut files = DumpFiles::new("test_random_priority_promotion_solver");
            let game = random_parity_game(rng, true, 100, 5, 3);

            files.dump("input.pg", |writer| write_pg(writer, &game)).unwrap();

            let (solution, strategy) = solve_priority_promotion(&game);
            verify_solution(&game, &solution, &strategy);
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_priority_promotion_matches_zielonka() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, true, 50, 4, 3);

            let (pp_solution, _) = solve_priority_promotion(&game);
            let (zielonka_solution, _) = solve_zielonka(&game, false);

            assert_eq!(
                pp_solution[0], zielonka_solution[0],
                "Even winning sets differ between priority promotion and Zielonka"
            );
            assert_eq!(
                pp_solution[1], zielonka_solution[1],
                "Odd winning sets differ between priority promotion and Zielonka"
            );
        });
    }
}
