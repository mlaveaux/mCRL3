//! A discovered set that stores state vectors as maximally shared B-trees.
//!
//! During explicit-state exploration every reachable state must be stored once
//! and recognised again when it is rediscovered. [`DiscoveredSet`] keeps each
//! state vector as a [`cranelift_bforest::Map`] mapping a position to its value.
//! All of these per-state B-trees draw their nodes from a *single* shared
//! [`cranelift_bforest::MapForest`] arena, so the nodes are pooled (maximally
//! shared) across every discovered state instead of every state owning its own
//! heap allocation. A [`hashbrown::HashTable`] over the state contents provides
//! the deduplication that maps equal state vectors onto the same [`StateRef`].

use std::hash::BuildHasher;

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
pub struct StateRef(u32);

impl StateRef {
    /// Returns the underlying index as a `usize`.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// A set of `u32` state vectors that deduplicates equal vectors and assigns
/// each a stable [`StateRef`].
///
/// The state vectors are stored as B-trees that share a single node pool; see
/// the [module documentation](self) for the rationale.
pub struct DiscoveredSet {
    /// Shared node pool backing every stored state B-tree.
    forest: MapForest<u32, u32>,
    /// Stored states indexed by [`StateRef`]; each maps `position -> value`.
    states: Vec<Map<u32, u32>>,
    /// Precomputed hash for each stored state, kept parallel to `states` so the
    /// hash table can be resized without reconstructing state vectors.
    hashes: Vec<u64>,
    /// Hash index from a state's content hash to its raw index into `states`.
    table: HashTable<usize>,
    /// Hasher used to fingerprint state vectors.
    hasher: FxBuildHasher,
}

impl DiscoveredSet {
    /// Creates a new empty discovered set.
    pub fn new() -> DiscoveredSet {
        DiscoveredSet {
            forest: MapForest::new(),
            states: Vec::new(),
            hashes: Vec::new(),
            table: HashTable::new(),
            hasher: FxBuildHasher,
        }
    }

    /// Creates a new empty discovered set with room for at least `capacity`
    /// states before reallocating.
    pub fn with_capacity(capacity: usize) -> DiscoveredSet {
        DiscoveredSet {
            forest: MapForest::new(),
            states: Vec::with_capacity(capacity),
            hashes: Vec::with_capacity(capacity),
            table: HashTable::with_capacity(capacity),
            hasher: FxBuildHasher,
        }
    }

    /// Returns the number of distinct states in the set.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Inserts `state` and returns its handle together with a boolean that is
    /// true when the state was newly inserted and false when it was already
    /// present.
    pub fn insert(&mut self, state: &[u32]) -> (StateRef, bool) {
        let hash = self.hasher.hash_one(state);

        // Look the state up first; the immutable borrow ends before we mutate
        // the forest and table below on the miss path.
        {
            let DiscoveredSet { table, forest, states, .. } = &*self;
            if let Some(&index) = table.find(hash, |&index| map_eq(forest, &states[index], state)) {
                return (StateRef(index as u32), false);
            }
        }

        let map = self.build_map(state);
        let index = self.states.len();
        self.states.push(map);
        self.hashes.push(hash);

        let DiscoveredSet { table, hashes, .. } = self;
        table.insert_unique(hash, index, |&index| hashes[index]);

        (StateRef(index as u32), true)
    }

    /// Returns the handle of `state` if it is present, or `None` otherwise.
    pub fn index(&self, state: &[u32]) -> Option<StateRef> {
        let hash = self.hasher.hash_one(state);
        let DiscoveredSet { table, forest, states, .. } = self;
        table
            .find(hash, |&index| map_eq(forest, &states[index], state))
            .map(|&index| StateRef(index as u32))
    }

    /// Returns true if `state` is present in the set.
    pub fn contains(&self, state: &[u32]) -> bool {
        self.index(state).is_some()
    }

    /// Reconstructs the state vector for `reference` into the freshly cleared
    /// `out` buffer. Reusing a buffer avoids an allocation per lookup, which
    /// matters on the hot exploration path. Returns false if the reference is
    /// out of range.
    pub fn get_into(&self, reference: StateRef, out: &mut Vec<u32>) -> bool {
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
    pub fn get(&self, reference: StateRef) -> Option<Vec<u32>> {
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

    /// Builds a B-tree for `state` in the shared forest, keyed by position.
    fn build_map(&mut self, state: &[u32]) -> Map<u32, u32> {
        let mut map = Map::new();
        for (position, &value) in state.iter().enumerate() {
            map.insert(position as u32, value, &mut self.forest, &());
        }
        map
    }
}

impl Default for DiscoveredSet {
    fn default() -> DiscoveredSet {
        DiscoveredSet::new()
    }
}

/// Returns true if the B-tree `map` (stored in `forest`) represents exactly the
/// state vector `state`.
///
/// States are stored with positions `0..state.len()`, so iterating the map
/// yields values in position order and we can compare against `state` directly.
fn map_eq(forest: &MapForest<u32, u32>, map: &Map<u32, u32>, state: &[u32]) -> bool {
    let mut matched = 0;
    for (position, value) in map.iter(forest) {
        debug_assert_eq!(position as usize, matched, "states are stored with dense positions");
        if matched >= state.len() || state[matched] != value {
            return false;
        }
        matched += 1;
    }
    matched == state.len()
}
