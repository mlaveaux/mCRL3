use streaming_iterator::StreamingIterator;

use crate::Data;
use crate::DataRef;
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
    if ldd == storage.empty_vector() || ldd == storage.empty_set() {
        Iter {
            storage,
            vector: Vec::new(),
            current: Vec::new(),
            stack: Vec::new(),
            finished: false,
        }
    } else {
        Iter {
            storage,
            vector: Vec::new(),
            current: Vec::new(),
            stack: vec![ldd.clone()],
            finished: false,
        }
    }
}

// Calls `f` with `&mut storage` and each vector in the LDD.  Unlike `iter`, the closure receives a
// mutable storage reference so it can insert nodes without a separate collection step.
pub fn for_each_mut<F>(storage: &mut Storage, ldd: &Ldd, mut f: F)
where
    F: FnMut(&mut Storage, &[Value]),
{
    if ldd == storage.empty_vector() || ldd == storage.empty_set() {
        return;
    }

    let mut vector: Vec<Value> = Vec::new();
    let mut stack: Vec<Ldd> = vec![ldd.clone()];

    loop {
        // Descend to the next leaf (node whose down == empty_vector).
        loop {
            let (value, down) = {
                let DataRef(value, down, _) = storage.get_ref(stack.last().unwrap());
                let is_leaf = down == *storage.empty_vector();
                let down_protected = (!is_leaf).then(|| storage.protect(&down));
                (value, down_protected)
            };
            vector.push(value);
            match down {
                Some(down_ldd) => stack.push(down_ldd),
                None => break, // reached leaf
            }
        }

        // Storage is no longer borrowed — the closure may mutate it freely.
        f(storage, &vector);

        // Ascend to find the next right sibling.
        while let Some(current) = stack.pop() {
            vector.pop();
            let right = {
                let DataRef(_, _, right) = storage.get_ref(&current);
                if right != *storage.empty_set() {
                    Some(storage.protect(&right))
                } else {
                    None
                }
            };
            if let Some(right_ldd) = right {
                stack.push(right_ldd);
                break;
            }
        }

        if stack.is_empty() {
            break;
        }
    }
}

// Returns an iterator over all nodes in the given LDD. Visits each node only if the predicate holds.
pub fn iter_nodes<'a, P>(storage: &'a Storage, ldd: &Ldd, filter: P) -> IterNode<'a, P>
where
    P: Fn(&Ldd) -> bool,
{
    let mut stack = Vec::new();

    if ldd != storage.empty_set() {
        stack.push((ldd.clone(), false));
    }

    IterNode {
        storage,
        stack,
        predicate: filter,
    }
}

pub struct IterNode<'a, P>
where
    P: Fn(&Ldd) -> bool,
{
    storage: &'a Storage,
    stack: Vec<(Ldd, bool)>,
    predicate: P,
}

impl<P> Iterator for IterNode<'_, P>
where
    P: Fn(&Ldd) -> bool,
{
    type Item = (Ldd, Data);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((current, visited)) = self.stack.pop() {
            let data = self.storage.get(&current);

            if visited {
                return Some((current, data));
            }

            let Data(_, down, right) = &data;

            // Next time we can actually process the current node.
            self.stack.push((current.clone(), true));

            // Add unvisited children to stack
            if (self.predicate)(down) {
                self.stack.push((down.clone(), false));
            }

            if (self.predicate)(right) {
                self.stack.push((right.clone(), false));
            }
        }

        None
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
    /// Stores the values of the returned vector.
    vector: Vec<Value>,
    /// Stores the current path in the LDD.
    current: Vec<Value>,
    /// Stores the stack for the depth-first search (only non 'true' or 'false' nodes)
    stack: Vec<Ldd>,
    /// Indicates whether the iteration is finished.
    finished: bool,
}

impl StreamingIterator for Iter<'_> {
    type Item = Vec<Value>;

    fn advance(&mut self) {
        // Find the next vector by going down the chain.
        loop {
            if let Some(current) = self.stack.last() {
                let DataRef(value, down, _) = self.storage.get_ref(current);
                self.vector.push(value);
                if down == *self.storage.empty_vector() {
                    // Reserve sufficient space in current
                    if self.current.len() < self.vector.len() {
                        self.current.resize(self.vector.len(), Value::default());
                    }

                    self.current.copy_from_slice(&self.vector);
                    break; // Stop iteration.
                } else {
                    self.stack.push(self.storage.protect(&down));
                }
            } else {
                // No more elements to iterate.
                self.finished = true;
                return;
            }
        }

        // Go up the chain to find the next right sibling that is not 'false'.
        while let Some(current) = self.stack.pop() {
            self.vector.pop();
            let DataRef(_, _, right) = self.storage.get_ref(&current);

            if right != *self.storage.empty_set() {
                self.stack.push(self.storage.protect(&right)); // This is the first right sibling.
                break;
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.finished { None } else { Some(&self.current) }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use merc_utilities::random_test;

    use crate::test_utility::from_iter;
    use crate::test_utility::random_vector_set;

    // Test the iterator implementation.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_random_iter() {
        random_test(100, |rng| {
            let mut storage = Storage::new();

            let set = random_vector_set(rng, 32, 10, 10);
            let ldd = from_iter(&mut storage, set.iter());

            assert!(
                iter(&storage, &ldd).count() == set.len(),
                "Number of iterations does not match the number of elements in the set."
            );

            let mut iter = iter(&storage, &ldd);
            while let Some(vector) = iter.next() {
                assert!(set.contains(vector), "Found element not in the set.");
            }
        })
    }
}
