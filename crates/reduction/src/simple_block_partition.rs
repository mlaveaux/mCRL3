#![forbid(unsafe_code)]

use std::fmt;

use merc_collections::{Block, BlockIndex, BlockIter, BlockPartition};
use merc_lts::StateIndex;

use crate::Partition;

/// A partition that explicitly stores a list of blocks and their indexing into
/// the list of elements. Similar to [super::BlockPartition] but without taking
/// the stability of individual elements into account.
#[derive(Debug)]
pub struct MarkedBlockPartition {
    partition: BlockPartition<bool>,
}

impl MarkedBlockPartition {
    /// Create an initial partition where all the states are in a single block
    /// 0. And all the elements in the block are marked.
    pub fn new(num_of_elements: usize) -> Self {
        Self {
            partition: BlockPartition::new(num_of_elements),
        }
    }

    /// Marks the given block as stable
    pub fn mark_block_stable(&mut self, block_index: BlockIndex) {
        *self.partition.block_annotation(block_index) = true;
    }

    /// Return a reference to the given block.
    pub fn block(&self, block_index: BlockIndex) -> &Block<bool> {
        &self.partition.block(block_index)
    }

    /// Splits a block into two blocks according to the given predicate. If the
    /// predicate holds for all or none of the elements, no split occurs.
    pub fn split_block(
        &mut self,
        block_index: BlockIndex,
        predicate: impl Fn(StateIndex) -> bool,
    ) -> Option<BlockIndex> {
        *self.partition.block_annotation(block_index) = false;
        self.partition.split_block(block_index, |element| predicate(StateIndex::new(element)))
    }

    /// Returns the number of blocks in the partition.
    pub fn num_of_blocks(&self) -> usize {
        self.partition.num_of_blocks()
    }

    /// Returns an iterator over the elements of a given block.
    pub fn iter_block(&self, block_index: BlockIndex) -> BlockIter<'_> {
        self.partition.iter_block(block_index)
    }

    /// Returns an iterator over all blocks in the partition.
    pub fn iter(&self) -> impl Iterator<Item = &Block<bool>> {
        self.partition.iter()
    }

    /// Returns an iterator over all blocks in the partition.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Block<bool>> {
        self.partition.iter_mut()
    }
}

impl fmt::Display for MarkedBlockPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.partition)
    }
}

impl Partition for MarkedBlockPartition {
    fn block_number(&self, state_index: StateIndex) -> BlockIndex {
        if !cfg!(debug_assertions) {
            panic!("block_number is only available in debug mode");
        }
        
        // Note that this is O(n) in the number of blocks. This could be improved
        // by storing a mapping from state index to block index. However, this
        // is only used in the comparison functions, so it is not a big issue.
        for block_index in 0..self.partition.num_of_blocks() {
            for element in self.partition.iter_block(BlockIndex::new(block_index)) {
                if element == *state_index {
                    return BlockIndex::new(block_index);
                }
            }
        }

        panic!("State index {:?} not found in partition {:?}", state_index, self);
    }

    fn num_of_blocks(&self) -> usize {
        self.partition.num_of_blocks()
    }

    fn len(&self) -> usize {
        self.partition.len()
    }
}

/// A [super::Block] that stores a subset of the elements in a partition, but
/// with individual stable elements.
///
/// # Details
///
/// It uses `start` and `end` to indicate a range start..end of elements in the
/// partition. The stable flag indicates whether the block is stable.
#[derive(Clone, Copy, Debug)]
pub struct SimpleBlock {
    begin: usize,
    end: usize,
    stable: bool,
}

impl SimpleBlock {
    /// Creates a new block that is not marked.
    pub fn new(begin: usize, end: usize) -> SimpleBlock {
        debug_assert!(begin < end, "The range of this block is incorrect");

        SimpleBlock {
            begin,
            end,
            stable: false,
        }
    }

    /// Returns an iterator over the elements in this block.
    pub fn iter<'a>(&self, elements: &'a Vec<StateIndex>) -> SimpleBlockIter<'a> {
        SimpleBlockIter {
            elements,
            index: self.begin,
            end: self.end,
        }
    }

    /// Returns the number of elements in the block.
    pub fn len(&self) -> usize {
        self.assert_consistent();

        self.end - self.begin
    }

    /// Returns true iff the block is empty.
    pub fn is_empty(&self) -> bool {
        self.assert_consistent();

        self.begin == self.end
    }

    /// Returns true iff the block is stable.
    pub fn is_stable(&self) -> bool {
        self.stable
    }

    /// Marks the block as stable.
    pub fn mark_stable(&mut self) {
        self.stable = true
    }

    /// Returns true iff the block is consistent.
    fn assert_consistent(self) {
        debug_assert!(self.begin < self.end, "The range of block {self:?} is incorrect");
    }
}

pub struct SimpleBlockIter<'a> {
    elements: &'a Vec<StateIndex>,
    index: usize,
    end: usize,
}

impl Iterator for SimpleBlockIter<'_> {
    type Item = StateIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.end {
            let element = self.elements[self.index];
            self.index += 1;
            Some(element)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_block_partition() {
        let mut partition = MarkedBlockPartition::new(10);

        assert_eq!(partition.num_of_blocks(), 1);

        let initial_block = BlockIndex::new(0);
        assert_eq!(partition.block(initial_block).len(), 10);

        let block_index = partition
            .split_block(BlockIndex::new(0), |state| *state < *StateIndex::new(5))
            .unwrap();

        assert_eq!(partition.num_of_blocks(), 2);
        assert_eq!(partition.block(initial_block).len(), 5);
        assert_eq!(partition.block(block_index).len(), 5);
    }
}
