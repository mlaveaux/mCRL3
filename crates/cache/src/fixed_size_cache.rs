//! Cache implementations including fixed-size caches with configurable eviction policies.
//!
//! # Example
//!
//! ```
//! use crate::fixed_size_cache::FixedSizeCache;
//! use crate::FifoPolicy;
//!
//! let mut cache = FixedSizeCache::new(2, FifoPolicy::new());
//!
//! cache.insert(1, "one");
//! cache.insert(2, "two");
//!
//! assert_eq!(cache.get(&1), Some(&"one"));
//! assert_eq!(cache.get(&2), Some(&"two"));
//!
//! // Insert a third item, which should evict the first one (FIFO policy)
//! cache.insert(3, "three");
//!
//! assert_eq!(cache.get(&1), None);
//! assert_eq!(cache.get(&2), Some(&"two"));
//! assert_eq!(cache.get(&3), Some(&"three"));
//! ```
//!

use hashbrown::HashMap;
use std::hash::Hash;

use crate::CachePolicy;

/// A cache that keeps track of key-value pairs with a maximum size and configurable
/// eviction policy.
#[derive(Debug)]
pub struct FixedSizeCache<K, V, P: CachePolicy<K>> {
    map: HashMap<K, V>,
    policy: P,
    maximum_size: usize,
}

impl<K, V, P> FixedSizeCache<K, V, P>
where
    K: Eq + Hash + Clone,
    P: CachePolicy<K>,
{
    /// Creates a new cache with the specified maximum size and policy.
    pub fn new(max_size: usize, policy: P) -> Self {
        let capacity = if max_size == 0 {
            usize::MAX
        } else {
            max_size.next_power_of_two()
        };

        Self {
            map: HashMap::with_capacity(capacity),
            policy,
            maximum_size: capacity,
        }
    }

    /// Returns an iterator over the cache entries.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }

    /// Clears all entries from the cache.
    pub fn clear(&mut self) {
        self.map.clear();
        self.policy.clear();
    }

    /// Returns true if the key exists in the cache.
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Finds a value in the cache by its key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    /// Inserts a key-value pair into the cache.
    /// Returns the previous value if the key existed.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.map.len() + 1 > self.maximum_size {
            // Get candidate for removal
            if let Some(candidate) = self.policy.replacement_candidate(&self.map) {
                self.map.remove(&candidate);
            }
        }

        let result = self.map.insert(key.clone(), value);
        self.policy.inserted(key);
        result
    }
}

/// A function cache that memoizes function results.
#[derive(Debug)]
pub struct FunctionCache<K, V, P, F>
where
    P: CachePolicy<K>,
    F: Fn(&K) -> V,
{
    cache: FixedSizeCache<K, V, P>,
    function: F,
}

impl<K, V, P, F> FunctionCache<K, V, P, F>
where
    K: Eq + Hash + Clone,
    P: CachePolicy<K>,
    F: Fn(&K) -> V,
{
    /// Creates a new function cache with the specified function and maximum size.
    pub fn new(function: F, max_size: usize, policy: P) -> Self {
        Self {
            cache: FixedSizeCache::new(max_size, policy),
            function,
        }
    }

    /// Retrieves or computes the value for the given key.
    pub fn get(&mut self, key: &K) -> &V {
        if !self.cache.contains_key(key) {
            let value = (self.function)(key);
            self.cache.insert(key.clone(), value);
        }
        self.cache.get(key).unwrap()
    }

    /// Clears all cached values.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::FifoPolicy;

    use super::*;

    #[test]
    fn test_fixed_size_cache() {
        let mut cache = FixedSizeCache::new(2, FifoPolicy::new());

        assert!(cache.insert(1, "one").is_none());
        assert!(cache.insert(2, "two").is_none());

        // Should evict first entry
        assert!(cache.insert(3, "three").is_none());

        assert!(cache.get(&1).is_none());
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_function_cache() {
        let f = |x: &i32| x * x;
        let mut cache = FunctionCache::new(f, 2, FifoPolicy::new());

        assert_eq!(*cache.get(&2), 4);
        assert_eq!(*cache.get(&3), 9);
        assert_eq!(*cache.get(&2), 4); // Should be cached

        // Should evict first entry (2)
        assert_eq!(*cache.get(&4), 16);

        // Should recompute
        assert_eq!(*cache.get(&2), 4);
    }

    #[test]
    fn test_function_cache_eviction() {
        let f = |x: &i32| x * x;
        let mut cache = FunctionCache::new(f, 2, FifoPolicy::new());

        assert_eq!(*cache.get(&1), 1);
        assert_eq!(*cache.get(&2), 4);

        // Should evict the first entry (1)
        assert_eq!(*cache.get(&3), 9);
        assert!(cache.cache.get(&1).is_none());
        assert_eq!(*cache.get(&2), 4);
        assert_eq!(*cache.get(&3), 9);
    }

    #[test]
    fn test_function_cache_clear() {
        let f = |x: &i32| x * x;
        let mut cache = FunctionCache::new(f, 2, FifoPolicy::new());

        assert_eq!(*cache.get(&1), 1);
        assert_eq!(*cache.get(&2), 4);

        cache.clear();

        assert!(cache.cache.get(&1).is_none());
        assert!(cache.cache.get(&2).is_none());
    }

    #[test]
    fn test_function_cache_contains_key() {
        let f = |x: &i32| x * x;
        let mut cache = FunctionCache::new(f, 2, FifoPolicy::new());

        assert_eq!(*cache.get(&1), 1);
        assert!(cache.cache.contains_key(&1));
        assert!(!cache.cache.contains_key(&2));
    }
}
