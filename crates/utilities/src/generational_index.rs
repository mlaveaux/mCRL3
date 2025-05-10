//! Provides a generational index implementation that offers generation checking
//! in debug builds while having zero runtime cost in release builds.

use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

/// A generational index that stores both an index and a generation counter.
/// The generation is only tracked in debug builds to avoid overhead in release.
///
/// This allows detecting use-after-free scenarios in debug mode while
/// maintaining zero overhead in release mode.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GenerationalIndex<I: Copy + Into<usize> = usize> {
    /// The raw index value
    index: I,

    #[cfg(debug_assertions)]
    /// Generation counter, only available in debug builds
    generation: usize,
}

impl Default for GenerationalIndex<usize> {
    fn default() -> Self {
        GenerationalIndex {
            index: 0,
            #[cfg(debug_assertions)]
            generation: usize::MAX,
        }
    }
}

impl<I: Copy + Into<usize>> Deref for GenerationalIndex<I> {
    type Target = I;

    /// Deref implementation to access the underlying index value.
    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl<I: Copy + Into<usize>> GenerationalIndex<I> {
    /// Creates a new generational index with the specified index.
    #[cfg(debug_assertions)]
    fn new(index: I, generation: usize) -> Self {
        Self { index, generation }
    }

    /// Creates a new generational index with the specified index and generation.
    #[cfg(not(debug_assertions))]
    fn new(index: I) -> Self {
        Self { index }
    }
}

/// A counter that keeps track of generational indices.
/// This helps manage generations of indices to detect use-after-free and similar issues.
#[derive(Clone, Debug, Default)]
pub struct GenerationCounter {
    /// Current generation count, only stored in debug builds
    #[cfg(debug_assertions)]
    current_generation: Vec<usize>,
}

impl GenerationCounter {
    /// Creates a new generation counter.
    pub fn new() -> Self {
        #[cfg(debug_assertions)]
        {
            Self {
                current_generation: Vec::new(),
            }
        }

        #[cfg(not(debug_assertions))]
        Self {}
    }
}

impl GenerationCounter {
    /// Creates a new generational index with the given index and the next generation.
    pub fn create_index<I>(&mut self, index: I) -> GenerationalIndex<I>
    where
        I: Copy + Into<usize>,
    {
        #[cfg(debug_assertions)]
        {
            let generation = if self.current_generation.len() <= index.into() {
                self.current_generation.resize(index.into() + 1, 0);
                0
            } else {
                let generation = &mut self.current_generation[index.into()];
                *generation = generation.wrapping_add(1);
                *generation
            };

            GenerationalIndex::new(index, generation)
        }

        #[cfg(not(debug_assertions))]
        {
            GenerationalIndex::new(index)
        }
    }

    /// Returns a generational index with the given index and the current generation.
    pub fn recall_index<I>(&self, index: I) -> GenerationalIndex<I>
    where
        I: Copy + Into<usize>,
    {
        #[cfg(debug_assertions)]
        {
            GenerationalIndex::new(index, self.current_generation[index.into()])
        }
        #[cfg(not(debug_assertions))]
        {
            GenerationalIndex::new(index)
        }
    }

    /// Returns the underlying index, checks if the generation is correct.
    pub fn get_index<I>(&self, index: GenerationalIndex<I>) -> I
    where
        I: Copy + Into<usize> + fmt::Debug,
    {
        #[cfg(debug_assertions)]
        {
            if self.current_generation[index.index.into()] != index.generation {
                panic!("Attempting to access an invalid index: {:?}", index);
            }
        }

        index.index
    }
}

// Standard trait implementations for GenerationalIndex

impl<I> PartialEq for GenerationalIndex<I>
where
    I: Copy + Into<usize> + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        // TODO: Should we have a default index?
        #[cfg(debug_assertions)]
        {
            if self.generation == usize::MAX || other.generation == usize::MAX {
                return false;
            }

            debug_assert_eq!(
                self.generation, other.generation,
                "Comparing indices of different generations"
            );
        }

        self.index == other.index
    }
}

impl<I> Eq for GenerationalIndex<I> where I: Copy + Into<usize> + Eq {}

impl<I> PartialOrd for GenerationalIndex<I>
where
    I: Copy + Into<usize> + PartialOrd + Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            self.generation, other.generation,
            "Comparing indices of different generations"
        );

        self.index.partial_cmp(&other.index)
    }
}

impl<I> Ord for GenerationalIndex<I>
where
    I: Copy + Into<usize> + Eq + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            self.generation, other.generation,
            "Comparing indices of different generations"
        );
        self.index.cmp(&other.index)
    }
}

impl<I> Hash for GenerationalIndex<I>
where
    I: Copy + Into<usize> + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<I> fmt::Debug for GenerationalIndex<I>
where
    I: Copy + Into<usize> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(debug_assertions)]
        {
            write!(
                f,
                "GenerationalIndex(index: {:?}, generation: {})",
                self.index, self.generation
            )
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
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_generational_index_equality() {
        let mut counter = GenerationCounter::new();
        let idx1 = counter.create_index(42usize);
        let idx2 = counter.create_index(42usize);
        let idx4 = counter.create_index(43usize);

        let idx3 = counter.recall_index(42usize);

        assert_ne!(idx1, idx4);
        assert_eq!(idx2, idx3);

        // This panics since idx1 and idx2 are from different generations
        assert_eq!(idx1, idx2);
    }
}
