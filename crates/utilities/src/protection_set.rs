use core::panic;
use std::ops::Index;

/// The protection set keeps track of nodes that should not be garbage
/// collected since they are being referenced by instances.
#[derive(Debug, Default)]
pub struct ProtectionSet<T> {
    roots: Vec<Entry<T>>, // The set of root active nodes.
    free: Option<usize>,
    number_of_insertions: u64,
    size: usize,
}

#[derive(Debug)]
enum Entry<T> {
    Filled(T),
    Free(usize),
}

impl<T> ProtectionSet<T> {
    pub fn new() -> Self {
        ProtectionSet {
            roots: vec![],
            free: None,
            number_of_insertions: 0,
            size: 0,
        }
    }

    /// Returns the number of insertions into the protection set.
    pub fn number_of_insertions(&self) -> u64 {
        self.number_of_insertions
    }

    /// Returns maximum number of active instances.
    pub fn maximum_size(&self) -> usize {
        self.roots.capacity()
    }

    /// Returns the number of roots in the protection set
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns whether the protection set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over all root indices in the protection set.
    pub fn iter(&self) -> ProtSetIter<T> {
        ProtSetIter {
            current: 0,
            protection_set: self,
        }
    }

    /// Returns whether the protection set contains the given index.
    pub fn contains_root(&self, index: usize) -> bool {
        matches!(self.roots[index], Entry::Filled(_))
    }

    /// Adds the given object to the protection set and returns its index.
    pub fn protect(&mut self, object: T) -> usize {
        self.number_of_insertions += 1;
        self.size += 1;

        match self.free {
            Some(first) => {
                match &self.roots[first] {
                    Entry::Free(next) => {
                        if first == *next {
                            // The list is empty as its first element points to itself.
                            self.free = None;
                        } else {
                            // Update free to be the next element in the list.
                            self.free = Some(*next);
                        }
                    }
                    Entry::Filled(_) => {
                        panic!("The free list should not point a filled entry");
                    }
                }

                self.roots[first] = Entry::Filled(object);
                first
            }
            None => {
                // If free list is empty insert new entry into roots.
                self.roots.push(Entry::Filled(object));
                self.roots.len() - 1
            }
        }
    }

    /// Remove protection from the given LDD node. Note that index must be the
    /// index returned by the [ProtectionSet::protect] call.
    pub fn unprotect(&mut self, index: usize) {
        debug_assert!(
            matches!(self.roots[index], Entry::Filled(_)),
            "Index {index} is not valid"
        );
        self.size -= 1;

        match self.free {
            Some(next) => {
                self.roots[index] = Entry::Free(next);
            }
            None => {
                self.roots[index] = Entry::Free(index);
            }
        };

        self.free = Some(index);
    }
}

impl<T> Index<usize> for ProtectionSet<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.roots[index] {
            Entry::Filled(value) => value,
            Entry::Free(_) => {
                panic!("Attempting to index free spot {}", index);
            }
        }
    }
}

pub struct ProtSetIter<'a, T> {
    current: usize,
    protection_set: &'a ProtectionSet<T>,
}

impl<'a, T> Iterator for ProtSetIter<'a, T> {
    type Item = (&'a T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // Find the next valid entry, return it when found or None when end of roots is reached.
        while self.current < self.protection_set.roots.len() {
            if let Entry::Filled(object) = &self.protection_set.roots[self.current] {
                self.current += 1;
                return Some((object, self.current - 1));
            } else {
                self.current += 1;
            }
        }

        None
    }
}

impl<'a, T> IntoIterator for &'a ProtectionSet<T> {
    type Item = (&'a T, usize);
    type IntoIter = ProtSetIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    use crate::test_logger;
    use crate::test_rng;

    #[test]
    fn test_protection_set() {
        let _ = test_logger();
        let mut protection_set = ProtectionSet::<usize>::new();

        // Protect a number of indices and record their roots.
        let mut indices: Vec<usize> = Vec::new();

        let mut rng = test_rng();

        for _ in 0..5000 {
            indices.push(protection_set.protect(rng.random_range(0..1000)));
        }

        // Unprotect a number of roots.
        for index in 0..2500 {
            assert!(protection_set[indices[index]] <= 1000);
            protection_set.unprotect(indices[index]);
            indices.remove(index);
        }

        // Protect more to test the freelist
        for _ in 0..1000 {
            indices.push(protection_set.protect(rng.random_range(0..1000)));
        }

        for index in &indices {
            assert!(
                matches!(protection_set.roots[*index], Entry::Filled(_)),
                "All indices that are not unprotected should occur in the protection set"
            );
        }

        assert_eq!(
            protection_set.iter().count(),
            6000 - 2500,
            "This is the number of roots remaining"
        );
        assert_eq!(protection_set.number_of_insertions(), 6000);
        assert!(protection_set.maximum_size() >= 5000);
        assert!(!protection_set.is_empty());

        println!("{:?}", protection_set);
    }
}
