use std::borrow::Borrow;
use std::fmt;
use std::mem::transmute;
use std::ops::Deref;

use delegate::delegate;

use mcrl3_macros::mcrl3_derive_terms;
use mcrl3_macros::mcrl3_ignore;
use mcrl3_macros::mcrl3_term;

use crate::ATerm;

use crate::ATermArgs;
use crate::ATermIndex;
use crate::ATermRef;
use crate::Markable;
use crate::Marker;
use crate::Symb;
use crate::Symbol;
use crate::SymbolRef;
use crate::THREAD_TERM_POOL;
use crate::Term;
use crate::TermIterator;
use crate::Transmutable;

fn is_string_term<'a, 'b>(t: &'b impl Term<'a, 'b>) -> bool {
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
        #[mcrl3_ignore]
        pub fn new(string: impl Into<String> + AsRef<str>) -> Self {
            THREAD_TERM_POOL.with_borrow(|tp| ATermString {
                term: tp.create_constant(&Symbol::new(string, 0)),
            })
        }

        /// Get the value of the string
        pub fn value(&self) -> &str {
            self.term.get_head_symbol().name()
        }
    }

    #[mcrl3_ignore]
    impl From<&str> for ATermString {
        fn from(s: &str) -> Self {
            ATermString::new(s)
        }
    }

    impl fmt::Display for ATermString {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value())
        }
    }
}

pub use inner::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let _ = mcrl3_utilities::test_logger();

        let s = ATermString::new("test");
        assert_eq!(s.value(), "test");
        assert_eq!(s.to_string(), "test");
    }
}
