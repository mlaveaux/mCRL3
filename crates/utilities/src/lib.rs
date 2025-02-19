//! Utility types and functions for the mCRL3 toolset.

// Forbid unsafe code in this crate.
#![forbid(unsafe_code)]

mod bitstream;
mod execution_timer;
mod helper;
mod indexed_set;
mod number_postfix;
mod parse_numbers;
mod progress_meter;
mod protection_set;
mod text_utility;
mod timer;
mod timing;
mod fixed_cache_policy;
mod fixed_size_cache;
mod line_iterator;
mod progress;

pub use bitstream::*;
pub use execution_timer::ExecutionTimer;
pub use helper::*;
pub use indexed_set::*;
pub use number_postfix::NumberPostfixGenerator;
pub use parse_numbers::*;
pub use progress_meter::*;
pub use protection_set::*;
pub use text_utility::*;
pub use timer::*;
pub use timing::*;
pub use fixed_cache_policy::*;
pub use fixed_size_cache::*;
pub use line_iterator::*;
pub use progress::*;

pub fn test_logger() -> Result<(), log::SetLoggerError> {
    env_logger::builder().is_test(true).try_init()
}
