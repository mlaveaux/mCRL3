use std::fmt;
use std::marker::PhantomData;

use itertools::Itertools;
use mcrl2_sys::atermpp::ffi::_aterm;

use super::THREAD_TERM_POOL;
use crate::ATerm;
use crate::ATermRef;

pub struct ATermList<T> {
    term: ATerm,
    _marker: PhantomData<T>,
}

impl<T: From<ATerm>> ATermList<T> {
    /// Obtain the head, i.e. the first element, of the list. Will panic if the
    /// list is empty.
    pub fn head(&self) -> T {
        // Guard explicitly: on an empty list `arg(0)` is out of range, which is
        // undefined (the FFI does not bounds-check in release), so panic instead.
        assert!(!self.is_empty(), "head() called on an empty list");
        self.term.arg(0).protect().into()
    }

    /// Returns the head if it exists, or None if the list is empty.
    pub fn try_head(&self) -> Option<T> {
        if self.is_empty() { None } else { Some(self.head()) }
    }
}

impl<T> ATermList<T> {
    /// Creates a new ATermList from the given term.
    pub fn new(term: ATerm) -> Self {
        debug_assert!(term.term.is_list(), "Term is not a list: {:?}", term);
        ATermList {
            term,
            _marker: PhantomData,
        }
    }

    /// Returns true iff the list is empty.
    pub fn is_empty(&self) -> bool {
        self.term.is_empty_list()
    }

    /// Obtain the tail, i.e. the remainder, of the list. Will panic if the list
    /// is empty.
    pub fn tail(&self) -> ATermList<T> {
        // Guard explicitly: on an empty list `arg(1)` is out of range, which is
        // undefined (the FFI does not bounds-check in release), so panic instead.
        assert!(!self.is_empty(), "tail() called on an empty list");
        self.term.arg(1).protect().into()
    }

    /// Returns the tail if it exists, or None if the list is empty.
    pub fn try_tail(&self) -> Option<ATermList<T>> {
        if self.is_empty() { None } else { Some(self.tail()) }
    }

    pub fn get(&self) -> &_aterm {
        self.term.get()
    }

    /// Casts the list to another type. This does not check whether the cast is
    /// valid.
    pub fn cast<U>(&self) -> ATermList<U> {
        ATermList {
            term: self.term.clone(),
            _marker: PhantomData,
        }
    }

    /// Constructs a new empty list.
    pub fn empty() -> Self {
        ATermList {
            term: THREAD_TERM_POOL.with_borrow(|tp| ATerm::constant(&tp.empty_list_symbol())),
            _marker: PhantomData,
        }
    }

    /// Constructs a new list with the given item as head and this list as tail.
    pub fn cons(&self, item: T) -> Self
    where
        T: Into<ATerm>,
    {
        let item: ATerm = item.into();
        ATermList {
            term: THREAD_TERM_POOL
                .with_borrow(|tp| ATerm::with_args(&tp.list_symbol(), &[item.copy(), self.term.copy()])),
            _marker: PhantomData,
        }
    }

    /// Constructs a new list from a double-ended iterator that is consumed.
    pub fn from_double_iter<I>(iter: I) -> Self
    where
        T: Into<ATerm>,
        I: DoubleEndedIterator<Item = T>,
    {
        let mut list = Self::empty();
        for item in iter.rev() {
            list = list.cons(item);
        }
        list
    }
}

impl<T: From<ATerm>> ATermList<T> {
    /// Returns an iterator over all elements in the list.
    pub fn iter(&self) -> ATermListIter<T> {
        ATermListIter { current: self.clone() }
    }

