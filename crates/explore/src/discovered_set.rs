//! A discovered set that stores state vectors as maximally shared sequences.

use merc_unsafety::ConcurrentIndexedSet;

use crate::BTreeForest;
use crate::BTreeForestContext;
use crate::Slot;
use crate::Tree;

/// A stable handle to a state stored in a [`DiscoveredSet`].
///
/// References are dense and assigned in insertion order starting from zero.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct StateRef(usize);

impl StateRef {
    /// Returns the underlying index as a `usize`.
    pub fn index(self) -> usize {
        self.0
    }
}

/// A set of `T` state vectors that deduplicates equal vectors and assigns each
/// a stable [`StateRef`].
///
/// The set is thread-safe: states can be inserted and looked up concurrently
/// through `&self` from multiple threads.
pub struct DiscoveredSet<T = usize>
where
    T: Copy,
{
    /// Hash-consed forest backing every stored state sequence.
    forest: BTreeForest<T, 2>,
    /// Deduplicates canonical roots and assigns each a dense index, which *is*
    /// the [`StateRef`]. Reconstructing a state vector for a [`StateRef`] looks
    /// the root up by index and iterates it back through the forest.
    states: ConcurrentIndexedSet<Tree>,
}

impl<T> DiscoveredSet<T>
where
    T: Slot,
{
    /// Creates a new empty discovered set.
    pub fn new() -> DiscoveredSet<T> {
        DiscoveredSet {
            forest: BTreeForest::new(),
            states: ConcurrentIndexedSet::new(),
        }
    }

    /// Creates a new empty discovered set with room for at least `capacity`
    /// states before reallocating.
    pub fn with_capacity(capacity: usize) -> DiscoveredSet<T> {
        DiscoveredSet {
            forest: BTreeForest::new(),
            states: ConcurrentIndexedSet::with_capacity(capacity),
        }
    }

    /// Inserts `state` and returns its handle together with a boolean that is
    /// true when the state was newly inserted and false when it was already
    /// present.
    pub fn insert(&self, state: &[T]) -> (StateRef, bool) {
        self.insert_with(state, &mut BTreeForestContext::new())
    }

    /// Inserts `state` like [`DiscoveredSet::insert`], reusing the scratch
    /// buffers in `context` to avoid a reallocation per insert on hot paths.
    /// Each thread must use its own context.
    pub fn insert_with(&self, state: &[T], context: &mut BTreeForestContext) -> (StateRef, bool) {
        let root = self.forest.insert_with(state, context);
        let (index, is_new) = self.states.insert(root);
        (StateRef(index), is_new)
    }

    /// Returns the handle of `state` if it is present, or `None` otherwise.
    pub fn index(&self, state: &[T]) -> Option<StateRef> {
        self.index_with(state, &mut BTreeForestContext::new())
    }

    /// Looks up `state` like [`DiscoveredSet::index`], reusing the scratch
    /// buffers in `context` to avoid a reallocation per lookup on hot paths.
    /// Each thread must use its own context.
    pub fn index_with(&self, state: &[T], context: &mut BTreeForestContext) -> Option<StateRef> {
        let root = self.forest.insert_with(state, context);
        self.states.index(&root).map(StateRef)
    }

    /// Returns true if `state` is present in the set.
    pub fn contains(&self, state: &[T]) -> bool {
        self.index(state).is_some()
    }

    /// Reconstructs the state vector for `reference` into the freshly cleared
    /// `out` buffer. Reusing a buffer avoids an allocation per lookup, which
    /// matters on the hot exploration path. Returns false if the reference is
    /// out of range.
    pub fn get_into(&self, reference: StateRef, out: &mut Vec<T>) -> bool {
        out.clear();
        let Some(&tree) = self.states.get_by_index(reference.index()) else {
            return false;
        };
        out.extend(self.forest.iter(tree));
        true
    }

    /// Returns the state vector for `reference`, allocating a fresh [`Vec`].
    ///
    /// Prefer [`DiscoveredSet::get_into`] on hot paths to reuse a buffer.
    pub fn get(&self, reference: StateRef) -> Option<Vec<T>> {
        let &tree = self.states.get_by_index(reference.index())?;
        Some(self.forest.iter(tree).collect())
    }

    /// Removes all states, invalidating every previously returned
    /// [`StateRef`].
    pub fn clear(&mut self) {
        self.forest.clear();
        self.states.clear();
    }
}

impl<T> DiscoveredSet<T>
where
    T: Copy,
{
    /// Returns the number of distinct states in the set.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

impl<T> Default for DiscoveredSet<T>
where
    T: Slot,
{
    fn default() -> DiscoveredSet<T> {
        DiscoveredSet::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::thread;

    use super::DiscoveredSet;

    #[test]
    fn test_insert_dedup() {
        let set: DiscoveredSet<usize> = DiscoveredSet::new();

        let (first, is_new) = set.insert(&[1, 2, 3]);
        assert!(is_new);
        let (again, is_new) = set.insert(&[1, 2, 3]);
        assert!(!is_new, "an existing state is not newly inserted");
        assert_eq!(first, again, "equal states share a handle");
        assert_eq!(set.len(), 1);

        let mut out = Vec::new();
        assert!(set.get_into(first, &mut out));
        assert_eq!(out, vec![1, 2, 3]);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_concurrent_insert() {
        let set: Arc<DiscoveredSet<usize>> = Arc::new(DiscoveredSet::new());
        let new_count = Arc::new(AtomicUsize::new(0));
        let num_threads = 8;
        let num_states = 1000usize;

        // Every thread inserts the same set of distinct states, so each state is
        // contended but must be counted as new exactly once overall.
        let mut handles = Vec::new();
        for _ in 0..num_threads {
            let set = Arc::clone(&set);
            let new_count = Arc::clone(&new_count);
            handles.push(thread::spawn(move || {
                for i in 0..num_states {
                    let (reference, is_new) = set.insert(&[i, i + 1, i + 2]);
                    assert!(reference.index() < num_states, "indices stay dense");
                    if is_new {
                        new_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(
            new_count.load(Ordering::Relaxed),
            num_states,
            "each state inserted once"
        );
        assert_eq!(set.len(), num_states);

        // Indices are dense (a permutation of `0..num_states`) and each handle
        // reconstructs to the right state vector.
        let mut seen = vec![false; num_states];
        let mut out = Vec::new();
        for i in 0..num_states {
            let reference = set.index(&[i, i + 1, i + 2]).expect("state must be present");
            assert!(!seen[reference.index()], "indices are unique");
            seen[reference.index()] = true;
            assert!(set.get_into(reference, &mut out));
            assert_eq!(out, vec![i, i + 1, i + 2]);
        }
    }
}
