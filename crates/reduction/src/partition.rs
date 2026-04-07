use merc_collections::BlockIndex;
use merc_collections::IndexedPartition;
use merc_lts::StateIndex;

/// A trait for partition refinement algorithms that expose the block number for
/// every state. Can be used to compute the quotient labelled transition system.
///
/// The invariants are that the union of all blocks is the original set, and
/// that each block contains distinct elements
pub trait Partition {
    /// Returns the block number for the given state.
    fn block_number(&self, state_index: StateIndex) -> BlockIndex;

    /// Returns the number of blocks in the partition.
    fn num_of_blocks(&self) -> usize;

    /// Returns the number of elements in the partition.
    fn len(&self) -> usize;

    /// Returns whether the partition is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Partition for IndexedPartition {
    fn block_number(&self, state_index: StateIndex) -> BlockIndex {
        self.block(state_index.value())
    }

    fn num_of_blocks(&self) -> usize {
        self.num_of_blocks()
    }

    fn len(&self) -> usize {
        self.len()
    }
}

/// Combines two partitions into a new partition.
pub fn combine_partition<P: Partition>(left: IndexedPartition, right: &P) -> IndexedPartition {
    let mut combined_partition = left.clone();

    for (element_index, block) in left.iter_elements() {
        let new_block = right.block_number(StateIndex::new(block.value()));

        combined_partition.set_block(element_index, new_block);
    }

    combined_partition
}
