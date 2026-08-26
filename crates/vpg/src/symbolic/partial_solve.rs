use std::ops::ControlFlow;

use oxidd::ldd::LDDFunction;

use merc_symbolic::intersect;
use merc_symbolic::merge;
use merc_utilities::MercError;

use crate::Player;
use crate::symbolic::AttractorProgress;
use crate::symbolic::RecursionProgress;
use crate::symbolic::SymbolicParityGame;
use crate::symbolic::SymbolicSolution;
use crate::symbolic::symbolic_zielonka::ExtendedParityGame;
use crate::symbolic::symbolic_zielonka::empty_like;
use crate::symbolic::symbolic_zielonka::empty_strategy_pair;
use crate::symbolic::symbolic_zielonka::includes;
use crate::symbolic::symbolic_zielonka::pack_strategy_pair;
use crate::symbolic::symbolic_zielonka::total_graph_with_early_exit;
use crate::symbolic::symbolic_zielonka::unpack_strategy_pair;
use crate::symbolic::symbolic_zielonka::zielonka;

/// Which of the two ways [`detect_solitair_cycles`]/[`detect_forced_cycles`]/
/// [`detect_fatal_attractors`] (and their `_within_safe_vertices` counterparts) stay sound
/// against `incomplete`'s unlearned edges. Resolved once, right after `total` is known (via
/// [`Self::resolve`]), and threaded from there as one value — replacing what used to be a
/// `safe_variant: bool` paired independently with an `Option<[LDDFunction; 2]>` that every caller
/// had to keep in sync by hand (see the `.expect("safe_variant is ...")` calls this replaced).
///
/// The two constructions are *provably equal*, not a soundness trade-off: for [`detect_forced_cycles`]
/// and [`detect_fatal_attractors`] this equality is exactly the paper's Propositions 2 and 3
/// (`C_for^α = C_s-for^α`, `F_s^α = F^α`), which rest on Lemma 3 (`cpre_α(⅁ ∩ safe_α(⅁), X) =
/// spre_α(⅁, X)` for `X ⊆ safe_α(⅁)`); for [`detect_solitair_cycles`] the cycle search itself
/// needs neither (Proposition 1: `C_sol^α(⅁) ⊆ safe_α(⅁)` unconditionally), so the two variants
/// only actually differ in how [`accept_cycle`]'s closing attractor is computed, via Lemma 4
/// (`Attr_α(⅁ ∩ safe_α(⅁), X) = SAttr_α(⅁, X)`). Nor is [`Self::Restricted`] cheaper:
/// [`Self::resolve`] pays two extra full `safe_vertices` attractor fixed points *up front* to
/// build it, exactly the cost the safe-attractor mechanism ([`Self::Safe`]) exists to avoid by
/// folding one `minus(incomplete)` into each control-predecessor step instead — matching the
/// paper's own measurements, where the safe-attractor variant is "almost always beneficial with
/// minimal overhead". [`Self::Restricted`] earns its keep only when `safe_vertices` was already
/// computed anyway for some other reason (as [`partial_solve`] does).
///
/// Named `SafetyMode` rather than after either of the "safe" concepts it distinguishes, since the
/// paper this module ports overloads that word for two different things: the *safe attractor*/
/// *safe control predecessor* mechanism ([`Self::Safe`], mCRL2's `safe_control_predecessors`/
/// `safe_attractor` — folds `incomplete` into the search directly) and the *α-safe vertex set*
/// ([`Self::Restricted`], [`SymbolicParityGame::safe_vertices`] — restricts the search to a
/// precomputed subgame instead). See [`detect_solitair_cycles`]'s doc comment for which one each
/// public entry point uses.
enum SafetyMode {
    /// Fold `incomplete` directly into every attractor/control-predecessor call.
    Safe,
    /// Restrict the whole search up front to [`SymbolicParityGame::safe_vertices`].
    Restricted([LDDFunction; 2]),
}

impl SafetyMode {
    /// Resolves the mode: `Restricted` (computing both players' `safe_vertices` against `total`)
    /// when `restrict_to_safe_vertices`, else `Safe`.
    fn resolve(
        restrict_to_safe_vertices: bool,
        game: &SymbolicParityGame,
        total: &LDDFunction,
        incomplete: &LDDFunction,
        attractor_progress: &AttractorProgress,
    ) -> Result<Self, MercError> {
        if restrict_to_safe_vertices {
            Ok(Self::Restricted([
                game.safe_vertices(Player::Even, total, incomplete, attractor_progress)?,
                game.safe_vertices(Player::Odd, total, incomplete, attractor_progress)?,
            ]))
        } else {
            Ok(Self::Safe)
        }
    }
}

