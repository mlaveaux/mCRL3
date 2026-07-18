mod data_specification;
mod inference;
mod ir;
mod number_encoding;
mod resolution;
mod signature;

// The internal passes are flattened to the crate root for convenience; their
// exact module is not part of the interface. Only the items below marked `pub`
// are exposed outside the crate.
pub(crate) use data_specification::*;
pub(crate) use inference::*;
#[allow(unused_imports)]
pub(crate) use ir::*;
pub(crate) use resolution::*;
#[allow(unused_imports)]
pub(crate) use signature::*;

pub use data_specification::DataSpecification;
pub use inference::InferenceError;
pub use number_encoding::NumberEncoding;
pub use signature::WellTypedError;
