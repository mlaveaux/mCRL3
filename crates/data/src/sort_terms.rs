use core::fmt;
use std::borrow::Borrow;
use std::ops::Deref;

use delegate::delegate;

use mcrl3_aterm::ATerm;
use mcrl3_aterm::ATermArgs;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::Markable;
use mcrl3_aterm::Marker;
use mcrl3_aterm::StrRef;
use mcrl3_aterm::Symb;
use mcrl3_aterm::SymbolRef;
use mcrl3_aterm::Term;
use mcrl3_aterm::TermIterator;
use mcrl3_macros::mcrl3_derive_terms;
use mcrl3_macros::mcrl3_term;

use crate::is_sort_expression;

// This module is only used internally to run the proc macro.
#[mcrl3_derive_terms]
mod inner {

    use super::*;

    #[mcrl3_term(is_sort_expression)]
    pub struct SortExpression {
        term: ATerm,
    }

    impl SortExpression {
        /// Returns the name of the sort.
        pub fn name(&self) -> StrRef {
            self.term.arg(0).get_head_symbol().name()
        }
    }

    impl fmt::Display for SortExpression {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }
}

pub use inner::*;