/// Solves as much of `epg.vertices` as [`SymbolicParityGame::compute_total_graph`] and two plain
/// [`zielonka`] calls, restricted to the *safe* vertices of each player.
pub fn partial_solve(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    partial_solution: SymbolicSolution,
    recursion_progress: &RecursionProgress,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    let SymbolicSolution { mut winning, strategy } = partial_solution;
    let mut strategy = unpack_strategy_pair(strategy);

    let total = match total_graph_with_early_exit(epg, incomplete, &mut winning, &mut strategy, attractor_progress)? {
        ControlFlow::Break(solution) => return Ok(solution),
        ControlFlow::Continue(total) => total,
    };
    let game = epg.game;

    let safe_even = game.safe_vertices(Player::Even, &total, incomplete, attractor_progress)?;
    let mut solution0 = zielonka(game, &safe_even, recursion_progress, attractor_progress)?;
    solution0.winning[0] = solution0.winning[0].union(&winning[0])?;
    if game.compute_strategy() {
        let mut s = solution0.strategy.take().expect("compute_strategy is set");
        s[0] = s[0].union(strategy[0].as_ref().expect("compute_strategy is set"))?;
        solution0.strategy = Some(s);
    }

    if includes(&solution0.winning[0], epg.initial_vertex)? {
        solution0.winning[1] = winning[1].clone();
        if game.compute_strategy() {
            let mut s = solution0.strategy.take().expect("compute_strategy is set");
            s[1] = strategy[1].clone().expect("compute_strategy is set");
            solution0.strategy = Some(s);
        }
        return Ok(solution0);
    }

    let safe_odd = game.safe_vertices(Player::Odd, &total, incomplete, attractor_progress)?;
    let mut solution1 = zielonka(game, &safe_odd, recursion_progress, attractor_progress)?;
    solution1.winning[1] = solution1.winning[1].union(&winning[1])?;
    if game.compute_strategy() {
        let mut s = solution1.strategy.take().expect("compute_strategy is set");
        s[1] = s[1].union(strategy[1].as_ref().expect("compute_strategy is set"))?;
        solution1.strategy = Some(s);
    }

    solution1.winning[0] = solution0.winning[0].clone();
    if game.compute_strategy() {
        let mut s = solution1.strategy.take().expect("compute_strategy is set");
        s[0] = solution0.strategy.expect("compute_strategy is set")[0].clone();
        solution1.strategy = Some(s);
    }

    Ok(solution1)
}

/// Detects "solitair" winning cycles: a set `U` of vertices all owned by `alpha`, each with at
/// least one edge staying inside `U`, so `alpha` can simply choose to loop inside `U` forever
/// (winning it outright, since `U` only ever contains vertices at `alpha`'s own parity).
///
/// Port of `detect_solitair_cycles`, using [`SafetyMode::Safe`] — see
/// [`detect_solitair_cycles_within_safe_vertices`] for the [`SafetyMode::Restricted`] variant,
/// and [`partial_solve`] for why `incomplete = ∅` is the only case exercised today.
pub fn detect_solitair_cycles(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    partial_solution: SymbolicSolution,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    detect_solitair_cycles_impl(epg, incomplete, false, partial_solution, attractor_progress)
}

/// Like [`detect_solitair_cycles`], but using [`SafetyMode::Restricted`]: restricts the whole
/// search up front to [`SymbolicParityGame::safe_vertices`] instead of folding `incomplete`
/// directly into every attractor call — see [`SafetyMode`]'s doc comment for why this finds the
/// same dominion, at a higher up-front cost, rather than a different (cheaper or more
/// conservative) one.
pub fn detect_solitair_cycles_within_safe_vertices(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    partial_solution: SymbolicSolution,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    detect_solitair_cycles_impl(epg, incomplete, true, partial_solution, attractor_progress)
}

