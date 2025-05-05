//!
//! A list of terms, where T is the type of the elements in the list.
//!

use std::marker::PhantomData;

use crate::ATerm;
use crate::ATermRef;
use crate::Term;
use crate::THREAD_TERM_POOL;

/// Returns true iff the term is a list term.
pub fn is_list_term<'a, 'b>(t: &'b impl Term<'a, 'b>) -> bool {
    THREAD_TERM_POOL.with_borrow(|tp| *tp.list_symbol() == t.get_head_symbol())
}

/// Returns true iff the term is an empty list.
pub fn is_empty_list_term<'a, 'b>(t: &'b impl Term<'a, 'b>) -> bool {
    THREAD_TERM_POOL.with_borrow(|tp| *tp.empty_list_symbol() == t.get_head_symbol())
}

pub struct ATermList<T> {
    term: ATerm,
    _marker: PhantomData<T>,
}

impl<T: From<ATerm>> ATermList<T> {
    /// Obtain the head, i.e. the first element, of the list.
    pub fn head(&self) -> T {
        self.term.arg(0).protect().into()
    }
}

impl<T> ATermList<T> {
    /// Constructs a new list from an iterator that is consumed.
    pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Self 
        where T: Into<ATerm>  
    {
        let mut list = Self::empty();
        for item in iter {
            list = list.cons(item);
        }
        list
    }

    /// Constructs a new list with the given item as the head and the current list as the tail.
    pub fn cons(&self, item: T) -> Self
        where T: Into<ATerm> 
    {
        ATermList {
            term: THREAD_TERM_POOL.with_borrow(|tp| ATerm::with_args(tp.list_symbol(), &[item.into()])),
            _marker: PhantomData,
        }
    }

    /// Returns the empty list.
    pub fn empty() -> Self {
        ATermList {
            term: THREAD_TERM_POOL.with_borrow(|tp| ATerm::constant(tp.empty_list_symbol())),
            _marker: PhantomData,
        }
    }

    /// Returns true iff the list is empty.
    pub fn is_empty(&self) -> bool {
        is_empty_list_term(&self.term)
    }

    /// Obtain the tail, i.e. the remainder, of the list.
    pub fn tail(&self) -> ATermList<T> {
        self.term.arg(1).into()
    }

    /// Returns an iterator over all elements in the list.
    pub fn iter(&self) -> ATermListIter<T> {
        ATermListIter { current: self.clone() }
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
        debug_assert!(is_list_term(&value), "Can only convert a aterm_list");
        ATermList::<T> {
            term: value,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> From<ATermRef<'a>> for ATermList<T> {
    fn from(value: ATermRef<'a>) -> Self {
        debug_assert!(is_list_term(&value), "Can only convert a aterm_list");
        ATermList::<T> {
            term: value.protect(),
            _marker: PhantomData,
        }
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

pub struct ATermListIter<T> {
    current: ATermList<T>,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_list_term() {
        use super::*;
        use crate::ATermInt;

        let list = ATermList::from_iter(vec![ATermInt::new(1), ATermInt::new(2), ATermInt::new(3)]);
        assert_eq!(list.head().value(), 1);
        assert_eq!(list.tail().head().value(), 2);
        assert_eq!(list.tail().tail().head().value(), 3);
        assert!(list.tail().tail().tail().is_empty());
    }
}