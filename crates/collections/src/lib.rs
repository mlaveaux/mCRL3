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

pub use block_partition::Block;
pub use block_partition::BlockIter;
pub use block_partition::BlockPartition;
pub use compressed_vec::ByteCompressedVec;
pub use compressed_vec::ByteCompressedVecIterator;
pub use compressed_vec::CompressedEntry;
pub use compressed_vec::CompressedVecMetrics;
pub use graph::Graph;
pub use indexed_partition::BlockIndex;
pub use indexed_partition::BlockTag;
pub use indexed_partition::IndexedPartition;
pub use indexed_set::IndexedSet;
pub use indexed_set::Iter;
pub use indexed_set::SetIndex;
pub use scc_decomposition::scc_decomposition;
pub use scc_decomposition::scc_decomposition_iterative;
pub use vecbag::VecBag;
pub use vecset::VecSet;