fn detect_solitair_cycles_impl(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    restrict_to_safe_vertices: bool,
    partial_solution: SymbolicSolution,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    let SymbolicSolution { mut winning, strategy } = partial_solution;
    let mut strategy = unpack_strategy_pair(strategy);

    let total = match total_graph_with_early_exit(epg, incomplete, &mut winning, &mut strategy, attractor_progress)? {
        ControlFlow::Break(solution) => return Ok(solution),
        ControlFlow::Continue(total) => total,
    };
    let game = epg.game;

    let vplayer = game.players(&total)?;
    let parity = game.parity(&total)?;
    let mode = SafetyMode::resolve(restrict_to_safe_vertices, game, &total, incomplete, attractor_progress)?;

    for alpha in [Player::Even, Player::Odd] {
        let i = alpha.to_index();

        let mut u = empty_like(game)?;
        let mut u_next = intersect(&parity[i], &vplayer[i])?;
        if let SafetyMode::Restricted(safe) = &mode {
            u_next = intersect(&u_next, &safe[i])?;
        }

        while u != u_next {
            u = u_next.clone();
            u_next = game.predecessors(&u, &u)?;
        }

        accept_cycle(
            game,
            alpha,
            &u,
            &total,
            &vplayer,
            &mode,
            incomplete,
            &mut winning,
            &mut strategy,
            attractor_progress,
        )?;
    }

    Ok(SymbolicSolution {
        winning,
        strategy: pack_strategy_pair(strategy),
    })
}

/// Detects "forced" winning cycles: a set `U` of vertices (any owner) such that `alpha` can
/// *force* play to stay inside `U` forever regardless of what its opponent does — unlike
/// [`detect_solitair_cycles`], `U` is not restricted to `alpha`'s own vertices, so an
/// opponent-owned vertex only joins `U` once every one of its edges is proven to stay inside.
///
/// Port of `detect_forced_cycles`, using [`SafetyMode::Safe`] — see
/// [`detect_forced_cycles_within_safe_vertices`] for the [`SafetyMode::Restricted`] variant.
pub fn detect_forced_cycles(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    partial_solution: SymbolicSolution,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    detect_forced_cycles_impl(epg, incomplete, false, partial_solution, attractor_progress)
}

/// Like [`detect_forced_cycles`], but using [`SafetyMode::Restricted`] — see
/// [`SafetyMode`]'s doc comment for what actually differs between the two.
pub fn detect_forced_cycles_within_safe_vertices(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    partial_solution: SymbolicSolution,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    detect_forced_cycles_impl(epg, incomplete, true, partial_solution, attractor_progress)
}

fn detect_forced_cycles_impl(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    restrict_to_safe_vertices: bool,
    partial_solution: SymbolicSolution,
    attractor_progress: &AttractorProgress,
) -> Result<SymbolicSolution, MercError> {
    let SymbolicSolution { mut winning, strategy } = partial_solution;
    let mut strategy = unpack_strategy_pair(strategy);

    let total = match total_graph_with_early_exit(epg, incomplete, &mut winning, &mut strategy, attractor_progress)? {
        ControlFlow::Break(solution) => return Ok(solution),
        ControlFlow::Continue(total) => total,
    };
    let game = epg.game;

    let vplayer = game.players(&total)?;
    let parity = game.parity(&total)?;
    let mode = SafetyMode::resolve(restrict_to_safe_vertices, game, &total, incomplete, attractor_progress)?;

    for alpha in [Player::Even, Player::Odd] {
        let i = alpha.to_index();

        let mut u = empty_like(game)?;
        let mut u_next = parity[i].clone();
        if let SafetyMode::Restricted(safe) = &mode {
            u_next = intersect(&u_next, &safe[i])?;
        }

        while u != u_next {
            u = u_next.clone();
            u_next = match &mode {
                SafetyMode::Safe => intersect(
                    &u,
                    &game.control_predecessors_within(alpha, &u, &total, &vplayer, Some(incomplete))?,
                )?,
                SafetyMode::Restricted(safe) => intersect(
                    &u,
                    &game.control_predecessors_within(alpha, &u, &safe[i], &vplayer, None)?,
                )?,
            };
        }

        accept_cycle(
            game,
            alpha,
            &u,
            &total,
            &vplayer,
            &mode,
            incomplete,
            &mut winning,
            &mut strategy,
            attractor_progress,
        )?;
    }

    Ok(SymbolicSolution {
        winning,
        strategy: pack_strategy_pair(strategy),
    })
}

