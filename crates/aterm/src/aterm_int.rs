use std::fmt;

use merc_macros::merc_derive_terms;

use crate::Symb;
use crate::Term;
use crate::storage::THREAD_TERM_POOL;

/// Returns true if the term is an [ATermInt] term.
pub fn is_int_term<'a, 'b, T: Term<'a, 'b>>(t: &'b T) -> bool {
    THREAD_TERM_POOL.with_borrow(|tp| *tp.int_symbol() == t.get_head_symbol())
}

/// Returns true if the given symbol is the special integer symbol used by [ATermInt].
pub fn is_int_symbol<'a, 'b, S: Symb<'a, 'b>>(f: &'b S) -> bool {
    THREAD_TERM_POOL.with_borrow(|tp| *tp.int_symbol() == f.copy())
}

#[merc_derive_terms]
mod inner {
    use delegate::delegate;

    use merc_macros::merc_ignore;
    use merc_macros::merc_term;

    use crate::ATerm;
    use crate::ATermArgs;
    use crate::ATermIndex;
    use crate::ATermRef;
    use crate::Markable;
    use crate::SymbolRef;
    use crate::Term;
    use crate::TermIterator;
    use crate::Transmutable;
    use crate::is_int_term;
    use crate::storage::Marker;
    use crate::storage::SharedTermInt;
    use crate::storage::THREAD_TERM_POOL;

    /// This is a wrapper around the [ATerm] type that stores a single `u64` using an annotation.
    #[merc_term(is_int_term)]
    pub struct ATermInt {
        term: ATerm,
    }

    impl ATermInt {
        #[merc_ignore]
        pub fn new(value: usize) -> ATermInt {
            THREAD_TERM_POOL.with_borrow(|tp| ATermInt {
                term: tp.create_int(value),
            })
        }

        /// Returns the value of the integer term.
        ///
        /// # Safety
        ///
        /// This method assumes that the term is indeed an integer term, which
        /// should be guaranteed by the constructor and the `is_int_term`
        /// function. Otherwise, it leads to undefined behaviour.
        pub fn value(&self) -> usize {
            unsafe { self.shared().ptr().cast::<SharedTermInt>().as_ref().value() }
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
    use merc_utilities::test_logger;

    use crate::ATermInt;
    use crate::is_int_term;

    #[test]
    fn test_int_term() {
        let _ = test_logger();

        let int_term = ATermInt::new(42);
        assert_eq!(int_term.value(), 42);
        assert!(is_int_term(&int_term));
    }
}
