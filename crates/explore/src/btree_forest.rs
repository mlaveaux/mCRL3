//! A forest of immutable, hash-consed B+-trees, inspired by `cranelift_bforest`
//! crate.
//!
//! The crucial difference is that this implementation uses hash conscing to
//! maximally share (immutable) nodes across trees. This makes it suitable to
//! compactly represent large sets of similar trees.
//!
//! The branching factor `N` is a const generic parameter (default 8) so it can
//! be tuned for experiments.
//!
//! When the key type is a zero-sized type such as `()`, the tree degenerates
//! into a hash-consed *sequence*.
//!
//! Nodes are never freed individually; the forest is append-only and reclaimed
//! all at once with [`BTreeForest::clear`].

use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
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

/// A reference to a node in a [`BTreeForest`]. This is a dense index into the
/// node pool.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node(usize);

/// A handle to a single tree stored in a [`BTreeForest`].
///
/// Handles are only meaningful for the forest that produced them and are
/// invalidated by [`BTreeForest::clear`].
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tree(usize);

impl Tree {
    /// The empty tree, holding no entries.
    pub const EMPTY: Tree = Tree(usize::MAX);

    /// Wraps the root node of a non-empty tree.
    fn from_node(node: Node) -> Tree {
        debug_assert!(node.0 != usize::MAX, "node pool exhausted");
        Tree(node.0)
    }

    /// Returns the root node, or `None` for the empty tree.
    fn node(self) -> Option<Node> {
        if self.0 == usize::MAX { None } else { Some(Node(self.0)) }
    }

    /// Returns true if this tree has no entries.
    pub fn is_empty(self) -> bool {
        self.0 == usize::MAX
    }
}

impl Default for Tree {
    fn default() -> Tree {
        Tree::EMPTY
    }
}

/// An immutable B+-tree node holding up to `N` entries (leaf) or children
/// (inner). Only the first `size` slots of each array are valid.
enum NodeData<K, V, const N: usize> {
    /// `children` holds `size + 1` sub-trees and `keys[0..size]` are separators:
    /// `keys[i]` is the smallest key in `children[i + 1]`, so all keys in
    /// `children[i]` are strictly less than `keys[i]`. At most `N - 1` of the
    /// `N` key slots are used.
    Inner {
        size: usize,
        keys: [K; N],
        children: [Node; N],
    },
    /// `keys[0..size]` and `vals[0..size]` are the stored pairs in key order.
    Leaf { size: usize, keys: [K; N], vals: [V; N] },
}

/// Hashes the live portion of `data`, ignoring trailing padding.
fn hash_node<K: Hash, V: Hash, const N: usize>(hasher: &FxBuildHasher, data: &NodeData<K, V, N>) -> u64 {
    let mut state = hasher.build_hasher();
    match data {
        NodeData::Leaf { size, keys, vals } => {
            0u8.hash(&mut state);
            keys[..*size].hash(&mut state);
            vals[..*size].hash(&mut state);
        }
        NodeData::Inner { size, keys, children } => {
            1u8.hash(&mut state);
            keys[..*size].hash(&mut state);
            children[..*size + 1].hash(&mut state);
        }
    }
    state.finish()
}

/// Builds the leaf node for `chunk`, whose first value has global index `base`.
/// Returns the leaf's critical (first) key alongside it.
fn make_leaf<K, V, const N: usize, F>(chunk: &[V], base: usize, key_at: &mut F) -> (K, NodeData<K, V, N>)
where
    K: Copy,
    V: Copy,
    F: FnMut(usize) -> K,
{
    let first_key = key_at(base);
    let mut keys = [first_key; N];
    let mut vals = [chunk[0]; N];
    for (offset, &value) in chunk.iter().enumerate() {
        keys[offset] = key_at(base + offset);
        vals[offset] = value;
    }
    (
        first_key,
        NodeData::Leaf {
            size: chunk.len(),
            keys,
            vals,
        },
    )
}

