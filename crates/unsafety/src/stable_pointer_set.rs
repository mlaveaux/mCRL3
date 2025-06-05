use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ptr::NonNull;

use allocator_api2::alloc::Allocator;
use allocator_api2::alloc::Global;
use allocator_api2::alloc::Layout;
use hashbrown::Equivalent;
use hashbrown::HashSet;

#[cfg(debug_assertions)]
use crate::AtomicRefCounter;

/// A safe wrapper around a raw pointer that allows immutable dereferencing. This remains valid as long as the `StablePointerSet` remains
/// valid, which is not managed by the borrow checker.
///
/// Comparisons are based on the pointer's address, not the value it points to.
#[repr(C)]
#[derive(Clone)]
pub struct StablePointer<T> {
    /// The raw pointer to the element.
    /// This is a NonNull pointer, which means it is guaranteed to be non-null.
    ptr: NonNull<T>,

    /// Keep track of reference counts in debug mode.
    #[cfg(debug_assertions)]
    reference_counter: AtomicRefCounter<()>,
}

/// Check that the Option<StablePointer> is the same size as a usize for release builds.
#[cfg(not(debug_assertions))]
const _: () = assert!(std::mem::size_of::<Option<StablePointer<usize>>>() == std::mem::size_of::<usize>());

impl<T> StablePointer<T> {
    /// Returns true if this is the last reference to the pointer.
    fn is_last_reference(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            // There is a reference in the table, and the one of `self.ptr`.
            self.reference_counter.strong_count() == 2
        }
        #[cfg(not(debug_assertions))]
        {
            true
        }
    }
}

impl<T> PartialEq for StablePointer<T> {
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: This is safe because we are comparing pointers, which is a valid operation.
        self.ptr == other.ptr
    }
}

impl<T> Eq for StablePointer<T> {}

impl<T> Ord for StablePointer<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // SAFETY: This is safe because we are comparing pointers, which is a valid operation.
        self.ptr.cmp(&(other.ptr))
    }
}

impl<T> PartialOrd for StablePointer<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // SAFETY: This is safe because we are comparing pointers, which is a valid operation.
        Some(self.ptr.cmp(&(other.ptr)))
    }
}

impl<T> Hash for StablePointer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // SAFETY: This is safe because we are hashing pointers, which is a valid operation.
        self.ptr.hash(state);
    }
}

unsafe impl<T: Send> Send for StablePointer<T> {}
unsafe impl<T: Sync> Sync for StablePointer<T> {}

impl<T> StablePointer<T> {
    /// Returns a copy of the StablePointer.
    ///
    /// # Safety
    /// The caller must ensure the pointer points to a valid T that outlives the returned StablePointer.
    pub fn copy(&self) -> Self {
        Self {
            ptr: self.ptr,
            #[cfg(debug_assertions)]
            reference_counter: self.reference_counter.clone(),
        }
    }

    /// Creates a new StablePointer from a boxed element.
    fn from_entry(entry: &Entry<T>) -> Self {
        Self {
            ptr: entry.ptr,
            #[cfg(debug_assertions)]
            reference_counter: entry.reference_counter.clone(),
        }
    }

    /// This returns a unique usize of the pointer. This CANNOT be used as a valid pointer.
    pub fn address(&self) -> usize {
        self.ptr.as_ptr() as usize
    }
}

impl<T> Deref for StablePointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // The caller must ensure the pointer points to a valid T that outlives this StablePointer.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: fmt::Debug> fmt::Debug for StablePointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StablePointer").field(&**self).finish()
    }
}

/// A set that provides stable pointers to its elements.
///
/// Similar to `IndexedSet` but uses pointers instead of indices for direct access to elements.
/// Elements are stored in stable memory locations using a custom allocator, with the hash set maintaining references.
///
/// The set can use a custom hasher type for potentially better performance based on workload characteristics.
/// Uses an allocator for memory management, defaulting to the global allocator.
pub struct StablePointerSet<T, S = RandomState, A = Global>
where
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
    index: HashSet<Entry<T>, S>,

    allocator: A,
}

