//! A forest of immutable, hash-consed B+-trees, inspired by `cranelift_bforest`
//! crate.
//!
//! The crucial difference is that this implementation uses hash conscing to
//! maximally share (immutable) nodes across trees. This makes it suitable to
//! compactly represent large sets of similar sequences.
//!
//! Each tree is a hash-consed *sequence* of values: positions are implicit in
//! iteration order, so there are no separator keys. The branching factor `N` is
//! a const generic parameter (default 8) so it can be tuned for experiments.

use std::hash::BuildHasher;
use std::hash::Hash;
use std::mem;

use allocator_api2::alloc::Allocator;
use allocator_api2::alloc::Global;
use allocator_api2::vec::Vec;
use hashbrown::HashTable;
use rustc_hash::FxBuildHasher;

/// The default branching factor, matches a standard cache line of words.
const DEFAULT_BRANCHING: usize = 8;

/// Upper bound on the height of any tree.
const MAX_DEPTH: usize = 48;

/// A value that can be stored in a [`BTreeForest`] node slot.
///
/// Nodes are untyped `[V; N]` arrays with neither a length nor a leaf/inner
/// discriminant, so the slot type itself must represent three things: a real
/// leaf value, an interior child reference, and an unused (empty) slot.
///
/// Implementations must keep these disjoint: [`Slot::EMPTY`] must never equal a
/// real value, and a slot produced by [`Slot::from_child`] is only ever decoded
/// with [`Slot::as_child`] when the traversal already knows (from the node's
/// height) that the slot holds a child reference.
pub trait Slot: Copy + Eq + Hash {
    /// Sentinel stored in unused trailing slots, in place of a per-node length.
    /// It must be a value the producer of real values never emits.
    const EMPTY: Self;

    /// Encodes a child node index into a slot value.
    fn from_child(index: usize) -> Self;

    /// Decodes a slot that the caller already knows (from the node's height)
    /// holds a child index.
    fn as_child(self) -> usize;
}

impl Slot for usize {
    const EMPTY: usize = usize::MAX;

    fn from_child(index: usize) -> usize {
        index
    }

    fn as_child(self) -> usize {
        self
    }
}

impl Slot for u32 {
    const EMPTY: u32 = u32::MAX;

    fn from_child(index: usize) -> u32 {
        // A `u32` slot caps the node pool at `u32::MAX`; the `usize` index is
        // guaranteed to fit because the pool cannot have grown beyond that.
        index as u32
    }

    fn as_child(self) -> usize {
        self as usize
    }
}

/// A handle to a single tree stored in a [`BTreeForest`].
///
/// Handles are only meaningful for the forest that produced them and are
/// invalidated by [`BTreeForest::clear`]. The handle carries the root node
/// index together with the tree's height, which iteration uses to tell leaf
/// values from child references without inspecting the nodes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tree {
    /// Index of the root node, or `usize::MAX` for the empty tree.
    root: usize,
    /// Height of the root. Zero when the root is a leaf-level node holding
    /// values.
    height: u8,
}

impl Tree {
    /// The empty tree, holding no entries.
    pub const EMPTY: Tree = Tree {
        root: usize::MAX,
        height: 0,
    };

    /// Returns true if this tree has no entries.
    pub fn is_empty(self) -> bool {
        self.root == usize::MAX
    }
}

impl Default for Tree {
    fn default() -> Tree {
        Tree::EMPTY
    }
}

/// Hashes a node's full `[V; N]` slot array. Empty padding is canonical (always
/// [`Slot::EMPTY`]), so the whole array can be hashed without a live length.
fn hash_node<V: Hash, const N: usize>(hasher: &FxBuildHasher, data: &[V; N]) -> u64 {
    hasher.hash_one(data)
}

