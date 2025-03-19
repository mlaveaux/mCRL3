use std::ops::Deref;
use std::borrow::Borrow;

use delegate::delegate;

use mcrl3_macros::mcrl3_derive_terms;
use mcrl3_macros::mcrl3_term;

use crate::ATermArgs;
use crate::ATermRef;
use crate::Markable;
use crate::Marker;
use crate::SymbolRef;
use crate::TermIterator;use crate::Term;
use crate::{ATerm, THREAD_TERM_POOL};

pub fn is_int_term<'a>(t: &impl Term<'a>) -> bool {
    t.is_int()
}

#[mcrl3_derive_terms]
mod inner {
    use super::*;

    #[mcrl3_term(is_int_term)]
    pub struct ATermInt {
        term: ATerm,
    }

    impl ATermInt {
        pub fn new(value: usize) -> ATermInt {
            THREAD_TERM_POOL.with_borrow_mut(|tp| {
                ATermInt {
                    term: tp.create_int(value),
                }
            })
        }
    }
}

pub use inner::*;