/// Records `U` (a solitair or forced winning cycle for `alpha`, already found by the caller) as
/// won, with an overapproximate `merge(U, U)` strategy — cut down to real edges by
/// [`SymbolicParityGame::apply_strategy`], the same trick every other strategy contribution in
/// this module relies on — and extends `winning`/`strategy` with the attractor into `U`.
///
/// Shared by [`detect_solitair_cycles_impl`] and [`detect_forced_cycles_impl`], which differ only
/// in how they compute `u`, not in what happens once they have it.
#[allow(clippy::too_many_arguments)]
fn accept_cycle(
    game: &SymbolicParityGame,
    alpha: Player,
    u: &LDDFunction,
    total: &LDDFunction,
    vplayer: &[LDDFunction; 2],
    mode: &SafetyMode,
    incomplete: &LDDFunction,
    winning: &mut [LDDFunction; 2],
    strategy: &mut [Option<LDDFunction>; 2],
    attractor_progress: &AttractorProgress,
) -> Result<(), MercError> {
    let i = alpha.to_index();

    if game.compute_strategy() {
        let extra = merge(game.manager(), u, u)?;
        let existing = strategy[i].take().expect("compute_strategy is set");
        strategy[i] = Some(existing.union(&extra)?);
    }

    let (attracted, attr_strategy) = match mode {
        SafetyMode::Safe => game.attractor(alpha, u, total, vplayer, Some(incomplete), None, attractor_progress)?,
        SafetyMode::Restricted(safe) => game.attractor(alpha, u, &safe[i], vplayer, None, None, attractor_progress)?,
    };

    winning[i] = winning[i].union(&attracted)?;
    if game.compute_strategy() {
        let existing = strategy[i].take().expect("compute_strategy is set");
        strategy[i] = Some(existing.union(&attr_strategy.expect("compute_strategy is set"))?);
    }

    Ok(())
}

/// Detects fatal attractors: for each priority `c` (processed from least to most significant),
/// searches for a set of priority-`c` vertices that `alpha = Player::from_priority(c)` can force
/// play to always return to — winning `alpha` the whole attractor into that set.
///
/// Port of `detect_fatal_attractors`, using [`SafetyMode::Safe`] — see
/// [`detect_fatal_attractors_within_safe_vertices`] for the [`SafetyMode::Restricted`] variant.
/// Unlike [`detect_solitair_cycles`]/[`detect_forced_cycles`], this has no
/// [`SymbolicSolution`]-shaped input/output: mCRL2's own version takes and returns raw winning
/// sets with no strategy at all (its internal `safe_attractor` calls still compute one when
/// [`SymbolicParityGame::compute_strategy`] is set, but it is discarded — ported faithfully here
/// rather than "improved", since fatal-attractor strategies are genuinely underspecified: a
/// vertex can belong to fatal attractors for *different* priorities on different iterations, and
/// mCRL2 does not attempt to reconcile that into one strategy).
///
/// `w0`/`w1` seed the winning sets exactly like mCRL2's optional `W0`/`W1` parameters (pass
/// empty sets to start from scratch).
pub fn detect_fatal_attractors(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    w0: &LDDFunction,
    w1: &LDDFunction,
    attractor_progress: &AttractorProgress,
) -> Result<[LDDFunction; 2], MercError> {
    detect_fatal_attractors_impl(epg, incomplete, false, w0, w1, attractor_progress)
}

/// Like [`detect_fatal_attractors`], but using [`SafetyMode::Restricted`] — see
/// [`SafetyMode`]'s doc comment for what actually differs between the two.
pub fn detect_fatal_attractors_within_safe_vertices(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    w0: &LDDFunction,
    w1: &LDDFunction,
    attractor_progress: &AttractorProgress,
) -> Result<[LDDFunction; 2], MercError> {
    detect_fatal_attractors_impl(epg, incomplete, true, w0, w1, attractor_progress)
}

