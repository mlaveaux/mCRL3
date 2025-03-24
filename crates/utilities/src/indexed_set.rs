use core::panic;
use std::hash::Hash;
use std::ops::Index;
use std::ops::IndexMut;

use rustc_hash::FxHashMap;

pub struct IndexedSet<T> {
    table: Vec<Entry<T>>,
    index: FxHashMap<T, usize>,
    free: Option<usize>, // A list of free nodes.
}

enum Entry<T> {
    Filled(T),
    Empty(usize),
}

/// A macro to return the pat type of an enum class target, and panics otherwise.
///
/// Usage cast!(instance, type)
macro_rules! cast {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            // #1
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat)); // #2
        }
    }};
}

impl<T> IndexedSet<T> {
    /// Creates a new empty IndexedSet.
    pub fn new() -> IndexedSet<T> {
        IndexedSet {
            table: Vec::default(),
            index: FxHashMap::default(),
            free: None,
        }
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns a reference to the element at the given index, if it exists.
    pub fn get(&self, index: usize) -> Option<&T> {
        if let Some(entry) = self.table.get(index) {
            match entry {
                Entry::Filled(element) => Some(element),
                Entry::Empty(_) => None,
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
    pub fn iter(&self) -> Iter<T> {
        Iter {
            reference: self,
            index: 0,
        }
    }

    /// Returns a mutable iterator over the elements in the set.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        let iter = self
            .table
            .iter_mut()
            .filter(|element| matches!(element, Entry::Filled(_)))
            .map(|element| cast!(element, Entry::Filled))
            .enumerate();

        IterMut { iter: Box::new(iter) }
    }
}

impl<T: Eq + Hash + Clone> IndexedSet<T> {
    /// Inserts the given element into the set
    /// Teturns the corresponding index and a boolean indicating if the element was inserted.
    pub fn insert(&mut self, value: T) -> (usize, bool) {
        match self.index.entry(value.clone()) {
            std::collections::hash_map::Entry::Occupied(entry) => (*entry.get(), false),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let index = match self.free {
                    Some(first) => {
                        let next = match self.table[first] {
                            Entry::Empty(x) => x,
                            Entry::Filled(_) => panic!("The free list contains a filled element"),
                        };

                        if first == next {
                            // The list is now empty as its first element points to itself.
                            self.free = None;
                        } else {
                            // Update free to be the next element in the list.
                            self.free = Some(next);
                        }

                        self.table[first] = Entry::Filled(value);
                        first
                    }
                    None => {
                        // No free positions so insert new.
                        self.table.push(Entry::Filled(value));
                        self.table.len() - 1
                    }
                };
                entry.insert(index);
                (index, true)
            }
        }
    }

    /// Erases all elements for which f(index, element) returns false. Allows
    /// modifying the given element (as long as the hash/equality does not change).
    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, &mut T) -> bool,
    {
        for (index, element) in self.table.iter_mut().enumerate() {
            if let Entry::Filled(value) = element {
                if !f(index, value) {
                    self.index.remove(value);

                    match self.free {
                        Some(next) => {
                            *element = Entry::Empty(next);
                        }
                        None => {
                            *element = Entry::Empty(index);
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

            self.table[index] = Entry::Empty(next);
            self.free = Some(index);
        }
    }
}

impl<T> Default for IndexedSet<T> {
    fn default() -> IndexedSet<T> {
        IndexedSet::new()
    }
}

impl<T> Index<usize> for IndexedSet<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        cast!(&self.table[index], Entry::Filled)
    }
}

impl<T> IndexMut<usize> for IndexedSet<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        cast!(&mut self.table[index], Entry::Filled)
    }
}

pub struct Iter<'a, T> {
    reference: &'a IndexedSet<T>,
    index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.reference.table.len() {
            if let Entry::Filled(element) = &self.reference.table[self.index] {
                self.index += 1;
                return Some((self.index - 1, element));
            }
            self.index += 1;
        }

        None
    }
}

pub struct IterMut<'a, T> {
    iter: Box<dyn Iterator<Item = (usize, &'a mut T)> + 'a>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T> IntoIterator for &'a IndexedSet<T> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use ahash::HashMap;
    use rand::Rng;

    use super::IndexedSet;

    #[test]
    fn test_construction() {
        let mut rand = rand::rng();

        let mut input = vec![];
        for _ in 0..100 {
            input.push(rand.random::<u64>() as usize);
        }

        let mut indices: HashMap<usize, usize> = HashMap::default();

        // Insert several elements and keep track of the resulting indices.
        let mut set: IndexedSet<usize> = IndexedSet::default();
        for element in &input {
            let index = set.insert(*element).0;

            assert!(
                indices.get(&index).is_none() || indices.get(&index).unwrap() == element,
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
                "Index {index} should still match element {value}"
            );
        }
    }
}
