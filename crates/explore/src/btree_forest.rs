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
//!
//! The forest is thread-safe: nodes can be interned concurrently from multiple
//! threads through `&self`.

use std::hash::Hash;
use std::mem;

use merc_unsafety::ShardedHashMap;

/// The default branching factor, matches a standard cache line of words.
const DEFAULT_BRANCHING: usize = 8;

/// Shards per per-height interning table.
const FOREST_SHARDS: usize = 16;

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

/// Number of bits reserved for the height in the packed `Tree` field.
/// Six bits suffice because `MAX_DEPTH` (48) is less than 64.
const HEIGHT_BITS: u32 = 6;
const HEIGHT_MASK: u64 = (1 << HEIGHT_BITS) - 1;

/// All-ones in the 58-bit root field; used as the empty-tree sentinel.
const ROOT_EMPTY: u64 = (1u64 << (64 - HEIGHT_BITS)) - 1;

/// A handle to a single tree stored in a [`BTreeForest`].
///
/// Handles are only meaningful for the forest that produced them and are
/// invalidated by [`BTreeForest::clear`]. The handle packs the root node
/// index (58 bits, upper) and the tree's height (6 bits, lower) into a
/// single `u64`, halving the struct size versus separate fields.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tree {
    /// Bits [63:6] hold the root node index; bits [5:0] hold the height.
    /// When bits [63:6] equal `ROOT_EMPTY` the tree is empty.
    packed: u64,
}

impl Tree {
    /// The empty tree, holding no entries.
    pub const EMPTY: Tree = Tree {
        packed: ROOT_EMPTY << HEIGHT_BITS,
    };

    /// Creates a new tree with the given root and height.
    fn new(root: usize, height: u8) -> Tree {
        debug_assert!((root as u64) < ROOT_EMPTY, "node pool exhausted");
        Tree {
            packed: ((root as u64) << HEIGHT_BITS) | (height as u64),
        }
    }

    /// Returns the root node index and height of this tree.
    fn root(self) -> usize {
        (self.packed >> HEIGHT_BITS) as usize
    }

    /// Returns the height of this tree.
    fn height(self) -> u8 {
        (self.packed & HEIGHT_MASK) as u8
    }

    /// Returns true if this tree has no entries.
    pub fn is_empty(self) -> bool {
        self.packed >> HEIGHT_BITS == ROOT_EMPTY
    }
}

impl Default for Tree {
    fn default() -> Tree {
        Tree::EMPTY
    }
}

/// A forest of hash-consed sequences of `V` with branching factor `N`.
///
/// Values are expected to be small `Copy` types, matching the
/// `cranelift_bforest` design tradeoffs. The branching factor `N` defaults to 8
/// and can be overridden to experiment with node sizes; it must be at least two.
///
/// Because a value and a child index could share a bit pattern, interning is
/// partitioned by height: nodes are only ever deduplicated against other nodes
/// at the same height, so `[5, 7]` interpreted as values never collides with
/// `[5, 7]` interpreted as child references.
///
/// Nodes are never freed individually; the forest is append-only and reclaimed
/// all at once with [`BTreeForest::clear`]. Interning happens through `&self`,
/// so the forest can be shared between threads and populated concurrently.
pub struct BTreeForest<V, const N: usize = DEFAULT_BRANCHING>
where
    V: Copy,
{
    /// The shared node pool. The append-only vector hands out the dense node
    /// index itself, so a node's slot index is its interned identity.
    nodes: boxcar::Vec<[V; N]>,
    /// One interning index per height, mapping a node's content to its index in
    /// `nodes`. Avoids duplicating the `nodes` information by passing the hash
    /// and equality functions as closures.
    tables: Vec<ShardedHashMap<usize>>,
}

/// Reusable scratch buffers for [`BTreeForest::insert_with`], so repeated
/// inserts on a hot path do not reallocate.
///
/// The buffers only hold node indices, so a single context works for any
/// [`BTreeForest`] regardless of its value type or branching factor.
#[derive(Debug, Default, Clone)]
pub struct BTreeForestContext {
    /// Child indices of the level just built.
    src: Vec<usize>,
    /// Child indices of the level currently being built.
    dst: Vec<usize>,
}

impl BTreeForestContext {
    /// Creates an empty context.
    pub fn new() -> BTreeForestContext {
        BTreeForestContext::default()
    }
}

