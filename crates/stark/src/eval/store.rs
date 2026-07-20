//! The flat evaluator store: one `Vec<Value>` indexed directly by [SlotId],
//! matching `IR_LOWERING_PLAN.md`'s slot layout (`[0, n_variables)` state,
//! `[n_variables, n_globals)` `const`/`param`, `[n_globals, n_slots)` scratch)
//! instead of `StarkStore.java`'s `variable -> value` closure/map.

use rand::Rng;

use crate::ir::IrProgram;
use crate::ir::SlotId;
use crate::value::EvalError;
use crate::value::Value;

use super::expr::eval;

/// `store[slot]`, sized to [IrProgram::n_slots] and indexed by [SlotId] —
/// see the module doc comment.
#[derive(Clone, Debug)]
pub(crate) struct Store {
    slots: Vec<Value>,
}

impl Store {
    /// Builds a store sized to `program` and runs startup initialisation:
    /// every [crate::ir::GlobalInit] (`const`/`param`) in declaration order
    /// (already a valid order — no forward references), then every
    /// variable's `initial_value`.
    ///
    /// `rng` is threaded through even though `typecheck.rs` disallows
    /// sampling directly in a global/variable initializer (`random_allowed:
    /// false` there) — a *function call* reached from one is still allowed
    /// to sample internally (`random_allowed: true` for function bodies), so
    /// [eval] needs an `Rng` regardless of whether this particular call
    /// tree happens to use it.
    /// Every slot starts as `Integer(0)` rather than a dedicated "unset"
    /// marker. `Value::Error` used to serve as that marker, which conflated
    /// "not written yet" with "an operation failed" (see `value.rs`); with
    /// errors moved to `Result`, no marker is needed, because no slot is ever
    /// read before it is written: globals and variables are initialised here
    /// in dependency order, and lowering guarantees a function's argument and
    /// `let` slots are written at the call/binding before its body can load
    /// them (`IR_LOWERING_PLAN.md`, "Why one flat slot space works").
    pub(crate) fn new<R: Rng + ?Sized>(program: &IrProgram, rng: &mut R) -> Result<Store, EvalError> {
        let mut store = Store {
            slots: vec![Value::Integer(0); program.n_slots() as usize],
        };
        for global in program.globals() {
            let value = eval(program, &mut store, rng, global.value)?;
            store.set(global.slot, value);
        }
        for variable in program.variables() {
            let value = eval(program, &mut store, rng, variable.initial_value)?;
            store.set(variable.slot, value);
        }
        Ok(store)
    }

    pub(crate) fn load(&self, slot: SlotId) -> Value {
        self.slots[slot.value() as usize]
    }

    pub(crate) fn set(&mut self, slot: SlotId, value: Value) {
        self.slots[slot.value() as usize] = value;
    }

    /// The `[0, n_variables)` prefix that a simulation checkpoints — exactly
    /// what `EvolutionSequence`/`SampleSet` would sample in the Java
    /// reference (see `EVALUATOR_PLAN.md`'s Step 5).
    pub(crate) fn state_prefix(&self, program: &IrProgram) -> &[Value] {
        &self.slots[0..program.n_variables() as usize]
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use test_log::test;

    use super::*;
    use crate::UntypedStarkSpecification;
    use crate::lower;

    fn lower_source(source: &str) -> IrProgram {
        let spec = UntypedStarkSpecification::parse(source)
            .expect("should parse")
            .check()
            .expect("should check");
        lower(&spec).expect("should lower")
    }

    #[test]
    fn runs_globals_then_variable_initial_values() {
        let program = lower_source(
            r"
            const c = 2;
            param p = c * 3;
            global variables {
              int x range [0, 100] = p + 1;
            }
            ",
        );
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let store = Store::new(&program, &mut rng).expect("should initialise");

        let state = store.state_prefix(&program);
        assert_eq!(state, &[Value::Integer(7)]);
    }
}
