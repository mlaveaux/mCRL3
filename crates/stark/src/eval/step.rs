//! One macro-step for the whole system: every component's controller cursor
//! advances, its buffered updates are applied, then the environment runs
//! against the post-controller state and its own updates are applied.
//! Mirrors `ControlledSystem.sampleNext`:
//!
//! ```java
//! public SystemState sampleNext(RandomGenerator rg) {
//!     EffectStep<Controller> step = controller.next(rg, state);
//!     int c_step = state.getStep();
//!     DataState newState = environment.apply(rg, state.apply(step.effect()));
//!     newState.setStep(c_step+1);
//!     return new ControlledSystem(step.next(), environment, newState);
//! }
//! ```
//!
//! [Cursor] replaces Java's recursive `Controller` object tree
//! (`StepController`/`ExecController`/`AssignmentController`/`NilController`/
//! `ParallelController`) with a plain value: a controller's entire "next
//! state" is exactly "which named state, and how many ticks left before it's
//! live" (`StepController`'s idle count is the only state Java's controller
//! tree actually threads through `next()`). Walking a state's body is one
//! flat recursion over [CommandNode] instead of a tree of controller
//! objects, since lowering already collapsed the controller AST into that
//! arena (see `IR_LOWERING_PLAN.md`'s Step 4).

use rand::Rng;

use crate::ir::CommandNode;
use crate::ir::CommandRef;
use crate::ir::IrProgram;
use crate::ir::IrStateId;
use crate::ir::SlotId;
use crate::value::Value;

use super::expr::eval;
use super::store::Store;

