use std::collections::HashMap;
use std::iter::FromIterator;

/// A generic enumerator that assigns unique indices to values.
/// Values are stored in insertion order and maintain stable indices.
#[derive(Debug, Clone)]
pub struct IndexedSet<T: Eq + std::hash::Hash> {
    index_to_value: Vec<T>,
    value_to_index: HashMap<T, usize>,
}

impl<T: Eq + std::hash::Hash + Clone> IndexedSet<T> {
    pub fn new() -> Self {
        IndexedSet {
            index_to_value: Vec::new(),
            value_to_index: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: T) -> usize {
        if let Some(&index) = self.value_to_index.get(&value) {
            return index;
        }

        let index = self.index_to_value.len();
        self.index_to_value.push(value.clone());
        self.value_to_index.insert(value, index);
        index
    }

    pub fn get_index(&self, value: &T) -> Option<usize> {
        self.value_to_index.get(value).copied()
    }

    pub fn get_value(&self, index: usize) -> Option<&T> {
        self.index_to_value.get(index)
    }

    pub fn len(&self) -> usize {
        self.index_to_value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index_to_value.is_empty()
    }

    /// Creates an enumerator pre-populated with the given values
    pub fn from_values(values: impl IntoIterator<Item = T>) -> Self {
        let mut enumerator = Self::new();
        for value in values {
            enumerator.add(value);
        }
        enumerator
    }

    /// Returns an iterator over indices and values in insertion order
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.index_to_value.iter().enumerate()
    }

    /// Returns an iterator over indices in insertion order
    pub fn indices(&self) -> impl Iterator<Item = usize> {
        0..self.len()
    }

    /// Returns an iterator over values in insertion order
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.index_to_value.iter()
    }

    /// Removes all elements while preserving capacity
    pub fn clear(&mut self) {
        self.index_to_value.clear();
        self.value_to_index.clear();
    }

    /// Reserves capacity for at least `additional` more elements
    pub fn reserve(&mut self, additional: usize) {
        self.index_to_value.reserve(additional);
        self.value_to_index.reserve(additional);
    }

    /// Returns true if the enumerator contains the value
    pub fn contains(&self, value: &T) -> bool {
        self.value_to_index.contains_key(value)
    }
}

// Standard trait implementations
impl<T: Eq + std::hash::Hash + Clone> Default for IndexedSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + std::hash::Hash + Clone> FromIterator<T> for IndexedSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_values(iter)
    }
}

impl<T: Eq + std::hash::Hash + Clone> Extend<T> for IndexedSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.add(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iteration() {
        let mut set = IndexedSet::new();
        set.add("a");
        set.add("b");
        set.add("c");

        // Test iterator
        let mut iter = set.iter();
        assert_eq!(iter.next(), Some((0, &"a")));
        assert_eq!(iter.next(), Some((1, &"b")));
        assert_eq!(iter.next(), Some((2, &"c")));
        assert_eq!(iter.next(), None);

        // Test values iterator
        let values: Vec<_> = set.values().collect();
        assert_eq!(values, vec![&"a", &"b", &"c"]);

        // Test indices iterator
        let indices: Vec<_> = set.indices().collect();
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_from_iter() {
        let values = vec!["a", "b", "c"];
        let set: IndexedSet<_> = values.iter().cloned().collect();
        
        assert_eq!(set.len(), 3);
        assert_eq!(set.get_index(&"b"), Some(1));
    }

    #[test]
    fn test_extend() {
        let mut set = IndexedSet::new();
        set.add("a");
        
        set.extend(vec!["b", "c"]);
        assert_eq!(set.len(), 3);
        assert_eq!(set.get_index(&"c"), Some(2));
    }

    #[test]
    fn test_clear_and_reserve() {
        let mut set = IndexedSet::new();
        set.extend(vec!["a", "b", "c"]);
        
        set.clear();
        assert!(set.is_empty());
        
        set.reserve(10);
        set.add("d");
        assert_eq!(set.get_index(&"d"), Some(0));
    }
}