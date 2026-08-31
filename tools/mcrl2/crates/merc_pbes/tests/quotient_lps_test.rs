//! Integration tests for [`QuotientLps`], moved out of `src/quotient_lps.rs`
//! since they only exercise the crate's public API.

use std::sync::Arc;

use mcrl2::Pbes;
use mcrl2::SrfPbes;
use merc_explore::CacheLPS;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_utilities::MercError;
use merc_utilities::Timing;
use merc_vpg::PG;
use merc_vpg::ParityGameBuilder;
use merc_vpg::VertexIndex;
use merc_vpg::solve_zielonka;

use merc_pbes::Bsgs;
use merc_pbes::Canonicaliser;
use merc_pbes::GapConfig;
use merc_pbes::PbesLps;
use merc_pbes::PbesSrfLps;
use merc_pbes::Permutation;
use merc_pbes::QuotientLps;
use merc_pbes::explore_pbes_impl;
use merc_pbes::symmetry_parameter_basis;

fn gap_config() -> GapConfig {
    GapConfig {
        executable: "gap".to_string(),
        dump_script: None,
    }
}

/// Converts `pbes` to SRF form and unifies its parameter lists (with the same
/// flags `symmetry_parameter_basis` uses), as every SRF-based explorer
/// requires (see `PbesSrfLps::new`).
fn unified_srf(pbes: &Pbes) -> SrfPbes {
    let mut srf = SrfPbes::from(pbes).expect("Failed to convert to SRF");
    srf.unify_parameters(false, true).expect("Failed to unify parameters");
    srf
}

