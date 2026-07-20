//! The public entry point for running a specification. [Simulation] owns the
//! store and every component's controller cursor and steps the whole system
//! one macro-step at a time, matching `ControlledSystem`'s role in the Java
//! reference (see `eval::step`'s doc comment for the exact per-step
//! ordering).
//!
//! Deliberately **push-based**: [Simulation::run] takes an [Observer] and
//! calls it after every step, rather than building an eager
//! `Vec<Vec<Value>>` trajectory. A caller can stop early, aggregate on the
//! fly, or (later) drive an ensemble of independently-seeded [Simulation]s
//! to build the `SampleSet`-style evolution sequence `EvolutionSequence.java`
//! models — `SampleSet<SystemState>`, sampled and regenerated lazily via
//! `generateUpTo` — without [Simulation] itself needing to change: an
//! ensemble driver is just "N `Simulation`s, one `Observer` that collects
//! across them," built on top of this, not into it.

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::ir::IrProgram;
use crate::value::EvalError;
use crate::value::Value;

use super::system::SystemState;

/// Notified after every macro-step (see [Simulation::run]).
pub trait Observer {
    /// `step` is the number of macro-steps taken so far (`1` after the
    /// first); `state` is the `[0, n_variables)` state prefix — exactly what
    /// `EvolutionSequence`/`SampleSet` would checkpoint in the Java
    /// reference.
    fn on_step(&mut self, step: u64, state: &[Value]);
}

/// An [Observer] that records every state it's given — the eager
/// `Vec<Vec<Value>>` trajectory, for callers that do want the whole thing
/// materialised (most tests, small examples) rather than driving the push
/// callback themselves.
#[derive(Default)]
pub struct RecordingObserver {
    pub trajectory: Vec<Vec<Value>>,
}

impl Observer for RecordingObserver {
    fn on_step(&mut self, _step: u64, state: &[Value]) {
        self.trajectory.push(state.to_vec());
    }
}

/// A running instance of a checked, lowered specification: the store, every
/// component's controller cursor, the step counter, and the RNG stream.
/// Mirrors `ControlledSystem`, minus the `Controller`/`DataStateFunction`
/// indirection lowering already collapsed into `program`.
pub struct Simulation<'a, R: Rng> {
    program: &'a IrProgram,
    state: SystemState,
    rng: R,
    step: u64,
}

