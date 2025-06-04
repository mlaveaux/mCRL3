//! Utility types and functions for the mCRL3 toolset.
//!
//! Forbid unsafe code in this crate. If unsafe code is needed it should be in the `mcrl3_unsafety` crate.
#![forbid(unsafe_code)]

#[macro_use]
mod cast_macro;

mod bitstream;
mod debug_trace;
mod display_pair;
mod error;
mod fixed_cache_policy;
mod fixed_size_cache;
mod generational_index;
mod global_allocator;
mod helper;
mod indexed_set;
mod line_iterator;
mod no_hasher;
mod number_postfix;
mod parse_numbers;
mod progress;
mod protection_set;
mod random_test;
mod simple_timer;
mod test_logger;
mod text_utility;
mod timing;

pub use bitstream::*;
pub use display_pair::*;
pub use error::*;
pub use fixed_cache_policy::*;
pub use fixed_size_cache::*;
pub use generational_index::*;
pub use global_allocator::*;
pub use helper::*;
pub use indexed_set::*;
pub use line_iterator::*;
pub use no_hasher::*;
pub use number_postfix::*;
pub use parse_numbers::*;
pub use progress::*;
pub use protection_set::*;
pub use random_test::*;
pub use simple_timer::*;
pub use test_logger::*;
pub use text_utility::*;
pub use timing::*;
