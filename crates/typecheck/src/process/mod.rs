//! Whole-process-specification type checking: [`ProcessSpecification`] extends
//! [`crate::DataSpecification`] to also check actions, process bodies, and `init`.

mod check;
mod disambiguation;
mod error;
mod process_specification;

pub use disambiguation::disambiguate_process_specification;
pub use error::ProcessError;
pub use process_specification::ProcessSpecification;