    /// Returns the number of elements in the list.
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Converts the list to a `Vec<T>`. Will convert every `ATerm` into `T``.
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().collect()
    }
}

impl<T> Clone for ATermList<T> {
    fn clone(&self) -> Self {
        ATermList {
            term: self.term.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> From<ATermList<T>> for ATerm {
    fn from(value: ATermList<T>) -> Self {
        value.term
    }
}

impl<T: From<ATerm>> Iterator for ATermListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_empty() {
            None
        } else {
            let head = self.current.head();
            self.current = self.current.tail();
            Some(head)
        }
    }
}

impl<T> From<ATerm> for ATermList<T> {
    fn from(value: ATerm) -> Self {
        Self::new(value)
    }
}

impl<T: From<ATerm>> IntoIterator for ATermList<T> {
    type IntoIter = ATermListIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: From<ATerm>> IntoIterator for &ATermList<T> {
    type IntoIter = ATermListIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: From<ATerm> + fmt::Debug> fmt::Debug for ATermList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}]", self.iter().format(", "))
    }
}

pub struct ATermListIter<T> {
    current: ATermList<T>,
}

// TODO: This should be generated by a macro
pub struct ATermListRef<'a, T> {
    term: ATermRef<'a>,
    _marker: PhantomData<T>,
}

impl<T> ATermListRef<'_, T> {
    /// Returns true iff the list is empty.
    pub fn is_empty(&self) -> bool {
        self.term.is_empty_list()
    }

    /// Protects the list by creating an owned version of it.
    pub fn protect(&self) -> ATermList<T> {
        ATermList::new(self.term.protect())
    }

    /// Returns a cheap copy of this term
    pub fn copy(&self) -> ATermListRef<'_, T> {
        ATermListRef {
            term: self.term.copy(),
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over all elements in the list.
    pub fn iter(&self) -> ATermListIterRef<'_, T> {
        ATermListIterRef { current: self.copy() }
    }
}

impl<'a, T: From<ATerm>> ATermListRef<'a, T> {
    pub fn head(&self) -> T {
        self.term.arg(0).protect().into()
    }

    pub fn tail(&'a self) -> ATermListRef<'a, T> {
        self.term.arg(1).into()
    }
}

impl<'a, T> From<ATermRef<'a>> for ATermListRef<'a, T> {
    fn from(value: ATermRef<'a>) -> Self {
        debug_assert!(value.is_list(), "Term is not a list: {:?}", value);
        ATermListRef {
            term: value,
            _marker: PhantomData,
        }
    }
}

/// The same as [ATermListIter], but for [ATermListRef].
pub struct ATermListIterRef<'a, T> {
    current: ATermListRef<'a, T>,
}

impl<'a, T: From<ATerm>> Iterator for ATermListIterRef<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_empty() {
            None
        } else {
            // TODO: This is a bit ugly because ATermListRef does not implement all term functions.
            let head = self.current.head();
            // SAFETY: `tail()` is a subterm of `self.current`, so `self.current`
            // is a parent term of it.
            self.current = unsafe { self.current.tail().term.upgrade(&self.current.term) }.into();
            Some(head)
        }
    }
}

#[cfg(test)]
mod tests {
    use merc_utilities::test_logger;

    use crate::ATerm;
    use crate::ATermList;

    #[test]
    fn test_aterm_list() {
        test_logger();
        let list: ATermList<ATerm> = ATerm::from_string("[f,g,h,i]").unwrap().into();

        assert!(!list.is_empty());

        // Convert into normal vector.
        let values: Vec<ATerm> = list.iter().collect();

        assert_eq!(values[0], ATerm::from_string("f").unwrap());
        assert_eq!(values[1], ATerm::from_string("g").unwrap());
        assert_eq!(values[2], ATerm::from_string("h").unwrap());
        assert_eq!(values[3], ATerm::from_string("i").unwrap());
    }

    #[test]
    fn test_from_double_iter() {
        let list = ATermList::from_double_iter(
            vec![
                ATerm::from_string("f").unwrap(),
                ATerm::from_string("g").unwrap(),
                ATerm::from_string("h").unwrap(),
            ]
            .into_iter(),
        );

        assert_eq!(list.head(), ATerm::from_string("f").unwrap());
        assert_eq!(list.tail().head(), ATerm::from_string("g").unwrap());
        assert_eq!(list.tail().tail().head(), ATerm::from_string("h").unwrap());
        assert!(list.tail().tail().tail().is_empty());
    }

    /// `head()` on an empty list used to call `arg(0)` without a bounds check,
    /// which is undefined behaviour through the FFI in release builds. After the
    /// fix an explicit `assert!` turns it into a deterministic panic.
    #[test]
    #[should_panic(expected = "head() called on an empty list")]
    fn head_on_empty_list_panics() {
        let empty: ATermList<ATerm> = ATermList::empty();
        let _ = empty.head();
    }

    /// Same as `head_on_empty_list_panics` but for `tail()` which would access
    /// `arg(1)` out of bounds.
    #[test]
    #[should_panic(expected = "tail() called on an empty list")]
    fn tail_on_empty_list_panics() {
        let empty: ATermList<ATerm> = ATermList::empty();
        let _ = empty.tail();
    }
}