impl<V, const N: usize> BTreeForest<V, N>
where
    V: Slot,
{
    /// Creates a new empty forest.
    pub fn new() -> BTreeForest<V, N> {
        const { assert!(N >= 2, "branching factor must be at least two") };
        // One interning table per representable height. The height field is
        // `HEIGHT_BITS` wide, so heights range over `0..(1 << HEIGHT_BITS)`.
        let tables = (0..(1usize << HEIGHT_BITS))
            .map(|_| ShardedHashMap::with_shards(FOREST_SHARDS))
            .collect();
        BTreeForest {
            nodes: boxcar::Vec::new(),
            tables,
        }
    }

    /// Interns the sequence `values` and returns a handle to its tree, using a
    /// throwaway [`BTreeForestContext`]. Prefer [`BTreeForest::insert_with`] on
    /// hot paths to reuse the scratch buffers.
    pub fn insert(&self, values: &[V]) -> Tree {
        self.insert_with(values, &mut BTreeForestContext::new())
    }

    /// Interns the sequence `values` and returns a handle to its tree, reusing
    /// any already-interned nodes. Equal sequences always yield the same
    /// handle. 
    /// 
    /// # Details
    /// 
    /// The tree is assembled bottom-up: contiguous chunks of `values` become
    /// leaf-level nodes, then those are grouped into interior nodes, and so on
    /// until a single root remains.
    pub fn insert_with(&self, values: &[V], context: &mut BTreeForestContext) -> Tree {
        if values.is_empty() {
            return Tree::EMPTY;
        }

        // `src` is the level just built, `dst` the one being built. The buffers
        // live in `context` so repeated inserts do not reallocate, but they are
        // not shared between threads.
        let BTreeForestContext { src, dst } = context;

        // Each chunk of `values` is copied straight into a node, padded with
        // `EMPTY`.
        src.clear();
        for chunk in values.chunks(N) {
            let mut slots = [V::EMPTY; N];
            for (offset, &value) in chunk.iter().enumerate() {
                debug_assert!(value != V::EMPTY, "value collides with the empty-slot sentinel");
                slots[offset] = value;
            }
            src.push(self.intern(0, slots));
        }

        // Group the previous level into nodes of child indices until one node
        // remains.
        let mut height = 0u8;
        while src.len() > 1 {
            height += 1;
            dst.clear();
            for chunk in src.chunks(N) {
                let mut slots = [V::EMPTY; N];
                for (offset, &child) in chunk.iter().enumerate() {
                    slots[offset] = V::from_child(child);
                }
                dst.push(self.intern(height, slots));
            }
            mem::swap(src, dst);
        }

        Tree::new(src[0], height)
    }

    /// Interns `data` at `height`, returning the index of the existing identical
    /// node if one is present, or of a freshly allocated node otherwise.
    fn intern(&self, height: u8, data: [V; N]) -> usize {
        // The table is specific to this height, so entries are compared and
        // hashed purely by their node content.
        let table = &self.tables[height as usize];
        let hash = table.hash(&data);
        let eq = |&index: &usize| self.node(index) == data;

        // Fast path: an existing node is found while only read-locking the shard.
        if let Some(index) = table.find(hash, eq) {
            return index;
        }

        // `push` appends the node and returns its dense index. The slot is
        // published before `push` returns, so any thread that later reads the
        // index from `tables` can resolve it in `nodes`. `make` runs only on a
        // miss and while the shard is write-locked, so the index is unique.
        table
            .find_or_insert_with(
                hash,
                eq,
                |&index| table.hash(&self.node(index)),
                || {
                    let index = self.nodes.push(data);
                    debug_assert!((index as u64) < ROOT_EMPTY, "node pool exhausted");
                    index
                },
            )
            .0
    }

    /// Returns a copy of the interned node at `index`, which must exist.
    fn node(&self, index: usize) -> [V; N] {
        *self.nodes.get(index).expect("interned node must exist")
    }
}

/// Returns the maximum useful tree depth for a branching factor of `n`.
///
/// A tree of height `h` can hold at most `n^(h+1)` leaf values, so once
/// `h+1 > usize::BITS / floor(log2(n))` the tree would need more nodes than
/// a `usize` can index. This ceiling is therefore a tight upper bound on any
/// reachable depth.
pub const fn max_depth(n: usize) -> usize {
    // floor(log2(n))
    let log2_n = (usize::BITS - n.leading_zeros() - 1) as usize;
    // ceil(usize::BITS / log2_n)
    (usize::BITS as usize).div_ceil(log2_n)
}

/// Upper bound on the height of any tree.
const MAX_DEPTH: usize = max_depth(2);

impl<V, const N: usize> BTreeForest<V, N>
where
    V: Slot,
{
    /// Returns an iterator over the values of `tree` in sequence order.
    pub fn iter(&self, tree: Tree) -> Iter<'_, V, N> {
        let mut stack = [([V::EMPTY; N], 0u32, 0u8); MAX_DEPTH];
        let mut depth = 0;
        if !tree.is_empty() {
            let root = *self.nodes.get(tree.root()).expect("root node must exist");
            stack[0] = (root, 0, tree.height());
            depth = 1;
        }
        Iter {
            nodes: &self.nodes,
            stack,
            depth,
        }
    }
}

