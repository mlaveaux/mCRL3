use std::ops::ControlFlow;

use log::debug;
use log::info;
use log::trace;

use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;

use merc_io::LargeFormatter;
use merc_io::TimeProgress;
use merc_symbolic::intersect;
use merc_symbolic::merge;
use merc_utilities::MercError;

use crate::Player;
use crate::Repeat;
use crate::SymbolicParityGame;
use crate::symbolic::AttractorProgress;

/// Progress reported by [`zielonka`]'s recursion: the recursion depth and the size of `V` at
/// that call. Separate from [`AttractorProgress`] because the two fire at very different
/// granularities — a single `zielonka` call can run many attractor iterations, each touching far
/// fewer vertices than a whole recursive call does.
pub type RecursionProgress = TimeProgress<(usize, usize)>;

/// The winning sets of both players in a symbolic parity game, indexed by [`Player::to_index`].
///
/// `strategy` is `None` unless the [`SymbolicParityGame`] this solution was computed for was
/// built with `compute_strategy: true`; when present, `strategy[Player::to_index()]` is that
/// player's winning strategy over the doubled, interleaved global vector `[from_0, to_0, from_1,
/// to_1, …]` (see [`SymbolicParityGame::apply_strategy`]).
pub struct SymbolicSolution {
    pub winning: [LDDFunction; 2],
    pub strategy: Option<[LDDFunction; 2]>,
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
pub(crate) fn includes(winning: &LDDFunction, vertex: &LDDFunction) -> Result<bool, MercError> {
    Ok(vertex.minus(winning)?.is_empty())
}

/// The "problem instance" every on-the-fly solving entry point in this module and
/// [`crate::symbolic::partial_solve`] operates over: the game itself, the subgame [`Self::vertices`]
/// currently being solved, its [`Self::sinks`], and the [`Self::initial_vertex`] whose winner every
/// entry point returns as soon as it is known. Bundles the four parameters
/// [`solve_symbolic_zielonka`] and every function in [`crate::symbolic::partial_solve`] take
/// identically, so they travel together as one named value instead of four positional arguments
/// that always appear in the same order anyway.
///
/// Deliberately does *not* also carry `incomplete`, `safe_variant`, or the recursion/attractor
/// progress trackers: those vary per call in ways `game`/`vertices`/`sinks`/`initial_vertex` don't
/// (see [`crate::symbolic::partial_solve`]'s `SafetyMode` for the `incomplete`/`safe_variant`
/// pair specifically).
pub struct ExtendedParityGame<'a> {
    pub game: &'a SymbolicParityGame,
    pub initial_vertex: &'a LDDFunction,
    pub vertices: &'a LDDFunction,
    pub sinks: &'a LDDFunction,
}

/// The shared preamble every entry point in [`crate::symbolic::partial_solve`] runs first:
/// removes the winning regions from `epg.vertices` via [`SymbolicParityGame::compute_total_graph`]
/// (growing `winning`/`strategy` in place), then returns early with the accumulated
/// [`SymbolicSolution`] if that alone already resolved `epg.initial_vertex` — otherwise continues
/// with the resulting total subgraph.
///
/// Factored out because every partial-solving accelerator needs exactly this preamble, byte for
/// byte, before it can even start searching for its own accelerator-specific dominion shape.
pub(crate) fn total_graph_with_early_exit(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    winning: &mut [LDDFunction; 2],
    strategy: &mut [Option<LDDFunction>; 2],
    attractor_progress: &AttractorProgress,
) -> Result<ControlFlow<SymbolicSolution, LDDFunction>, MercError> {
    let total = epg.game.compute_total_graph(
        epg.vertices,
        epg.sinks,
        winning,
        strategy,
        Some(incomplete),
        attractor_progress,
    )?;

    if includes(&winning[0], epg.initial_vertex)? || includes(&winning[1], epg.initial_vertex)? {
        return Ok(ControlFlow::Break(SymbolicSolution {
            winning: winning.clone(),
            strategy: pack_strategy_pair(strategy.clone()),
        }));
    }

    Ok(ControlFlow::Continue(total))
}

