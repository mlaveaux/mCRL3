use std::borrow::Borrow;
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

pub fn is_int_term<'a, 'b>(t: &'b impl Term<'a, 'b>) -> bool {
    t.is_int()
}

#[mcrl3_derive_terms]
mod inner {
    use mcrl3_macros::mcrl3_ignore;

    use super::*;

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
            self.term.shared().address() - 1
        }
    }
}

pub use inner::*;