impl<'a> Simulation<'a, StdRng> {
    /// Builds a simulation seeded from a `u64`, for reproducibility.
    /// **Not** bit-compatible with the Java reference's Mersenne-Twister
    /// stream — a different PRNG makes that infeasible, so only the
    /// *distributions* match; this port's own stream is reproducible from
    /// this seed, which is what matters for regression tests and for
    /// building an ensemble from independent substreams later. See
    /// `EVALUATOR_PLAN.md`'s "Deliberate deviations".
    pub fn new(program: &'a IrProgram, seed: u64) -> Result<Simulation<'a, StdRng>, EvalError> {
        Simulation::with_rng(program, StdRng::seed_from_u64(seed))
    }
}

impl<'a, R: Rng> Simulation<'a, R> {
    /// Builds a simulation from an already-constructed RNG — the seam a test
    /// uses to inject a deterministic/scripted generator.
    ///
    /// Fails if evaluating a `const`/`param` or a variable's initial value
    /// fails (e.g. a `const` that divides by zero), since there is no valid
    /// initial state to run from in that case.
    pub fn with_rng(program: &'a IrProgram, mut rng: R) -> Result<Simulation<'a, R>, EvalError> {
        let state = SystemState::new(program, &mut rng)?;
        log::debug!(
            "initialised a simulation over {} slots, {} of them variables",
            program.n_slots(),
            program.n_variables()
        );
        Ok(Simulation {
            program,
            state,
            rng,
            step: 0,
        })
    }

    /// The current `[0, n_variables)` state prefix.
    pub fn state(&self) -> &[Value] {
        self.state.variables(self.program)
    }

    /// The number of macro-steps taken so far.
    pub fn step_count(&self) -> u64 {
        self.step
    }

    /// Runs one macro-step — see `eval::step`'s doc comment for the exact
    /// controller-then-environment ordering.
    ///
    /// On an [EvalError] the step counter does not advance and the state is
    /// left as it was before the step, so a caller that wants to report the
    /// failure can still inspect [Simulation::state] for the state that
    /// triggered it.
    pub fn step(&mut self) -> Result<(), EvalError> {
        self.state.sample_next(self.program, &mut self.rng)?;
        self.step += 1;
        log::trace!("step {}: {:?}", self.step, self.state());
        Ok(())
    }

    /// Runs `steps` macro-steps, calling `observer.on_step` after each one.
    /// Push-based rather than returning a trajectory, so a caller can stop
    /// early or aggregate incrementally instead of paying for an eagerly
    /// collected `Vec` it may not fully need — see the module doc comment.
    ///
    /// Stops at the first failing step and returns its [EvalError]; the
    /// observer has already been called for every step that did succeed.
    pub fn run(&mut self, steps: u64, observer: &mut impl Observer) -> Result<(), EvalError> {
        for _ in 0..steps {
            self.step()?;
            observer.on_step(self.step, self.state());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;
    use crate::UntypedStarkSpecification;
    use crate::lower;

    fn build(source: &str) -> IrProgram {
        let spec = UntypedStarkSpecification::parse(source)
            .expect("should parse")
            .check()
            .expect("should check");
        lower(&spec).expect("should lower")
    }

    #[test]
    fn run_pushes_one_state_per_step_to_the_observer() {
        let program = build(
            r"
            global variables {
              int x = 0;
            }
            environment {
              x' = x + 1;
            }
            ",
        );
        let mut simulation = Simulation::new(&program, 0).expect("should initialise");
        let mut observer = RecordingObserver::default();
        simulation.run(5, &mut observer).expect("should run");

        assert_eq!(
            observer.trajectory,
            vec![
                vec![Value::Integer(1)],
                vec![Value::Integer(2)],
                vec![Value::Integer(3)],
                vec![Value::Integer(4)],
                vec![Value::Integer(5)],
            ]
        );
        assert_eq!(simulation.step_count(), 5);
        assert_eq!(simulation.state(), &[Value::Integer(5)]);
    }

    #[test]
    fn same_seed_is_deterministic() {
        let program = build(
            r"
            global variables {
              real x = 0.0;
            }
            environment {
              x' = R;
            }
            ",
        );
        let mut a = Simulation::new(&program, 123).expect("should initialise");
        let mut b = Simulation::new(&program, 123).expect("should initialise");
        for _ in 0..10 {
            a.step().expect("should step");
            b.step().expect("should step");
        }
        assert_eq!(a.state(), b.state());
    }

    #[test]
    fn different_seeds_diverge() {
        let program = build(
            r"
            global variables {
              real x = 0.0;
            }
            environment {
              x' = R;
            }
            ",
        );
        let mut a = Simulation::new(&program, 1).expect("should initialise");
        let mut b = Simulation::new(&program, 2).expect("should initialise");
        a.step().expect("should step");
        b.step().expect("should step");
        assert_ne!(a.state(), b.state());
    }

    #[test]
    fn observer_can_stop_early_by_running_fewer_steps() {
        let program = build(
            r"
            global variables {
              int x = 0;
            }
            environment {
              x' = x + 1;
            }
            ",
        );
        let mut simulation = Simulation::new(&program, 0).expect("should initialise");
        let mut observer = RecordingObserver::default();
        simulation.run(2, &mut observer).expect("should run");
        assert_eq!(observer.trajectory.len(), 2);
        assert_eq!(simulation.state(), &[Value::Integer(2)]);
    }
}
