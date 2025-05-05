use core::panic;
use std::fmt;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;

use hashbrown::Equivalent;
use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use hashbrown::hash_map::EntryRef;

use rustc_hash::FxBuildHasher;

use crate::GenerationCounter;
use crate::GenerationalIndex;

/// A type-safe index for use with IndexedSet.
/// This newtype prevents accidental usage of raw usize values as indices.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Default)]
pub struct SetIndex(GenerationalIndex<usize>);

impl Deref for SetIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for SetIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SetIndex({})", self.0)
    }
}

impl fmt::Display for SetIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


/// A set that assigns a unique index to every entry. The index can be used to access the inserted entry.
///
/// TODO: Remove the need for duplicating the key in the hash map.
pub struct IndexedSet<T, S = FxBuildHasher> {
    table: Vec<IndexSetEntry<T>>,
    index: HashMap<T, usize, S>,
    /// A list of free nodes, where the value is the first free node.
    free: Option<usize>,
    /// The number of generations
    generation_counter: GenerationCounter,
}

/// An entry in the indexed set, which can either be filled or empty.
enum IndexSetEntry<T> {
    Filled(T),
    Empty(usize),
}

/// A macro to return the pat type of an enum class target, and panics otherwise.
///
/// Usage cast!(instance, type)
macro_rules! cast {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat));
        }
    }};
}

impl<T, S: BuildHasher + Default> IndexedSet<T, S> {
    /// Creates a new empty IndexedSet with the default hasher.
    pub fn new() -> IndexedSet<T, S> {
        IndexedSet {
            table: Vec::default(),
            index: HashMap::default(),
            free: None,
            generation_counter: GenerationCounter::new(),
        }
    }

    /// Creates a new empty IndexedSet with the specified hasher.
    pub fn with_hasher(hash_builder: S) -> IndexedSet<T, S> {
        IndexedSet {
            table: Vec::default(),
            index: HashMap::with_hasher(hash_builder),
            free: None,
            generation_counter: GenerationCounter::new(),
        }
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the element at the given index, if it exists.
    pub fn get(&self, index: SetIndex) -> Option<&T> {
        if let Some(entry) = self.table.get(self.generation_counter.get_index(index.0)) {
            match entry {
                IndexSetEntry::Filled(element) => Some(element),
                IndexSetEntry::Empty(_) => None,
            }
        } else {
            None
        }
    }

    /// Returns the capacity of the set.
    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }

    /// Returns an iterator over the elements in the set.
    pub fn iter(&self) -> Iter<T, S> {
        Iter {
            reference: self,
            index: 0,
            generation_counter: &self.generation_counter,
        }
    }

    /// Returns a mutable iterator over the elements in the set.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        let iter = self
            .table
            .iter_mut()
            .filter(|element| matches!(element, IndexSetEntry::Filled(_)))
            .map(|element| cast!(element, IndexSetEntry::Filled))
            .enumerate()
            .map(|(idx, elem)| (SetIndex(self.generation_counter.recall_index(idx)), elem));

        IterMut { iter: Box::new(iter) }
    }
}

impl<T: Hash + Eq, S: BuildHasher> IndexedSet<T, S> {
    /// Inserts the given element into the set
    ///
    /// Returns the corresponding index and a boolean indicating if the element was inserted.
    pub fn insert_equiv<'a, Q>(&mut self, value: &'a Q) -> (SetIndex, bool)
    where
        Q: Hash + Equivalent<T>,
        T: From<&'a Q>,
    {
        match self.index.entry_ref(value) {
            EntryRef::Occupied(entry) => (SetIndex(self.generation_counter.recall_index(*entry.get())), false),
            EntryRef::Vacant(entry) => {
                let index = match self.free {
                    Some(first) => {
                        let next = match self.table[first] {
                            IndexSetEntry::Empty(x) => x,
                            IndexSetEntry::Filled(_) => panic!("The free list contains a filled element"),
                        };

                        if first == next {
                            // The list is now empty as its first element points to itself.
                            self.free = None;
                        } else {
                            // Update free to be the next element in the list.
                            self.free = Some(next);
                        }

                        self.table[first] = IndexSetEntry::Filled(value.into());
                        first
                    }
                    None => {
                        // No free positions so insert new.
                        self.table.push(IndexSetEntry::Filled(value.into()));
                        self.table.len() - 1
                    }
                };

                entry.insert(index);
                (SetIndex(self.generation_counter.create_index(index)), true)
            }
        }
    }

    /// Inserts the given element into the set
    ///
    /// Returns the corresponding index and a boolean indicating if the element was inserted.
    pub fn insert(&mut self, value: T) -> (SetIndex, bool)
    where
        T: Clone,
    {
        match self.index.entry(value.clone()) {
            Entry::Occupied(entry) => (SetIndex(self.generation_counter.recall_index(*entry.get())), false),
            Entry::Vacant(entry) => {
                let index = match self.free {
                    Some(first) => {
                        let next = match self.table[first] {
                            IndexSetEntry::Empty(x) => x,
                            IndexSetEntry::Filled(_) => panic!("The free list contains a filled element"),
                        };

                        if first == next {
                            // The list is now empty as its first element points to itself.
                            self.free = None;
                        } else {
                            // Update free to be the next element in the list.
                            self.free = Some(next);
                        }

                        self.table[first] = IndexSetEntry::Filled(value);
                        first
                    }
                    None => {
                        // No free positions so insert new.
                        self.table.push(IndexSetEntry::Filled(value));
                        self.table.len() - 1
                    }
                };

                entry.insert(index);
                (SetIndex(self.generation_counter.create_index(index)), true)
            }
        }
    }
}

