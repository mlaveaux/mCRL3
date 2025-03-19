use std::ops::Deref;
use std::borrow::Borrow;

use mcrl3_macros::mcrl3_derive_terms;
use mcrl3_macros::mcrl3_term;
use mcrl3_macros::mcrl3_ignore;

use crate::{ATerm, Symb, Term, THREAD_TERM_POOL};
use crate::Symbol;
use crate::ATermRef;
use crate::Markable;
use crate::Marker;

use crate::ATermArgs;
use crate::TermIterator;
use crate::SymbolRef;
use crate::StrRef;

fn is_string_term<'a>(t: &impl Term<'a>) -> bool {
    t.get_head_symbol().arity() == 0
}

#[mcrl3_derive_terms]
mod inner {
    use super::*;

    #[mcrl3_term(is_string_term)]
    pub struct ATermString {
        term: ATerm,
    }

    impl ATermString {
        pub fn new(string: impl Into<String>) -> ATermString {
            THREAD_TERM_POOL.with_borrow(|tp| {
                ATermString {
                    term: tp.create_constant(&Symbol::new(string, 0)),
                }
            })
        }

        /// Get the value of the string
        pub fn value(&self) -> StrRef<'_> {
            let arg = self.term.arg(0);
            
            arg.get_head_symbol().name()
        }
    }

    #[mcrl3_ignore]
    impl From<&str> for ATermString {
        fn from(s: &str) -> Self {
            ATermString::new(s)
        }
    }
}

pub use inner::*;