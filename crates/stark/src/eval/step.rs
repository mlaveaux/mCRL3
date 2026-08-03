//! One macro-step for the whole system: every component's controller cursor
//! advances, its buffered updates are applied, then the environment runs
//! against the post-controller state and its own updates are applied.
//! That ordering — controller, then apply, then environment, then apply — is
//! the original's, and the two apply points are what make assignments read
//! the pre-phase state (see [PendingUpdate]).
//!
//! [Cursor] replaces the original's recursive tree of controller objects with
//! a plain value: a controller's entire "next state" is exactly "which named
//! state, and how many ticks left before it's live", the idle count being the
//! only thing that tree actually threads from one tick to the next. Walking a
//! state's body is one flat recursion over [CommandNode], since lowering
//! already collapsed the controller AST into that arena.

use rand::Rng;

use crate::ir::CommandNode;
use crate::ir::CommandRef;
use crate::ir::IrProgram;
use crate::ir::IrStateId;
use crate::ir::SlotId;
use crate::value::EvalError;
use crate::value::Value;

use super::expr::eval;
use super::store::Store;

/// One component's continuation between macro-steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cursor {
    /// The component ran off the end of a body with no `step`/`exec`: no
    /// effect, self-loop, forever.
    Nil,
    /// Live in `state` this tick; walk its body now.
    Run(IrStateId),
    /// Idling: `remaining` more ticks with no effect, then `Run(target)`.
    /// `remaining` is always `>= 1` by construction (see [Walk::Transitioned]
    /// at `CommandNode::Step`, which produces `Cursor::Run` directly for a
    /// zero-or-negative step count).
    Idle { remaining: u32, target: IrStateId },
}

/// A buffered `target' = value` reached while walking a command tree.
/// Pushed into a list during the walk and applied only once the whole step
/// (controller or environment) has run — see [CommandNode]'s doc comment on
/// why updates must not write through immediately.
#[derive(Clone, Copy, Debug)]
struct PendingUpdate {
    target: SlotId,
    value: Value,
}

/// The outcome of walking one command subtree for the current tick.
enum Walk {
    /// Fell off the end with no `step`/`exec` — an enclosing `Sequence`
    /// should keep walking its next sibling, if any.
    FellThrough,
    /// Hit a transition; this component's tick is over. Carries the cursor
    /// to use starting the *next* tick.
    Transitioned(Cursor),
}

/// Runs one macro-step: every component's cursor advances (all reading the
/// same pre-step state — their updates are buffered and only applied once
/// every cursor has run, matching the original's parallel composition, which
/// concatenates both sides' effects before the single apply), then the
/// environment runs against the post-controller state.
///
/// A failing evaluation anywhere in the step aborts the whole step with that
/// [EvalError] — the buffered updates from the failed phase are dropped rather
/// than half-applied, so `store` is left holding the last state that was
/// computed successfully.
pub(crate) fn macro_step<R: Rng + ?Sized>(
    program: &IrProgram,
    store: &mut Store,
    rng: &mut R,
    cursors: &mut [Cursor],
) -> Result<(), EvalError> {
    let budget = total_states(program);
    let mut updates = Vec::new();
    for cursor in cursors.iter_mut() {
        let mut exec_budget = budget;
        *cursor = run_component(program, store, rng, &mut updates, &mut exec_budget, *cursor)?;
    }
    apply_updates(store, &updates);

    if let Some(environment) = program.environment() {
        let mut env_updates = Vec::new();
        // The environment never contains `Step`/`Exec` (see `CommandNode`'s
        // doc comment), so no budget should ever be spent; 0 is a defensive
        // fallback that still can't panic or loop if that invariant is ever
        // violated by a malformed IR.
        let mut exec_budget = 0;
        run_command(program, store, rng, &mut env_updates, &mut exec_budget, environment)?;
        apply_updates(store, &env_updates);
    }
    Ok(())
}

fn apply_updates(store: &mut Store, updates: &[PendingUpdate]) {
    for update in updates {
        store.set(update.target, update.value);
    }
}

/// The total number of controller states across every component — an exec
/// chain can visit each state at most once without repeating, so this bounds
/// how many same-tick `exec` hops [run_command] will follow before
/// concluding the specification has an `exec` cycle with no intervening
/// `step` (which would otherwise recurse forever) and forcibly ending the
/// component's tick instead of overflowing the stack.
fn total_states(program: &IrProgram) -> u32 {
    program
        .components()
        .iter()
        .map(|component| component.states.len() as u32)
        .sum()
}

