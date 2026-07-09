use std::collections::HashMap;
use std::hash::Hash;

use crate::CachePolicy;

/// A cache that keeps track of key-value pairs with a maximum size and
/// configurable eviction policy. A maximum size of zero means the cache is
/// unbounded.
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
        let maximum_size = if max_size == 0 {
            usize::MAX
        } else {
            max_size.next_power_of_two()
        };

        Self {
            // Only preallocate for bounded caches; an unbounded cache grows on demand.
            map: HashMap::with_capacity(if max_size == 0 { 0 } else { maximum_size }),
            policy,
            maximum_size,
        }
    }

    /// Returns the number of entries in the cache.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the cache contains no entries.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
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

    /// Finds a value in the cache by its key, marking the entry as accessed for
    /// the eviction policy.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let value = self.map.get(key);
        if value.is_some() {
            self.policy.touch(key);
        }
        value
    }

    /// Inserts a key-value pair into the cache, evicting an entry chosen by the
    /// policy when the cache is full. Returns the previous value if the key
    /// existed.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if !self.map.contains_key(&key) && self.map.len() + 1 > self.maximum_size {
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

#[cfg(test)]
mod tests {
    use crate::FifoPolicy;

    use super::FixedSizeCache;

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
    fn test_fixed_size_cache_unbounded() {
        let mut cache = FixedSizeCache::new(0, FifoPolicy::new());

        for i in 0..100 {
            assert!(cache.insert(i, i * i).is_none());
        }

        assert_eq!(cache.len(), 100);
        assert_eq!(cache.get(&0), Some(&0));
    }

    #[test]
    fn test_fixed_size_cache_replace_when_full() {
        let mut cache = FixedSizeCache::new(2, FifoPolicy::new());

        assert!(cache.insert(1, "one").is_none());
        assert!(cache.insert(2, "two").is_none());

        // Replacing a resident key must not evict another entry.
        assert_eq!(cache.insert(2, "TWO"), Some("two"));
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"TWO"));
    }
}
