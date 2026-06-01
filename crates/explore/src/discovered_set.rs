//! A discovered set that stores state vectors as maximally shared B-trees.

use std::hash::BuildHasher;
use std::hash::Hash;

use allocator_api2::alloc::Allocator;
use allocator_api2::alloc::Global;
use allocator_api2::vec::Vec;
use hashbrown::HashTable;
use rustc_hash::FxBuildHasher;

use crate::BTreeForest;
use crate::Tree;

/// A stable handle to a state stored in a [`DiscoveredSet`].
///
/// References are dense and assigned in insertion order starting from zero. A
/// 32-bit representation is used to match the [`BTreeForest`] storage, which is
/// optimised for 32-bit keys and values.
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
/// The state vectors are stored as hash-consed B-trees in a shared
/// [`BTreeForest`], so positions and values common to many states are stored
/// only once. The auxiliary collections allocate from `A`.
pub struct DiscoveredSet<T = usize, A = Global>
where
    T: Copy,
    A: Allocator,
{
    /// Hash-consed forest backing every stored state B-tree. The key type is a
    /// unit (zero-sized) type because positions are implicit in iteration order
    /// and never looked up; this keeps leaves storing only values and lets nodes
    /// be shared by content regardless of position.
    forest: BTreeForest<(), T, A, 2>,
    /// Stored states indexed by [`StateRef`]; each is the root of the state's
    /// value B-tree.
    states: Vec<Tree, A>,
    /// Hash index from a state's canonical root to its raw index into `states`.
    /// Hash consing makes the root a unique fingerprint of the whole vector, so
    /// no separate content hash or comparison is needed.
    table: HashTable<usize, A>,
    /// Hasher used to fingerprint roots in `table`.
    hasher: FxBuildHasher,
}

impl<T> DiscoveredSet<T, Global>
where
    T: Copy + Eq + Hash,
{
    /// Creates a new empty discovered set backed by the global allocator.
    pub fn new() -> DiscoveredSet<T, Global> {
        DiscoveredSet::new_in(Global)
    }

    /// Creates a new empty discovered set with room for at least `capacity`
    /// states before reallocating, backed by the global allocator.
    pub fn with_capacity(capacity: usize) -> DiscoveredSet<T, Global> {
        DiscoveredSet::with_capacity_in(capacity, Global)
    }
}

impl<T, A> DiscoveredSet<T, A>
where
    T: Copy + Eq + Hash,
    A: Allocator + Clone,
{
    /// Creates a new empty discovered set whose auxiliary collections allocate
    /// from `alloc`.
    pub fn new_in(alloc: A) -> DiscoveredSet<T, A> {
        DiscoveredSet {
            forest: BTreeForest::new_in(alloc.clone()),
            states: Vec::new_in(alloc.clone()),
            table: HashTable::new_in(alloc),
            hasher: FxBuildHasher,
        }
    }

    /// Creates a new empty discovered set with room for at least `capacity`
    /// states before reallocating, whose auxiliary collections allocate from
    /// `alloc`.
    pub fn with_capacity_in(capacity: usize, alloc: A) -> DiscoveredSet<T, A> {
        DiscoveredSet {
            forest: BTreeForest::new_in(alloc.clone()),
            states: Vec::with_capacity_in(capacity, alloc.clone()),
            table: HashTable::with_capacity_in(capacity, alloc),
            hasher: FxBuildHasher,
        }
    }

    /// Inserts `state` and returns its handle together with a boolean that is
    /// true when the state was newly inserted and false when it was already
    /// present.
    pub fn insert(&mut self, state: &[T]) -> (StateRef, bool) {
        // Intern the state's B-tree. Hash consing makes the resulting root a
        // canonical fingerprint of the whole vector: equal vectors always yield
        // the same root, so the root alone identifies the state. Re-interning an
        // already known state allocates no new nodes.
        let root = self.forest.build(state, |_| ());
        let hash = self.hasher.hash_one(root);

        let DiscoveredSet {
            table, states, hasher, ..
        } = self;
        if let Some(&index) = table.find(hash, |&index| states[index] == root) {
            return (StateRef(index), false);
        }

        let index = states.len();
        states.push(root);
        table.insert_unique(hash, index, |&index| hasher.hash_one(states[index]));

        (StateRef(index), true)
    }

    /// Returns the handle of `state` if it is present, or `None` otherwise.
    pub fn index(&self, state: &[T]) -> Option<StateRef> {
        // Resolve the canonical root without mutating the forest; a missing node
        // means the state was never inserted.
        let root = self.forest.find(state, |_| ())?;
        let hash = self.hasher.hash_one(root);
        self.table
            .find(hash, |&index| self.states[index] == root)
            .map(|&index| StateRef(index))
    }

    /// Returns true if `state` is present in the set.
    pub fn contains(&self, state: &[T]) -> bool {
        self.index(state).is_some()
    }
}

impl<T, A> DiscoveredSet<T, A>
where
    T: Copy,
    A: Allocator,
{
    /// Returns the number of distinct states in the set.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Reconstructs the state vector for `reference` into the freshly cleared
    /// `out` buffer. Reusing a buffer avoids an allocation per lookup, which
    /// matters on the hot exploration path. Returns false if the reference is
    /// out of range.
    pub fn get_into(&self, reference: StateRef, out: &mut std::vec::Vec<T>) -> bool {
        out.clear();
        match self.states.get(reference.index()) {
            Some(&tree) => {
                out.extend(self.forest.iter(tree).map(|(_, value)| value));
                true
            }
            None => false,
        }
    }

    /// Returns the state vector for `reference`, allocating a fresh [`Vec`].
    ///
    /// Prefer [`DiscoveredSet::get_into`] on hot paths to reuse a buffer.
    pub fn get(&self, reference: StateRef) -> Option<std::vec::Vec<T>> {
        let &tree = self.states.get(reference.index())?;
        Some(self.forest.iter(tree).map(|(_, value)| value).collect())
    }

    /// Removes all states, invalidating every previously returned
    /// [`StateRef`].
    pub fn clear(&mut self) {
        self.forest.clear();
        self.states.clear();
        self.table.clear();
    }
}

impl<T> Default for DiscoveredSet<T, Global>
where
    T: Copy + Eq + Hash,
{
    fn default() -> DiscoveredSet<T, Global> {
        DiscoveredSet::new()
    }
}
