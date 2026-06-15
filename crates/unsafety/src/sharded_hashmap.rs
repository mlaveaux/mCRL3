//! A sharded concurrent hash table exposing a raw, hash-and-equality-closure
//! API.
//!
//! Unlike a `HashMap<K, V>`, this table does not own or hash a key type: every
//! operation is parameterised by a precomputed hash and an equality closure, so
//! the stored element can be a compact handle (for example an index) whose hash
//! and identity are derived from data living elsewhere. This makes it suitable
//! as an interning index that does not duplicate the interned payload.
//!
//! Concurrency comes from striping the elements across a fixed, power-of-two
//! number of shards, each a [`parking_lot::RwLock`] around a
//! [`hashbrown::HashTable`]. The shard is selected from the hash bits just below
//! the top seven that `hashbrown` reserves for its control byte, so shard
//! selection does not degrade in-shard probing.
//!
//! Two API layers are provided:
//!
//! * the raw [`ShardedHashMap::find`], [`ShardedHashMap::find_or_insert_with`]
//!   and [`ShardedHashMap::remove`], where the caller supplies the hash and the
//!   equality (and, for insertion, the rehashing) closures; and
//! * `DashSet`-shaped convenience methods ([`ShardedHashMap::insert`],
//!   [`ShardedHashMap::get`], [`ShardedHashMap::contains`],
//!   [`ShardedHashMap::remove_equiv`]) that hash through the table's own
//!   [`BuildHasher`] for callers that store self-describing elements.

use std::hash::BuildHasher;
use std::hash::Hash;
use std::thread::available_parallelism;

use crossbeam_utils::CachePadded;
use equivalent::Equivalent;
use hashbrown::HashTable;
use hashbrown::hash_table::Entry;
use parking_lot::RwLock;
use rustc_hash::FxBuildHasher;

/// Caps the default shard count so machines with many cores do not allocate an
/// unreasonable number of locks.
const MAX_DEFAULT_SHARDS: usize = 256;

/// Number of high hash bits `hashbrown` reserves for its control byte; the shard
/// is selected from the bits immediately below them.
const CONTROL_BITS: u32 = 7;

/// A concurrent hash table that stores bare elements of type `T`, looked up by a
/// caller-supplied hash and equality closure. See the module documentation.
pub struct ShardedHashMap<T, S = FxBuildHasher> {
    /// Element shards, selected by a slice of the hash. Always a power of two in
    /// length so the shard index is a cheap bit extraction.
    shards: Box<[CachePadded<RwLock<HashTable<T>>>]>,
    /// Builds the hashes the convenience methods use; exposed via
    /// [`ShardedHashMap::hash`] so raw callers hash consistently with the table.
    hasher: S,
    /// `64 - log2(shards.len())`; shifts the hash so the shard index ends up in
    /// the low bits. Unused when there is a single shard.
    shard_shift: u32,
}

/// Returns a sensible default shard count for the running machine.
fn default_shard_count() -> usize {
    let parallelism = available_parallelism().map(|n| n.get()).unwrap_or(1);
    (parallelism * 4).clamp(1, MAX_DEFAULT_SHARDS).next_power_of_two()
}

impl<T> ShardedHashMap<T, FxBuildHasher> {
    /// Creates a table with a machine-dependent number of shards.
    pub fn new() -> ShardedHashMap<T, FxBuildHasher> {
        ShardedHashMap::with_hasher(FxBuildHasher)
    }

    /// Creates a table with at least `shards` shards (rounded up to a power of
    /// two). Fewer shards reduce memory; more reduce write contention.
    pub fn with_shards(shards: usize) -> ShardedHashMap<T, FxBuildHasher> {
        ShardedHashMap::with_shards_and_hasher(shards, FxBuildHasher)
    }
}

impl<T, S: Default> Default for ShardedHashMap<T, S> {
    fn default() -> ShardedHashMap<T, S> {
        ShardedHashMap::with_hasher(S::default())
    }
}

impl<T, S> ShardedHashMap<T, S> {
    /// Creates a table with a machine-dependent number of shards and the given
    /// hash builder.
    pub fn with_hasher(hasher: S) -> ShardedHashMap<T, S> {
        ShardedHashMap::build(default_shard_count(), 0, hasher)
    }

