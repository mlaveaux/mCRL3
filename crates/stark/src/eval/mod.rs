//! The evaluator: executes a checked, lowered [crate::ir::IrProgram] —
//! expression evaluation, function calls, sampling, and simulation stepping.
//! See `EVALUATOR_PLAN.md` for the full design.
//!
//! ```text
//! parse -> resolve -> typecheck -> lower -> IrProgram -> [ evaluate ]
//! ```
//!
//! The store (`Store`, one flat `Vec<Value>` indexed by [crate::ir::SlotId])
//! and the per-component controller cursor (`Cursor`) are internal
//! implementation details, not part of this module's public surface — the
//! only things a caller needs are [Simulation] and [Observer].

mod expr;
mod sim;
mod step;
mod store;

pub use sim::Observer;
pub use sim::RecordingObserver;
pub use sim::Simulation;
