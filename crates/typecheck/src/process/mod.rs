//! Whole-process-specification type checking: [`ProcessSpecification`] extends
//! [`crate::DataSpecification`] to also check actions, process bodies, and `init`.

mod check;
mod error;
mod process_specification;
mod reparse;

pub use error::ProcessError;
pub use process_specification::ProcessSpecification;
