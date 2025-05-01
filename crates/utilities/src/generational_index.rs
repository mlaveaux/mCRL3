//! Provides a generational index implementation that offers generation checking
//! in debug builds while having zero runtime cost in release builds.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// A generational index that stores both an index and a generation counter.
/// The generation is only tracked in debug builds to avoid overhead in release.
///
/// This allows detecting use-after-free scenarios in debug mode while
/// maintaining zero overhead in release mode.
#[derive(Copy, Clone)]
pub struct GenerationalIndex<I: Copy = usize> {
    /// The raw index value
    index: I,
    
    #[cfg(debug_assertions)]
    /// Generation counter, only available in debug builds
    generation: usize,
}

impl<I: Copy> Deref for GenerationalIndex<I> {
    type Target = I;

    /// Deref implementation to access the underlying index value.
    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl<I: Copy> GenerationalIndex<I> {
    /// Creates a new generational index with the specified index and generation.
    ///
    /// In release builds, the generation parameter is ignored.
    fn new(index: I, generation: usize) -> Self {
        #[cfg(debug_assertions)]
        {
            Self {
                index,
                generation,
            }
        }
        
        #[cfg(not(debug_assertions))]
        Self {
            index,
        }
    }
}

/// A counter that keeps track of generational indices.
/// This helps manage generations of indices to detect use-after-free and similar issues.
#[derive(Clone, Debug, Default)]
pub struct GenerationCounter
{
    /// Current generation count, only stored in debug builds
    #[cfg(debug_assertions)]
    current_generation: usize,
}

impl GenerationCounter {
    /// Creates a new generational counter starting at generation 1.
    pub fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            current_generation: 0,
        }
    }

    /// Returns the current generation count.
    fn increment_generation(&mut self) -> usize{
        #[cfg(debug_assertions)]
        {
            self.current_generation.wrapping_add(1)
        }

        #[cfg(not(debug_assertions))]
        1
    }
}

impl GenerationCounter {    
    /// Creates a new generational index with the given index and the current generation.
    pub fn create_index<I>(&mut self, index: I) -> GenerationalIndex<I> 
        where I: Copy + Eq + Ord + Hash + fmt::Debug
    {
        GenerationalIndex::new(index, self.increment_generation())
    }
}

// Standard trait implementations for GenerationalIndex

impl<I> PartialEq for GenerationalIndex<I>
    where I: Copy + Eq
{
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(self.generation, other.generation, "Comparing indices of different generations");

        self.index == other.index
    }
}

impl<I> Eq for GenerationalIndex<I>
    where I: Copy + Eq
{}

impl<I> PartialOrd for GenerationalIndex<I>
    where I: Copy + PartialOrd + Eq
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        debug_assert_eq!(self.generation, other.generation, "Comparing indices of different generations");

        self.index.partial_cmp(&other.index)
    }
}

impl<I> Ord for GenerationalIndex<I>
    where I: Copy + Eq + Ord
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        debug_assert_eq!(self.generation, other.generation, "Comparing indices of different generations");
        self.index.cmp(&other.index)
    }
}

impl<I> Hash for GenerationalIndex<I>
    where I: Copy + Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<I> fmt::Debug for GenerationalIndex<I>
    where I: Copy + fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(debug_assertions)]
        {
            write!(f, "GenerationalIndex(index: {:?}, generation: {})", self.index, self.generation)
        }
        #[cfg(not(debug_assertions))]
        {
            write!(f, "GenerationalIndex(index: {:?})", self.index)
        }
    }
}

impl fmt::Display for GenerationalIndex<usize> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generational_index_equality() {
        let mut counter = GenerationCounter::new();
        let idx1 = counter.create_index(42);
        let idx2 = counter.create_index(42);
        let idx3 = counter.create_index(42);
        let idx4 = counter.create_index(43);
        
        assert_eq!(idx1, idx2);
        
        #[cfg(debug_assertions)]
        {
            assert_ne!(idx1, idx3);
        }
        
        assert_ne!(idx1, idx4);
    }
}