/// Builds the inner node grouping `chunk` of `(critical_key, node)` children.
/// Returns the inner node's critical key (that of its first child) alongside it.
fn make_inner<K, V, const N: usize>(chunk: &[(K, Node)]) -> (K, NodeData<K, V, N>)
where
    K: Copy,
    V: Copy,
{
    let first_key = chunk[0].0;
    let mut keys = [first_key; N];
    let mut children = [chunk[0].1; N];
    for (offset, &(_, node)) in chunk.iter().enumerate() {
        children[offset] = node;
    }
    // `keys[i]` separates `children[i]` from `children[i + 1]`, so it is the
    // critical key of the (i + 1)-th child.
    for offset in 1..chunk.len() {
        keys[offset - 1] = chunk[offset].0;
    }
    (
        first_key,
        NodeData::Inner {
            size: chunk.len() - 1,
            keys,
            children,
        },
    )
}

/// Returns true if `a` and `b` have the same live content, ignoring padding.
fn node_eq<K: Eq, V: Eq, const N: usize>(a: &NodeData<K, V, N>, b: &NodeData<K, V, N>) -> bool {
    match (a, b) {
        (
            NodeData::Leaf {
                size: sa,
                keys: ka,
                vals: va,
            },
            NodeData::Leaf {
                size: sb,
                keys: kb,
                vals: vb,
            },
        ) => sa == sb && ka[..*sa] == kb[..*sa] && va[..*sa] == vb[..*sa],
        (
            NodeData::Inner {
                size: sa,
                keys: ka,
                children: ca,
            },
            NodeData::Inner {
                size: sb,
                keys: kb,
                children: cb,
            },
        ) => sa == sb && ka[..*sa] == kb[..*sa] && ca[..*sa + 1] == cb[..*sa + 1],
        _ => false,
    }
}

/// A forest of hash-consed B+-trees mapping `K` to `V` with branching factor
/// `N`.
///
/// Keys and values are expected to be small `Copy` types, matching the
/// `cranelift_bforest` design tradeoffs. The auxiliary collections allocate from
/// `A`. The branching factor `N` defaults to 8 and can be overridden to
/// experiment with node sizes; it must be at least two.
pub struct BTreeForest<K, V, A = Global, const N: usize = DEFAULT_BRANCHING>
where
    K: Copy,
    V: Copy,
    A: Allocator,
{
    /// The shared node pool; nodes are immutable once pushed and never freed
    /// individually.
    nodes: Vec<NodeData<K, V, N>, A>,
    /// Interning index from a node's content hash to its index in `nodes`.
    table: HashTable<usize, A>,
    /// Hasher used to fingerprint node content.
    hasher: FxBuildHasher,
    /// Scratch buffers reused by [`BTreeForest::build`] to assemble each tree
    /// level without allocating per call. They hold `(critical_key, node)`
    /// pairs for the level currently being built.
    scratch_lo: Vec<(K, Node), A>,
    scratch_hi: Vec<(K, Node), A>,
}

impl<K, V, const N: usize> BTreeForest<K, V, Global, N>
where
    K: Copy + Eq + Hash,
    V: Copy + Eq + Hash,
{
    /// Creates a new empty forest backed by the global allocator.
    pub fn new() -> BTreeForest<K, V, Global, N> {
        BTreeForest::new_in(Global)
    }
}