impl<T: Eq + Hash, S: BuildHasher> IndexedSet<T, S> {
    /// Erases all elements for which f(index, element) returns false. Allows
    /// modifying the given element (as long as the hash/equality does not change).
    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(SetIndex, &mut T) -> bool,
    {
        for (index, element) in self.table.iter_mut().enumerate() {
            if let IndexSetEntry::Filled(value) = element {
                if !f(SetIndex(self.generation_counter.recall_index(index)), value) {
                    self.index.remove(value);

                    match self.free {
                        Some(next) => {
                            *element = IndexSetEntry::Empty(next);
                        }
                        None => {
                            *element = IndexSetEntry::Empty(index);
                        }
                    };
                    self.free = Some(index);
                }
            };
        }
    }

    /// Removes the given element from the set.
    pub fn remove(&mut self, element: &T) {
        if let Some(index) = self.index.remove(element) {
            let next = match self.free {
                Some(next) => next,
                None => index,
            };

            self.table[index] = IndexSetEntry::Empty(next);
            self.free = Some(index);
        }
    }
}

impl<T, S: BuildHasher + Default> Default for IndexedSet<T, S> {
    fn default() -> IndexedSet<T, S> {
        IndexedSet::new()
    }
}

impl<T, S: BuildHasher> Index<SetIndex> for IndexedSet<T, S> {
    type Output = T;

    fn index(&self, index: SetIndex) -> &Self::Output {
        cast!(&self.table[*index], IndexSetEntry::Filled)
    }
}

impl<T, S: BuildHasher> IndexMut<SetIndex> for IndexedSet<T, S> {
    fn index_mut(&mut self, index: SetIndex) -> &mut Self::Output {
        cast!(&mut self.table[*index], IndexSetEntry::Filled)
    }
}

pub struct Iter<'a, T, S> {
    reference: &'a IndexedSet<T, S>,
    index: usize,
    generation_counter: &'a GenerationCounter,
}

impl<'a, T, S> Iterator for Iter<'a, T, S> {
    type Item = (SetIndex, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.reference.table.len() {
            let current_index = self.index;
            self.index += 1;

            if let IndexSetEntry::Filled(element) = &self.reference.table[current_index] {
                return Some((SetIndex(self.generation_counter.recall_index(current_index)), element));
            }
        }

        None
    }
}

pub struct IterMut<'a, T> {
    iter: Box<dyn Iterator<Item = (SetIndex, &'a mut T)> + 'a>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (SetIndex, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T, S: BuildHasher + Default> IntoIterator for &'a IndexedSet<T, S> {
    type Item = (SetIndex, &'a T);
    type IntoIter = Iter<'a, T, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_logger;
    use crate::test_rng;

    use super::*;

    use ahash::RandomState;
    use rand::Rng;
    use std::collections::HashMap;

    #[test]
    fn test_random_indexed_set_construction() {
        let _ = test_logger();
        let mut rand = test_rng();

        let mut input = vec![];
        for _ in 0..100 {
            input.push(rand.random::<u64>() as usize);
        }

        let mut indices: HashMap<usize, SetIndex> = HashMap::default();

        // Insert several elements and keep track of the resulting indices.
        let mut set: IndexedSet<usize> = IndexedSet::default();
        for element in &input {
            let index = set.insert(*element).0;

            assert!(
                indices.contains_key(&index) || **indices.get(&index).unwrap() == *element,
                "Index was already used for another element"
            );
            indices.insert(*element, index);
        }

        for value in &mut input.iter().take(10) {
            set.remove(value);
            indices.remove(value);
        }

        // Check consistency of the indexed set.
        for (index, value) in &set {
            assert_eq!(
                indices[value], index,
                "The resulting index does not match the returned value"
            );
        }

        for (value, index) in &indices {
            assert!(
                set.get(*index) == Some(value),
                "Index {} should still match element {:?}",
                *index,
                value
            );
        }
    }

    #[test]
    fn test_custom_hasher() {
        let _ = test_logger();
        let mut rand = test_rng();

        // Create a set with a custom hasher (AHasher)
        let mut set: IndexedSet<usize, RandomState> = IndexedSet::with_hasher(RandomState::new());

        let mut indices = HashMap::new();

        // Insert some elements
        for _ in 0..50 {
            let value = rand.random::<u64>() as usize;
            let (index, _) = set.insert(value);
            indices.insert(value, index);
        }

        // Verify all elements are accessible
        for (value, index) in &indices {
            assert_eq!(set.get(*index), Some(value));
        }

        // Remove some elements
        let mut to_remove = indices.keys().cloned().collect::<Vec<_>>();
        to_remove.truncate(10);

        for value in to_remove {
            set.remove(&value);
            indices.remove(&value);
        }

        // Check consistency after removal
        for (value, index) in &indices {
            assert_eq!(set.get(*index), Some(value));
        }

        // Verify iteration works correctly
        for (index, value) in &set {
            assert_eq!(indices[value], index);
        }
    }
}