/// Solves `epg.game` restricted to `epg.vertices`, with `epg.sinks` as the deadlocks to resolve,
/// and returns the winner of `epg.initial_vertex` together with the winning partition.
///
/// When `allow_early_termination` is true, the solver will terminate as soon as
/// `epg.initial_vertex` is resolved.
pub fn solve_symbolic_zielonka(
    epg: &ExtendedParityGame,
    allow_early_termination: bool,
) -> Result<(Player, SymbolicSolution), MercError> {
    let game = epg.game;
    let empty = game.manager().with_manager_shared(LDDFunction::empty_set)?;
    let mut winning = [empty.clone(), empty];
    let mut strategy = empty_strategy_pair(game)?;
    let attractor_progress = new_attractor_progress();

    let total = game.compute_total_graph(
        epg.vertices,
        epg.sinks,
        &mut winning,
        &mut strategy,
        None,
        &attractor_progress,
    )?;

    let already_resolved = allow_early_termination
        && (includes(&winning[0], epg.initial_vertex)? || includes(&winning[1], epg.initial_vertex)?);

    if !already_resolved {
        let recursion_progress = new_recursion_progress();
        let solution = zielonka(game, &total, &recursion_progress, &attractor_progress)?;
        winning[0] = winning[0].union(&solution.winning[0])?;
        winning[1] = winning[1].union(&solution.winning[1])?;
        union_strategy_pair_in_place(&mut strategy, solution.strategy)?;
    }

    let strategy = pack_strategy_pair(strategy);

    if includes(&winning[0], epg.initial_vertex)? {
        Ok((Player::Even, SymbolicSolution { winning, strategy }))
    } else if includes(&winning[1], epg.initial_vertex)? {
        Ok((Player::Odd, SymbolicSolution { winning, strategy }))
    } else {
        Err("solve_symbolic_zielonka: initial vertex was not resolved by the solver".into())
    }
}

/// Builds the throttled progress tracker [`SymbolicParityGame::attractor`] reports through:
/// iteration number and attractor size so far, logged at most once every 5 seconds.
fn new_attractor_progress() -> AttractorProgress {
    TimeProgress::new(
        |(iteration, size)| {
            info!(
                "attractor: iteration {iteration}, {} vertices so far",
                LargeFormatter(size)
            );
        },
        5,
    )
}

/// Builds the throttled progress tracker [`zielonka`]'s recursion reports through: depth and
/// `|V|` at that call, logged at most once every 5 seconds.
fn new_recursion_progress() -> RecursionProgress {
    TimeProgress::new(
        |(depth, size)| {
            info!(
                "zielonka: recursion depth {depth}, {} vertices remaining",
                LargeFormatter(size)
            );
        },
        5,
    )
}

/// `[None, None]` when `game` was not built with `compute_strategy`, otherwise `[Some(∅),
/// Some(∅)]` — the accumulator [`SymbolicParityGame::compute_total_graph`] grows in place.
pub(crate) fn empty_strategy_pair(game: &SymbolicParityGame) -> Result<[Option<LDDFunction>; 2], MercError> {
    if game.compute_strategy() {
        let empty = game.manager().with_manager_shared(LDDFunction::empty_set)?;
        Ok([Some(empty.clone()), Some(empty)])
    } else {
        Ok([None, None])
    }
}

/// Unions `addition` (a `zielonka` result's strategy) into `strategy` in place, or is a no-op
/// when strategies are not being computed.
fn union_strategy_pair_in_place(
    strategy: &mut [Option<LDDFunction>; 2],
    addition: Option<[LDDFunction; 2]>,
) -> Result<(), MercError> {
    if let Some(addition) = addition {
        for player in [Player::Even, Player::Odd] {
            let i = player.to_index();
            let existing = strategy[i].take().expect("compute_strategy is set, so this is Some");
            strategy[i] = Some(existing.union(&addition[i])?);
        }
    }
    Ok(())
}