impl<T> Default for StablePointerSet<T, RandomState, Global>
where
    T: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StablePointerSet<T, RandomState, Global>
where
    T: Hash + Eq,
{
    /// Creates an empty StablePointerSet with the default hasher and global allocator.
    pub fn new() -> Self {
        Self {
            index: hashbrown::HashSet::default(),
            allocator: Global,
        }
    }

    /// Creates an empty StablePointerSet with the specified capacity, default hasher, and global allocator.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            index: hashbrown::HashSet::with_capacity_and_hasher(capacity, RandomState::new()),
            allocator: Global,
        }
    }
}

impl<T, S> StablePointerSet<T, S, Global>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    /// Creates an empty StablePointerSet with the specified hasher and global allocator.
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            index: HashSet::with_hasher(hasher),
            allocator: Global,
        }
    }

    /// Creates an empty StablePointerSet with the specified capacity, hasher, and global allocator.
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            index: HashSet::with_capacity_and_hasher(capacity, hasher),
            allocator: Global,
        }
    }
}

impl<T, S, A> StablePointerSet<T, S, A>
where
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
    /// Creates an empty StablePointerSet with the specified allocator and default hasher.
    pub fn new_in(allocator: A) -> Self
    where
        S: Default,
    {
        Self {
            index: HashSet::with_hasher(S::default()),
            allocator,
        }
    }

    /// Creates an empty StablePointerSet with the specified capacity, allocator, and default hasher.
    pub fn with_capacity_in(capacity: usize, allocator: A) -> Self
    where
        S: Default,
    {
        Self {
            index: HashSet::with_capacity_and_hasher(capacity, S::default()),
            allocator,
        }
    }

    /// Creates an empty StablePointerSet with the specified hasher and allocator.
    pub fn with_hasher_in(hasher: S, allocator: A) -> Self {
        Self {
            index: HashSet::with_hasher(hasher),
            allocator,
        }
    }

    /// Creates an empty StablePointerSet with the specified capacity, hasher, and allocator.
    pub fn with_capacity_and_hasher_in(capacity: usize, hasher: S, allocator: A) -> Self {
        Self {
            index: HashSet::with_capacity_and_hasher(capacity, hasher),
            allocator,
        }
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity of the set.
    pub fn capacity(&self) -> usize {
        self.index.capacity()
    }

    /// Inserts an element into the set.
    ///
    /// If the set did not have this value present, `true` is returned along
    /// with a stable pointer to the inserted element.
    ///
    /// If the set already had this value present, `false` is returned along
    /// with a stable pointer to the existing element.
    pub fn insert(&mut self, value: T) -> (StablePointer<T>, bool) {
        debug_assert!(std::mem::size_of::<T>() > 0, "Zero-sized types not supported");

        // Check if we already have this value
        let raw_ptr = self.get(&value);

        if let Some(ptr) = raw_ptr {
            // We already have this value, return pointer to existing element
            return (ptr, false);
        }

        // Insert new value using allocator
        let entry = Entry::new(value, &self.allocator);
        let ptr = StablePointer::from_entry(&entry);

        // First add to storage, then to index
        let inserted = self.index.insert(entry);

        debug_assert!(inserted, "Value should not already exist in the index");

        (ptr, true)
    }

    /// Inserts an element into the set using an equivalent value.
    ///
    /// This version takes a reference to an equivalent value and creates the value to insert
    /// only if it doesn't already exist in the set. Returns a stable pointer to the element
    /// and a boolean indicating whether the element was inserted.
    pub fn insert_equiv<'a, Q>(&mut self, value: &'a Q) -> (StablePointer<T>, bool)
    where
        Q: Hash + Equivalent<T>,
        T: From<&'a Q>,
    {
        debug_assert!(std::mem::size_of::<T>() > 0, "Zero-sized types not supported");

        // Check if we already have this value
        let raw_ptr = self.get(value);

        if let Some(ptr) = raw_ptr {
            // We already have this value, return pointer to existing element
            return (ptr, false);
        }

        // Convert the reference to an actual value only if needed
        let value_to_insert = T::from(value);

        // Insert new value using allocator
        let entry = Entry::new(value_to_insert, &self.allocator);
        let ptr = StablePointer::from_entry(&entry);

        // First add to storage, then to index
        let inserted = self.index.insert(entry);

        debug_assert!(inserted, "Value should not already exist in the index");

        (ptr, true)
    }

    /// Returns `true` if the set contains a value.
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Eq + Hash,
        Q: ?Sized + Hash + Equivalent<T>,
    {
        self.get(value).is_some()
    }

    /// Returns a stable pointer to a value in the set, if present.
    ///
    /// Searches for a value equal to the provided reference and returns a pointer to the stored element.
    /// The returned pointer remains valid until the element is removed from the set.
    pub fn get<Q>(&self, value: &Q) -> Option<StablePointer<T>>
    where
        T: Eq + Hash,
        Q: ?Sized + Hash + Equivalent<T>,
    {
        // Find the boxed element that contains an equivalent value
        let boxed = self.index.get(&LookUp(value))?;

        // SAFETY: The pointer is valid as long as the set is valid.
        let ptr = StablePointer::from_entry(boxed);
        Some(ptr)
    }

    /// Returns an iterator over the elements of the set.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.index.iter().map(|boxed| unsafe { boxed.ptr.as_ref() })
    }

    /// Removes an element from the set using its stable pointer. This is very inefficient and requires O(n) time.
    ///
    /// Returns true if the element was found and removed.
    pub fn remove(&mut self, pointer: StablePointer<T>) -> bool {
        debug_assert!(
            pointer.is_last_reference(),
            "Pointer must be the last reference to the element"
        );
        // SAFETY: This is the last reference to the element, so it is safe to remove it.
        let t = pointer.deref();
        self.index.remove(&LookUp(t))
    }

    /// Retains only the elements specified by the predicate, modifying the set in-place.
    ///
    /// The predicate closure is called with a mutable reference to each element and must
    /// return true if the element should remain in the set.
    ///
    /// # Safety
    ///
    /// It invalidates any StablePointers to removed elements
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&StablePointer<T>) -> bool,
    {
        // First pass: determine what to keep/remove without modifying the collection
        self.index.retain(|element| {
            let ptr = StablePointer::from_entry(element);

            if !predicate(&ptr) {
                // One reference in the table, and one that is constructed above as `ptr`.
                // #[cfg(debug_assertions)]
                // debug_assert_eq!(
                //     Arc::strong_count(&element.reference_counter),
                //     2,
                //     "No other references to should exist when removed"
                // );
                return false;
            }

            true
        });
    }

    /// Clears the set, removing all values and invalidating all pointers.
    ///
    /// # Safety
    /// This is unsafe because it invalidates all pointers to the elements in the set.
    pub fn clear(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(
            self.index.iter().all(|x| x.reference_counter.strong_count() == 1),
            "All pointers must be the last reference to the element"
        );

        // Manually deallocate all entries before clearing
        for entry in self.index.drain() {
            entry.deallocate(&self.allocator);
        }

        debug_assert!(self.index.is_empty(), "Index should be empty after draining");
    }
}

