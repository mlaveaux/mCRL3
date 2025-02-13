//! Cache implementations including fixed-size caches with configurable eviction policies.

mod fixed_size_cache;
mod policy;

pub use fixed_size_cache::FixedSizeCache;
pub use policy::*;
