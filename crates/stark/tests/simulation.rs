//! Runs the evaluator end-to-end over a sample of example specifications for
//! a fixed number of steps under a fixed seed, and asserts every step
//! succeeds — a smoke test for `EVALUATOR_PLAN.md`'s Milestone B
//! (simulation).
//!
//! This used to assert only that no step produced an all-`Value::Error`
//! state, which was the strongest check available while a failed evaluation
//! was a *value*: a single errored variable, or a guard that silently
//! evaluated to `false` because its expression failed, both slipped through.
//! Now that evaluation returns a `Result` (see `value.rs`), any failure
//! anywhere in a step surfaces here as an `Err`.
//!
//! The sample covers specifications the *evaluator* handles today: every
//! example lowers (`tests/lowering.rs` covers all of them), but evaluating
//! `perturbation`/`distance`/`formula` is Milestone C and not yet
//! implemented, so specs relying on them are exercised only up to lowering.

use merc_stark::UntypedStarkSpecification;
use merc_stark::eval::RecordingObserver;
use merc_stark::eval::Simulation;
use merc_stark::lower;
use test_case::test_case;

#[test_case(include_str!("../../../examples/stark/random_walk.stark") ; "random_walk.stark")]
#[test_case(include_str!("../../../examples/stark/multiscler.stark") ; "multiscler.stark")]
#[test_case(include_str!("../../../examples/stark/polistil_race.stark") ; "polistil_race.stark")]
fn runs_fifty_steps_without_erroring(source: &str) {
    let spec = UntypedStarkSpecification::parse(source)
        .unwrap_or_else(|e| panic!("failed to parse: {e}"))
        .check()
        .unwrap_or_else(|d| panic!("failed to check:\n{}", d.render(source)));
    let program = lower(&spec).unwrap_or_else(|d| panic!("failed to lower:\n{}", d.render(source)));

    let mut simulation = Simulation::new(&program, 0).unwrap_or_else(|e| panic!("failed to initialise: {e}"));
    let mut observer = RecordingObserver::default();
    if let Err(e) = simulation.run(50, &mut observer) {
        panic!("failed at step {}: {e}", simulation.step_count() + 1);
    }

    assert_eq!(observer.trajectory.len(), 50);
}

#[test]
fn same_seed_reproduces_the_same_trajectory() {
    let source = include_str!("../../../examples/stark/random_walk.stark");
    let spec = UntypedStarkSpecification::parse(source)
        .expect("should parse")
        .check()
        .expect("should check");
    let program = lower(&spec).expect("should lower");

    let mut a = Simulation::new(&program, 42).expect("should initialise");
    let mut observer_a = RecordingObserver::default();
    a.run(20, &mut observer_a).expect("should run");

    let mut b = Simulation::new(&program, 42).expect("should initialise");
    let mut observer_b = RecordingObserver::default();
    b.run(20, &mut observer_b).expect("should run");

    assert_eq!(observer_a.trajectory, observer_b.trajectory);
}
