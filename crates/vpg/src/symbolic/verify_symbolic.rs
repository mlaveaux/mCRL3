use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;

use merc_utilities::MercError;

use crate::PG;
use crate::ParityGameBuilder;
use crate::Player;
use crate::solve_zielonka;
use crate::symbolic::SymbolicParityGame;
use crate::symbolic::SymbolicSolution;
use crate::symbolic::convert_symbolic_parity_game;
use crate::verify_solution;

/// Independently certifies a [`SymbolicSolution`] against the game it was computed for.
///
/// # Details
///
/// [`solve_zielonka`]/[`verify_solution`] already give the explicit path a certificate checker:
/// a strategy is only trusted once it has been shown, by an independent solitaire-game fixed
/// point, to actually win every vertex it claims. The symbolic solver has no strategy of its
/// own to hand to that checker (see the module-level plan doc, §6), so this reuses the checker
/// a different way: decode the *solved* symbolic game into an explicit [`crate::ParityGame`]
/// with [`convert_symbolic_parity_game`], re-solve that explicit game from scratch with a fresh
/// [`solve_zielonka`] call (which does compute a strategy), certify that strategy with
/// [`verify_solution`], and only then compare the certified winner of every vertex against what
/// `solution` claims.
///
/// This is strictly stronger than comparing just the initial vertex's winner (as
/// `solve_symbolic_test.rs`'s `assert_symbolic_matches_explicit` already does one level up, via
/// the *SRF* exploration path rather than this LDD decoding): every vertex the symbolic solver
/// resolved is checked, and the explicit side is checked by construction, not by trusting
/// `solve_zielonka` to agree with itself.
///
/// `vertices` must be the same reachable-vertex set `solution` was computed over (i.e. the
/// `total` graph `compute_total_graph` returned, unioned back with its sinks — in practice, the
/// same `vertices` passed to [`crate::solve_symbolic_zielonka`]). Panics (via [`verify_solution`])
/// if the certified strategy is inconsistent, or if the two solvers disagree on any vertex.
pub fn verify_symbolic_solution(
    manager: &LDDManagerRef,
    game: &SymbolicParityGame,
    vertices: &LDDFunction,
    solution: &SymbolicSolution,
) -> Result<(), MercError> {
    let (decoded, cubes) = convert_symbolic_parity_game(manager, game, vertices)?;
    let total = make_parity_game_total(&decoded);

    let (certified, strategy) = solve_zielonka(&total, true);
    verify_solution(&total, &certified, &strategy.expect("compute_strategy was true"));

    for (index, cube) in cubes.iter().enumerate() {
        let singleton = manager.with_manager_shared(|m| LDDFunction::singleton(m, cube))?;
        let symbolic_winner = solution
            .winner(&singleton)
            .ok_or("verify_symbolic_solution: vertex was not resolved by the symbolic solver")?;
        let certified_winner = if certified[Player::Even.to_index()][index] {
            Player::Even
        } else {
            Player::Odd
        };

        if symbolic_winner != certified_winner {
            panic!(
                "verify_symbolic_solution: vertex {index} (cube {cube:?}) is won by {symbolic_winner} \
                 symbolically but by {certified_winner} in the independently certified solution"
            );
        }
    }

    Ok(())
}

/// Totalizes `game` using [`ParityGameBuilder::finish`]'s own `make_total`: every vertex without
/// an outgoing edge gets a self-loop whose priority is [`Player::opponent`] of its owner
/// (`ParityGame::from_edges` sets it to `owner.opponent().to_index()`), so the self-loop's unique
/// play is won by the *opponent* of the sink's owner. That is the same convention
/// [`SymbolicParityGame::compute_total_graph`] uses for its sinks (a disjunctive PBES equation
/// with no enabled summand is `false`) — this function exists only so both paths share that one
/// implementation instead of each re-deriving it, since [`convert_symbolic_parity_game`]
/// deliberately keeps sinks rather than fabricating self-loops itself.
pub(crate) fn make_parity_game_total<G: PG>(game: &G) -> crate::ParityGame {
    let mut builder = ParityGameBuilder::with_capacity(game.initial_vertex(), game.num_of_edges());
    for vertex in game.iter_vertices() {
        builder.add_vertex(vertex, game.owner(vertex), game.priority(vertex));
    }
    for vertex in game.iter_vertices() {
        for edge in game.outgoing_edges(vertex) {
            builder.add_edge(vertex, edge.to());
        }
    }
    builder.finish(true, true)
}

#[cfg(test)]
mod tests {
    use rand::RngExt;

    use oxidd::ManagerRef;
    use oxidd::ldd::LDDFunction;

    use merc_utilities::random_test;

    use crate::ExtendedParityGame;
    use crate::encode_parity_game;
    use crate::random_parity_game;
    use crate::solve_symbolic_zielonka;

    use super::verify_symbolic_solution;

    /// Cross-checks [`verify_symbolic_solution`] against random *total* games: `symbolic_zielonka`'s
    /// own random test already cross-checks the solution against the explicit solver, so this only
    /// needs to show the certificate itself is accepted.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_verify_symbolic_solution_random_total() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, true, 60, 5, 3);

            let manager = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
            let radix = rng.random_range(2..=5);
            let num_groups = rng.random_range(1..=3);
            let (symbolic, all_vertices, cubes) =
                encode_parity_game(&manager, &game, rng, radix, num_groups, false).unwrap();

            let empty_sinks = symbolic.sinks(&all_vertices, &all_vertices).unwrap();
            let initial = manager
                .with_manager_shared(|m| LDDFunction::singleton(m, &cubes[0]))
                .unwrap();

            // Force a full solve (no early termination) so every vertex is covered.
            let epg = ExtendedParityGame {
                game: &symbolic,
                initial_vertex: &initial,
                vertices: &all_vertices,
                sinks: &empty_sinks,
            };
            let (_, solution) = solve_symbolic_zielonka(&epg, false).unwrap();

            verify_symbolic_solution(&manager, &symbolic, &all_vertices, &solution).unwrap();
        });
    }

    /// Same as [`test_verify_symbolic_solution_random_total`], but for random *non-total* games,
    /// so the deadlock/sink handling is exercised too.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_verify_symbolic_solution_random_non_total() {
        random_test(100, |rng| {
            let game = random_parity_game(rng, false, 60, 5, 3);

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

            verify_symbolic_solution(&manager, &symbolic, &all_vertices, &solution).unwrap();
        });
    }
}
