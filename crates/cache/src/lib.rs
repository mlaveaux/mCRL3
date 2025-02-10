//! Cache implementations including fixed-size caches with configurable eviction policies.

mod policy;
mod fixed_size_cache;

pub use policy::*;
pub use fixed_size_cache::FixedSizeCache;