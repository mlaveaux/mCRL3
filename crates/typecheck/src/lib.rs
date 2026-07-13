mod alias;
mod context;
mod data_specification;
mod desugar;
mod inference;
mod is_finite;
mod is_well_typed;
mod lower;
mod lowering;
mod name_resolution;
mod non_empty;
mod normalize;
mod resolved_sort;
mod signature;
mod sort_resolution;
mod standard_sorts;
mod system_check;
mod system_defined;
mod system_resolution;
mod unification;

// The internal passes are flattened to the crate root for convenience; their
// exact module is not part of the interface. Only the items below marked `pub`
// are exposed outside the crate.
pub(crate) use alias::*;
pub(crate) use context::*;
pub(crate) use data_specification::*;
pub(crate) use desugar::*;
pub(crate) use inference::*;
#[allow(unused_imports)]
pub(crate) use is_finite::*;
pub(crate) use is_well_typed::*;
pub(crate) use lower::*;
#[allow(unused_imports)]
pub(crate) use lowering::*;
pub(crate) use name_resolution::*;
pub(crate) use non_empty::*;
pub(crate) use normalize::*;
pub(crate) use resolved_sort::*;
pub(crate) use signature::*;
pub(crate) use sort_resolution::*;
pub(crate) use standard_sorts::*;
pub(crate) use system_check::*;
pub(crate) use system_defined::*;
pub(crate) use system_resolution::*;
pub(crate) use unification::*;

pub use data_specification::DataSpecification;
pub use inference::InferenceError;
pub use is_well_typed::WellTypedError;
