use log::debug;
use log::trace;

use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;

use merc_utilities::MercError;

use crate::Player;
use crate::Repeat;
use crate::SymbolicParityGame;

/// The winning sets of both players in a symbolic parity game, indexed by [`Player::to_index`].
pub struct SymbolicSolution {
    pub winning: [LDDFunction; 2],
}

impl SymbolicSolution {
    /// Returns the winner of `vertex`, or `None` if `vertex` was not part of the solved game.
    pub fn winner(&self, vertex: &LDDFunction) -> Option<Player> {
        if includes(&self.winning[Player::Even.to_index()], vertex).expect("LDD operation should not run out of memory")
        {
            Some(Player::Even)
        } else if includes(&self.winning[Player::Odd.to_index()], vertex)
            .expect("LDD operation should not run out of memory")
        {
            Some(Player::Odd)
        } else {
            None
        }
    }
}

/// Returns whether `vertex` is a subset of `winning` (mCRL2's Sylvan-native `includes`, which
/// oxidd has no equivalent for).
fn includes(winning: &LDDFunction, vertex: &LDDFunction) -> Result<bool, MercError> {
    Ok(vertex.minus(winning)?.is_empty())
}

/// Solves `game` restricted to `vertices`, with `sinks` as the deadlocks to resolve (assigned to
/// the opponent of their owner — see [`SymbolicParityGame::compute_total_graph`]), and returns
/// the winner of `initial_vertex` together with the full winning partition.
///
/// Mirrors `symbolic_pbessolve_algorithm::solve`, with its `partial_solution` and
/// `allow_early_termination` parameters dropped for the MVP: there is no partial solution to
/// resume from yet, and early termination (skip the recursive solve entirely when
/// `compute_total_graph`'s sink handling already resolved `initial_vertex`) is unconditional.
pub fn solve_symbolic_zielonka(
    game: &SymbolicParityGame,
    initial_vertex: &LDDFunction,
    vertices: &LDDFunction,
    sinks: &LDDFunction,
) -> Result<(Player, SymbolicSolution), MercError> {
    let empty = game.manager().with_manager_shared(LDDFunction::empty_set)?;
    let mut winning = [empty.clone(), empty];

    let total = game.compute_total_graph(vertices, sinks, &mut winning)?;

    if !includes(&winning[0], initial_vertex)? && !includes(&winning[1], initial_vertex)? {
        let solution = zielonka(game, &total, 0)?;
        winning[0] = winning[0].union(&solution.winning[0])?;
        winning[1] = winning[1].union(&solution.winning[1])?;
    }

    if includes(&winning[0], initial_vertex)? {
        Ok((Player::Even, SymbolicSolution { winning }))
    } else if includes(&winning[1], initial_vertex)? {
        Ok((Player::Odd, SymbolicSolution { winning }))
    } else {
        Err("solve_symbolic_zielonka: initial vertex was not resolved by the solver".into())
    }
}

/// The recursive Zielonka solver, restricted to the vertex set `v`.
fn zielonka(game: &SymbolicParityGame, v: &LDDFunction, depth: usize) -> Result<SymbolicSolution, MercError> {
    let indent = Repeat::new(" ", depth);

    if v.is_empty() {
        let empty = game.manager().with_manager_shared(LDDFunction::empty_set)?;
        return Ok(SymbolicSolution {
            winning: [empty.clone(), empty],
        });
    }

    let vplayer = game.players(v)?;
    let (priority, u) = game
        .max_priority(v)?
        .expect("v is non-empty, so it must contain at least one priority");
    let alpha = Player::from_priority(priority);
    let not_alpha = alpha.opponent();

    debug!("{indent}|V| = {}, priority = {priority}, player = {alpha}", v.len());
    trace!("{indent}U (highest priority vertices) has {} elements", u.len());

    let a = game.attractor(alpha, &u, v, &vplayer, None)?;
    trace!("{indent}A (attractor of U) has {} elements", a.len());

    let v_minus_a = v.minus(&a)?;
    let solution_v_minus_a = zielonka(game, &v_minus_a, depth + 1)?;

    let solution = if solution_v_minus_a.winning[not_alpha.to_index()].is_empty() {
        let mut winning = [empty_like(game)?, empty_like(game)?];
        winning[alpha.to_index()] = a.union(&solution_v_minus_a.winning[alpha.to_index()])?;
        SymbolicSolution { winning }
    } else {
        let b = game.attractor(
            not_alpha,
            &solution_v_minus_a.winning[not_alpha.to_index()],
            v,
            &vplayer,
            None,
        )?;
        trace!(
            "{indent}B (attractor of the opponent's win in V \\ A) has {} elements",
            b.len()
        );

        let v_minus_b = v.minus(&b)?;
        let mut solution = zielonka(game, &v_minus_b, depth + 1)?;
        solution.winning[not_alpha.to_index()] = solution.winning[not_alpha.to_index()].union(&b)?;
        solution
    };

    #[cfg(debug_assertions)]
    {
        let partition = solution.winning[0].union(&solution.winning[1])?;
        debug_assert!(partition == *v, "zielonka: winning sets must partition V");
    }

    Ok(solution)
}

fn empty_like(game: &SymbolicParityGame) -> Result<LDDFunction, MercError> {
    Ok(game.manager().with_manager_shared(LDDFunction::empty_set)?)
}

#[cfg(test)]
mod tests {
    use rand::RngExt;