    /// Creates a table with at least `shards` shards (rounded up to a power of
    /// two) and the given hash builder.
    pub fn with_shards_and_hasher(shards: usize, hasher: S) -> ShardedHashMap<T, S> {
        ShardedHashMap::build(shards, 0, hasher)
    }

    /// Creates a table sized to hold at least `capacity` elements before
    /// reallocating, with the given hash builder.
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> ShardedHashMap<T, S> {
        let shards = default_shard_count();
        ShardedHashMap::build(shards, capacity.div_ceil(shards), hasher)
    }

    /// Builds a table with `shards` (rounded up to a power of two) shards, each
    /// pre-sized for `per_shard_capacity` elements.
    fn build(shards: usize, per_shard_capacity: usize, hasher: S) -> ShardedHashMap<T, S> {
        let shards = shards.max(1).next_power_of_two();
        let shard_shift = u64::BITS - shards.trailing_zeros();
        let shards = (0..shards)
            .map(|_| CachePadded::new(RwLock::new(HashTable::with_capacity(per_shard_capacity))))
            .collect();
        ShardedHashMap {
            shards,
            hasher,
            shard_shift,
        }
    }

    /// Returns the shard a hash maps to.
    fn shard(&self, hash: u64) -> &RwLock<HashTable<T>> {
        // With a single shard the shift would be 64 (undefined for `>>`), so
        // short-circuit. Otherwise drop the top `CONTROL_BITS` (used by
        // `hashbrown`'s control byte) and take the next `log2(shards)` bits.
        let index = if self.shards.len() == 1 {
            0
        } else {
            ((hash << CONTROL_BITS) >> self.shard_shift) as usize
        };
        &self.shards[index]
    }

    /// Returns the number of elements across all shards.
    pub fn len(&self) -> usize {
        self.shards.iter().map(|shard| shard.read().len()).sum()
    }

    /// Returns true if every shard is empty.
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|shard| shard.read().is_empty())
    }

    /// Returns the total capacity across all shards.
    pub fn capacity(&self) -> usize {
        self.shards.iter().map(|shard| shard.read().capacity()).sum()
    }

    /// Removes all elements, keeping the allocated shard capacity.
    pub fn clear(&self) {
        for shard in &self.shards {
            shard.write().clear();
        }
    }

    /// Retains only the elements for which `keep` returns true.
    pub fn retain(&self, mut keep: impl FnMut(&T) -> bool) {
        for shard in &self.shards {
            shard.write().retain(|element| keep(element));
        }
    }

    /// Calls `f` on a reference to every element. The shard holding each element
    /// is read-locked while `f` runs against it, so `f` must not call back into
    /// the same table.
    pub fn for_each(&self, mut f: impl FnMut(&T)) {
        for shard in &self.shards {
            let table = shard.read();
            for element in table.iter() {
                f(element);
            }
        }
    }

    /// Returns a clone of the element matching `hash`/`eq`, if present.
    pub fn find(&self, hash: u64, mut eq: impl FnMut(&T) -> bool) -> Option<T>
    where
        T: Clone,
    {
        self.shard(hash).read().find(hash, |element| eq(element)).cloned()
    }

    /// Returns a clone of the element matching `hash`/`eq`, inserting the value
    /// produced by `make` if none is present. `make` runs only on insertion and
    /// while the shard is write-locked; `hash_of` recomputes an element's hash
    /// when the shard grows. Returns the resident element and whether it was
    /// freshly inserted.
    pub fn find_or_insert_with(
        &self,
        hash: u64,
        mut eq: impl FnMut(&T) -> bool,
        hash_of: impl Fn(&T) -> u64,
        make: impl FnOnce() -> T,
    ) -> (T, bool)
    where
        T: Clone,
    {
        let mut table = self.shard(hash).write();
        match table.entry(hash, |element| eq(element), &hash_of) {
            Entry::Occupied(entry) => (entry.get().clone(), false),
            Entry::Vacant(entry) => (entry.insert(make()).get().clone(), true),
        }
    }

    /// Removes and returns the element matching `hash`/`eq`, if present.
    pub fn remove(&self, hash: u64, mut eq: impl FnMut(&T) -> bool) -> Option<T> {
        match self.shard(hash).write().find_entry(hash, |element| eq(element)) {
            Ok(occupied) => Some(occupied.remove().0),
            Err(_) => None,
        }
    }
}