#[allow(clippy::too_many_arguments)]
fn detect_fatal_attractors_impl(
    epg: &ExtendedParityGame,
    incomplete: &LDDFunction,
    restrict_to_safe_vertices: bool,
    w0: &LDDFunction,
    w1: &LDDFunction,
    attractor_progress: &AttractorProgress,
) -> Result<[LDDFunction; 2], MercError> {
    let mut winning = [w0.clone(), w1.clone()];
    let mut strategy = empty_strategy_pair(epg.game)?;

    let total = match total_graph_with_early_exit(epg, incomplete, &mut winning, &mut strategy, attractor_progress)? {
        ControlFlow::Break(solution) => return Ok(solution.winning),
        ControlFlow::Continue(total) => total,
    };
    let game = epg.game;

    let vplayer = game.players(&total)?;
    let mode = SafetyMode::resolve(restrict_to_safe_vertices, game, &total, incomplete, attractor_progress)?;

    // Ascending order: under merc's max-parity encoding this is mCRL2's own descending (least to
    // most significant) rank order — see `SymbolicParityGame::max_priority`'s doc comment for
    // the general inversion rule this follows.
    for (&priority, block) in game.priorities() {
        let alpha = Player::from_priority(priority);
        let i = alpha.to_index();
        let search_space = match &mode {
            SafetyMode::Safe => &total,
            SafetyMode::Restricted(safe) => &safe[i],
        };

        let mut x = match &mode {
            SafetyMode::Safe => block.clone(),
            SafetyMode::Restricted(_) => intersect(block, search_space)?,
        };
        let mut y = empty_like(game)?;

        while !x.is_empty() && x != y {
            y = x.clone();
            let z = game.monotone_attractor(
                &x,
                alpha,
                priority,
                search_space,
                &vplayer,
                matches!(mode, SafetyMode::Safe).then_some(incomplete),
                None,
            )?;

            if includes(&z, &x)? {
                let (attracted, _) = match &mode {
                    SafetyMode::Safe => {
                        game.attractor(alpha, &z, &total, &vplayer, Some(incomplete), None, attractor_progress)?
                    }
                    SafetyMode::Restricted(_) => {
                        game.attractor(alpha, &z, search_space, &vplayer, None, None, attractor_progress)?
                    }
                };
                winning[i] = winning[i].union(&attracted)?;
                break;
            }
            x = intersect(&x, &z)?;
        }
    }

    Ok(winning)
}

#[cfg(test)]
mod tests {
    use rand::RngExt;
    use rand::SeedableRng;

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

    use super::AttractorProgress;
    use super::ExtendedParityGame;
    use super::SymbolicSolution;
    use super::detect_fatal_attractors;
    use super::detect_fatal_attractors_within_safe_vertices;
    use super::detect_forced_cycles;
    use super::detect_forced_cycles_within_safe_vertices;
    use super::detect_solitair_cycles;
    use super::detect_solitair_cycles_within_safe_vertices;
    use super::partial_solve;
    use crate::symbolic::RecursionProgress;

    /// A progress tracker that never actually prints (interval far longer than any test run),
    /// for tests that need one to pass but have nothing useful to say about it.
    fn silent_attractor_progress() -> AttractorProgress {
        AttractorProgress::new(|_| {}, 3600)
    }

    /// See [`silent_attractor_progress`].
    fn silent_recursion_progress() -> RecursionProgress {
        RecursionProgress::new(|_| {}, 3600)
    }

    /// Cross-checks [`partial_solve`] against the explicit [`solve_zielonka`] for *soundness* on
    /// random total games, with `incomplete = ∅`: whichever vertices it does resolve must agree
    /// with the oracle. It is not required to resolve everything — `incomplete = ∅` makes every
    /// vertex *safe*, but `partial_solve` still returns as soon as `initial_vertex`'s winner is
    /// decided (see its doc comment), so a real subset of `v` going unresolved is expected, not a
    /// bug.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_partial_solve_is_sound() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, true, 60, 5, 3);
            let (expected, _) = solve_zielonka(&game, false);

            let manager = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
            let radix = rng.random_range(2..=5);
            let num_groups = rng.random_range(1..=3);
            let (symbolic, all_vertices, cubes) =
                encode_parity_game(&manager, &game, rng, radix, num_groups, false).unwrap();

            let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
            let incomplete = manager.with_manager_shared(LDDFunction::empty_set).unwrap();
            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();

            let empty = manager.with_manager_shared(LDDFunction::empty_set).unwrap();
            let partial_solution = SymbolicSolution {
                winning: [empty.clone(), empty],
                strategy: None,
            };