    use oxidd::ManagerRef;
    use oxidd::ldd::LDDFunction;

    use merc_io::DumpFiles;
    use merc_utilities::random_test;

    use crate::PG;
    use crate::ParityGameBuilder;
    use crate::Player;
    use crate::Priority;
    use crate::VertexIndex;
    use crate::encode_parity_game;
    use crate::random_parity_game;
    use crate::solve_zielonka;
    use crate::write_pg;

    use super::solve_symbolic_zielonka;

    /// Cross-checks [`solve_symbolic_zielonka`] against the explicit [`solve_zielonka`] on random
    /// total games.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_symbolic_zielonka_solver() {
        random_test(100, |rng| {
            let files = DumpFiles::new("test_random_symbolic_zielonka_solver");
            let game = random_parity_game(rng, true, 60, 5, 3);
            files.dump("input.pg", |writer| write_pg(writer, &game)).unwrap();

            let (expected, _) = solve_zielonka(&game, false);

            let manager = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
            let radix = rng.random_range(2..=5);
            let num_groups = rng.random_range(1..=3);
            let (symbolic, all_vertices, cubes) = encode_parity_game(&manager, &game, rng, radix, num_groups).unwrap();

            let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
            assert!(empty_sinks.is_empty(), "a total game must have no sinks");

            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();
            let (_, solution) = solve_symbolic_zielonka(&symbolic, &initial, &all_vertices, &empty_sinks).unwrap();

            for v in game.iter_vertices() {
                let vertex = manager
                    .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[*v]))
                    .unwrap();
                let symbolic_winner = solution.winner(&vertex).expect("every vertex should be solved");
                let expected_winner = if expected[Player::Even.to_index()][*v] {
                    Player::Even
                } else {
                    Player::Odd
                };
                assert_eq!(symbolic_winner, expected_winner, "vertex {v}");
            }
        });
    }

    /// Cross-checks the sink handling of [`crate::SymbolicParityGame::compute_total_graph`]
    /// against an explicit oracle: a deadlock is resolved by giving it a self-loop whose priority
    /// has the parity of the *opponent* of its owner (so the self-loop's unique play is won by
    /// that opponent), matching `compute_total_graph`'s `winning[Even] |= sinks ∩ Odd-owned`.
    ///
    /// `solve_symbolic_zielonka` itself terminates as soon as `initial_vertex` is resolved (it
    /// has no `allow_early_termination` knob to disable that, matching mCRL2's default), so a
    /// non-total game's full winning partition is not guaranteed by that entry point alone —
    /// only the initial vertex's winner is. To still cross-check every vertex, this drives
    /// `compute_total_graph` + the private `zielonka` recursion directly (available to this
    /// `mod tests` as a sibling of `symbolic_zielonka`), bypassing early termination.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_symbolic_zielonka_solver_non_total() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, false, 60, 5, 3);

            // Build the explicit oracle: same game, but every deadlock gets a self-loop and a
            // priority whose parity is the opponent of its owner.
            let mut builder = ParityGameBuilder::new(VertexIndex::new(0));
            for v in game.iter_vertices() {
                builder.add_vertex(v, game.owner(v), game.priority(v));
            }
            for v in game.iter_vertices() {
                for edge in game.outgoing_edges(v) {
                    builder.add_edge(v, edge.to());
                }
            }
            for v in game.iter_vertices() {
                if game.outgoing_edges(v).next().is_none() {
                    let opponent = game.owner(v).opponent();
                    let priority = if opponent == Player::Even {
                        Priority::new(0)
                    } else {
                        Priority::new(1)
                    };
                    builder.add_vertex(v, game.owner(v), priority);
                    builder.add_edge(v, v);
                }
            }
            let expected_game = builder.finish(false, true);
            let (expected, _) = solve_zielonka(&expected_game, false);

            let manager = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
            let radix = rng.random_range(2..=5);
            let num_groups = rng.random_range(1..=3);
            let (symbolic, all_vertices, cubes) = encode_parity_game(&manager, &game, rng, radix, num_groups).unwrap();

            let sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();

            // Full solve, bypassing solve_symbolic_zielonka's early termination.
            let empty = manager.with_manager_shared(LDDFunction::empty_set).unwrap();
            let mut winning = [empty.clone(), empty];
            let total = symbolic
                .compute_total_graph(&all_vertices, &sinks, &mut winning)
                .unwrap();
            let zielonka_solution = super::zielonka(&symbolic, &total, 0).unwrap();
            winning[0] = winning[0].union(&zielonka_solution.winning[0]).unwrap();
            winning[1] = winning[1].union(&zielonka_solution.winning[1]).unwrap();
            let solution = super::SymbolicSolution { winning };

            // The early-terminating entry point must agree with the full solve on the initial vertex.
            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();
            let (early_winner, _) = solve_symbolic_zielonka(&symbolic, &initial, &all_vertices, &sinks).unwrap();
            assert_eq!(Some(early_winner), solution.winner(&initial));

            for v in game.iter_vertices() {
                let vertex = manager
                    .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[*v]))
                    .unwrap();
                let symbolic_winner = solution.winner(&vertex).expect("every vertex should be solved");
                let expected_winner = if expected[Player::Even.to_index()][*v] {
                    Player::Even
                } else {
                    Player::Odd
                };
                assert_eq!(symbolic_winner, expected_winner, "vertex {v}");
            }
        });
    }
}
