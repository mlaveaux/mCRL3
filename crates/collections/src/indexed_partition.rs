#![forbid(unsafe_code)]

use std::fmt;

use merc_utilities::TagIndex;

/// A zero sized tag for the block.
pub struct BlockTag {}

/// The index for blocks.
pub type BlockIndex = TagIndex<usize, BlockTag>;

/// Defines a partition based on an explicit indexing of elements to their block
/// number.
#[derive(Debug)]
pub struct IndexedPartition {
    /// Stores a mapping from element index to block number.
    partition: Vec<BlockIndex>,

    /// Keeps track of the number of blocks defined.
    num_of_blocks: usize,
}

impl IndexedPartition {
    /// Create a new partition where all elements are in a single block.
    pub fn new(num_of_elements: usize) -> IndexedPartition {
        IndexedPartition {
            partition: vec![BlockIndex::new(0); num_of_elements],
            num_of_blocks: 1,
        }
    }

    /// Create a new partition with the given partitioning.
    pub fn with_partition(partition: Vec<BlockIndex>, num_of_blocks: usize) -> IndexedPartition {
        IndexedPartition {
            partition,
            num_of_blocks,
        }
    }

    /// Iterates over the blocks in the partition.
    pub fn iter(&self) -> impl Iterator<Item = BlockIndex> + '_ {
        self.partition.iter().copied()
    }

    /// Sets the block number of the given element
    /// 
    /// # Details
    /// 
    /// This assumes that the blocks are dense, otherwise the partition overestimates the total number of blocks present returned from `len`.
    pub fn set_block(&mut self, element_index: usize, block_number: BlockIndex) {
        self.num_of_blocks = self.num_of_blocks.max(block_number.value() + 1);

        self.partition[element_index] = block_number;
    }

    /// Returns the block number of the given element.
    pub fn block(&self, element_index: usize) -> BlockIndex {
        self.partition[element_index]
    }

    /// Returns the number of blocks in the partition.
    pub fn len(&self) -> usize {
        self.partition.len()
    }

    /// Returns whether the partition is empty.
    pub fn is_empty(&self) -> bool {
        self.partition.is_empty()
    }

    /// Returns the number of blocks in the partition.
    pub fn num_of_blocks(&self) -> usize {
        self.num_of_blocks
    }

    /// Returns the underlying partition vector.
    pub fn partition(&self) -> &Vec<BlockIndex> {
        &self.partition
    }
}

/// Reorders the blocks of the given partition according to the given permutation.
pub fn reorder_partition<P>(partition: IndexedPartition, permutation: P) -> IndexedPartition
where
    P: Fn(BlockIndex) -> BlockIndex,
{
    let mut new_partition = IndexedPartition::new(partition.partition.len());

    for (element_index, block) in partition.iter().enumerate() {
        new_partition.set_block(element_index, permutation(block));
    }

    new_partition
}

impl fmt::Display for IndexedPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ ")?;

        let mut first = true;

        for block_index in 0..self.partition.len() {
            // Print all elements with the same block number.
            let mut first_block = true;
            for (element_index, _) in self.iter().enumerate().filter(|(_, value)| *value == block_index) {
                if !first_block {
                    write!(f, ", ")?;
                } else {
                    if !first {
                        write!(f, ", ")?;
                    }

                    write!(f, "{{")?;
                }

                write!(f, "{element_index}")?;
                first_block = false;
            }

            if !first_block {
                write!(f, "}}")?;
                first = false;
            }
        }

        write!(f, " }}")
    }
}
