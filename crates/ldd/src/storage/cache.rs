use ahash::RandomState;
use core::hash::Hash;
use std::hash::BuildHasher;

use crate::Ldd;
use crate::LddRef;
use crate::Storage;

use super::ldd::LddIndex;
use super::ldd::SharedProtectionSet;

/// The operation cache can significantly speed up operations by caching
/// intermediate results. This is necessary since the maximal sharing means that
/// the same inputs can be encountered many times while evaluating the
/// operation.
///
/// For all operations defined in `operations.rs` where caching helps we
/// introduce a cache. The cache that belongs to one operation is identified by
/// the value of [UnaryFunction], [BinaryOperator] or [TernaryOperator].
pub struct OperationCache {
    protection_set: SharedProtectionSet,
    caches1: Vec<Cache<LddIndex, usize>>,
    caches2: Vec<Cache<(LddIndex, LddIndex), LddIndex>>,
    caches3: Vec<Cache<(LddIndex, LddIndex, LddIndex), LddIndex>>,
}

impl OperationCache {
    pub fn new(protection_set: SharedProtectionSet) -> OperationCache {
        OperationCache {
            protection_set,
            caches1: vec![Cache::new()],
            caches2: vec![Cache::new(); 3],
            caches3: vec![Cache::new()],
        }
    }

    /// Clear all existing caches. This must be done during garbage collection
    /// since caches have references to elements in the node table that are not
    /// protected.
    pub fn clear(&mut self) {
        for cache in self.caches1.iter_mut() {
            cache.clear();
        }

        for cache in self.caches2.iter_mut() {
            cache.clear();
        }

        for cache in self.caches3.iter_mut() {
            cache.clear();
        }
    }

    /// Returns the number of elements in the operation cache.
    pub fn len(&self) -> usize {
        let mut result: usize = 0;

        for cache in self.caches1.iter() {
            result += cache.len();
        }

        for cache in self.caches2.iter() {
            result += cache.len();
        }

        for cache in self.caches3.iter() {
            result += cache.len();
        }

        result
    }

    /// Returns true iff the operation cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Puts a limit on the operation cache size. This will ensure that
    /// self.len() <= n if self.limit(n) has been set.
    pub fn limit(&mut self, size: usize) {
        for cache in self.caches1.iter_mut() {
            cache.limit(size / 4);
        }

        for cache in self.caches2.iter_mut() {
            cache.limit(size / 4);
        }

        for cache in self.caches3.iter_mut() {
            cache.limit(size / 4);
        }
    }

    fn get_cache1(&mut self, operator: &UnaryFunction) -> &mut Cache<LddIndex, usize> {
        match operator {
            UnaryFunction::Len => &mut self.caches1[0],
        }
    }

    fn get_cache2(&mut self, operator: &BinaryOperator) -> &mut Cache<(LddIndex, LddIndex), LddIndex> {
        match operator {
            BinaryOperator::Union => &mut self.caches2[0],
            BinaryOperator::Merge => &mut self.caches2[1],
            BinaryOperator::Minus => &mut self.caches2[2],
        }
    }

    fn get_cache3(&mut self, operator: &TernaryOperator) -> &mut Cache<(LddIndex, LddIndex, LddIndex), LddIndex> {
        match operator {
            TernaryOperator::RelationalProduct => &mut self.caches3[0],
        }
    }

    /// Create an Ldd from the given index. Only safe because this is a private function.
    fn create(&mut self, index: LddIndex) -> Ldd {
        Ldd::new(&self.protection_set, index)
    }
}

/// Implements an associative mapping between key value pairs, but has a limit
/// on the maximum amount of elements stored. The cache requires that default
/// values of K are never used in calls to get and insert, because these are
/// used to indicate empty cache entries.
pub struct Cache<K, V, S = RandomState> {
    table: Vec<(K, V)>,
    hash_builder: S,
}

impl<K: Default + Clone, V: Clone + Default> Cache<K, V, RandomState> {
    pub fn new() -> Cache<K, V, RandomState> {
        Cache {
            table: vec![Default::default(); 1024],
            hash_builder: RandomState::default(),
        }
    }
}

impl<K: Default + Clone, V: Clone + Default> Default for Cache<K, V, RandomState> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Default + Clone, V: Clone + Default, S> Cache<K, V, S> {
    /// Removes all elements stored in the cache.
    pub fn clear(&mut self) {
        let capacity = self.table.len();

        self.table.clear();
        self.table.resize(capacity, Default::default());
    }