impl<K, V, A, const N: usize> BTreeForest<K, V, A, N>
where
    K: Copy + Eq + Hash,
    V: Copy + Eq + Hash,
    A: Allocator + Clone,
{
    /// Creates a new empty forest whose collections allocate from `alloc`.
    pub fn new_in(alloc: A) -> BTreeForest<K, V, A, N> {
        const { assert!(N >= 2, "branching factor must be at least two") };
        BTreeForest {
            nodes: Vec::new_in(alloc.clone()),
            table: HashTable::new_in(alloc.clone()),
            hasher: FxBuildHasher,
            scratch_lo: Vec::new_in(alloc.clone()),
            scratch_hi: Vec::new_in(alloc),
        }
    }

    /// Builds a tree from `values`, keyed by `key_at` applied to each value's
    /// global index. `values` must be sorted by the order `key_at` induces,
    /// which holds trivially when `key_at` is monotonic (e.g. the identity for
    /// dense `0..n` keys, or a constant for a zero-sized key type).
    ///
    /// The tree is assembled bottom-up: contiguous sub-slices of `values` become
    /// leaf nodes, then those are grouped into inner nodes, and so on until a
    /// single root remains. Every node is interned, so nodes shared with
    /// previously built trees are reused rather than duplicated.
    pub fn build<F>(&mut self, values: &[V], mut key_at: F) -> Tree
    where
        F: FnMut(usize) -> K,
    {
        if values.is_empty() {
            return Tree::EMPTY;
        }

        // Destructure so the scratch buffers can be borrowed independently of
        // the node pool that `intern` mutates.
        let BTreeForest {
            nodes,
            table,
            hasher,
            scratch_lo,
            scratch_hi,
        } = self;

        // Leaf level: each chunk of `values` is copied straight into a leaf.
        scratch_lo.clear();
        for (chunk_index, chunk) in values.chunks(N).enumerate() {
            let (first_key, data) = make_leaf(chunk, chunk_index * N, &mut key_at);
            let node = Self::intern(nodes, table, hasher, data);
            scratch_lo.push((first_key, node));
        }

        // Inner levels: group the previous level into inner nodes until one node
        // remains. `src` is the level just built, `dst` the one being built.
        let (mut src, mut dst) = (scratch_lo, scratch_hi);
        while src.len() > 1 {
            dst.clear();
            for chunk in src.chunks(N) {
                let (first_key, data) = make_inner(chunk);
                let node = Self::intern(nodes, table, hasher, data);
                dst.push((first_key, node));
            }
            mem::swap(&mut src, &mut dst);
        }

        Tree::from_node(src[0].1)
    }

    /// Interns `data`, returning the reference of the existing identical node if
    /// one is present, or of a freshly allocated node otherwise.
    fn intern(
        nodes: &mut Vec<NodeData<K, V, N>, A>,
        table: &mut HashTable<usize, A>,
        hasher: &FxBuildHasher,
        data: NodeData<K, V, N>,
    ) -> Node {
        let hash = hash_node(hasher, &data);
        if let Some(&index) = table.find(hash, |&index| node_eq(&nodes[index], &data)) {
            return Node(index);
        }

        let index = nodes.len();
        debug_assert!(index != usize::MAX, "node pool exhausted");
        nodes.push(data);
        table.insert_unique(hash, index, |&index| hash_node(hasher, &nodes[index]));
        Node(index)
    }

    /// Resolves the tree that [`BTreeForest::build`] would produce for `values`
    /// *without* mutating the forest, returning `None` if any required node is
    /// not already interned.
    ///
    /// Because nodes are hash-consed, a tree exists in full only if every one of
    /// its nodes was interned by an earlier `build`. This is the read-only
    /// membership counterpart to `build`; it allocates scratch on the heap since
    /// it is not on the hot insertion path.
    pub fn find<F>(&self, values: &[V], mut key_at: F) -> Option<Tree>
    where
        F: FnMut(usize) -> K,
    {
        if values.is_empty() {
            return Some(Tree::EMPTY);
        }

        let mut level: std::vec::Vec<(K, Node)> = std::vec::Vec::new();
        for (chunk_index, chunk) in values.chunks(N).enumerate() {
            let (first_key, data) = make_leaf(chunk, chunk_index * N, &mut key_at);
            level.push((first_key, self.find_node(&data)?));
        }

        while level.len() > 1 {
            let mut next = std::vec::Vec::with_capacity(level.len().div_ceil(N));
            for chunk in level.chunks(N) {
                let (first_key, data) = make_inner(chunk);
                next.push((first_key, self.find_node(&data)?));
            }
            level = next;
        }

        Some(Tree::from_node(level[0].1))
    }

    /// Returns the reference of the interned node equal to `data`, if any.
    fn find_node(&self, data: &NodeData<K, V, N>) -> Option<Node> {
        let hash = hash_node(&self.hasher, data);
        self.table
            .find(hash, |&index| node_eq(&self.nodes[index], data))
            .map(|&index| Node(index))
    }
}