            let epg = ExtendedParityGame {
                game: &symbolic,
                initial_vertex: &initial,
                vertices: &all_vertices,
                sinks: &empty_sinks,
            };
            let solution = partial_solve(
                &epg,
                &incomplete,
                partial_solution,
                &silent_recursion_progress(),
                &silent_attractor_progress(),
            )
            .unwrap();

            let initial_winner = if expected[Player::Even.to_index()][0] {
                Player::Even
            } else {
                Player::Odd
            };
            assert_eq!(
                solution.winner(&initial),
                Some(initial_winner),
                "initial vertex must be resolved"
            );

            for v in game.iter_vertices() {
                let vertex = manager
                    .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[*v]))
                    .unwrap();
                if let Some(winner) = solution.winner(&vertex) {
                    let expected_winner = if expected[Player::Even.to_index()][*v] {
                        Player::Even
                    } else {
                        Player::Odd
                    };
                    assert_eq!(winner, expected_winner, "vertex {v}");
                }
            }
        });
    }

    /// Cross-checks [`detect_solitair_cycles`], [`detect_forced_cycles`] and
    /// [`detect_fatal_attractors`] (and their `_within_safe_vertices` counterparts) for
    /// *soundness* against the explicit [`solve_zielonka`] on random total games: unlike
    /// [`partial_solve`], these accelerators only ever resolve part of the game (a
    /// solitair/forced/fatal cycle need not exist at all in a given random game), so the property
    /// to check is that whatever they *do* claim agrees with the oracle — completeness is checked
    /// separately, on hand-built fixtures guaranteed to contain a cycle, below.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_cycle_detectors_are_sound() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, true, 60, 5, 3);
            let (expected, _) = solve_zielonka(&game, false);

            let manager = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
            let radix = rng.random_range(2..=5);
            let num_groups = rng.random_range(1..=3);
            let (symbolic, all_vertices, cubes) =
                encode_parity_game(&manager, &game, rng, radix, num_groups, false).unwrap();

            let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
            let incomplete = manager.with_manager_shared(LDDFunction::empty_set).unwrap();
            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();
            let empty = manager.with_manager_shared(LDDFunction::empty_set).unwrap();

            let epg = ExtendedParityGame {
                game: &symbolic,
                initial_vertex: &initial,
                vertices: &all_vertices,
                sinks: &empty_sinks,
            };

            let expected_winner = |v: usize| {
                if expected[Player::Even.to_index()][v] {
                    Player::Even
                } else {
                    Player::Odd
                }
            };

            let fresh_solution = || SymbolicSolution {
                winning: [empty.clone(), empty.clone()],
                strategy: None,
            };

            // "safe" mode: folds `incomplete` directly into every attractor call.
            let solitair_safe =
                detect_solitair_cycles(&epg, &incomplete, fresh_solution(), &silent_attractor_progress()).unwrap();
            let forced_safe =
                detect_forced_cycles(&epg, &incomplete, fresh_solution(), &silent_attractor_progress()).unwrap();
            let fatal_safe =
                detect_fatal_attractors(&epg, &incomplete, &empty, &empty, &silent_attractor_progress()).unwrap();

            // "within_safe_vertices" mode: restricts the search up front to `safe_vertices`.
            let solitair_restricted = detect_solitair_cycles_within_safe_vertices(
                &epg,
                &incomplete,
                fresh_solution(),
                &silent_attractor_progress(),
            )
            .unwrap();
            let forced_restricted = detect_forced_cycles_within_safe_vertices(
                &epg,
                &incomplete,
                fresh_solution(),
                &silent_attractor_progress(),
            )
            .unwrap();
            let fatal_restricted = detect_fatal_attractors_within_safe_vertices(
                &epg,
                &incomplete,
                &empty,
                &empty,
                &silent_attractor_progress(),
            )
            .unwrap();

            for v in game.iter_vertices() {
                let vertex = manager
                    .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[*v]))
                    .unwrap();

                for (label, solitair, forced, fatal) in [
                    ("safe", &solitair_safe, &forced_safe, &fatal_safe),
                    (
                        "within_safe_vertices",
                        &solitair_restricted,
                        &forced_restricted,
                        &fatal_restricted,
                    ),
                ] {
                    if let Some(winner) = solitair.winner(&vertex) {
                        assert_eq!(winner, expected_winner(*v), "solitair: vertex {v}, mode={label}");
                    }
                    if let Some(winner) = forced.winner(&vertex) {
                        assert_eq!(winner, expected_winner(*v), "forced: vertex {v}, mode={label}");
                    }
                    for player in [Player::Even, Player::Odd] {
                        if merc_symbolic::element_of(&manager, &cubes[*v], &fatal[player.to_index()]) {
                            assert_eq!(player, expected_winner(*v), "fatal: vertex {v}, mode={label}");
                        }
                    }
                }
            }
        });
    }

    /// A tiny fixed game with a genuine solitair cycle: vertex 0 (Even-owned, priority 0, so
    /// `Player::from_priority` maps it to Even — matching its owner, exactly the coincidence
    /// [`detect_solitair_cycles`] needs) has a self-loop, alongside an edge into vertex 1 that
    /// leads nowhere useful for Even. Even can simply always choose the self-loop, so vertex 0
    /// (and, by attraction, vertex 1, since Odd's only move from 1 is back to 0) must be won by
    /// Even — checks completeness where the random test above only checks soundness.
    fn solitair_cycle_fixture() -> crate::ParityGame {
        let mut builder = ParityGameBuilder::new(VertexIndex::new(0));
        builder.add_vertex(VertexIndex::new(0), Player::Even, Priority::new(0));
        builder.add_vertex(VertexIndex::new(1), Player::Odd, Priority::new(1));
        builder.add_edge(VertexIndex::new(0), VertexIndex::new(0));
        builder.add_edge(VertexIndex::new(0), VertexIndex::new(1));
        builder.add_edge(VertexIndex::new(1), VertexIndex::new(0));
        builder.finish(false, true)
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_detect_solitair_cycles_finds_the_fixture_cycle() {
        let files = DumpFiles::new("test_detect_solitair_cycles_finds_the_fixture_cycle");
        let game = solitair_cycle_fixture();
        files.dump("input.pg", |writer| write_pg(writer, &game)).unwrap();

        let manager = oxidd::ldd::new_manager(1 << 12, 1 << 12, 1);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let (symbolic, all_vertices, cubes) = encode_parity_game(&manager, &game, &mut rng, 2, 1, false).unwrap();

        let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
        let empty = manager.with_manager_shared(LDDFunction::empty_set).unwrap();
        let incomplete = empty.clone();
        let initial = manager
            .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
            .unwrap();

        let partial_solution = SymbolicSolution {
            winning: [empty.clone(), empty],
            strategy: None,
        };

        let epg = ExtendedParityGame {
            game: &symbolic,
            initial_vertex: &initial,
            vertices: &all_vertices,
            sinks: &empty_sinks,
        };
        let solution = detect_solitair_cycles_within_safe_vertices(
            &epg,
            &incomplete,
            partial_solution,
            &silent_attractor_progress(),
        )
        .unwrap();

        for v in [0usize, 1] {
            let vertex = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[v]))
                .unwrap();
            assert_eq!(
                solution.winner(&vertex),
                Some(Player::Even),
                "vertex {v} must be resolved (won by Even) by the solitair cycle at vertex 0"
            );
        }
    }

    /// A tiny fixed game with a genuine *forced* (but not solitair) cycle: vertex 0 is Odd-owned
    /// with a *single* outgoing edge to vertex 1 (Even-owned), which has a single edge straight
    /// back to vertex 0 — both at priority 0 (even). [`detect_forced_cycles`]'s seed is every
    /// vertex sharing `alpha`'s own priority *parity*, regardless of owner, so both vertices seed
    /// it; Odd has no choice at all (degree 1), so it is trivially "forced" to stay inside `{0,
    /// 1}`, and the whole 2-cycle is won by Even (its only priority, 0, is even).
    /// [`detect_solitair_cycles`] cannot find this: its seed is restricted to vertices `alpha`
    /// itself *owns*, i.e. just `{1}` here, and `{1}` alone does not self-sustain (vertex 1's
    /// only edge leaves it, to vertex 0) — only [`detect_forced_cycles`] can bring vertex 0's
    /// forced, single-edge cooperation into the picture.
    fn forced_cycle_fixture() -> crate::ParityGame {
        let mut builder = ParityGameBuilder::new(VertexIndex::new(0));
        builder.add_vertex(VertexIndex::new(0), Player::Odd, Priority::new(0));
        builder.add_vertex(VertexIndex::new(1), Player::Even, Priority::new(0));
        builder.add_edge(VertexIndex::new(0), VertexIndex::new(1));
        builder.add_edge(VertexIndex::new(1), VertexIndex::new(0));
        builder.finish(false, true)
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_detect_forced_cycles_finds_the_fixture_cycle() {
        let files = DumpFiles::new("test_detect_forced_cycles_finds_the_fixture_cycle");
        let game = forced_cycle_fixture();
        files.dump("input.pg", |writer| write_pg(writer, &game)).unwrap();

        // Sanity check that this really is a forced-but-not-solitair cycle for the explicit
        // solver too, i.e. that the fixture tests what it claims to.
        let (expected, _) = solve_zielonka(&game, false);
        for v in [0usize, 1] {
            assert!(
                expected[Player::Even.to_index()][v],
                "fixture sanity check: vertex {v} should be Even's"
            );
        }

        let manager = oxidd::ldd::new_manager(1 << 12, 1 << 12, 1);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let (symbolic, all_vertices, cubes) = encode_parity_game(&manager, &game, &mut rng, 2, 1, false).unwrap();

        let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
        let empty = manager.with_manager_shared(LDDFunction::empty_set).unwrap();
        let incomplete = empty.clone();
        let initial = manager
            .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
            .unwrap();
        let epg = ExtendedParityGame {
            game: &symbolic,
            initial_vertex: &initial,
            vertices: &all_vertices,
            sinks: &empty_sinks,
        };

        // The solitair detector must not resolve anything here: vertex 0 (the only vertex at an
        // even priority owned by anyone) is owned by Odd, so it never seeds the search.
        let solitair_partial_solution = SymbolicSolution {
            winning: [empty.clone(), empty.clone()],
            strategy: None,
        };
        let solitair = detect_solitair_cycles_within_safe_vertices(
            &epg,
            &incomplete,
            solitair_partial_solution,
            &silent_attractor_progress(),
        )
        .unwrap();
        for v in [0usize, 1] {
            let vertex = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[v]))
                .unwrap();
            assert_eq!(
                solitair.winner(&vertex),
                None,
                "vertex {v} must NOT be resolved by the solitair detector (no vertex here owns its own escape)"
            );
        }

        let forced_partial_solution = SymbolicSolution {
            winning: [empty.clone(), empty],
            strategy: None,
        };
        let forced = detect_forced_cycles_within_safe_vertices(
            &epg,
            &incomplete,
            forced_partial_solution,
            &silent_attractor_progress(),
        )
        .unwrap();
        for v in [0usize, 1] {
            let vertex = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[v]))
                .unwrap();
            assert_eq!(
                forced.winner(&vertex),
                Some(Player::Even),
                "vertex {v} must be resolved (won by Even) by the forced cycle {{0, 1}}"
            );
        }
    }

    /// The same [`forced_cycle_fixture`] game, this time checking [`detect_fatal_attractors`]:
    /// both vertices share the one priority (0, even), and vertex 0 can always be returned to, so
    /// `{0, 1}` is a textbook fatal attractor for Even.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_detect_fatal_attractors_finds_the_fixture_cycle() {
        let game = forced_cycle_fixture();

        let manager = oxidd::ldd::new_manager(1 << 12, 1 << 12, 1);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let (symbolic, all_vertices, cubes) = encode_parity_game(&manager, &game, &mut rng, 2, 1, false).unwrap();

        let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
        let empty = manager.with_manager_shared(LDDFunction::empty_set).unwrap();
        let incomplete = empty.clone();
        let initial = manager
            .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
            .unwrap();
        let epg = ExtendedParityGame {
            game: &symbolic,
            initial_vertex: &initial,
            vertices: &all_vertices,
            sinks: &empty_sinks,
        };

        let winning = detect_fatal_attractors_within_safe_vertices(
            &epg,
            &incomplete,
            &empty,
            &empty,
            &silent_attractor_progress(),
        )
        .unwrap();

        for v in [0usize, 1] {
            assert!(
                merc_symbolic::element_of(&manager, &cubes[v], &winning[Player::Even.to_index()]),
                "vertex {v} must be won by Even via the fatal attractor {{0, 1}}"
            );
        }
    }
}
