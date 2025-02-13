use std::fmt;

use crate::Partition;

/// Defines a partition based on an explicit indexing of elements to their block
/// number.
#[derive(Debug)]
pub struct IndexedPartition {
    partition: Vec<usize>,

    num_of_blocks: usize,
}

impl IndexedPartition {
    /// Create a new partition where all elements are in a single block.
    pub fn new(num_of_elements: usize) -> IndexedPartition {
        IndexedPartition {
            partition: vec![0; num_of_elements],
            num_of_blocks: 1,
        }
    }

    /// Create a new partition with the given partitioning.
    pub fn with_partition(partition: Vec<usize>, num_of_blocks: usize) -> IndexedPartition {
        IndexedPartition {
            partition,
            num_of_blocks,
        }
    }

    /// Iterates over the elements in the partition.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.partition.iter().copied()
    }

    /// Sets the block number of the given element
    pub fn set_block(&mut self, element_index: usize, block_number: usize) {
        // TODO: This assumes that the blocks are dense, otherwise it overestimates the number of blocks.
        self.num_of_blocks = self.num_of_blocks.max(block_number + 1);

        self.partition[element_index] = block_number;
    }
}

/// Combines two partitions into a new partition.
pub fn combine_partition(left: IndexedPartition, right: &impl Partition) -> IndexedPartition {
    let mut combined_partition = IndexedPartition::new(left.partition.len());

    for (element_index, block) in left.partition.iter().enumerate() {
        let new_block = right.block_number(*block);

        combined_partition.set_block(element_index, new_block);
    }

    combined_partition
}

/// Reorders the blocks of the given partition according to the given permutation.
pub fn reorder_partition<P>(partition: IndexedPartition, permutation: P) -> IndexedPartition
where
    P: Fn(usize) -> usize,
{
    let mut new_partition = IndexedPartition::new(partition.len());

    for (element_index, block) in partition.iter().enumerate() {
        new_partition.set_block(element_index, permutation(block));
    }

    new_partition
}

impl PartialEq for IndexedPartition {
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
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

                write!(f, "{}", element_index)?;
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

impl Partition for IndexedPartition {
    fn block_number(&self, state_index: usize) -> usize {
        self.partition[state_index]
    }

    fn num_of_blocks(&self) -> usize {
        self.num_of_blocks
    }

    fn len(&self) -> usize {
        self.partition.len()
    }
}