impl<V, const N: usize> BTreeForest<V, N>
where
    V: Slot,
{
    /// Returns the number of distinct nodes currently held by the forest. This
    /// is the deduplicated node count, not the total number of entries across
    /// all trees.
    pub fn node_count(&self) -> usize {
        self.nodes.count()
    }

    /// Removes every tree from the forest, invalidating all outstanding
    /// [`Tree`] handles.
    pub fn clear(&mut self) {
        self.nodes.clear();
        for table in &self.tables {
            table.clear();
        }
    }
}

impl<V, const N: usize> Default for BTreeForest<V, N>
where
    V: Slot,
{
    fn default() -> BTreeForest<V, N> {
        BTreeForest::new()
    }
}

/// In-order iterator over the values of a single tree.
///
/// The descent stack is a fixed-size array, so no heap allocation is performed,
/// which keeps reconstruction cheap on hot paths. Each frame caches a copy of
/// its node's slot array, so a node is fetched from the shared pool at most once
/// per descent rather than on every value.
pub struct Iter<'a, V, const N: usize>
where
    V: Copy,
{
    nodes: &'a boxcar::Vec<[V; N]>,
    /// Frames on the path from the root, each holding the node's slot array, the
    /// index of the next slot to visit in it, and the node's height.
    stack: [([V; N], u32, u8); MAX_DEPTH],
    depth: usize,
}

impl<V, const N: usize> Iterator for Iter<'_, V, N>
where
    V: Slot,
{
    type Item = V;

    fn next(&mut self) -> Option<V> {
        while self.depth > 0 {
            let top = self.depth - 1;
            let slot = self.stack[top].1 as usize;
            let height = self.stack[top].2;

            // A slot past the end or holding the empty sentinel exhausts this
            // node; pop back to its parent.
            if slot >= N || self.stack[top].0[slot] == V::EMPTY {
                self.depth -= 1;
                continue;
            }

            let value = self.stack[top].0[slot];
            self.stack[top].1 += 1;

            if height == 0 {
                // Height zero means the slot holds a leaf value.
                return Some(value);
            }

            // Otherwise it is a child reference; descend into it.
            assert!(self.depth < max_depth(N), "tree deeper than max_depth({N})");
            let child = *self.nodes.get(value.as_child()).expect("child node must exist");
            self.stack[self.depth] = (child, 0, height - 1);
            self.depth += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::thread;

    use super::BTreeForest;
    use super::BTreeForestContext;
    use super::Tree;

    #[test]
    fn test_insert_with_reuses_context() {
        let forest: BTreeForest<usize, 2> = BTreeForest::new();
        let mut context = BTreeForestContext::new();

        // Reusing one context across sequences of varying length (so the scratch
        // buffers are cleared and regrown) must yield the same handles as the
        // allocating path and reconstruct correctly.
        let sequences: [&[usize]; 4] = [&[1, 2, 3, 4, 5], &[9], &[], &[7, 7, 7, 7, 7, 7, 7]];
        for seq in sequences {
            let tree = forest.insert_with(seq, &mut context);
            assert_eq!(forest.insert(seq), tree, "context path matches the allocating path");
            let got: Vec<usize> = forest.iter(tree).collect();
            assert_eq!(got, seq.to_vec());
        }
    }

    #[test]
    fn test_insert_and_iter() {
        let forest: BTreeForest<usize, 2> = BTreeForest::new();

        let seq = [1usize, 2, 3, 4, 5];
        let tree = forest.insert(&seq);
        let got: Vec<usize> = forest.iter(tree).collect();
        assert_eq!(got, seq.to_vec());

        // The empty sequence maps to the empty tree.
        assert!(forest.insert(&[]).is_empty());

        // Equal sequences are interned to the same handle and share nodes.
        let nodes = forest.node_count();
        assert_eq!(forest.insert(&seq), tree);
        assert_eq!(forest.node_count(), nodes, "re-interning must not allocate nodes");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_concurrent_intern() {
        let forest: Arc<BTreeForest<usize, 4>> = Arc::new(BTreeForest::new());
        let num_threads = 8;

        let mut handles = Vec::new();
        for _ in 0..num_threads {
            let forest = Arc::clone(&forest);
            handles.push(thread::spawn(move || {
                let mut interned = Vec::new();
                for i in 0..200usize {
                    let seq: Vec<usize> = (0..(i % 17 + 1)).map(|j| j + i).collect();
                    let tree = forest.insert(&seq);
                    // The tree reconstructs to exactly what was inserted, even
                    // while other threads intern concurrently.
                    let got: Vec<usize> = forest.iter(tree).collect();
                    assert_eq!(got, seq);
                    interned.push((seq, tree));
                }
                interned
            }));
        }

        // Equal sequences must intern to equal handles regardless of which
        // thread interned them.
        let mut seen: HashMap<Vec<usize>, Tree> = HashMap::new();
        for handle in handles {
            for (seq, tree) in handle.join().unwrap() {
                match seen.get(&seq) {
                    Some(&prev) => assert_eq!(prev, tree, "equal sequences interned to different trees"),
                    None => {
                        seen.insert(seq, tree);
                    }
                }
            }
        }
    }
}
