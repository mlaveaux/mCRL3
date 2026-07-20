//! The evaluator: executes a checked, lowered [crate::ir::IrProgram] —
//! expression evaluation, function calls, sampling, and simulation stepping.
//! See `EVALUATOR_PLAN.md` for the full design.
//!
//! ```text
//! parse -> resolve -> typecheck -> lower -> IrProgram -> [ evaluate ]
//! ```
//!
//! The store (`Store`, one flat `Vec<Value>` indexed by [crate::ir::SlotId]),
//! the per-component controller cursor (`Cursor`), the sampled `SystemState`
//! and the `EvolutionSequence` of sample sets are internal implementation
//! details, not part of this module's public surface.
//!
//! There are two entry points, one per thing you can ask of a specification:
//!
//! - [Simulation] — *run* it. One trajectory, stepped on demand, states
//!   pushed to an [Observer]. This is Milestone B of `EVALUATOR_PLAN.md`.
//! - [Analysis] — *verify* it. Checks the specification's `formula` and
//!   `distance` declarations by comparing an ensemble of trajectories against
//!   a perturbed copy of itself, yielding a [TruthValue] (or a raw distance).
//!   This is Milestone C.
//!
//! Every entry point is fallible: evaluation returns `Result<_, EvalError>`
//! rather than propagating an absorbing error *value* the way the Java
//! reference's `StarkValue.ERROR_VALUE` does — see `value.rs`'s "Errors are a
//! `Result`, not a value" for why.

mod distance;
mod expr;
mod formula;
mod perturbation;
mod robust;
mod sequence;
mod sim;
mod step;
mod store;
mod system;

pub use crate::value::EvalError;
pub use distance::Ci;
pub use formula::TruthValue;
pub use robust::Analysis;
pub use robust::AnalysisOptions;
pub use sequence::EvolutionSequence;
pub use sim::Observer;
pub use sim::RecordingObserver;
pub use sim::Simulation;
