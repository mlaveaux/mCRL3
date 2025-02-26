use crate::{ATerm, THREAD_TERM_POOL};
use crate::Symbol;
use mcrl3_macros::mcrl3_derive_terms;
use std::ops::Deref;

use std::borrow::Borrow;
use crate::ATermRef;
use crate::Markable;
use crate::Marker;

fn is_string_term(_: &ATermRef<'_>) -> bool {
    true
}

#[mcrl3_derive_terms]
mod inner {
    use mcrl3_macros::mcrl3_term;

    use super::*;

    #[mcrl3_term(is_string_term)]
    pub struct ATermString {
        term: ATerm,
    }

    impl ATermString {
        pub fn new(string: impl Into<String>) -> ATermString {
            THREAD_TERM_POOL.with_borrow_mut(|tp| {
                ATermString {
                    term: tp.create_constant(&Symbol::new(string, 0)),
                }
            })
        }

        pub fn to_string(&self) -> String {
            self.term.to_string()
        }
    }
}

pub use inner::*;