use crate::Data;
use crate::Ldd;
use crate::Storage;
use crate::Value;

// Returns an iterator over all right siblings of the given LDD.
pub fn iter_right<'a>(storage: &'a Storage, ldd: &Ldd) -> IterRight<'a> {
    IterRight {
        storage,
        current: ldd.clone(),
    }
}

// Returns an iterator over all vectors contained in the given LDD.
pub fn iter<'a>(storage: &'a Storage, ldd: &Ldd) -> Iter<'a> {
    if ldd == storage.empty_set() {
        Iter {
            storage,
            vector: Vec::new(),
            stack: Vec::new(),
        }
    } else {
        Iter {
            storage,
            vector: Vec::new(),
            stack: vec![ldd.clone()],
        }
    }
}

pub struct IterRight<'a> {
    storage: &'a Storage,
    current: Ldd,
}

impl Iterator for IterRight<'_> {
    type Item = Data;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == *self.storage.empty_set() {
            None
        } else {
            // Progress to the right LDD.
            let Data(value, down, right) = self.storage.get(&self.current);
            self.current = right.clone();
            Some(Data(value, down, right))
        }
    }
}

pub struct Iter<'a> {
    storage: &'a Storage,
    vector: Vec<Value>, // Stores the values of the returned vector.
    stack: Vec<Ldd>,    // Stores the stack for the depth-first search (only non 'true' or 'false' nodes)
}

impl Iterator for Iter<'_> {
    type Item = Vec<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the next vector by going down the chain.
        let vector: Vec<Value>;
        loop {
            let current = self.stack.last()?;

            let Data(value, down, _) = self.storage.get(current);
            self.vector.push(value);
            if down == *self.storage.empty_vector() {
                vector = self.vector.clone();
                break; // Stop iteration.
            } else {
                self.stack.push(down.clone());
            }
        }

        // Go up the chain to find the next right sibling that is not 'false'.
        while let Some(current) = self.stack.pop() {
            self.vector.pop();
            let Data(_, _, right) = self.storage.get(&current);

            if right != *self.storage.empty_set() {
                self.stack.push(right); // This is the first right sibling.
                break;
            }
        }

        Some(vector)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use mcrl3_utilities::random_test;

    use crate::test_utility::{from_iter, random_vector_set};

    // Test the iterator implementation.
    #[test]
    fn test_random_iter() {
        random_test(100, |rng| {
            let mut storage = Storage::new();

            let set = random_vector_set(rng, 32, 10, 10);
            let ldd = from_iter(&mut storage, set.iter());

            assert!(
                iter(&storage, &ldd).count() == set.len(),
                "Number of iterations does not match the number of elements in the set."
            );

            for vector in iter(&storage, &ldd) {
                assert!(set.contains(&vector), "Found element not in the set.");
            }
        })
    }
}
