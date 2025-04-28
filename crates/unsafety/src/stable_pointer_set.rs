use std::ptr::NonNull;
use std::{hash::Hash, borrow::Borrow, ops::Deref, fmt};
use hashbrown::Equivalent;
use std::hash::BuildHasher;
use std::collections::hash_map::RandomState;

/// A safe wrapper around a raw pointer that allows immutable dereferencing. This remains valid as long as the `StablePointerSet` remains
/// valid, which is not managed by the borrow checker.
///
/// This type is guaranteed to point to a valid, immutable T managed by a StablePointerSet.
/// 
/// Comparisons are based on the pointer's address, not the value it points to.
#[derive(Copy, Clone, Hash)]
pub struct StablePointer<T>(NonNull<T>);

/// Check that the Option<StablePointer> is the same size as a usize.
const _: () = assert!(std::mem::size_of::<Option<StablePointer<usize>>>() == std::mem::size_of::<usize>());

impl<T> PartialEq for StablePointer<T> {
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: This is safe because we are comparing pointers, which is a valid operation.
        self.address() == other.address()
    }
}

impl<T> Eq for StablePointer<T> {}

impl<T> Ord for StablePointer<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // SAFETY: This is safe because we are comparing pointers, which is a valid operation.
        self.address().cmp(&(other.address()))
    }
}

impl<T> PartialOrd for StablePointer<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // SAFETY: This is safe because we are comparing pointers, which is a valid operation.
        self.address().partial_cmp(&(other.address()))
    }
}

unsafe impl<T> Send for StablePointer<T> {}
unsafe impl<T> Sync for StablePointer<T> {}

impl<T> StablePointer<T> {
    /// Should never be dereferenced, as it is a dangling pointer.
    pub fn dangling() -> Self {
        // SAFETY: This is safe because we are creating a dangling pointer, which is valid.
        Self(NonNull::dangling())
    }

    /// Returns a unique index for the StablePointer. Note that this index cannot be converted into a pointer.
    pub fn address(&self) -> usize {
        self.0.addr().into()
    }
    
    /// Returns a copy of the StablePointer.
    /// 
    /// # Safety
    // This is a no-op, as the pointer is already stable and valid.
    // The caller must ensure the pointer is valid and outlives this StablePointer.
    pub unsafe fn copy(&self) -> Self {
        Self::new(self.0)
    }

    /// Creates a new StablePointer from a raw pointer.
    /// 
    /// # Safety
    /// The caller must ensure the pointer points to a valid T that outlives this StablePointer.
    fn new(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }
}

impl<T> Deref for StablePointer<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        // SAFETY: StablePointerSet guarantees this pointer points to a valid T
        // that lives at least as long as the StablePointerSet
        unsafe { self.0.as_ref() }
    }
}

impl<T: fmt::Debug> fmt::Debug for StablePointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StablePointer")
            .field(&**self)
            .finish()
    }
}

/// A set that provides stable pointers to its elements.
///
/// Similar to `IndexedSet` but uses pointers instead of indices for direct access to elements.
/// Elements are stored in stable memory locations, with the hash set maintaining references.
/// This design allows for:
/// 1. Direct element access without re-indexing
/// 2. Element access during collection resize operations
/// 3. Configurable hashing strategy for workload optimization
///
/// The set can use a custom hasher type for potentially better performance based on workload characteristics.
pub struct StablePointerSet<T, S = RandomState> 
where
    T: Hash + Eq,
    S: BuildHasher,
{
    index: hashbrown::HashSet<StablePointer<T>, S>,
    storage: Vec<Box<T>>,
}

impl<T> StablePointerSet<T, RandomState> 
where
    T: Hash + Eq,
{
    /// Creates an empty StablePointerSet with the default hasher.
    pub fn new() -> Self {
        Self {
            index: hashbrown::HashSet::default(),
            storage: Vec::new(),
        }
    }

    /// Creates an empty StablePointerSet with the specified capacity and default hasher.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            index: hashbrown::HashSet::with_capacity_and_hasher(capacity, RandomState::new()),
            storage: Vec::with_capacity(capacity),
        }
    }
}