impl<T, S> ShardedHashMap<T, S>
where
    S: BuildHasher,
{
    /// Hashes `value` with the table's hash builder. Raw callers use this so the
    /// hashes they pass to [`ShardedHashMap::find`] and friends are consistent
    /// with the table's convenience methods.
    pub fn hash<Q>(&self, value: &Q) -> u64
    where
        Q: ?Sized + Hash,
    {
        self.hasher.hash_one(value)
    }

    /// Inserts `value` unless an equal element is already present. Returns the
    /// resident element and whether it was freshly inserted.
    pub fn insert(&self, value: T) -> (T, bool)
    where
        T: Hash + Eq + Clone,
    {
        let hash = self.hash(&value);
        let mut table = self.shard(hash).write();
        match table.entry(
            hash,
            |element| element == &value,
            |element| self.hasher.hash_one(element),
        ) {
            Entry::Occupied(entry) => (entry.get().clone(), false),
            Entry::Vacant(entry) => (entry.insert(value).get().clone(), true),
        }
    }

    /// Returns a clone of the element equivalent to `value`, if present.
    pub fn get<Q>(&self, value: &Q) -> Option<T>
    where
        T: Clone,
        Q: ?Sized + Hash + Equivalent<T>,
    {
        let hash = self.hash(value);
        self.shard(hash)
            .read()
            .find(hash, |element| value.equivalent(element))
            .cloned()
    }

    /// Returns true if an element equivalent to `value` is present.
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        Q: ?Sized + Hash + Equivalent<T>,
    {
        let hash = self.hash(value);
        self.shard(hash)
            .read()
            .find(hash, |element| value.equivalent(element))
            .is_some()
    }

    /// Removes and returns the element equivalent to `value`, if present.
    pub fn remove_equiv<Q>(&self, value: &Q) -> Option<T>
    where
        Q: ?Sized + Hash + Equivalent<T>,
    {
        let hash = self.hash(value);
        match self
            .shard(hash)
            .write()
            .find_entry(hash, |element| value.equivalent(element))
        {
            Ok(occupied) => Some(occupied.remove().0),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::thread;

    use super::ShardedHashMap;

    #[test]
    fn insert_dedup_get_remove() {
        let map: ShardedHashMap<u64> = ShardedHashMap::with_shards(4);

        assert_eq!(map.insert(7), (7, true));
        assert_eq!(map.insert(7), (7, false), "duplicate is not reinserted");
        assert_eq!(map.len(), 1);
        assert!(map.contains(&7));
        assert_eq!(map.get(&7), Some(7));
        assert_eq!(map.get(&8), None);

        assert_eq!(map.remove_equiv(&7), Some(7));
        assert!(map.is_empty());
        assert_eq!(map.remove_equiv(&7), None);
    }

    /// Exercises the raw API the way the forest does: the table stores indices
    /// into a side vector and the equality/hash closures dereference it.
    #[test]
    fn raw_interning_dedup() {
        let map: ShardedHashMap<usize> = ShardedHashMap::with_shards(4);
        let store: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

        let intern = |value: &'static str| -> usize {
            let hash = map.hash(value);
            let eq = |&index: &usize| store.lock().unwrap()[index] == value;
            if let Some(index) = map.find(hash, eq) {
                return index;
            }
            map.find_or_insert_with(
                hash,
                eq,
                |&index: &usize| map.hash(store.lock().unwrap()[index]),
                || {
                    let mut store = store.lock().unwrap();
                    store.push(value);
                    store.len() - 1
                },
            )
            .0
        };

        let a = intern("alpha");
        let b = intern("beta");
        assert_eq!(intern("alpha"), a, "equal payloads share an index");
        assert_ne!(a, b);
        assert_eq!(map.len(), 2);
        assert_eq!(store.lock().unwrap().len(), 2, "no payload pushed for a duplicate");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn concurrent_insert_counts_each_once() {
        let map: Arc<ShardedHashMap<usize>> = Arc::new(ShardedHashMap::with_shards(8));
        let inserted = Arc::new(AtomicUsize::new(0));
        let threads = 8;
        let values = 1000usize;

        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let map = Arc::clone(&map);
                let inserted = Arc::clone(&inserted);
                thread::spawn(move || {
                    for value in 0..values {
                        if map.insert(value).1 {
                            inserted.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(inserted.load(Ordering::Relaxed), values, "each value inserted once");
        assert_eq!(map.len(), values);
    }
}