    /// Puts a limit on the maximum self.len() of this cache.
    pub fn limit(&mut self, size: usize) {
        let power_of_two = size.next_power_of_two();

        self.table.clear();
        self.table.resize(power_of_two, Default::default());
    }

    /// Returns the amount of elements in the cache.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns true iff the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: Default + Eq + Hash, V, S: BuildHasher> Cache<K, V, S> {
    /// Check whether key is in the storage, if so returns Some(value) and None otherwise.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        debug_assert!(*key != K::default(), "The key may never be equal to its default value.");

        // Compute the index in the table.
        let index = self.hash_builder.hash_one(key) % (self.table.len() as u64);

        let entry = &self.table[index as usize];
        if entry.0 == *key { Some(&entry.1) } else { None }
    }

    /// Inserts the given key value pair into the cache. Might evict other pairs in the cache.
    pub fn insert(&mut self, key: K, value: V) {
        debug_assert!(key != K::default(), "The key may never be equal to its default value.");

        // Compute the index in the table.

        let index = self.hash_builder.hash_one(&key) % (self.table.len() as u64);
        self.table[index as usize] = (key, value);
    }
}

impl<K: Clone, V: Clone, S: Clone> Clone for Cache<K, V, S> {
    fn clone(&self) -> Self {
        Cache {
            table: self.table.clone(),
            hash_builder: self.hash_builder.clone(),
        }
    }
}

/// Any function from LDD -> usize.
pub enum UnaryFunction {
    Len,
}

/// Any operator from LDD x LDD -> LDD.
pub enum BinaryOperator {
    Union,
    Merge,
    Minus,
}

/// Any operator from LDD x LDD x LDD -> LDD.
pub enum TernaryOperator {
    RelationalProduct,
}

/// Implements an operation cache for a unary LDD operator.
pub fn cache_unary_function<F>(storage: &mut Storage, operator: UnaryFunction, a: &LddRef, f: F) -> usize
where
    F: Fn(&mut Storage, &LddRef<'_>) -> usize,
{
    let key = a.index();
    if let Some(result) = storage.operation_cache().get_cache1(&operator).get(&key) {
        *result
    } else {
        let result = f(storage, a);
        storage.operation_cache().get_cache1(&operator).insert(key, result);
        result
    }
}

/// Implements an operation cache for a binary LDD operator.
pub fn cache_binary_op<F>(storage: &mut Storage, operator: BinaryOperator, a: &LddRef, b: &LddRef, f: F) -> Ldd
where
    F: Fn(&mut Storage, &LddRef<'_>, &LddRef<'_>) -> Ldd,
{
    let key = (a.index(), b.index());
    if let Some(result) = storage.operation_cache().get_cache2(&operator).get(&key) {
        let result = *result; // Necessary to decouple borrow from storage and the call to create.
        storage.operation_cache().create(result)
    } else {
        let result = f(storage, a, b);
        storage
            .operation_cache()
            .get_cache2(&operator)
            .insert(key, result.index());
        result
    }
}

/// Implements an operation cache for a commutative binary LDD operator, i.e.,
/// an operator f such that f(a,b) = f(b,a) for all LDD a and b.
pub fn cache_comm_binary_op<F>(storage: &mut Storage, operator: BinaryOperator, a: &LddRef, b: &LddRef, f: F) -> Ldd
where
    F: Fn(&mut Storage, &LddRef<'_>, &LddRef<'_>) -> Ldd,
{
    // Reorder the inputs to improve caching behaviour (can potentially half the cache size)
    if a.index() < b.index() {
        cache_binary_op(storage, operator, a, b, f)
    } else {
        cache_binary_op(storage, operator, b, a, f)
    }
}

/// Implements an operation cache for a terniary LDD operator.
pub fn cache_terniary_op<F>(
    storage: &mut Storage,
    operator: TernaryOperator,
    a: &LddRef,
    b: &LddRef,
    c: &LddRef,
    f: F,
) -> Ldd
where
    F: Fn(&mut Storage, &LddRef<'_>, &LddRef<'_>, &LddRef<'_>) -> Ldd,
{
    let key = (a.index(), b.index(), c.index());
    if let Some(result) = storage.operation_cache().get_cache3(&operator).get(&key) {
        let result = *result; // Necessary to decouple borrow from storage and the call to create.
        storage.operation_cache().create(result)
    } else {
        let result = f(storage, a, b, c);
        storage
            .operation_cache()
            .get_cache3(&operator)
            .insert(key, result.index());
        result
    }
}
