use std::path::Path;

use mcrl2::Pbes;
use mcrl2::SrfPbes;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_symbolic::SymbolicLpsOptions;
use merc_utilities::Timing;
use merc_vpg::ExtendedParityGame;
use merc_vpg::PG;
use merc_vpg::ParityGameBuilder;
use merc_vpg::Player;
use merc_vpg::VertexIndex;
use merc_vpg::check_strategy;
use merc_vpg::solve_symbolic_zielonka;
use merc_vpg::solve_zielonka;
use merc_vpg::verify_symbolic_solution;

use merc_pbes::explore_pbes_symbolic_game;
use merc_pbes::explore_srf_pbes;

/// Converts `pbes` to SRF form and unifies its parameter lists for the explicit path, as
/// `explore_srf_pbes` requires (see `PbesSrfLps::new`).
fn unified_srf(pbes: &Pbes) -> SrfPbes {
    let mut srf = SrfPbes::from(pbes).expect("Failed to convert to SRF");
    srf.unify_parameters(false, true).expect("Failed to unify parameters");
    srf
}

/// Converts `pbes` to SRF form and unifies its parameter lists for the symbolic path
/// (`ignore_ce_equations = true`, unlike the explicit path — see `SymbolicExploreArgs::build_srf`
/// in `main.rs`).
fn unified_symbolic_srf(pbes: &Pbes) -> SrfPbes {
    let mut srf = SrfPbes::from(pbes).expect("Failed to convert to SRF");
    srf.unify_parameters(true, false).expect("Failed to unify parameters");
    srf
}

/// Solves `pbes` both explicitly (SRF exploration + Zielonka) and symbolically (LDD reachability +
/// the symbolic Zielonka solver), and asserts the initial vertex has the same winner in both.
/// Also independently certifies the *entire* symbolic winning partition, twice: natively via
/// [`check_strategy`] (the same certificate `--verify-solution` computes, scaling the way the
/// rest of symbolic solving does) and, as a second, differently-implemented cross-check, by
/// decoding to an explicit game via [`verify_symbolic_solution`].
///
/// A reachable deadlock (a PVI with no enabled summand) is resolved identically by both paths:
/// `explore_common.rs`'s `builder.finish(true, true)` (`ParityGame::from_edges`'s `make_total`)
/// gives the deadlocked vertex a self-loop whose priority is set to the *opponent* of its owner's
/// index, and `SymbolicParityGame::compute_total_graph` assigns the deadlock to the winning set of
/// the opponent of its owner directly — both are the standard "stuck = lose" game convention, so
/// they agree by construction. Verified directly against `a.text.pbes`/`b.text.pbes` (which do
/// reach deadlocks) and the trivial `val(true)`/`val(false)` fixtures below.
fn assert_symbolic_matches_explicit(pbes: &Pbes) {
    // Normalise to positive normal form so the SRF converter accepts it (see
    // `explore_pbes_test.rs`'s `assert_general_matches_srf`).
    let mut pbes = pbes.clone();
    pbes.normalize();
    let pbes = &pbes;

    let game = explore_srf_pbes(
        unified_srf(pbes),
        ExplorationStrategy::Bfs,
        CachingStrategy::None,
        false,
        &Timing::new(),
        ParityGameBuilder::new(VertexIndex::new(0)),
    )
    .expect("SRF exploration failed");
    let (solution, _) = solve_zielonka(&game, false);
    let explicit_winner = if solution[Player::Even.to_index()][game.initial_vertex().value()] {
        Player::Even
    } else {
        Player::Odd
    };

    let storage = oxidd::ldd::new_manager(1 << 20, 1 << 20, 1);
    let symbolic = explore_pbes_symbolic_game(
        &storage,
        unified_symbolic_srf(pbes),
        &SymbolicLpsOptions::default(),
        false,
        true,
        &Timing::new(),
    )
    .expect("symbolic exploration failed");

    // Force a full solve (no early termination), so the returned partition — and hence what
    // `verify_symbolic_solution` below certifies — covers every reachable vertex, not only
    // whatever `compute_total_graph`'s sink-attractor step happened to resolve on its own.
    let (symbolic_winner, symbolic_solution) = solve_symbolic_zielonka(
        &ExtendedParityGame {
            game: &symbolic.game,
            initial_vertex: &symbolic.initial_vertex,
            vertices: &symbolic.vertices,
            sinks: &symbolic.sinks,
        },
        false,
    )
    .expect("symbolic solve failed");

    assert_eq!(
        explicit_winner, symbolic_winner,
        "explicit and symbolic solvers disagree on the initial vertex's winner"
    );

    check_strategy(
        &symbolic.game,
        &symbolic.initial_vertex,
        &symbolic.vertices,
        &symbolic_solution,
    )
    .expect("native strategy certification failed");

    verify_symbolic_solution(&storage, &symbolic.game, &symbolic.vertices, &symbolic_solution)
        .expect("symbolic solution failed independent certification");
}