/// Converts the `Option` accumulator pair into the `Option<[_; 2]>` shape [`SymbolicSolution`]
/// exposes.
pub(crate) fn pack_strategy_pair(strategy: [Option<LDDFunction>; 2]) -> Option<[LDDFunction; 2]> {
    let [even, odd] = strategy;
    match (even, odd) {
        (Some(even), Some(odd)) => Some([even, odd]),
        _ => None,
    }
}

/// The inverse of [`pack_strategy_pair`]: unpacks a [`SymbolicSolution`]'s strategy into the
/// `Option`-per-player accumulator shape [`SymbolicParityGame::compute_total_graph`] grows.
pub(crate) fn unpack_strategy_pair(strategy: Option<[LDDFunction; 2]>) -> [Option<LDDFunction>; 2] {
    match strategy {
        Some([even, odd]) => [Some(even), Some(odd)],
        None => [None, None],
    }
}

/// The recursive Zielonka solver, restricted to the vertex set `v` — entry point that hides the
/// recursion-depth bookkeeping [`zielonka_rec`] needs from every caller (mirroring
/// [`crate::solve_zielonka`]'s own `solve_zielonka`/`solve_zielonka_impl::zielonka_rec` split),
/// so none of [`solve_symbolic_zielonka`] nor [`crate::symbolic::partial_solve`]'s three callers
/// have to remember to pass `depth: 0`.
pub(crate) fn zielonka(
    game: &SymbolicParityGame,
    v: &LDDFunction,
    recursion_progress: &RecursionProgress,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    recursion_progress.reset();
    zielonka_rec(game, v, 0, recursion_progress, attractor_progress)
}

/// The recursive worker behind [`zielonka`]; see there for the entry point.
fn zielonka_rec(
    game: &SymbolicParityGame,
    v: &LDDFunction,
    depth: usize,
    recursion_progress: &RecursionProgress,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    let indent = Repeat::new(" ", depth);

    if v.is_empty() {
        let empty = empty_like(game)?;
        let strategy = game
            .compute_strategy()
            .then(|| Ok::<_, MercError>([empty.clone(), empty.clone()]))
            .transpose()?;
        return Ok(SymbolicSolution {
            winning: [empty.clone(), empty],
            strategy,
        });
    }

    recursion_progress.print((depth, v.len()));

    let vplayer = game.players(v)?;
    let (priority, u) = game
        .max_priority(v)?
        .expect("v is non-empty, so it must contain at least one priority");
    let alpha = Player::from_priority(priority);
    let not_alpha = alpha.opponent();

    debug!(
        "{indent}zielonka: |V| = {}, priority = {priority}, player = {alpha}",
        v.len()
    );
    trace!("{indent}U (highest priority vertices) has {} elements", u.len());

    let (a, a_strategy) = game.attractor(alpha, &u, v, &vplayer, None, None, attractor_progress)?;
    trace!("{indent}A (attractor of U) has {} elements", a.len());

    let v_minus_a = v.minus(&a)?;
    let solution_v_minus_a = zielonka_rec(game, &v_minus_a, depth + 1, recursion_progress, attractor_progress)?;

    let solution = if solution_v_minus_a.winning[not_alpha.to_index()].is_empty() {
        let mut winning = [empty_like(game)?, empty_like(game)?];
        winning[alpha.to_index()] = a.union(&solution_v_minus_a.winning[alpha.to_index()])?;

        let strategy = if game.compute_strategy() {
            // Vertices pulled into A via a predecessor edge already have a strategy move from
            // `a_strategy`, but A's own seed set U does not (it was the *target*, not pulled in
            // by anyone). Since this branch wins the whole of `v` for `alpha`, any successor
            // that stays inside `v` is a sound move for an `alpha`-owned vertex of U — hence the
            // cartesian product `merge(U ∩ Vplayer[alpha], v)`, later cut down to real edges by
            // `apply_strategy`.
            let u_alpha = intersect(&u, &vplayer[alpha.to_index()])?;
            let extra = merge(game.manager(), &u_alpha, v)?;
            let combined = a_strategy
                .expect("compute_strategy is set")
                .union(&solution_v_minus_a.strategy.as_ref().expect("compute_strategy is set")[alpha.to_index()])?
                .union(&extra)?;

            let mut strategy = [empty_like(game)?, empty_like(game)?];
            strategy[alpha.to_index()] = combined;
            Some(strategy)
        } else {
            None
        };

        SymbolicSolution { winning, strategy }
    } else {
        let (b, b_strategy) = game.attractor(
            not_alpha,
            &solution_v_minus_a.winning[not_alpha.to_index()],
            v,
            &vplayer,
            None,
            None,
            attractor_progress,
        )?;
        trace!(
            "{indent}B (attractor of the opponent's win in V \\ A) has {} elements",
            b.len()
        );

        let v_minus_b = v.minus(&b)?;
        let mut solution = zielonka_rec(game, &v_minus_b, depth + 1, recursion_progress, attractor_progress)?;
        solution.winning[not_alpha.to_index()] = solution.winning[not_alpha.to_index()].union(&b)?;

        if game.compute_strategy() {
            let mut strategy = solution.strategy.take().expect("compute_strategy is set");
            let combined = solution_v_minus_a.strategy.expect("compute_strategy is set")[not_alpha.to_index()]
                .union(&b_strategy.expect("compute_strategy is set"))?
                .union(&strategy[not_alpha.to_index()])?;
            strategy[not_alpha.to_index()] = combined;
            solution.strategy = Some(strategy);
        }

        solution
    };

    #[cfg(debug_assertions)]
    {
        let partition = solution.winning[0].union(&solution.winning[1])?;
        debug_assert!(partition == *v, "zielonka: winning sets must partition V");
    }

    Ok(solution)
}