impl<T, S, A> Drop for StablePointerSet<T, S, A>
where
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(
            self.index.iter().all(|x| x.reference_counter.strong_count() == 1),
            "All pointers must be the last reference to the element"
        );

        // Manually deallocate all entries
        for entry in self.index.drain() {
            entry.deallocate(&self.allocator);
        }
    }
}

/// A helper struct to store the allocated element in the set.
///
/// Uses manual allocation instead of Box for custom allocator support.
/// Optionally stores a reference counter for debugging purposes in debug builds.
struct Entry<T> {
    /// Pointer to the allocated value
    ptr: NonNull<T>,

    #[cfg(debug_assertions)]
    reference_counter: AtomicRefCounter<()>,
}

unsafe impl<T: Send> Send for Entry<T> {}
unsafe impl<T: Sync> Sync for Entry<T> {}

impl<T> Entry<T> {
    /// Creates a new entry by allocating memory for the value using the provided allocator.
    fn new(value: T, allocator: &impl Allocator) -> Self {
        debug_assert!(std::mem::size_of::<T>() > 0, "Zero-sized types not supported");

        // Allocate memory for the value
        let layout = Layout::new::<T>();
        let ptr = allocator.allocate(layout).expect("Allocation failed").cast::<T>();

        // Write the value to the allocated memory
        unsafe {
            ptr.as_ptr().write(value);
        }

        Self {
            ptr,
            #[cfg(debug_assertions)]
            reference_counter: AtomicRefCounter::new(()),
        }
    }

