#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

#[macro_use]
mod cast_macro;

mod debug_trace;
mod error;
mod fixed_cache_policy;
mod fixed_size_cache;
mod generational_index;
mod helper;
mod kani_rng;
mod permutation;
mod pest_display_pair;
mod random_test;
mod sharded_counter;
mod tagged_index;
mod test_logger;
mod timing;
mod traversal;

pub use error::MercError;
pub use generational_index::GenerationCounter;
pub use generational_index::GenerationalIndex;
pub use helper::PhantomUnsend;
pub use helper::PhantomUnsync;
pub use permutation::is_valid_permutation;
pub use pest_display_pair::DisplayPair;
pub use random_test::random_test;
pub use random_test::random_test_threads;
pub use sharded_counter::ShardedCounter;
pub use tagged_index::MercIndex;
pub use tagged_index::TagIndex;
pub use test_logger::test_logger;
pub use test_logger::test_threads;
pub use timing::Timing;
pub use traversal::Step;
pub use traversal::Visit;
pub use fixed_cache_policy::CachePolicy;
pub use fixed_cache_policy::FifoPolicy;
pub use fixed_cache_policy::LruPolicy;
pub use fixed_cache_policy::NoPolicy;
pub use fixed_size_cache::FixedSizeCache;

#[cfg(kani)]
pub use kani_rng::*;