/// One component's continuation between macro-steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cursor {
    /// The component ran off the end of a body with no `step`/`exec` —
    /// `NilController`: no effect, self-loop, forever.
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
/// Mirrors Java's `DataStateUpdate`: pushed into a list during the walk,
/// applied only once the whole step (controller or environment) has run —
/// see [CommandNode]'s doc comment on why updates must not write through
/// immediately.
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
/// every cursor has run, matching `ParallelController`'s "both effects
/// concatenated before the single `apply`"), then the environment runs
/// against the post-controller state.
pub(crate) fn macro_step<R: Rng + ?Sized>(program: &IrProgram, store: &mut Store, rng: &mut R, cursors: &mut [Cursor]) {
    let budget = total_states(program);
    let mut updates = Vec::new();
    for cursor in cursors.iter_mut() {
        let mut exec_budget = budget;
        *cursor = run_component(program, store, rng, &mut updates, &mut exec_budget, *cursor);
    }
    apply_updates(store, &updates);

    if let Some(environment) = program.environment() {
        let mut env_updates = Vec::new();
        // The environment never contains `Step`/`Exec` (`IR_LOWERING_PLAN.md`
        // Step 4), so no budget should ever be spent; 0 is a defensive
        // fallback that still can't panic or loop if that invariant is ever
        // violated by a malformed IR.
        let mut exec_budget = 0;
        run_command(program, store, rng, &mut env_updates, &mut exec_budget, environment);
        apply_updates(store, &env_updates);
    }
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
) -> Cursor {
    match cursor {
        Cursor::Nil => Cursor::Nil,
        Cursor::Idle { remaining, target } => {
            if remaining <= 1 {
                Cursor::Run(target)
            } else {
                Cursor::Idle {
                    remaining: remaining - 1,
                    target,
                }
            }
        }
        Cursor::Run(state) => match program.state(state).body {
            Some(body) => match run_command(program, store, rng, updates, exec_budget, body) {
                Walk::FellThrough => Cursor::Nil,
                Walk::Transitioned(next) => next,
            },
            None => Cursor::Nil,
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
) -> Walk {
    match *program.command(id) {
        CommandNode::Assign(update) => {
            // `StarkValue.isTrue` semantics: a missing guard is
            // unconditionally true; a non-boolean guard is false, not an
            // error (see `Value::truthy`'s doc comment).
            let guarded = match update.guard {
                Some(guard) => eval(program, store, rng, guard).truthy(),
                None => true,
            };
            if guarded {
                let value = eval(program, store, rng, update.value);
                updates.push(PendingUpdate {
                    target: update.target,
                    value,
                });
            }
            Walk::FellThrough
        }
        CommandNode::IfThenElse {
            guard,
            then_branch,
            else_branch,
        } => {
            let branch = if eval(program, store, rng, guard).truthy() {
                then_branch
            } else {
                else_branch
            };
            match branch {
                Some(branch) => run_command(program, store, rng, updates, exec_budget, branch),
                None => Walk::FellThrough,
            }
        }
        CommandNode::Let { slot, value, body } => {
            let value = eval(program, store, rng, value);
            store.set(slot, value);
            match body {
                Some(body) => run_command(program, store, rng, updates, exec_budget, body),
                None => Walk::FellThrough,
            }
        }
        CommandNode::Sequence(left, right) => match run_command(program, store, rng, updates, exec_budget, left) {
            Walk::FellThrough => run_command(program, store, rng, updates, exec_budget, right),
            transitioned => transitioned,
        },
        CommandNode::Step { steps, target } => {
            // `StepController`: `k <= 0` behaves like an immediate
            // transition to `target` *starting next tick* (not this one —
            // this tick simply ends here); `k > 0` idles `k` further ticks
            // first.
            let k = match steps {
                Some(steps) => match eval(program, store, rng, steps) {
                    Value::Integer(v) => v,
                    // A non-integer step count can't arise from a checked
                    // program (`typecheck.rs` requires it numeric and
                    // lowering never produces a non-integer step count);
                    // treat it as "no delay" rather than panicking.
                    _ => 0,
                },
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
            Walk::Transitioned(cursor)
        }
        CommandNode::Exec(target) => {
            // Same-tick tail jump: `ExecController.next` immediately
            // delegates to `target`'s controller within the same call, so
            // its effects land in this tick too.
            if *exec_budget == 0 {
                log::error!(
                    "`exec` chain exceeded the total state budget while entering {target:?} — likely an `exec` \
                     cycle with no intervening `step`; ending this component's tick instead of looping forever"
                );
                return Walk::Transitioned(Cursor::Nil);
            }
            *exec_budget -= 1;
            match program.state(target).body {
                Some(body) => run_command(program, store, rng, updates, exec_budget, body),
                None => Walk::Transitioned(Cursor::Nil),
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
    use crate::lower;

    fn build(source: &str) -> IrProgram {
        let spec = UntypedStarkSpecification::parse(source)
            .expect("should parse")
            .check()
            .expect("should check");
        lower(&spec).expect("should lower")
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
        let mut store = Store::new(&program, &mut rng);
        let mut cursors = Vec::new();
        macro_step(&program, &mut store, &mut rng, &mut cursors);
        assert_eq!(store.state_prefix(&program), &[Value::Integer(2), Value::Integer(1)]);
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
        let mut store = Store::new(&program, &mut rng);
        let mut cursors: Vec<Cursor> = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();

        macro_step(&program, &mut store, &mut rng, &mut cursors);
        assert_eq!(store.state_prefix(&program), &[Value::Integer(1)]);

        macro_step(&program, &mut store, &mut rng, &mut cursors);
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
        let mut store = Store::new(&program, &mut rng);
        let mut cursors: Vec<Cursor> = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();

        macro_step(&program, &mut store, &mut rng, &mut cursors);
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
        let mut store = Store::new(&program, &mut rng);
        let mut cursors: Vec<Cursor> = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();

        macro_step(&program, &mut store, &mut rng, &mut cursors);
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
        let mut store = Store::new(&program, &mut rng);
        let mut cursors: Vec<Cursor> = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();

        macro_step(&program, &mut store, &mut rng, &mut cursors);
        // Both read x=1, y=1 from the *same* pre-step state, not one
        // another's freshly-buffered update.
        assert_eq!(store.state_prefix(&program), &[Value::Integer(11), Value::Integer(11)]);
    }
}