/// A forest of hash-consed sequences of `V` with branching factor `N`.
///
/// Values are expected to be small `Copy` types, matching the
/// `cranelift_bforest` design tradeoffs. The auxiliary collections allocate from
/// `A`. The branching factor `N` defaults to 8 and can be overridden to
/// experiment with node sizes; it must be at least two.
///
/// Because a value and a child index could share a bit pattern, interning is
/// partitioned by height: nodes are only ever deduplicated against other nodes
/// at the same height, so `[5, 7]` interpreted as values never collides with
/// `[5, 7]` interpreted as child references.
///
/// Nodes are never freed individually; the forest is append-only and reclaimed
/// all at once with [`BTreeForest::clear`].
pub struct BTreeForest<V, A = Global, const N: usize = DEFAULT_BRANCHING>
where
    V: Copy,
    A: Allocator,
{
    /// The shared node pool.
    nodes: Vec<[V; N], A>,
    /// Interning index from a node's content hash to its index in `nodes`.
    tables: Vec<HashTable<usize, A>, A>,
    /// Scratch buffers reused by [`BTreeForest::build`] to assemble each tree
    /// level without allocating per call. They hold the child node indices of
    /// the level currently being built.
    scratch_lo: Vec<usize, A>,
    scratch_hi: Vec<usize, A>,
    hasher: FxBuildHasher,
    /// Allocator used to create the per-height interning tables on demand.
    alloc: A,
}

impl<V, const N: usize> BTreeForest<V, Global, N>
where
    V: Slot,
{
    /// Creates a new empty forest backed by the global allocator.
    pub fn new() -> BTreeForest<V, Global, N> {
        BTreeForest::new_in(Global)
    }
}

impl<V, A, const N: usize> BTreeForest<V, A, N>
where
    V: Slot,
    A: Allocator + Clone,
{
    /// Creates a new empty forest whose collections allocate from `alloc`.
    pub fn new_in(alloc: A) -> BTreeForest<V, A, N> {
        const { assert!(N >= 2, "branching factor must be at least two") };
        BTreeForest {
            nodes: Vec::new_in(alloc.clone()),
            tables: Vec::new_in(alloc.clone()),
            hasher: FxBuildHasher,
            scratch_lo: Vec::new_in(alloc.clone()),
            scratch_hi: Vec::new_in(alloc.clone()),
            alloc,
        }
    }

    /// Interns the sequence `values` and returns a handle to its tree,
    /// reusing any already-interned nodes. Equal sequences always yield the
    /// same handle. The tree is assembled bottom-up: contiguous chunks of
    /// `values` become leaf-level nodes, then those are grouped into interior
    /// nodes, and so on until a single root remains.
    pub fn insert(&mut self, values: &[V]) -> Tree {
        if values.is_empty() {
            return Tree::EMPTY;
        }

        // Destructure so the scratch buffers can be borrowed independently of
        // the node pool and interning tables that `intern` mutates.
        let BTreeForest {
            nodes,
            tables,
            hasher,
            scratch_lo,
            scratch_hi,
            alloc,
        } = self;

        //  Each chunk of `values` is copied straight into a node, padded with
        // `EMPTY`.
        scratch_lo.clear();
        for chunk in values.chunks(N) {
            let mut slots = [V::EMPTY; N];
            for (offset, &value) in chunk.iter().enumerate() {
                debug_assert!(value != V::EMPTY, "value collides with the empty-slot sentinel");
                slots[offset] = value;
            }
            let node = Self::intern(nodes, tables, hasher, alloc, 0, slots);
            scratch_lo.push(node);
        }

        // Group the previous level into nodes of child indices until one node
        // remains. `src` is the level just built, `dst` the one being built.
        let (mut src, mut dst) = (scratch_lo, scratch_hi);
        let mut height = 0u8;
        while src.len() > 1 {
            height += 1;
            dst.clear();
            for chunk in src.chunks(N) {
                let mut slots = [V::EMPTY; N];
                for (offset, &child) in chunk.iter().enumerate() {
                    slots[offset] = V::from_child(child);
                }
                let node = Self::intern(nodes, tables, hasher, alloc, height as usize, slots);
                dst.push(node);
            }
            mem::swap(&mut src, &mut dst);
        }

        Tree { root: src[0], height }
    }

    /// Interns `data` at `height`, returning the index of the existing identical
    /// node if one is present, or of a freshly allocated node otherwise.
    fn intern(
        nodes: &mut Vec<[V; N], A>,
        tables: &mut Vec<HashTable<usize, A>, A>,
        hasher: &FxBuildHasher,
        alloc: &A,
        height: usize,
        data: [V; N],
    ) -> usize {
        while tables.len() <= height {
            tables.push(HashTable::new_in(alloc.clone()));
        }
        let table = &mut tables[height];

        let hash = hash_node(hasher, &data);
        if let Some(&index) = table.find(hash, |&index| nodes[index] == data) {
            return index;
        }

        let index = nodes.len();
        debug_assert!(index != usize::MAX, "node pool exhausted");
        nodes.push(data);
        table.insert_unique(hash, index, |&index| hash_node(hasher, &nodes[index]));
        index
    }



}

