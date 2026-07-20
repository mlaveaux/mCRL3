//! Runs a robustness analysis end to end over an example specification —
//! `EVALUATOR_PLAN.md`'s Milestone C, the counterpart of `simulation.rs`'s
//! Milestone B smoke test.
//!
//! The point of these tests is that the whole stack *runs and agrees with
//! itself*, not that any particular verdict is the "right" one: a verdict
//! depends on the sample size, and the small sizes used here (real analyses
//! use hundreds of samples) are chosen to keep the tests fast. What is
//! asserted is therefore structural — no evaluation fails, the same seed
//! reproduces the same answer, and a `nil` perturbation is at distance zero
//! from the unperturbed system, which must hold at any sample size.

use merc_stark::UntypedStarkSpecification;
use merc_stark::eval::Analysis;
use merc_stark::eval::AnalysisOptions;
use merc_stark::ir::IrProgram;
use merc_stark::lower;

/// Deliberately tiny: `\G[400,1000]` in the spec below drives the evolution
/// sequence out to a thousand steps, and every sample is a full trajectory.
fn options() -> AnalysisOptions {
    AnalysisOptions {
        sample_size: 2,
        scale: 1,
        bootstrap_replicas: 4,
        quantile: 1.96,
    }
}

fn build(source: &str) -> IrProgram {
    let spec = UntypedStarkSpecification::parse(source)
        .unwrap_or_else(|e| panic!("failed to parse: {e}"))
        .check()
        .unwrap_or_else(|d| panic!("failed to check:\n{}", d.render(source)));
    lower(&spec).unwrap_or_else(|d| panic!("failed to lower:\n{}", d.render(source)))
}

/// A biochemical model with one penalty, one perturbation, and a `\G`
/// distance over a long interval — the shape almost every example with a
/// `formula` has.
const ISOCITRATE: &str = include_str!("../../../examples/stark/isocitrate.stark");

#[test]
fn checks_a_formula_from_an_example_specification() {
    let program = build(ISOCITRATE);
    let mut analysis = Analysis::new(&program, 0, options()).expect("should initialise");
    let mut sequence = analysis.sample().expect("should sample");

    let formula = program.formula_decls()[0].root;
    analysis
        .check(&mut sequence, 0, formula)
        .unwrap_or_else(|e| panic!("three-valued check failed: {e}"));
}

#[test]
fn both_semantics_run_over_an_example_specification() {
    let program = build(ISOCITRATE);
    let mut analysis = Analysis::new(&program, 1, options()).expect("should initialise");
    let mut sequence = analysis.sample().expect("should sample");

    let formula = program.formula_decls()[0].root;
    analysis
        .check_boolean(&mut sequence, 0, formula)
        .unwrap_or_else(|e| panic!("boolean check failed: {e}"));
}

#[test]
fn the_same_seed_reproduces_the_same_distance() {
    let program = build(ISOCITRATE);
    let distance = program.distance_decls()[0].root;
    let perturbation = program.perturbation_decls()[0].root;

    let computed: Vec<f64> = (0..2)
        .map(|_| {
            let mut analysis = Analysis::new(&program, 7, options()).expect("should initialise");
            let mut sequence = analysis.sample().expect("should sample");
            analysis
                .distance_under(&mut sequence, 0, distance, perturbation)
                .expect("should compute")
        })
        .collect();

    assert_eq!(computed[0], computed[1]);
}

#[test]
fn a_nil_perturbation_leaves_the_system_at_distance_zero() {
    // Appending a `nil` perturbation to a real specification: perturbing by
    // nothing must be indistinguishable from not perturbing, whatever the
    // sample size and however stochastic the model is. This is the one
    // assertion in this file that is a genuine semantic invariant rather
    // than a smoke test.
    let program = build(&format!("{ISOCITRATE}\nperturbation nothing = nil;\n"));
    let mut analysis = Analysis::new(&program, 3, options()).expect("should initialise");
    let mut sequence = analysis.sample().expect("should sample");

    let distance = program.distance_decls()[0].root;
    let nothing = program
        .perturbation_decls()
        .iter()
        .find(|decl| decl.name == "nothing")
        .expect("the appended perturbation")
        .root;

    let computed = analysis
        .distance_under(&mut sequence, 0, distance, nothing)
        .expect("should compute");
    assert_eq!(computed, 0.0);
}
