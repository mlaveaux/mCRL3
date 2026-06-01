//! A discovered set that stores state vectors as maximally shared B-trees.

use std::hash::BuildHasher;
use std::hash::Hash;

use allocator_api2::alloc::Allocator;
use allocator_api2::alloc::Global;
use allocator_api2::vec::Vec;
use cranelift_bforest::Map;
use cranelift_bforest::MapForest;
use hashbrown::HashTable;
use rustc_hash::FxBuildHasher;

/// A stable handle to a state stored in a [`DiscoveredSet`].
///
/// References are dense and assigned in insertion order starting from zero. A
/// 32-bit representation is used to match the `cranelift_bforest` storage,
/// which is optimised for 32-bit keys and values.
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
/// The state vectors are stored as B-trees that share a single node pool; see
/// the [module documentation](self) for the rationale. The auxiliary
/// collections allocate from `A`.
pub struct DiscoveredSet<T = usize, A = Global>
where
    T: Copy,
    A: Allocator,
{
    /// Shared node pool backing every stored state B-tree.
    forest: MapForest<usize, T>,
    /// Stored states indexed by [`StateRef`]; each maps `position -> value`.
    states: Vec<Map<usize, T>, A>,
    /// Precomputed hash for each stored state, kept parallel to `states` so the
    /// hash table can be resized without reconstructing state vectors.
    hashes: Vec<u64, A>,
    /// Hash index from a state's content hash to its raw index into `states`.
    table: HashTable<usize, A>,
    /// Hasher used to fingerprint state vectors.
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
            forest: MapForest::new(),
            states: Vec::new_in(alloc.clone()),
            hashes: Vec::new_in(alloc.clone()),
            table: HashTable::new_in(alloc),
            hasher: FxBuildHasher,
        }
    }

    /// Creates a new empty discovered set with room for at least `capacity`
    /// states before reallocating, whose auxiliary collections allocate from
    /// `alloc`.
    pub fn with_capacity_in(capacity: usize, alloc: A) -> DiscoveredSet<T, A> {
        DiscoveredSet {
            forest: MapForest::new(),
            states: Vec::with_capacity_in(capacity, alloc.clone()),
            hashes: Vec::with_capacity_in(capacity, alloc.clone()),
            table: HashTable::with_capacity_in(capacity, alloc),
            hasher: FxBuildHasher,
        }
    }

    /// Inserts `state` and returns its handle together with a boolean that is
    /// true when the state was newly inserted and false when it was already
    /// present.
    pub fn insert(&mut self, state: &[T]) -> (StateRef, bool) {
        let hash = self.hasher.hash_one(state);

        // Look the state up first; the immutable borrow ends before we mutate
        // the forest and table below on the miss path.
        {
            let DiscoveredSet {
                table, forest, states, ..
            } = &*self;
            if let Some(&index) = table.find(hash, |&index| map_eq(forest, &states[index], state)) {
                return (StateRef(index), false);
            }
        }

        let map = self.build_map(state);
        let index = self.states.len();
        self.states.push(map);
        self.hashes.push(hash);

        let DiscoveredSet { table, hashes, .. } = self;
        table.insert_unique(hash, index, |&index| hashes[index]);

        (StateRef(index), true)
    }

    /// Returns the handle of `state` if it is present, or `None` otherwise.
    pub fn index(&self, state: &[T]) -> Option<StateRef> {
        let hash = self.hasher.hash_one(state);
        let DiscoveredSet {
            table, forest, states, ..
        } = self;
        table
            .find(hash, |&index| map_eq(forest, &states[index], state))
            .map(|&index| StateRef(index))
    }

    /// Returns true if `state` is present in the set.
    pub fn contains(&self, state: &[T]) -> bool {
        self.index(state).is_some()
    }

    /// Builds a B-tree for `state` in the shared forest, keyed by position.
    fn build_map(&mut self, state: &[T]) -> Map<usize, T> {
        let mut map = Map::new();
        for (position, &value) in state.iter().enumerate() {
            map.insert(position, value, &mut self.forest, &());
        }
        map
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
            Some(map) => {
                out.extend(map.iter(&self.forest).map(|(_, value)| value));
                true
            }
            None => false,
        }
    }

    /// Returns the state vector for `reference`, allocating a fresh [`Vec`].
    ///
    /// Prefer [`DiscoveredSet::get_into`] on hot paths to reuse a buffer.
    pub fn get(&self, reference: StateRef) -> Option<std::vec::Vec<T>> {
        let map = self.states.get(reference.index())?;
        Some(map.iter(&self.forest).map(|(_, value)| value).collect())
    }

    /// Removes all states, invalidating every previously returned
    /// [`StateRef`].
    pub fn clear(&mut self) {
        self.forest.clear();
        self.states.clear();
        self.hashes.clear();
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

/// Returns true if the B-tree `map` (stored in `forest`) represents exactly the
/// state vector `state`.
///
/// States are stored with positions `0..state.len()`, so iterating the map
/// yields values in position order and we can compare against `state` directly.
fn map_eq<T: Copy + Eq>(forest: &MapForest<usize, T>, map: &Map<usize, T>, state: &[T]) -> bool {
    let mut matched = 0;
    for (position, value) in map.iter(forest) {
        debug_assert_eq!(position, matched, "states are stored with dense positions");
        if matched >= state.len() || state[matched] != value {
            return false;
        }
        matched += 1;
    }
    matched == state.len()
}
