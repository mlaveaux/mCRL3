//! TODO: We could consider introducing "annotated" terms formally where the last argument is a pointer to the annotation.

use std::borrow::Borrow;
use std::fmt;
use std::mem::transmute;
use std::ops::Deref;

use delegate::delegate;

use mcrl3_macros::mcrl3_derive_terms;
use mcrl3_macros::mcrl3_term;

use crate::ATerm;
use crate::ATermArgs;
use crate::ATermIndex;
use crate::ATermRef;
use crate::Markable;
use crate::Marker;
use crate::SymbolRef;
use crate::THREAD_TERM_POOL;
use crate::Term;
use crate::TermIterator;
use crate::Transmutable;

/// Returns true if the term is an ATermInt term.
pub fn is_int_term<'a, 'b>(t: &'b impl Term<'a, 'b>) -> bool {
    THREAD_TERM_POOL.with_borrow(|tp| *tp.int_symbol() == t.get_head_symbol())
}

#[mcrl3_derive_terms]
mod inner {
    use mcrl3_macros::mcrl3_ignore;

    use super::*;

    /// This is a wrapper around the ATerm type that represents an integer.
    #[mcrl3_term(is_int_term)]
    pub struct ATermInt {
        term: ATerm,
    }

    impl ATermInt {
        #[mcrl3_ignore]
        pub fn new(value: usize) -> ATermInt {
            THREAD_TERM_POOL.with_borrow(|tp| ATermInt {
                term: tp.create_int(value),
            })
        }

        /// Returns the value of the integer term.
        pub fn value(&self) -> usize {
            // This is a bit ugly, but the first argument is the integer value
            self.shared().arguments()[0].shared().address() - 1
        }
    }
}

impl fmt::Display for ATermInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Display for ATermIntRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

pub use inner::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_term() {
        let int_term = ATermInt::new(42);
        assert_eq!(int_term.value(), 42);
        assert!(is_int_term(&int_term));
    }
}