pub(crate) fn empty_like(game: &SymbolicParityGame) -> Result<LDDFunction, MercError> {
    Ok(game.manager().with_manager_shared(LDDFunction::empty_set)?)
}

/// Checks that `solution.strategy` really does win the vertices
/// `solution.winning` claims, native, this scales to large models the same way
/// the rest of the symbolic solver does).
///
/// # Details
///
/// Restrict `game` to the initial vertex's winner's own strategy via
/// [`SymbolicParityGame::apply_strategy`], re-solve the restricted game from
/// scratch, and require the two solutions to agree.
///
/// Panics with a description of the mismatch if the strategy fails to certify.
pub fn check_strategy(
    game: &SymbolicParityGame,
    initial_vertex: &LDDFunction,
    vertices: &LDDFunction,
    solution: &SymbolicSolution,
) -> Result<(), MercError> {
    let winner = solution
        .winner(initial_vertex)
        .ok_or("check_strategy: initial vertex was not resolved by the solver")?;
    let strategy = solution
        .strategy
        .as_ref()
        .ok_or("check_strategy: solution has no strategy (game was not built with compute_strategy)")?;

    let restricted = game.apply_strategy(winner, &strategy[winner.to_index()])?;
    let new_sinks = restricted.sinks(vertices, vertices)?;

    let (new_winner, new_solution) = solve_symbolic_zielonka(
        &ExtendedParityGame {
            game: &restricted,
            initial_vertex,
            vertices,
            sinks: &new_sinks,
        },
        false,
    )?;

    if new_winner != winner {
        panic!(
            "check_strategy: restricting {winner}'s own strategy changed the winner of the initial \
             vertex to {new_winner}"
        );
    }

    let solved = solution.winning[0].union(&solution.winning[1])?;
    if includes(&solved, vertices)? {
        for player in [Player::Even, Player::Odd] {
            let i = player.to_index();
            if solution.winning[i] != new_solution.winning[i] {
                panic!(
                    "check_strategy: after restricting {winner}'s strategy, {player}'s winning set \
                     changed even though the original solution covered every vertex"
                );
            }
        }
    } else {
        for player in [Player::Even, Player::Odd] {
            let i = player.to_index();
            if !includes(&new_solution.winning[i], &solution.winning[i])? {
                panic!("check_strategy: after restricting {winner}'s strategy, {player}'s winning set shrank");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rand::RngExt;

    use oxidd::ManagerRef;
    use oxidd::ldd::LDDFunction;

    use merc_io::DumpFiles;
    use merc_utilities::random_test;

    use crate::PG;
    use crate::Player;
    use crate::encode_parity_game;
    use crate::random_parity_game;
    use crate::solve_zielonka;
    use crate::write_pg;

    use super::ExtendedParityGame;
    use super::solve_symbolic_zielonka;
    use crate::symbolic::verify_symbolic::make_parity_game_total;

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
            let (symbolic, all_vertices, cubes) =
                encode_parity_game(&manager, &game, rng, radix, num_groups, false).unwrap();

            let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
            assert!(empty_sinks.is_empty(), "a total game must have no sinks");

            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();
            let epg = ExtendedParityGame {
                game: &symbolic,
                initial_vertex: &initial,
                vertices: &all_vertices,
                sinks: &empty_sinks,
            };
            let (_, solution) = solve_symbolic_zielonka(&epg, true).unwrap();

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
    /// Also checks that early termination doesn't change the initial vertex's winner: since
    /// `solve_symbolic_zielonka(..., false)` fully solves the game (needed to cross-check every
    /// vertex against the oracle below, not just the initial one),
    /// `solve_symbolic_zielonka(..., true)` on the same game must agree with it there.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_symbolic_zielonka_solver_non_total() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, false, 60, 5, 3);

            // Build the explicit oracle: same game, but every deadlock gets a self-loop and a
            // priority whose parity is the opponent of its owner (shared with
            // `verify_symbolic_solution`'s decode path, since both need the same convention).
            let expected_game = make_parity_game_total(&game);
            let (expected, _) = solve_zielonka(&expected_game, false);

            let manager = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
            let radix = rng.random_range(2..=5);
            let num_groups = rng.random_range(1..=3);
            let (symbolic, all_vertices, cubes) =
                encode_parity_game(&manager, &game, rng, radix, num_groups, false).unwrap();

            let sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();

            let epg = ExtendedParityGame {
                game: &symbolic,
                initial_vertex: &initial,
                vertices: &all_vertices,
                sinks: &sinks,
            };
            let (_, solution) = solve_symbolic_zielonka(&epg, false).unwrap();

            let (early_winner, _) = solve_symbolic_zielonka(&epg, true).unwrap();
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

    /// Cross-checks [`super::check_strategy`] (native, LDD-level certification via
    /// [`crate::SymbolicParityGame::apply_strategy`]) against random *total* games.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_check_strategy_random_total() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, true, 60, 5, 3);

            let manager = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
            let radix = rng.random_range(2..=5);
            let num_groups = rng.random_range(1..=3);
            let (symbolic, all_vertices, cubes) =
                encode_parity_game(&manager, &game, rng, radix, num_groups, true).unwrap();

            let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();

            let epg = ExtendedParityGame {
                game: &symbolic,
                initial_vertex: &initial,
                vertices: &all_vertices,
                sinks: &empty_sinks,
            };
            let (_, solution) = solve_symbolic_zielonka(&epg, false).unwrap();

            super::check_strategy(&symbolic, &initial, &all_vertices, &solution).unwrap();
        });
    }

    /// Same as [`test_check_strategy_random_total`], but for random *non-total* games, so
    /// `apply_strategy`'s fresh-sink handling is exercised too.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_check_strategy_random_non_total() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, false, 60, 5, 3);

            let manager = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
            let radix = rng.random_range(2..=5);
            let num_groups = rng.random_range(1..=3);
            let (symbolic, all_vertices, cubes) =
                encode_parity_game(&manager, &game, rng, radix, num_groups, true).unwrap();

            let sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();

            let epg = ExtendedParityGame {
                game: &symbolic,
                initial_vertex: &initial,
                vertices: &all_vertices,
                sinks: &sinks,
            };
            let (_, solution) = solve_symbolic_zielonka(&epg, false).unwrap();

            super::check_strategy(&symbolic, &initial, &all_vertices, &solution).unwrap();
        });
    }
}
