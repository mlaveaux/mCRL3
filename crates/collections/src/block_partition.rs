#![forbid(unsafe_code)]

use std::fmt;

use itertools::Itertools;
use log::trace;

use crate::{BlockIndex, IndexedPartition};

/// A trait for blocks in a [BlockPartition].
pub trait Block: Clone {
    /// Creates a new block.
    fn new(start: usize, end: usize) -> Self;

    /// Returns the start index of the block.
    fn start(&self) -> usize;
    fn set_start(&mut self, start: usize);

    /// Returns the end index (exclusive) of the block.
    fn end(&self) -> usize;
    fn set_end(&mut self, end: usize);

    /// Returns the number of elements in the block.
    fn len(&self) -> usize {
        self.assert_consistent();
        self.end() - self.start()
    }

    /// Returns true iff the block is empty.
    fn is_empty(&self) -> bool {
        self.assert_consistent();
        self.start() == self.end()
    }

    /// Returns an iterator over the elements in this block.
    fn iter<'a>(&self, elements: &'a Vec<usize>) -> impl Iterator<Item = usize> + 'a;

    /// Returns true iff the block is consistent.
    fn assert_consistent(&self);
}

/// A partition that explicitly stores a list of blocks and their indexing into
/// the list of elements.
#[derive(Debug)]
pub struct BlockPartition<B: Block> {
    elements: Vec<usize>,
    blocks: Vec<B>,
}

impl<B: Block> BlockPartition<B> {
    /// Create an initial partition where all the states are in a single block
    /// 0. And all the elements in the block are marked.
    pub fn new(num_of_elements: usize) -> Self {
        debug_assert!(num_of_elements > 0, "Cannot partition the empty set");

        let blocks = vec![B::new(0, num_of_elements)];
        let elements = (0..num_of_elements).collect();

        Self { elements, blocks }
    }

    /// Create a block partition from an indexed partition.
    pub fn from_indexed_partition(partition: &IndexedPartition) -> Self {
        let mut blocks = vec![B::new(0, 0); partition.num_of_blocks()];

        // Figure out the number of elements per block.
        for element in partition.iter() {
            let end = blocks[element].end();
            blocks[element].set_end(end + 1);
        }

        // Compute the start index for each block.
        let mut start = 0;
        for block in &mut blocks {
            let end = block.end();
            block.set_start(start);
            block.set_end(start); // This will be updated when adding elements.
            start = end;
        }

        // Create the elements vector.
        let mut elements = vec![0; partition.len()];
        for (element_index, block_index) in partition.iter().enumerate() {
            // Add the element to the block, and update the end index.
            let block = &mut blocks[block_index];
            let pos = block.end();
            elements[pos] = element_index;
            block.set_end(pos + 1);
        }

        Self { elements, blocks }
    }

    /// Return a reference to the given block.
    pub fn block(&self, block_index: BlockIndex) -> &B {
        &self.blocks[block_index]
    }

    /// Splits a block into two blocks according to the given predicate. If the
    /// predicate holds for all or none of the elements, no split occurs.
    pub fn split_block(&mut self, block_index: BlockIndex, predicate: impl Fn(usize) -> bool) -> Option<BlockIndex> {
        // Size of the new block.
        let mut size = 0usize;

        for state in self.blocks[block_index].start()..self.blocks[block_index].end() {
            if predicate(self.elements[state]) {
                self.elements.swap(self.blocks[block_index].start() + size, state);
                size += 1;
            }
        }

        // The original block are now the first [begin, begin + size) elements
        if size == 0 || size == self.blocks[block_index].len() {
            // No split occurred
            return None;
        }

        // Create a new block for the remaining elements
        let new_block = B::new(self.blocks[block_index].start() + size, self.blocks[block_index].end());
        let last_block = self.blocks.len();
        self.blocks.push(new_block);

        // Update the original block
        let new_end = self.blocks[block_index].start() + size;
        self.blocks[block_index].set_end(new_end);

        trace!(
            "Split block {:?} into blocks {:?} and {:?}",
            block_index,
            block_index,
            BlockIndex::new(last_block)
        );
        Some(BlockIndex::new(last_block))
    }

    /// Returns the number of blocks in the partition.
    pub fn num_of_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// Returns an iterator over the elements of a given block.
    pub fn iter_block(&self, block_index: BlockIndex) -> SimpleBlockIter<'_> {
        SimpleBlockIter {
            elements: &self.elements,
            index: self.blocks[block_index].start(),
            end: self.blocks[block_index].end(),
        }
    }

    /// Returns an iterator over all blocks in the partition.
    pub fn iter(&self) -> impl Iterator<Item = &B> {
        self.blocks.iter()
    }

    /// Returns an iterator over all blocks in the partition.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut B> {
        self.blocks.iter_mut()
    }
}

impl<B: Block> fmt::Display for BlockPartition<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format = self
            .blocks
            .iter()
            .map(|block| format!("{{{}}}", block.iter(&self.elements).format(", ")))
            .format(", ");

        write!(f, "{{{}}}", format)
    }
}
/// A [super::Block] that stores a subset of the elements in a partition.
///
/// # Details
///
/// It uses `start` and `end` to indicate a range start..end of elements in the
/// partition.
#[derive(Clone, Copy, Debug)]
pub struct SimpleBlock {
    begin: usize,
    end: usize,
}

impl Block for SimpleBlock {
    fn new(begin: usize, end: usize) -> SimpleBlock {
        debug_assert!(begin < end, "The range of this block is incorrect");

        SimpleBlock { begin, end }
    }

    fn start(&self) -> usize {
        self.begin
    }

    fn set_start(&mut self, start: usize) {
        self.begin = start;
    }

    fn end(&self) -> usize {
        self.end
    }

    fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    fn iter<'a>(&self, elements: &'a Vec<usize>) -> impl Iterator<Item = usize> + 'a {
        SimpleBlockIter {
            elements,
            index: self.begin,
            end: self.end,
        }
    }

    fn assert_consistent(&self) {
        debug_assert!(self.begin < self.end, "The range of block {self:?} is incorrect");
    }
}

pub struct SimpleBlockIter<'a> {
    elements: &'a Vec<usize>,
    index: usize,
    end: usize,
}

impl Iterator for SimpleBlockIter<'_> {
    type Item = usize;

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
        let mut partition: BlockPartition<SimpleBlock> = BlockPartition::new(10);

        assert_eq!(partition.num_of_blocks(), 1);

        let initial_block = BlockIndex::new(0);
        assert_eq!(partition.block(initial_block).len(), 10);

        let block_index = partition.split_block(BlockIndex::new(0), |state| state < 5).unwrap();

        assert_eq!(partition.num_of_blocks(), 2);
        assert_eq!(partition.block(initial_block).len(), 5);
        assert_eq!(partition.block(block_index).len(), 5);
    }
}
