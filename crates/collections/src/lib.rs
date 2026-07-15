#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod block_partition;
mod compressed_vec;
mod graph;
mod indexed_partition;
mod indexed_set;
mod scc_decomposition;
mod vec_difference;
mod vecbag;
mod vecset;

pub use block_partition::{Block, BlockIter, BlockPartition};
pub use compressed_vec::{ByteCompressedVec, ByteCompressedVecIterator, CompressedEntry, CompressedVecMetrics};
pub use graph::Graph;
pub use indexed_partition::{BlockIndex, BlockTag, IndexedPartition};
pub use indexed_set::{IndexedSet, Iter, SetIndex};
pub use scc_decomposition::{scc_decomposition, scc_decomposition_iterative};
pub use vecbag::VecBag;
pub use vecset::VecSet;
