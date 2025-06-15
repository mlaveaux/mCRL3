//! Utility types and functions for the mCRL3 toolset.
//!
//! Forbid unsafe code in this crate. If unsafe code is needed it should be in the `mcrl3_unsafety` crate.
#![forbid(unsafe_code)]

#[macro_use]
mod cast_macro;

mod arbitrary_utility;
mod debug_trace;
mod display_pair;
mod error;
mod fixed_cache_policy;
mod fixed_size_cache;
mod generational_index;
mod global_allocator;
mod helper;
mod indexed_set;
mod no_hasher;
mod number_postfix;
mod protection_set;
mod random_test;
mod safe_index;
mod simple_timer;
mod test_logger;
mod timing;

pub use arbitrary_utility::*;
pub use display_pair::*;
pub use error::*;
pub use fixed_cache_policy::*;
pub use fixed_size_cache::*;
pub use generational_index::*;
pub use global_allocator::*;
pub use helper::*;
pub use indexed_set::*;
pub use no_hasher::*;
pub use number_postfix::*;
pub use protection_set::*;
pub use random_test::*;
pub use safe_index::*;
pub use simple_timer::*;
pub use test_logger::*;
pub use timing::*;