impl<T, S> StablePointerSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    /// Creates an empty StablePointerSet with the specified hasher.
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            index: hashbrown::HashSet::with_hasher(hasher),
            storage: Vec::new(),
        }
    }

    /// Creates an empty StablePointerSet with the specified capacity and hasher.
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            index: hashbrown::HashSet::with_capacity_and_hasher(capacity, hasher),
            storage: Vec::with_capacity(capacity),
        }
    }
    
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.index.len(), self.storage.len(), "Index and storage sizes must match");
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
        // Check if we already have this value
        let raw_ptr = self.get_raw_ptr(&value);

        if let Some(ptr) = raw_ptr {
            // We already have this value, return pointer to existing element
            return (ptr, false);
        }

        // Insert new value
        let boxed = Box::new(value);
        let ptr = unsafe {
            StablePointer::new(NonNull::new_unchecked(boxed.as_ref() as *const T as *mut T))
        };
        
        // First add to storage, then to index
        self.storage.push(boxed);
        let inserted = self.index.insert(unsafe { ptr.copy() });
        
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
        // Check if we already have this value
        let raw_ptr = self.get_raw_ptr_equiv(value);

        if let Some(ptr) = raw_ptr {
            // We already have this value, return pointer to existing element
            return (ptr, false);
        }

        // Convert the reference to an actual value only if needed
        let value_to_insert = T::from(value);
        
        // Insert new value
        let boxed = Box::new(value_to_insert);
        let ptr  = unsafe {
            StablePointer::new(NonNull::new_unchecked(boxed.as_ref() as *const T as *mut T))
        };

        // First add to storage, then to index
        self.storage.push(boxed);        
        let inserted = self.index.insert(unsafe { ptr.copy() });  

        debug_assert!(inserted, "Value should not already exist in the index");
        
        (ptr, true)
    }

    /// Returns `true` if the set contains a value.
    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get_raw_ptr(value).is_some()
    }

    /// Returns a stable pointer to a value in the set, if present.
    ///
    /// Searches for a value equal to the provided reference and returns a pointer to the stored element.
    /// The returned pointer remains valid until the element is removed from the set.
    pub fn get<Q: ?Sized>(&self, value: &Q) -> Option<StablePointer<T>>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get_raw_ptr(value)
    }

    /// Returns an iterator over the elements of the set.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.storage.iter().map(|boxed| boxed.as_ref())
    }

    /// Clears the set, removing all values.
    pub fn clear(&mut self) {
        self.index.clear();
        self.storage.clear();
    }

    /// Returns a stable pointer to the value in the set, if any, that is equal to the given value.
    fn get_raw_ptr<Q: ?Sized>(&self, value: &Q) -> Option<StablePointer<T>>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        for ptr in &self.index {
            if (**ptr).borrow() == value {
                return Some(unsafe { ptr.copy() });
            }
        }
        None
    }

    /// Returns a stable pointer to the value in the set, if any, that is equivalent to the given value.
    fn get_raw_ptr_equiv<Q>(&self, equiv: &Q) -> Option<StablePointer<T>>
    where
        Q: Equivalent<T>,
    {
        for ptr in &self.index {
            if equiv.equivalent(&**ptr) {
                return Some(unsafe { ptr.copy() });
            }
        }
        None
    }

    /// Removes an element from the set using its stable pointer. This is very inefficient and requires O(n) time.
    ///
    /// Returns the removed element if it was in the set.
    ///
    /// # Safety
    ///
    /// This is unsafe because:
    /// - The provided pointer must be valid and originally obtained from this set
    /// - Any other StablePointers to the removed element become dangling and must not be used
    /// - The caller takes ownership of the returned value
    pub unsafe fn remove(&mut self, pointer: StablePointer<T>) -> Option<T> {
        // First check if the pointer is in our index
        if !self.index.remove(&pointer) {
            return None;
        }

        // Find the element in storage
        let mut index_to_remove = None;
        for (i, boxed) in self.storage.iter().enumerate() {
            if std::ptr::eq(boxed.as_ref() as *const T, pointer.0.as_ptr()) {
                index_to_remove = Some(i);
                break;
            }
        }

        // If found, remove it from storage and return the value
        if let Some(idx) = index_to_remove {
            // Remove the box and convert it to the inner value
            let boxed = self.storage.swap_remove(idx);
            debug_assert_eq!(self.index.len(), self.storage.len(), "Index and storage sizes must match after removal");
            Some(*boxed)
        } else {
            // This should never happen if our index and storage are in sync
            debug_assert!(false, "Pointer found in index but not in storage - internal state corruption");
            None
        }
    }


    /// Retains only the elements specified by the predicate, modifying the set in-place.
    ///
    /// The predicate closure is called with a mutable reference to each element and must 
    /// return true if the element should remain in the set.
    ///
    /// # Safety
    ///
    /// This is unsafe because:
    /// - It invalidates any StablePointers to removed elements
    pub unsafe fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&StablePointer<T>) -> bool
    {
        // First pass: determine what to keep/remove without modifying the collection
        self.storage.retain(|element| {
            let ptr = unsafe {
                StablePointer::new(NonNull::new_unchecked(element.as_ref() as *const T as *mut T))
            };

            if !predicate(&ptr) {
                self.index.remove(&ptr);
                return false;
            }

            true
        });

        debug_assert_eq!(self.index.len(), self.storage.len(), 
            "Index and storage sizes must match after retention");
    }
}