/// Verifies that wrapping `PbesSrfLps` in `QuotientLps` (with trivial group)
/// produces the same parity game size as the unwrapped LPS.
#[test]
#[cfg_attr(miri, ignore)]
fn quotient_trivial_group_same_game_size() -> Result<(), MercError> {
    let pbes_text = r#"pbes
nu X(b: Bool) = X(true);
init X(true);"#;
    let pbes = Pbes::from_text(pbes_text)?;

    let lps = PbesSrfLps::new(unified_srf(&pbes))?;
    let n = lps.num_params();
    let bsgs = Arc::new(Bsgs::from_generators(&[], n, &gap_config())?);

    let timing = Timing::new();
    let plain_game = explore_pbes_impl(
        &lps,
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;

    let lps2 = PbesSrfLps::new(unified_srf(&pbes))?;
    let qlps = QuotientLps::new(lps2, Arc::new(Canonicaliser::Bsgs(bsgs)), 1);
    let quot_game = explore_pbes_impl(
        &qlps,
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;

    assert_eq!(plain_game.num_of_vertices(), quot_game.num_of_vertices());
    assert_eq!(plain_game.num_of_edges(), quot_game.num_of_edges());
    Ok(())
}

/// Verifies that `QuotientLps<CacheLPS<PbesSrfLps>>` compiles and produces a
/// valid game (wrapping order: cache inside, quotient outside).
#[test]
#[cfg_attr(miri, ignore)]
fn quotient_with_cache_compiles() -> Result<(), MercError> {
    let pbes_text = r#"pbes
nu X(b: Bool) = X(true);
init X(true);"#;
    let pbes = Pbes::from_text(pbes_text)?;
    let lps = PbesSrfLps::new(unified_srf(&pbes))?;
    let n = lps.num_params();
    let bsgs = Arc::new(Bsgs::from_generators(&[], n, &gap_config())?);

    let cached = CacheLPS::new(lps, CachingStrategy::Local);
    let qlps = QuotientLps::new(cached, Arc::new(Canonicaliser::Bsgs(bsgs)), 1);

    let timing = Timing::new();
    let game = explore_pbes_impl(
        &qlps,
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;
    assert!(game.num_of_vertices() > 0);
    Ok(())
}

/// A PBES whose two parameters are interchangeable, explored with the general
/// (non-SRF) explorer so the game also contains sink and subformula vertices.
///
/// Those carry a priority and an interned formula index where a propositional
/// variable instantiation carries parameters, so a quotient that permutes them
/// unconditionally corrupts them — the interned index in particular becomes a
/// formula that does not exist.
const SYMMETRIC_PBES: &str = r#"pbes
nu X(m: Nat, n: Nat) = X(m, n) && (Y(n, m) || Y((m + 1) mod 2, n));
mu Y(m: Nat, n: Nat) = X(m, n) || Y((n + 1) mod 2, m);
init X(0, 1);"#;

#[test]
#[cfg_attr(miri, ignore)]
fn quotient_preserves_winner_and_reduces_the_game() -> Result<(), MercError> {
    let pbes = Pbes::from_text(SYMMETRIC_PBES)?;
    let timing = Timing::new();

    let plain = explore_pbes_impl(
        &PbesLps::new(pbes.clone())?,
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;

    // `n` sizes the permutation group; the tool itself derives this same degree
    // from `symmetry_parameter_basis` rather than from a constructed LPS.
    let n = symmetry_parameter_basis(&pbes)?.len();
    let lps = PbesLps::new(pbes)?;
    let generators = vec![Permutation::from_cycle_notation("(0 1)")?];
    let bsgs = Arc::new(Bsgs::from_generators(&generators, n, &gap_config())?);
    let quotient = explore_pbes_impl(
        &QuotientLps::new(lps, Arc::new(Canonicaliser::Bsgs(bsgs)), 1),
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;

    assert!(
        quotient.num_of_vertices() < plain.num_of_vertices(),
        "swapping the two parameters is a symmetry, so the quotient must be smaller \
         (plain {} vertices, quotient {})",
        plain.num_of_vertices(),
        quotient.num_of_vertices()
    );

    let (plain_solution, _) = solve_zielonka(&plain, false);
    let (quotient_solution, _) = solve_zielonka(&quotient, false);
    assert_eq!(
        plain_solution[0][0], quotient_solution[0][0],
        "the quotient changed the winner of the initial vertex"
    );
    Ok(())
}

/// The SRF backend must reduce and preserve the winner just like the general
/// one — `--srf --symmetry` layers the quotient over [`PbesSrfLps`].
///
/// The two games are not the same size: SRF normalisation rewrites the
/// right-hand sides, so this compares the SRF game against its own quotient
/// rather than against the general explorer's.
#[test]
#[cfg_attr(miri, ignore)]
fn quotient_over_srf_preserves_winner_and_reduces_the_game() -> Result<(), MercError> {
    let pbes = Pbes::from_text(SYMMETRIC_PBES)?;
    let timing = Timing::new();

    let plain = explore_pbes_impl(
        &PbesSrfLps::new(unified_srf(&pbes))?,
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;

    let lps = PbesSrfLps::new(unified_srf(&pbes))?;
    assert_eq!(
        lps.num_params(),
        2,
        "the generator below is a transposition of the whole parameter vector"
    );
    // Both parameters have the same sort, so the transposition is `(0 1)`
    // whichever of the two orders `unify_parameters` happened to produce.
    let generators = vec![Permutation::from_cycle_notation("(0 1)")?];
    let bsgs = Arc::new(Bsgs::from_generators(&generators, lps.num_params(), &gap_config())?);
    let quotient = explore_pbes_impl(
        &QuotientLps::new(lps, Arc::new(Canonicaliser::Bsgs(bsgs)), 1),
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;

    assert!(
        quotient.num_of_vertices() < plain.num_of_vertices(),
        "swapping the two parameters is a symmetry, so the quotient must be smaller \
         (plain {} vertices, quotient {})",
        plain.num_of_vertices(),
        quotient.num_of_vertices()
    );

    let (plain_solution, _) = solve_zielonka(&plain, false);
    let (quotient_solution, _) = solve_zielonka(&quotient, false);
    assert_eq!(
        plain_solution[0][0], quotient_solution[0][0],
        "the quotient changed the winner of the initial vertex"
    );
    Ok(())
}

/// The same reduction must survive a cache layer underneath the quotient.
#[test]
#[cfg_attr(miri, ignore)]
fn quotient_over_cache_agrees_with_quotient_alone() -> Result<(), MercError> {
    let pbes = Pbes::from_text(SYMMETRIC_PBES)?;
    let generators = vec![Permutation::from_cycle_notation("(0 1)")?];
    let timing = Timing::new();

    // `n` sizes the permutation group; the tool itself derives this same degree
    // from `symmetry_parameter_basis` rather than from a constructed LPS.
    let n = symmetry_parameter_basis(&pbes)?.len();
    let lps = PbesLps::new(pbes.clone())?;
    let bsgs = Arc::new(Bsgs::from_generators(&generators, n, &gap_config())?);
    let uncached = explore_pbes_impl(
        &QuotientLps::new(lps, Arc::new(Canonicaliser::Bsgs(Arc::clone(&bsgs))), 1),
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;

    let cached = CacheLPS::new(PbesLps::new(pbes)?, CachingStrategy::Local);
    let cached = explore_pbes_impl(
        &QuotientLps::new(cached, Arc::new(Canonicaliser::Bsgs(bsgs)), 1),
        ExplorationStrategy::Bfs,
        &timing,
        ParityGameBuilder::new(VertexIndex::new(0)),
    )?;

    assert_eq!(uncached.num_of_vertices(), cached.num_of_vertices());
    assert_eq!(uncached.num_of_edges(), cached.num_of_edges());
    Ok(())
}