fn assert_symbolic_matches_explicit_from_text(text: &str) {
    let pbes = Pbes::from_text(text).expect("Failed to parse PBES");
    assert_symbolic_matches_explicit(&pbes);
}

fn assert_symbolic_matches_explicit_from_file(text_pbes_relative_path: &str) {
    let text_pbes_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(text_pbes_relative_path);
    assert!(
        text_pbes_path.exists(),
        "Text PBES file not found: {}",
        text_pbes_path.display()
    );

    let pbes = Pbes::from_text_file(text_pbes_path.to_str().unwrap()).expect("Failed to read text PBES");
    assert_symbolic_matches_explicit(&pbes);
}

#[test]
#[cfg_attr(miri, ignore)] // Oxidd does not work with miri
fn test_simple_mu_true() {
    assert_symbolic_matches_explicit_from_text("pbes mu X = val(true); init X;");
}

#[test]
#[cfg_attr(miri, ignore)] // Oxidd does not work with miri
fn test_simple_mu_false() {
    assert_symbolic_matches_explicit_from_text("pbes mu X = val(false); init X;");
}

#[test]
#[cfg_attr(miri, ignore)] // Oxidd does not work with miri
fn test_simple_nu_true() {
    assert_symbolic_matches_explicit_from_text("pbes nu X = val(true); init X;");
}

#[test]
#[cfg_attr(miri, ignore)] // Oxidd does not work with miri
fn test_simple_nu_false() {
    assert_symbolic_matches_explicit_from_text("pbes nu X = val(false); init X;");
}

#[test]
#[cfg_attr(miri, ignore)] // Oxidd does not work with miri
fn test_symbolic_a_text_pbes() {
    assert_symbolic_matches_explicit_from_file("../../../../examples/pbes/a.text.pbes");
}

#[test]
#[cfg_attr(miri, ignore)] // Oxidd does not work with miri
fn test_symbolic_b_text_pbes() {
    assert_symbolic_matches_explicit_from_file("../../../../examples/pbes/b.text.pbes");
}

#[test]
#[cfg_attr(miri, ignore)] // Oxidd does not work with miri
fn test_symbolic_c_text_pbes() {
    assert_symbolic_matches_explicit_from_file("../../../../examples/pbes/c.text.pbes");
}

#[test]
#[cfg_attr(miri, ignore)] // Oxidd does not work with miri
fn test_symbolic_random_pbes_seeds() {
    use merc_syntax::random_pbes;
    use rand::SeedableRng;

    for seed in 0u64..50 {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
        // Propositional only (no quantifiers, no integers): the symbolic path enumerates every
        // parameter's domain when interning it into an LDD value, so it needs a finite one.
        let pbes_ast = random_pbes(&mut rng, 3, 2, 3, false, false);
        let text = pbes_ast.to_string();
        assert_symbolic_matches_explicit_from_text(&text);
    }
}