fn run_component<R: Rng + ?Sized>(
    program: &IrProgram,
    store: &mut Store,
    rng: &mut R,
    updates: &mut Vec<PendingUpdate>,
    exec_budget: &mut u32,
    cursor: Cursor,
) -> Result<Cursor, EvalError> {
    match cursor {
        Cursor::Nil => Ok(Cursor::Nil),
        Cursor::Idle { remaining, target } => {
            if remaining <= 1 {
                Ok(Cursor::Run(target))
            } else {
                Ok(Cursor::Idle {
                    remaining: remaining - 1,
                    target,
                })
            }
        }
        Cursor::Run(state) => match program.state(state).body {
            Some(body) => match run_command(program, store, rng, updates, exec_budget, body)? {
                Walk::FellThrough => Ok(Cursor::Nil),
                Walk::Transitioned(next) => Ok(next),
            },
            None => Ok(Cursor::Nil),
        },
    }
}

fn run_command<R: Rng + ?Sized>(
    program: &IrProgram,
    store: &mut Store,
    rng: &mut R,
    updates: &mut Vec<PendingUpdate>,
    exec_budget: &mut u32,
    id: CommandRef,
) -> Result<Walk, EvalError> {
    match *program.command(id) {
        CommandNode::Assign(update) => {
            // A missing guard is unconditionally true. A *non-boolean* guard
            // is now an error: the original mapped it (and a failed
            // evaluation) to `false`, so an assignment whose guard divided by
            // zero silently didn't happen — see `Value::as_boolean`.
            let guarded = match update.guard {
                Some(guard) => eval(program, store, rng, guard)?.as_boolean("the guard of an assignment")?,
                None => true,
            };
            if guarded {
                let value = eval(program, store, rng, update.value)?;
                updates.push(PendingUpdate {
                    target: update.target,
                    value,
                });
            }
            Ok(Walk::FellThrough)
        }
        CommandNode::IfThenElse {
            guard,
            then_branch,
            else_branch,
        } => {
            let branch = if eval(program, store, rng, guard)?.as_boolean("the condition of an `if` command")? {
                then_branch
            } else {
                else_branch
            };
            match branch {
                Some(branch) => run_command(program, store, rng, updates, exec_budget, branch),
                None => Ok(Walk::FellThrough),
            }
        }
        CommandNode::Let { slot, value, body } => {
            let value = eval(program, store, rng, value)?;
            store.set(slot, value);
            match body {
                Some(body) => run_command(program, store, rng, updates, exec_budget, body),
                None => Ok(Walk::FellThrough),
            }
        }
        CommandNode::Sequence(left, right) => match run_command(program, store, rng, updates, exec_budget, left)? {
            Walk::FellThrough => run_command(program, store, rng, updates, exec_budget, right),
            transitioned => Ok(transitioned),
        },
        CommandNode::Step { steps, target } => {
            // `k <= 0` behaves like an immediate transition to `target`
            // *starting next tick* (not this one — this tick simply ends
            // here); `k > 0` idles `k` further ticks first.
            let k = match steps {
                // A non-integer step count can't arise from a checked program
                // (`typecheck.rs` requires it numeric and lowering never
                // produces a non-integer step count), so this reports a
                // compiler bug rather than silently meaning "no delay".
                Some(steps) => eval(program, store, rng, steps)?.as_integer("a `step` count")?,
                None => 0,
            };
            let cursor = if k <= 0 {
                Cursor::Run(target)
            } else {
                Cursor::Idle {
                    remaining: k as u32,
                    target,
                }
            };
            Ok(Walk::Transitioned(cursor))
        }
        CommandNode::Exec(target) => {
            // Same-tick tail jump: `exec` delegates to `target`'s body
            // within the same tick, so its effects land in this tick too.
            if *exec_budget == 0 {
                log::error!(
                    "`exec` chain exceeded the total state budget while entering {target:?} — likely an `exec` \
                     cycle with no intervening `step`; ending this component's tick instead of looping forever"
                );
                return Ok(Walk::Transitioned(Cursor::Nil));
            }
            *exec_budget -= 1;
            match program.state(target).body {
                Some(body) => run_command(program, store, rng, updates, exec_budget, body),
                None => Ok(Walk::Transitioned(Cursor::Nil)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use test_log::test;

    use super::*;
    use crate::UntypedStarkSpecification;
    use crate::value::EvalErrorKind;

    fn build(source: &str) -> IrProgram {
        let spec = UntypedStarkSpecification::parse(source)
            .expect("should parse");

        let typed_spec = crate::StarkSpecification::from_untyped(spec)
            .expect("should check");
        IrProgram::from_spec(&typed_spec).expect("should lower")
    }

    #[test]
    fn buffered_assignments_read_pre_step_state_swap() {
        let program = build(
            r"
            global variables {
              int x = 1;
              int y = 2;
            }
            environment {
              x' = y;
              y' = x;
            }
            ",
        );
        let mut rng = StdRng::seed_from_u64(0);
        let mut store = Store::new(&program, &mut rng).expect("should initialise");
        let mut cursors = Vec::new();
        macro_step(&program, &mut store, &mut rng, &mut cursors).expect("should step");
        assert_eq!(store.state_prefix(&program), &[Value::Integer(2), Value::Integer(1)]);
    }

    #[test]
    fn a_failing_guard_aborts_the_step_instead_of_reading_as_false() {
        // Originally the guard `1 / zero > 0` evaluated to the absorbing error
        // value, resulting in false.
        let program = build(
            r"
            global variables {
              int zero = 0;
              int x = 0;
            }
            environment {
              when 1 / zero > 0 x' = 1;
            }
            ",
        );
        let mut rng = StdRng::seed_from_u64(0);
        let mut store = Store::new(&program, &mut rng).expect("should initialise");
        let mut cursors = Vec::new();

        let error =
            macro_step(&program, &mut store, &mut rng, &mut cursors).expect_err("the `1 / zero` guard divides by zero");
        assert_eq!(error.kind, EvalErrorKind::DivisionByZero);
        // The failure is anchored to the offending `1 / zero`, not reported
        // as a bare class of error.
        assert!(error.span.is_some(), "a division by zero should carry its source span");
    }

    #[test]
    fn a_non_boolean_guard_aborts_the_step() {
        // `typecheck.rs` rejects a non-boolean guard, so this is built
        // straight against the IR: `Value::as_boolean` must report it rather
        // than answering `false` the way the original did.
        assert_eq!(
            Value::Integer(1).as_boolean("the guard of an assignment"),
            Err(EvalErrorKind::ExpectedBoolean {
                context: "the guard of an assignment",
                found: crate::value::ValueKind::Integer,
            })
        );
    }

    #[test]
    fn step_idles_the_requested_number_of_ticks() {
        let program = build(
            r"
            global variables {
              int ticks = 0;
            }
            component C {
              variables { }
              controller {
                state A {
                  ticks' = ticks + 1;
                  step B;
                }
                state B {
                  ticks' = ticks + 100;
                  step A;
                }
              }
              init A
            }
            ",
        );
        let mut rng = StdRng::seed_from_u64(0);
        let mut store = Store::new(&program, &mut rng).expect("should initialise");
        let mut cursors: Vec<Cursor> = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();

        macro_step(&program, &mut store, &mut rng, &mut cursors).expect("should step");
        assert_eq!(store.state_prefix(&program), &[Value::Integer(1)]);

        macro_step(&program, &mut store, &mut rng, &mut cursors).expect("should step");
        assert_eq!(store.state_prefix(&program), &[Value::Integer(101)]);
    }

    #[test]
    fn exec_transitions_within_the_same_tick() {
        let program = build(
            r"
            global variables {
              int touched = 0;
            }
            component C {
              variables { }
              controller {
                state A {
                  exec B;
                }
                state B {
                  touched' = 1;
                  step A;
                }
              }
              init A
            }
            ",
        );
        let mut rng = StdRng::seed_from_u64(0);
        let mut store = Store::new(&program, &mut rng).expect("should initialise");
        let mut cursors: Vec<Cursor> = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();

        macro_step(&program, &mut store, &mut rng, &mut cursors).expect("should step");
        assert_eq!(store.state_prefix(&program), &[Value::Integer(1)]);
    }

    #[test]
    fn environment_runs_after_controller_updates_are_applied() {
        let program = build(
            r"
            global variables {
              int x = 0;
              int seen = 0;
            }
            component C {
              variables { }
              controller {
                state A {
                  x' = 5;
                  step A;
                }
              }
              init A
            }
            environment {
              seen' = x;
            }
            ",
        );
        let mut rng = StdRng::seed_from_u64(0);
        let mut store = Store::new(&program, &mut rng).expect("should initialise");
        let mut cursors: Vec<Cursor> = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();

        macro_step(&program, &mut store, &mut rng, &mut cursors).expect("should step");
        // The environment reads `x` *after* the controller's `x' = 5` was
        // applied, so `seen` should be `5`, not the pre-step `0`.
        assert_eq!(store.state_prefix(&program), &[Value::Integer(5), Value::Integer(5)]);
    }

    #[test]
    fn parallel_components_read_the_same_pre_step_state() {
        let program = build(
            r"
            global variables {
              int x = 1;
              int y = 1;
            }
            component Reader1 {
              variables { }
              controller {
                state A {
                  x' = y + 10;
                  step A;
                }
              }
              init A
            }
            component Reader2 {
              variables { }
              controller {
                state A {
                  y' = x + 10;
                  step A;
                }
              }
              init A
            }
            ",
        );
        let mut rng = StdRng::seed_from_u64(0);
        let mut store = Store::new(&program, &mut rng).expect("should initialise");
        let mut cursors: Vec<Cursor> = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();

        macro_step(&program, &mut store, &mut rng, &mut cursors).expect("should step");
        // Both read x=1, y=1 from the *same* pre-step state, not one
        // another's freshly-buffered update.
        assert_eq!(store.state_prefix(&program), &[Value::Integer(11), Value::Integer(11)]);
    }
}