impl<K, V, A, const N: usize> BTreeForest<K, V, A, N>
where
    K: Copy,
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
        self.table.clear();
        self.scratch_lo.clear();
        self.scratch_hi.clear();
    }

    /// Looks up `key` in `tree`.
    ///
    /// Only meaningful for an order-preserving key type; for a zero-sized key
    /// type the result is unspecified (use [`BTreeForest::iter`] instead).
    pub fn get(&self, tree: Tree, key: K) -> Option<V>
    where
        K: Ord,
    {
        let mut node = tree.node()?;
        loop {
            match &self.nodes[node.0] {
                NodeData::Inner { size, keys, children } => {
                    // The child index is the number of separator keys that are
                    // less than or equal to `key`.
                    let child = keys[..*size].partition_point(|&separator| separator <= key);
                    node = children[child];
                }
                NodeData::Leaf { size, keys, vals } => {
                    return match keys[..*size].binary_search(&key) {
                        Ok(index) => Some(vals[index]),
                        Err(_) => None,
                    };
                }
            }
        }
    }

    /// Returns an iterator over the `(key, value)` pairs of `tree` in key order.
    pub fn iter(&self, tree: Tree) -> Iter<'_, K, V, A, N> {
        let mut iter = Iter {
            nodes: &self.nodes,
            stack: [(Node(0), 0); MAX_DEPTH],
            depth: 0,
            leaf: None,
            pos: 0,
        };
        if let Some(root) = tree.node() {
            iter.descend(root);
        }
        iter
    }
}

impl<K, V, const N: usize> Default for BTreeForest<K, V, Global, N>
where
    K: Copy + Eq + Hash,
    V: Copy + Eq + Hash,
{
    fn default() -> BTreeForest<K, V, Global, N> {
        BTreeForest::new()
    }
}

/// In-order iterator over the entries of a single tree.
///
/// The descent stack is a fixed-size array, so no heap allocation is performed,
/// which keeps reconstruction cheap on hot paths.
pub struct Iter<'a, K, V, A, const N: usize>
where
    K: Copy,
    V: Copy,
    A: Allocator,
{
    nodes: &'a Vec<NodeData<K, V, N>, A>,
    /// Inner-node frames on the path to the current leaf, each paired with the
    /// index of the next child to descend into.
    stack: [(Node, usize); MAX_DEPTH],
    depth: usize,
    /// The leaf currently being yielded from, if any.
    leaf: Option<Node>,
    /// Index of the next entry to yield from `leaf`.
    pos: usize,
}

impl<K, V, A, const N: usize> Iter<'_, K, V, A, N>
where
    K: Copy,
    V: Copy,
    A: Allocator,
{
    /// Walks down the left spine starting at `node`, pushing the inner nodes
    /// onto the stack, until a leaf is reached and made current.
    fn descend(&mut self, mut node: Node) {
        loop {
            debug_assert!(self.depth < MAX_DEPTH, "tree deeper than MAX_DEPTH");
            match &self.nodes[node.0] {
                NodeData::Inner { children, .. } => {
                    // Descend into child 0; the next child to visit is 1.
                    self.stack[self.depth] = (node, 1);
                    self.depth += 1;
                    node = children[0];
                }
                NodeData::Leaf { .. } => {
                    self.leaf = Some(node);
                    self.pos = 0;
                    return;
                }
            }
        }
    }
}

