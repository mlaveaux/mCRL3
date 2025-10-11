//! Utility types and functions for the mCRL3 toolset.
//!
//! Forbid unsafe code in this crate. If unsafe code is needed it should be in the `mcrl3_unsafety` crate.
#![forbid(unsafe_code)]

#[macro_use]
mod cast_macro;

mod arbitrary_utility;
mod compressed_vec;
mod debug_trace;
mod error;
mod fixed_cache_policy;
mod fixed_size_cache;
mod format;
mod generational_index;
mod helper;
mod indexed_set;
mod macros;
mod no_hasher;
mod number_postfix;
mod pest_display_pair;
mod protection_set;
mod random_test;
mod simple_timer;
mod tagged_index;
mod test_logger;
mod timing;

pub use arbitrary_utility::*;
pub use compressed_vec::*;
pub use error::*;
pub use fixed_cache_policy::*;
pub use fixed_size_cache::*;
pub use format::*;
pub use generational_index::*;
pub use helper::*;
pub use indexed_set::*;
pub use no_hasher::*;
pub use number_postfix::*;
pub use pest_display_pair::*;
pub use protection_set::*;
pub use random_test::*;
pub use simple_timer::*;
pub use tagged_index::*;
pub use test_logger::*;
pub use timing::*;