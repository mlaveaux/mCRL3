//! The perturbation coroutine: a value that, tick by tick, decides whether
//! the state it is attached to gets rewritten and how. The original's
//! three-method interface — "what is your effect *now*", "what are you
//! *next*", "are you done" — carries over verbatim, since that is the whole
//! semantics of a perturbation.
//!
//! Like [super::step]'s [Cursor](super::step::Cursor) replacing a recursive
//! tree of controller objects, [PerturbationState] replaces an object graph
//! of atomic, sequential, iterative and empty perturbations with one plain
//! enum: an atomic perturbation's *static* part (which slots, which value
//! expressions) stays in the [PerturbationIr] arena and is referenced by
//! [PerturbationId], so this value only carries what actually changes over
//! time — the countdowns.
//!
//! The original has two further cases, a delay wrapping a whole
//! sub-perturbation and an indefinitely repeating one, which the grammar
//! cannot produce; [PerturbationIr] matches the reachable subset exactly.
//! See `plan.md`.

use rand::Rng;

use crate::ir::IrProgram;
use crate::ir::PerturbationId;
use crate::ir::PerturbationIr;
use crate::value::EvalError;
use crate::value::EvalErrorKind;

use super::expr::eval;
use super::store::Store;

/// A perturbation's remaining schedule. Immutable: [PerturbationState::step]
/// returns the successor rather than mutating, as in the original.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PerturbationState {
    /// `nil`: no effect, self-loop, already done.
    None,
    /// `[..]@time`: fires `node`'s assignments once `after_steps` ticks have
    /// elapsed. `node` is always a [PerturbationIr::Atomic].
    Atomic { after_steps: i64, node: PerturbationId },
    /// `a ; b`: `first` runs until it is done, then `second`.
    Sequence(Box<PerturbationState>, Box<PerturbationState>),
    /// `a ^ n`: `body`, repeated `replica` times. `body` is kept *pristine*
    /// (never stepped), because each repetition is seeded from the original
    /// body rather than from the previous repetition's remainder.
    Iterative { replica: i64, body: Box<PerturbationState> },
}

impl PerturbationState {
    /// Builds the initial schedule for a perturbation declaration.
    ///
    /// The `@time` and `^iterations` counts are [crate::ir::ExprRef]s in the
    /// IR rather than folded constants, so they are evaluated here, once, at
    /// construction — the same point the original evaluates them while
    /// building the perturbation. `globals` is the store holding the
    /// program's `const`/`param` slots, which is all such a bound can legally
    /// refer to.
    pub(crate) fn build<R: Rng + ?Sized>(
        program: &IrProgram,
        globals: &mut Store,
        rng: &mut R,
        id: PerturbationId,
    ) -> Result<PerturbationState, EvalError> {
        Ok(match program.perturbation(id) {
            PerturbationIr::Nil => PerturbationState::None,
            // A `Reference` is already resolved to the referent's root node,
            // so following it is a plain recursion. The original shares one
            // perturbation object between every reference to a declaration;
            // building a fresh (equal) value per reference is equivalent,
            // because a perturbation is immutable — stepping it returns a new
            // one rather than mutating the shared instance.
            PerturbationIr::Reference(target) => PerturbationState::build(program, globals, rng, *target)?,
            PerturbationIr::Atomic { time, .. } => PerturbationState::Atomic {
                after_steps: eval(program, globals, rng, *time)?.as_integer("the `@` time of a perturbation")?,
                node: id,
            },
            PerturbationIr::Sequence(left, right) => PerturbationState::Sequence(
                Box::new(PerturbationState::build(program, globals, rng, *left)?),
                Box::new(PerturbationState::build(program, globals, rng, *right)?),
            ),
            PerturbationIr::Iteration { argument, iterations } => PerturbationState::Iterative {
                replica: eval(program, globals, rng, *iterations)?
                    .as_integer("the `^` iteration count of a perturbation")?,
                body: Box::new(PerturbationState::build(program, globals, rng, *argument)?),
            },
        })
    }

    /// The atomic node whose assignments fire on *this* tick, if any —
    /// `Perturbation.effect()`.
    pub(crate) fn effect(&self) -> Option<PerturbationId> {
        match self {
            PerturbationState::None => None,
            PerturbationState::Atomic { after_steps, node } => (*after_steps <= 0).then_some(*node),
            PerturbationState::Sequence(first, second) => {
                if first.is_done() {
                    second.effect()
                } else {
                    first.effect()
                }
            }
            PerturbationState::Iterative { replica, body } => {
                if *replica > 0 {
                    body.effect()
                } else {
                    None
                }
            }
        }
    }

    /// The schedule for the next tick — `Perturbation.step()`.
    pub(crate) fn step(self) -> PerturbationState {
        match self {
            PerturbationState::None => PerturbationState::None,
            PerturbationState::Atomic { after_steps, node } => {
                if after_steps <= 0 {
                    // An atomic perturbation fires exactly once.
                    PerturbationState::None
                } else {
                    PerturbationState::Atomic {
                        after_steps: after_steps - 1,
                        node,
                    }
                }
            }
            PerturbationState::Sequence(first, second) => {
                if first.is_done() {
                    second.step()
                } else {
                    PerturbationState::Sequence(Box::new(first.step()), second)
                }
            }
            PerturbationState::Iterative { replica, body } => {
                if replica > 0 {
                    // The current repetition advances one tick, with the
                    // remaining `replica - 1` repetitions queued behind it —
                    // each seeded from the *pristine* body, which is why
                    // `body` is never stepped in place.
                    PerturbationState::Sequence(
                        Box::new(body.as_ref().clone().step()),
                        Box::new(PerturbationState::Iterative {
                            replica: replica - 1,
                            body,
                        }),
                    )
                } else {
                    PerturbationState::None
                }
            }
        }
    }

