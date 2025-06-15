use std::fmt;
use std::{marker::PhantomData, ops::Deref};
use std::hash::Hash;


/// A safe index is an index that can only be compared with equivalent tags.
pub struct SafeIndex<T, Tag> {
    index: T,

    /// Ensures that the Tag is used by the struct
    marker: PhantomData<Tag>,
}

impl<T: PartialEq, Tag> Eq for SafeIndex<T, Tag> {}

impl<T: PartialEq, Tag> PartialEq for SafeIndex<T, Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T: Ord, Tag> Ord for SafeIndex<T, Tag> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T: PartialOrd, Tag> PartialOrd for SafeIndex<T, Tag> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<T: Hash, Tag> Hash for SafeIndex<T, Tag> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T: Clone, Tag>  Clone for SafeIndex<T, Tag> {
    fn clone(&self) -> Self {
        Self { index: self.index.clone(), marker: self.marker.clone() }
    }
}

impl<T: Copy, Tag> Copy for SafeIndex<T, Tag> {}

impl<T, Tag> SafeIndex<T, Tag> {
    pub fn new(index: T) -> Self {
        Self {
            index,
            marker: PhantomData::default(),
        }
    }
}

impl<T: Copy, Tag> SafeIndex<T, Tag> {
    pub fn value(&self) -> T {
        self.index
    }
}

impl<T: fmt::Debug, Tag> fmt::Debug for SafeIndex<T, Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.index.fmt(f)
    }
}

impl<T: fmt::Display, Tag> fmt::Display for SafeIndex<T, Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.index.fmt(f)
    }
}

impl<T, Tag> Deref for SafeIndex<T, Tag> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}