    /// Deallocates the memory used by this entry.
    fn deallocate(self, allocator: &impl Allocator) {
        debug_assert!(std::mem::size_of::<T>() > 0, "Zero-sized types not supported");

        unsafe {
            // Drop the value in place
            std::ptr::drop_in_place(self.ptr.as_ptr());

            // Deallocate the memory
            let layout = Layout::new::<T>();
            allocator.deallocate(self.ptr.cast(), layout);
        }
    }
}

impl<T> Deref for Entry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The pointer is valid as long as the Entry exists
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: PartialEq> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Hash> Hash for Entry<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: Eq> Eq for Entry<T> {}

/// A helper struct to look up elements in the set using a reference.
#[derive(Hash, PartialEq, Eq)]
struct LookUp<'a, T: ?Sized>(&'a T);

impl<T, Q: ?Sized> Equivalent<Entry<T>> for LookUp<'_, Q>
where
    Q: Equivalent<T>,
{
    fn equivalent(&self, other: &Entry<T>) -> bool {
        self.0.equivalent(&**other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use allocator_api2::alloc::System;
    use rustc_hash::FxHasher;
    use std::hash::BuildHasherDefault;

    #[test]
    fn test_insert_and_get() {
        let mut set = StablePointerSet::new();

        // Insert a value and ensure we get it back
        let (ptr1, inserted) = set.insert(42);
        assert!(inserted);
        assert_eq!(*ptr1, 42);

        // Insert the same value and ensure we get the same pointer
        let (ptr2, inserted) = set.insert(42);
        assert!(!inserted);
        assert_eq!(*ptr2, 42);

        // Pointers to the same value should be identical
        assert_eq!(ptr1, ptr2);

        // Verify that we have only one element
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_contains() {
        let mut set = StablePointerSet::new();
        set.insert(42);
        set.insert(100);

        assert!(set.contains(&42));
        assert!(set.contains(&100));
        assert!(!set.contains(&200));
    }

    #[test]
    fn test_get() {
        let mut set = StablePointerSet::new();
        set.insert(42);
        set.insert(100);

        let ptr = set.get(&42).expect("Value should exist");
        assert_eq!(*ptr, 42);

        let ptr = set.get(&100).expect("Value should exist");
        assert_eq!(*ptr, 100);

        assert!(set.get(&200).is_none(), "Value should not exist");
    }

    #[test]
    fn test_iteration() {
        let mut set = StablePointerSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let mut values: Vec<i32> = set.iter().copied().collect();
        values.sort();

        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_insert_equiv_ref() {
        #[derive(PartialEq, Eq, Debug)]
        struct TestValue {
            id: i32,
            name: String,
        }

        impl From<&i32> for TestValue {
            fn from(id: &i32) -> Self {
                TestValue {
                    id: *id,
                    name: format!("Value-{}", id),
                }
            }
        }

        impl Hash for TestValue {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl Equivalent<TestValue> for i32 {
            fn equivalent(&self, key: &TestValue) -> bool {
                *self == key.id
            }
        }

        let mut set: StablePointerSet<TestValue> = StablePointerSet::new();

        // Insert using equivalent reference (i32 -> TestValue)
        let (ptr1, inserted) = set.insert_equiv(&42);
        assert!(inserted, "Value should be inserted");
        assert_eq!(ptr1.id, 42);
        assert_eq!(ptr1.name, "Value-42");

        // Try inserting the same value again via equivalent
        let (ptr2, inserted) = set.insert_equiv(&42);
        assert!(!inserted, "Value should not be inserted again");
        assert_eq!(ptr1, ptr2, "Should return the same pointer");

        // Insert a different value
        let (ptr3, inserted) = set.insert_equiv(&100);
        assert!(inserted, "New value should be inserted");
        assert_eq!(ptr3.id, 100);
        assert_eq!(ptr3.name, "Value-100");

        // Ensure we have exactly two elements
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_stable_pointer_deref() {
        let mut set = StablePointerSet::new();
        let (ptr, _) = set.insert(42);

        // Test dereferencing
        let value: &i32 = &ptr;
        assert_eq!(*value, 42);

        // Test methods on the dereferenced value
        assert_eq!((*ptr).checked_add(10), Some(52));
    }

    #[test]
    fn test_remove() {
        let mut set = StablePointerSet::new();

        // Insert values
        let (ptr1, _) = set.insert(42);
        let (ptr2, _) = set.insert(100);
        assert_eq!(set.len(), 2);

        // Remove one value
        assert!(set.remove(ptr1));
        assert_eq!(set.len(), 1);

        // Remove other value
        assert!(set.remove(ptr2));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_retain() {
        let mut set = StablePointerSet::new();

        // Insert values
        set.insert(1);
        let (ptr2, _) = set.insert(2);
        set.insert(3);
        let (ptr4, _) = set.insert(4);
        assert_eq!(set.len(), 4);

        // Retain only even numbers
        set.retain(|x| **x % 2 == 0);

        // Verify results
        assert_eq!(set.len(), 2);
        assert!(!set.contains(&1));
        assert!(set.contains(&2));
        assert!(!set.contains(&3));
        assert!(set.contains(&4));

        // Verify that removed pointers are invalid and remaining are valid
        assert!(set.remove(ptr2));
        assert!(set.remove(ptr4));
    }

    #[test]
    fn test_custom_allocator() {
        // Test with System allocator
        let mut set: StablePointerSet<i32, RandomState, System> = StablePointerSet::new_in(System);

        // Insert some values
        let (ptr1, inserted) = set.insert(42);
        assert!(inserted);
        let (ptr2, inserted) = set.insert(100);
        assert!(inserted);

        // Check that everything works as expected
        assert_eq!(set.len(), 2);
        assert_eq!(*ptr1, 42);
        assert_eq!(*ptr2, 100);

        // Test contains
        assert!(set.contains(&42));
        assert!(set.contains(&100));
        assert!(!set.contains(&200));
    }

    #[test]
    fn test_custom_hasher_and_allocator() {
        // Use both custom hasher and allocator
        let mut set: StablePointerSet<i32, BuildHasherDefault<FxHasher>, System> =
            StablePointerSet::with_hasher_in(BuildHasherDefault::<FxHasher>::default(), System);

        // Insert some values
        let (ptr1, inserted) = set.insert(42);
        assert!(inserted);
        let (ptr2, inserted) = set.insert(100);
        assert!(inserted);

        // Check that everything works as expected
        assert_eq!(set.len(), 2);
        assert_eq!(*ptr1, 42);
        assert_eq!(*ptr2, 100);

        // Test contains
        assert!(set.contains(&42));
        assert!(set.contains(&100));
        assert!(!set.contains(&200));
    }
}