    /// Whether this schedule can still produce an effect. Note an `Atomic`
    /// is *never* done in the original, even once it has fired: only
    /// stepping retires it (to `None`), and only a `Sequence` ever asks.
    pub(crate) fn is_done(&self) -> bool {
        match self {
            PerturbationState::None => true,
            PerturbationState::Atomic { .. } => false,
            PerturbationState::Sequence(first, second) => first.is_done() && second.is_done(),
            PerturbationState::Iterative { replica, .. } => *replica <= 0,
        }
    }
}

/// Applies one atomic perturbation node's assignments to `store`.
///
/// **Buffered**, like a controller assignment: every right-hand side is
/// evaluated against the pre-perturbation store before any of them is
/// written, matching the original, which materialises the whole update list
/// against the pre-perturbation state before applying it.
pub(crate) fn apply_effect<R: Rng + ?Sized>(
    program: &IrProgram,
    store: &mut Store,
    rng: &mut R,
    node: PerturbationId,
) -> Result<(), EvalError> {
    let PerturbationIr::Atomic { assignments, .. } = program.perturbation(node) else {
        // `effect()` only ever returns the id of an `Atomic` node.
        return Err(EvalErrorKind::Unreachable("a non-atomic perturbation produced an effect").into());
    };
    let mut values = Vec::with_capacity(assignments.len());
    for assignment in assignments {
        values.push((assignment.target, eval(program, store, rng, assignment.value)?));
    }
    for (target, value) in values {
        store.set(target, value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use test_log::test;

    use super::*;
    use crate::StarkSpecification;
    use crate::UntypedStarkSpecification;

    /// Builds the program's single perturbation declaration's initial state.
    fn build(source: &str) -> (IrProgram, PerturbationState) {
        let untyped = UntypedStarkSpecification::parse(source).expect("should parse");
        let spec = StarkSpecification::from_untyped(untyped).expect("should check");
        let program = IrProgram::from_spec(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);
        let mut globals = Store::new(&program, &mut rng).expect("should initialise");
        let root = program.perturbation_decls().last().expect("a perturbation").root;
        let state = PerturbationState::build(&program, &mut globals, &mut rng, root).expect("should build");
        (program, state)
    }

    /// The sequence of ticks at which `state` produces an effect, over
    /// `ticks` ticks — the observable behaviour of a schedule.
    fn firing_ticks(mut state: PerturbationState, ticks: usize) -> Vec<usize> {
        let mut fired = Vec::new();
        for tick in 0..ticks {
            if state.effect().is_some() {
                fired.push(tick);
            }
            state = state.step();
        }
        fired
    }

    const PREAMBLE: &str = r"
        global variables {
          int x = 0;
        }
    ";

    #[test]
    fn an_atomic_perturbation_fires_once_at_its_time() {
        let (_, state) = build(&format!("{PREAMBLE} perturbation p = [x <- 1]@3;"));
        assert_eq!(firing_ticks(state, 8), vec![3]);
    }

    #[test]
    fn an_atomic_perturbation_at_time_zero_fires_immediately() {
        let (_, state) = build(&format!("{PREAMBLE} perturbation p = [x <- 1]@0;"));
        assert_eq!(firing_ticks(state, 5), vec![0]);
    }

    #[test]
    fn an_iteration_repeats_the_body_once_per_tick() {
        // `[x <- 1]@0` fires immediately, so iterating it `3` times fires on
        // three consecutive ticks — each repetition is queued behind the
        // previous one's remainder.
        let (_, state) = build(&format!("{PREAMBLE} perturbation p = ([x <- 1]@0)^3;"));
        assert_eq!(firing_ticks(state, 8), vec![0, 1, 2]);
    }

    #[test]
    fn a_sequence_runs_the_second_only_after_the_first_is_done() {
        let (_, state) = build(&format!("{PREAMBLE} perturbation p = [x <- 1]@1;[x <- 2]@2;"));
        // The first fires at tick 1 and retires; the second's own `@2`
        // countdown then starts from *there*, not from tick 0.
        assert_eq!(firing_ticks(state, 10), vec![1, 4]);
    }

    #[test]
    fn nil_never_fires() {
        let (_, state) = build(&format!("{PREAMBLE} perturbation p = nil;"));
        assert_eq!(firing_ticks(state, 5), Vec::<usize>::new());
        assert!(PerturbationState::None.is_done());
    }

    #[test]
    fn a_reference_behaves_like_the_declaration_it_names() {
        let (_, referenced) = build(&format!(
            "{PREAMBLE} perturbation base = [x <- 1]@2; perturbation p = base;"
        ));
        let (_, direct) = build(&format!("{PREAMBLE} perturbation p = [x <- 1]@2;"));
        assert_eq!(firing_ticks(referenced, 6), firing_ticks(direct, 6));
    }

    #[test]
    fn assignments_are_buffered_so_they_read_the_pre_perturbation_state() {
        let source = r"
            global variables {
              int x = 1;
              int y = 2;
            }
            perturbation swap = [x <- y, y <- x]@0;
        ";
        let untyped = UntypedStarkSpecification::parse(source).expect("should parse");
        let spec = StarkSpecification::from_untyped(untyped).expect("should check");
        let program = IrProgram::from_spec(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);
        let mut store = Store::new(&program, &mut rng).expect("should initialise");
        let root = program.perturbation_decls()[0].root;
        let state = PerturbationState::build(&program, &mut store.clone(), &mut rng, root).expect("should build");

        let node = state.effect().expect("should fire at tick 0");
        apply_effect(&program, &mut store, &mut rng, node).expect("should apply");

        use crate::value::Value;
        assert_eq!(store.state_prefix(&program), &[Value::Integer(2), Value::Integer(1)]);
    }
}