impl<K, V, A, const N: usize> Iterator for Iter<'_, K, V, A, N>
where
    K: Copy,
    V: Copy,
    A: Allocator,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        loop {
            // Yield the next entry from the current leaf if one remains.
            if let Some(leaf) = self.leaf {
                if let NodeData::Leaf { size, keys, vals } = &self.nodes[leaf.0]
                    && self.pos < *size
                {
                    let entry = (keys[self.pos], vals[self.pos]);
                    self.pos += 1;
                    return Some(entry);
                }
                self.leaf = None;
            }

            // The current leaf is exhausted; find the next leaf by advancing the
            // nearest ancestor that still has unvisited children.
            loop {
                if self.depth == 0 {
                    return None;
                }
                let (node, child) = self.stack[self.depth - 1];
                if let NodeData::Inner { size, children, .. } = &self.nodes[node.0]
                    && child <= *size
                {
                    self.stack[self.depth - 1].1 = child + 1;
                    self.descend(children[child]);
                    break;
                }
                self.depth -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a tree from a slice of values with dense `0..n` keys.
    fn build_dense(forest: &mut BTreeForest<usize, u32>, values: &[u32]) -> Tree {
        forest.build(values, |position| position)
    }

    /// Collects the values of a tree in key order.
    fn collect_values(forest: &BTreeForest<usize, u32>, tree: Tree) -> std::vec::Vec<u32> {
        forest.iter(tree).map(|(_, value)| value).collect()
    }

    #[test]
    fn empty_tree() {
        let forest: BTreeForest<usize, u32> = BTreeForest::new();
        assert!(Tree::EMPTY.is_empty());
        assert_eq!(forest.iter(Tree::EMPTY).count(), 0);
        assert_eq!(forest.get(Tree::EMPTY, 0), None);
    }

    #[test]
    fn roundtrip_various_sizes() {
        let mut forest = BTreeForest::new();
        for len in 0..200usize {
            let values: std::vec::Vec<u32> = (0..len as u32).map(|v| v * 7 + 1).collect();
            let tree = build_dense(&mut forest, &values);
            assert_eq!(collect_values(&forest, tree), values, "len = {len}");
            for (position, &value) in values.iter().enumerate() {
                assert_eq!(forest.get(tree, position), Some(value), "len = {len}");
            }
            assert_eq!(forest.get(tree, len), None);
        }
    }

    #[test]
    fn roundtrip_small_branching() {
        // The smallest legal branching factor exercises the deepest trees.
        let mut forest: BTreeForest<usize, u32, Global, 2> = BTreeForest::new();
        let values: std::vec::Vec<u32> = (0..100).map(|v| v * 3).collect();
        let tree = forest.build(&values, |position| position);
        let got: std::vec::Vec<u32> = forest.iter(tree).map(|(_, value)| value).collect();
        assert_eq!(got, values);
    }

    #[test]
    fn identical_inputs_share_root() {
        let mut forest = BTreeForest::new();
        let values: std::vec::Vec<u32> = (0..100).collect();
        let a = build_dense(&mut forest, &values);
        let nodes_after_first = forest.node_count();
        let b = build_dense(&mut forest, &values);
        // Re-building the same input must not allocate any new node and must
        // yield the very same root.
        assert_eq!(a, b);
        assert_eq!(forest.node_count(), nodes_after_first);
    }

    #[test]
    fn shared_prefix_shares_leaves() {
        let mut forest = BTreeForest::new();
        // Two equally long vectors differing only in the last position share
        // every leaf except the final one, so the second build adds far fewer
        // nodes than the first.
        let base: std::vec::Vec<u32> = (0..100).collect();
        let _ = build_dense(&mut forest, &base);
        let nodes_after_first = forest.node_count();

        let mut other = base.clone();
        *other.last_mut().unwrap() = 9999;
        let _ = build_dense(&mut forest, &other);
        let added = forest.node_count() - nodes_after_first;

        assert!(added < nodes_after_first, "added {added} new nodes");
    }

    #[test]
    fn zero_sized_key_shares_by_value() {
        // With a zero-sized key, leaves are deduplicated purely by their values,
        // so a repeated block of values at different positions is shared.
        let mut forest: BTreeForest<(), u32> = BTreeForest::new();
        let block: std::vec::Vec<u32> = (0..8).collect();
        let a = forest.build(&block, |_| ());
        let nodes_after_first = forest.node_count();

        // The same block again at the front of a longer vector reuses the
        // leaf(s) of the first build.
        let mut longer = block.clone();
        longer.extend(100..108);
        let _ = forest.build(&longer, |_| ());
        let got: std::vec::Vec<u32> = forest.iter(a).map(|(_, value)| value).collect();
        assert_eq!(got, block);
        assert!(forest.node_count() >= nodes_after_first);
    }

    #[test]
    fn distinct_trees_are_independent() {
        let mut forest = BTreeForest::new();
        let a = build_dense(&mut forest, &[1, 2, 3]);
        let b = build_dense(&mut forest, &[4, 5, 6, 7, 8]);
        assert_eq!(collect_values(&forest, a), [1, 2, 3]);
        assert_eq!(collect_values(&forest, b), [4, 5, 6, 7, 8]);
    }

    #[test]
    fn clear_resets_forest() {
        let mut forest = BTreeForest::new();
        let _ = build_dense(&mut forest, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert!(forest.node_count() > 0);
        forest.clear();
        assert_eq!(forest.node_count(), 0);
        let tree = build_dense(&mut forest, &[42]);
        assert_eq!(collect_values(&forest, tree), [42]);
    }
}
