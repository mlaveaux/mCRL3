//! One sampled system state: the store plus every component's controller
//! cursor, minus the controller/environment indirection that lowering
//! already collapsed into the [IrProgram] itself.
//!
//! This exists as its own type — rather than living inline in [super::sim] —
//! because robustness analysis needs *many* of them at once: a sample set is
//! a whole distribution of independently-sampled states at one time step (see
//! [super::sequence]), and a perturbed evolution sequence is built by cloning
//! one and applying a perturbation to the copy. A single simulation is then
//! just the degenerate one-sample case.

use rand::Rng;

use crate::ir::IrProgram;
use crate::value::EvalError;
use crate::value::Value;

use super::step::Cursor;
use super::step::macro_step;
use super::store::Store;

/// A complete sampled state of the system: everything a macro-step reads and
/// writes.
#[derive(Clone, Debug)]
pub(crate) struct SystemState {
    pub(crate) store: Store,
    pub(crate) cursors: Vec<Cursor>,
}

impl SystemState {
    /// Samples an initial state: runs startup initialisation (see
    /// [Store::new]) and puts every component in its `init` state.
    ///
    /// Sampling matters here, not just at each step: an initial value may
    /// itself be random (`x = R[0,10]`), so calling this `n` times with one
    /// RNG yields `n` *different* initial states drawn from the same initial
    /// distribution — which is exactly how a sample set is generated.
    pub(crate) fn new<R: Rng + ?Sized>(program: &IrProgram, rng: &mut R) -> Result<SystemState, EvalError> {
        let store = Store::new(program, rng)?;
        // Every component's `init` is a parallel composition of controller
        // states (`ComponentIr::initial`); flattening every component's
        // initial states into one `Vec<Cursor>` is exactly that composition:
        // nothing cares which "side" a cursor came from, only that every
        // cursor advances against the same pre-step state each tick (see
        // `eval::step`).
        let cursors = program
            .components()
            .iter()
            .flat_map(|component| component.initial.iter())
            .map(|&state| Cursor::Run(state))
            .collect();
        Ok(SystemState { store, cursors })
    }

    /// Advances this state by one macro-step, in place.
    ///
    /// On an [EvalError] the state is left as it was (the failed phase's
    /// buffered updates are dropped rather than half-applied), so a caller
    /// can report the state that triggered the failure.
    pub(crate) fn sample_next<R: Rng + ?Sized>(&mut self, program: &IrProgram, rng: &mut R) -> Result<(), EvalError> {
        macro_step(program, &mut self.store, rng, &mut self.cursors)
    }

    /// The `[0, n_variables)` state prefix — what a trajectory records and
    /// what a perturbation writes to.
    pub(crate) fn variables(&self, program: &IrProgram) -> &[Value] {
        self.store.state_prefix(program)
    }
}
