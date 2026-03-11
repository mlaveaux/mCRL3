use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use std::slice::Iter;

use itertools::Itertools;

#[macro_export]
macro_rules! vecset {
    () => {
        $crate::VecSet::new()
    };
    ($elem:expr; $n:expr) => {{
        let mut __set = $crate::VecSet::new();
        let __count: usize = $n;
        if __count > 0 {
            __set.insert($elem);
        }
        __set
    }};
    ($($x:expr),+ $(,)?) => {{
        let mut __set = $crate::VecSet::new();
        $( let _ = __set.insert($x); )*
        __set
    }};
}

///
/// A set that is internally represented by a sorted vector. Mostly useful for
/// a compact representation of sets that are not changed often.
///
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecSet<T> {
    /// The internal storage with the invariant that the array is sorted.
    sorted_array: Vec<T>,
}

impl<T: Ord> VecSet<T> {
    /// Creates a new, empty VecSet.
    pub fn new() -> Self {
        Self {
            sorted_array: Vec::new(),
        }
    }

    /// Creates a VecSet from the given vector without assuming anything of the given vector.
    pub fn from_vec(mut vec: Vec<T>) -> Self {
        vec.sort_unstable();
        vec.dedup();
        Self { sorted_array: vec }
    }

    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }

    /// Returns the capacity of the set.
    pub fn capacity(&self) -> usize {
        self.sorted_array.capacity()
    }

    /// Returns true iff the set contains the given element.
    pub fn contains(&self, element: &T) -> bool {
        self.sorted_array.binary_search(element).is_ok()
    }

    /// Clears the set, removing all elements.
    pub fn clear(&mut self) {
        self.sorted_array.clear();
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        // Removing elements does not change the order.
        self.sorted_array.retain(|e| f(e));
    }

    /// Returns true iff this set is a subset of the other set.
    pub fn is_subset(&self, other: &VecSet<T>) -> bool {
        let mut self_iter = self.sorted_array.iter();
        let mut other_iter = other.sorted_array.iter();

        // Traverse both sets in order, checking that all elements of self are in other.
        let mut self_next = self_iter.next();
        let mut other_next = other_iter.next();

        while let Some(self_val) = self_next {
            match other_next {
                Some(other_val) => {
                    match self_val.cmp(other_val) {
                        Ordering::Equal => {
                            self_next = self_iter.next();
                            other_next = other_iter.next();
                        }
                        Ordering::Greater => other_next = other_iter.next(),
                        Ordering::Less => return false, // self_val is smaller, not in other
                    }
                }
                None => return false, // other is exhausted
            }
        }

        true
    }

    /// Returns a new set only containing the given element.
    pub fn singleton(element: T) -> Self {
        Self {
            sorted_array: vec![element],
        }
    }

    /// Returns true iff the set is empty.
    pub fn is_empty(&self) -> bool {
        self.sorted_array.is_empty()
    }

    /// Returns the difference of this set and the other set.
    pub fn difference<'a>(&'a self, other: &'a VecSet<T>) -> impl Iterator<Item = &'a T> {
        Difference::<'a, T, _> {
            self_iter: self.sorted_array.iter(),
            other_iter: other.sorted_array.iter(),
            other_next: None,
            marker: PhantomData,
        }
    }

    /// Inserts the given element into the set, returns true iff the element was
    /// inserted.
    pub fn insert(&mut self, element: T) -> bool {
        // Finds the location where to insert the element to keep the array sorted.
        if let Err(position) = self.sorted_array.binary_search(&element) {
            self.sorted_array.insert(position, element);
            return true;
        }

        false
    }

    /// Extends this set with the elements from the given iterator, returns true iff at least one element was inserted.
    pub fn extend<'a, I: IntoIterator<Item = &'a T>>(&mut self, iter: I) -> bool
    where
        T: Clone + 'a,
    {
        let mut inserted = false;
        for element in iter {
            if self.insert(element.clone()) {
                inserted = true;
            }
        }

        inserted
    }

    /// Returns an iterator over the elements in the set, they are yielded in sorted order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.sorted_array.iter()
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.sorted_array.len()
    }

    /// Consumes the set and returns a vector with the elements in sorted order.
    pub fn to_vec(self) -> Vec<T> {
        self.sorted_array
    }
}

/// A lazy iterator that yields the difference of two sets. The elements are yielded in sorted order.
struct Difference<'a, T, I> {
    self_iter: I,
    other_iter: I,
    other_next: Option<&'a T>,
    marker: PhantomData<&'a T>,
}

impl<'a, T, I> Iterator for Difference<'a, T, I>
where
    I: Iterator<Item = &'a T>,
    T: Ord + PartialEq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.other_next.is_none() {
            self.other_next = self.other_iter.next();
        }

        while let Some(self_val) = self.self_iter.next() {
            loop {
                match self.other_next {
                    Some(other_val) => {
                        match self_val.cmp(other_val) {
                            Ordering::Equal => {
                                self.other_next = self.other_iter.next();
                                break;
                            }
                            Ordering::Greater => self.other_next = self.other_iter.next(),
                            Ordering::Less => return Some(self_val), // self_val is smaller, yield it
                        }
                    }
                    None => return Some(self_val), // other is exhausted, yield remaining elements of self
                }
            }
        }

        None
    }
}

impl<T: Ord> Default for VecSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a VecSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.sorted_array.iter()
    }
}

impl<T: fmt::Debug> fmt::Debug for VecSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{:?}}}", self.sorted_array.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rand::RngExt;

    use merc_utilities::random_test;

    use crate::VecSet;

    #[test]
    fn test_random_vecset_difference() {
        random_test(100, |rng| {
            let size = rng.random_range(0..20);
            let size2 = rng.random_range(0..20);
            let vec1: Vec<u32> = (0..size).map(|_| rng.random_range(0..100)).collect();
            let vec2: Vec<u32> = (0..size2).map(|_| rng.random_range(0..100)).collect();

            let set1 = VecSet::from_vec(vec1.clone());
            let set2 = VecSet::from_vec(vec2.clone());

            println!("left: {:?}", set1);
            println!("right: {:?}", set2);

            let difference: Vec<u32> = set1.difference(&set2).cloned().collect();
            let expected_difference: Vec<u32> = vec1
                .into_iter()
                .filter(|x| !vec2.contains(x))
                .sorted()
                .dedup()
                .collect();

            assert_eq!(difference, expected_difference);
        })
    }
}
