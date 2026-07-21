//! The evaluator: executes a checked, lowered [crate::ir::IrProgram] —
//! expression evaluation, function calls, sampling, and simulation stepping.
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
//!   pushed to an [Observer].
//! - [Analysis] — *verify* it. Checks the specification's `formula` and
//!   `distance` declarations by comparing an ensemble of trajectories against
//!   a perturbed copy of itself, yielding a [TruthValue] (or a raw distance).
//!
//! Every entry point is fallible: evaluation returns `Result<_, EvalError>`
//! rather than propagating an absorbing error *value* the way the original
//! does — see `value.rs` for why.

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
pub use crate::value::EvalErrorKind;
pub use distance::Ci;
pub use formula::TruthValue;
pub use robust::Analysis;
pub use robust::AnalysisOptions;
pub use sequence::EvolutionSequence;
pub use sim::Observer;
pub use sim::RecordingObserver;
pub use sim::Simulation;