impl<V, A, const N: usize> BTreeForest<V, A, N>
where
    V: Slot,
    A: Allocator,
{
    /// Returns an iterator over the values of `tree` in sequence order.
    pub fn iter(&self, tree: Tree) -> Iter<'_, V, A, N> {
        let mut stack = [(0usize, 0u32, 0u8); MAX_DEPTH];
        let mut depth = 0;
        if !tree.is_empty() {
            stack[0] = (tree.root, 0, tree.height);
            depth = 1;
        }
        Iter {
            nodes: &self.nodes,
            stack,
            depth,
        }
    }
}

impl<V, A, const N: usize> BTreeForest<V, A, N>
where
    V: Copy,
    A: Allocator,
{
    /// Returns the number of distinct nodes currently held by the forest. This
    /// is the deduplicated node count, not the total number of entries across
    /// all trees.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Removes every tree from the forest, invalidating all outstanding
    /// [`Tree`] handles.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.tables.clear();
        self.scratch_lo.clear();
        self.scratch_hi.clear();
    }
}

impl<V, const N: usize> Default for BTreeForest<V, Global, N>
where
    V: Slot,
{
    fn default() -> BTreeForest<V, Global, N> {
        BTreeForest::new()
    }
}

/// In-order iterator over the values of a single tree.
///
/// The descent stack is a fixed-size array, so no heap allocation is performed,
/// which keeps reconstruction cheap on hot paths.
pub struct Iter<'a, V, A, const N: usize>
where
    V: Copy,
    A: Allocator,
{
    nodes: &'a Vec<[V; N], A>,
    /// Frames on the path from the root, each holding the node, the index of the
    /// next slot to visit in it, and the node's height.
    stack: [(usize, u32, u8); MAX_DEPTH],
    depth: usize,
}

impl<V, A, const N: usize> Iterator for Iter<'_, V, A, N>
where
    V: Slot,
    A: Allocator,
{
    type Item = V;

    fn next(&mut self) -> Option<V> {
        while self.depth > 0 {
            let top = self.depth - 1;
            let (node, slot, height) = self.stack[top];
            let slot = slot as usize;
            let slots = &self.nodes[node];

            // A slot past the end or holding the empty sentinel exhausts this
            // node; pop back to its parent.
            if slot >= N || slots[slot] == V::EMPTY {
                self.depth -= 1;
                continue;
            }

            let value = slots[slot];
            self.stack[top].1 += 1;

            if height == 0 {
                // Height zero means the slot holds a leaf value.
                return Some(value);
            }

            // Otherwise it is a child reference; descend into it.
            debug_assert!(self.depth < MAX_DEPTH, "tree deeper than MAX_DEPTH");
            self.stack[self.depth] = (value.as_child(), 0, height - 1);
            self.depth += 1;
        }
        None
    }
}