impl<T, S> Drop for StablePointerSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    fn drop(&mut self) {
        // Ensure storage is cleared explicitly to make the drop order deterministic
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        
        let mut values: Vec<i32> = set.iter().map(|&v| v).collect();
        values.sort();
        
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_insert_equiv_ref() {
        #[derive(Hash, PartialEq, Eq, Debug)]
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
        let value: &i32 = &*ptr;
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
        unsafe {
            let removed = set.remove(ptr1);
            assert_eq!(removed, Some(42));
            assert_eq!(set.len(), 1);
            
            // The pointer is now invalid, using it would be unsafe
            
            // Try to remove same pointer again - should fail
            let removed_again = set.remove(ptr1);
            assert_eq!(removed_again, None);
            
            // Remove other value
            let removed2 = set.remove(ptr2);
            assert_eq!(removed2, Some(100));
            assert_eq!(set.len(), 0);
        }
    }

    #[test]
    fn test_retain_mut() {
        let mut set = StablePointerSet::new();
        
        // Insert values
        let (ptr1, _) = set.insert(1);
        let (ptr2, _) = set.insert(2);
        let (ptr3, _) = set.insert(3);
        let (ptr4, _) = set.insert(4);
        assert_eq!(set.len(), 4);
        
        // Retain only even numbers
        unsafe {
            set.retain(|x| **x % 2 == 0);
        }
        
        // Verify results
        assert_eq!(set.len(), 2);
        assert!(!set.contains(&1));
        assert!(set.contains(&2));
        assert!(!set.contains(&3));
        assert!(set.contains(&4));
        
        // Verify that removed pointers are invalid and remaining are valid
        unsafe {
            assert_eq!(set.remove(ptr2), Some(2));
            assert_eq!(set.remove(ptr4), Some(4));
            
            // These pointers are no longer valid - the remove should return None
            assert_eq!(set.remove(ptr1), None);
            assert_eq!(set.remove(ptr3), None);
        }
    }

    #[test]
    fn test_custom_hasher() {
        // Use FxHasher explicitly
        let mut set: StablePointerSet<i32, BuildHasherDefault<FxHasher>> = 
            StablePointerSet::with_hasher(BuildHasherDefault::<FxHasher>::default());
        
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