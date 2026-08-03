//! The flat evaluator store: one `Vec<Value>` indexed directly by [SlotId],
//! matching the IR's slot layout (`[0, n_variables)` state,
//! `[n_variables, n_globals)` `const`/`param`, `[n_globals, n_slots)` scratch)
//! instead of the original's `variable -> value` map.

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
    /// Builds a store sized to `program` and runs startup initialisation: every
    /// [crate::ir::GlobalInit] (`const`/`param`) in declaration order, then
    /// every variable's `initial_value`.
    pub(crate) fn new<R: Rng + ?Sized>(program: &IrProgram, rng: &mut R) -> Result<Store, EvalError> {
        let mut store = Store {
            // Just use any default value, should always be overwritten by the
            // initialisation below.
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

    /// The `[0, n_variables)` prefix that a simulation checkpoints.
    pub(crate) fn state_prefix(&self, program: &IrProgram) -> &[Value] {
        &self.slots[0..program.n_variables() as usize]
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use test_log::test;

    use crate::IrProgram;
    use crate::UntypedStarkSpecification;
    use crate::eval::store::Store;
    use crate::value::Value;

    fn lower_source(source: &str) -> IrProgram {
        let spec = UntypedStarkSpecification::parse(source).expect("should parse");

        let typed_spec = crate::StarkSpecification::from_untyped(spec).expect("should check");
        IrProgram::from_spec(&typed_spec).expect("should lower")
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
