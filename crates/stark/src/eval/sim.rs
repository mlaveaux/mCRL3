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
use crate::value::Value;

use super::step::Cursor;
use super::step::macro_step;
use super::store::Store;

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
    store: Store,
    cursors: Vec<Cursor>,
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
    pub fn new(program: &'a IrProgram, seed: u64) -> Simulation<'a, StdRng> {
        Simulation::with_rng(program, StdRng::seed_from_u64(seed))
    }
}

impl<'a, R: Rng> Simulation<'a, R> {
    /// Builds a simulation from an already-constructed RNG — the seam a test
    /// uses to inject a deterministic/scripted generator.
    pub fn with_rng(program: &'a IrProgram, mut rng: R) -> Simulation<'a, R> {
        let store = Store::new(program, &mut rng);
        // Every component's `init` is a parallel composition of controller
        // states (`ComponentIr::initial`); flattening every component's
        // initial states into one `Vec<Cursor>` is exactly that composition
        // — `ParallelController` doesn't care which "side" a cursor came
        // from, only that every cursor advances against the same pre-step
        // state each tick (see `eval::step`).
        let cursors = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();
        Simulation {
            program,
            store,
            cursors,
            rng,
            step: 0,
        }
    }

    /// The current `[0, n_variables)` state prefix.
    pub fn state(&self) -> &[Value] {
        self.store.state_prefix(self.program)
    }

    /// The number of macro-steps taken so far.
    pub fn step_count(&self) -> u64 {
        self.step
    }

    /// Runs one macro-step — see `eval::step`'s doc comment for the exact
    /// controller-then-environment ordering.
    pub fn step(&mut self) {
        macro_step(self.program, &mut self.store, &mut self.rng, &mut self.cursors);
        self.step += 1;
    }

    /// Runs `steps` macro-steps, calling `observer.on_step` after each one.
    /// Push-based rather than returning a trajectory, so a caller can stop
    /// early or aggregate incrementally instead of paying for an eagerly
    /// collected `Vec` it may not fully need — see the module doc comment.
    pub fn run(&mut self, steps: u64, observer: &mut impl Observer) {
        for _ in 0..steps {
            self.step();
            observer.on_step(self.step, self.state());
        }
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
        let mut simulation = Simulation::new(&program, 0);
        let mut observer = RecordingObserver::default();
        simulation.run(5, &mut observer);

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
        let mut a = Simulation::new(&program, 123);
        let mut b = Simulation::new(&program, 123);
        for _ in 0..10 {
            a.step();
            b.step();
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
        let mut a = Simulation::new(&program, 1);
        let mut b = Simulation::new(&program, 2);
        a.step();
        b.step();
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
        let mut simulation = Simulation::new(&program, 0);
        let mut observer = RecordingObserver::default();
        simulation.run(2, &mut observer);
        assert_eq!(observer.trajectory.len(), 2);
        assert_eq!(simulation.state(), &[Value::Integer(2)]);
    }
}
