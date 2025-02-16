use core::fmt;
use std::borrow::Borrow;
use std::ops::Deref;

use mcrl3_aterm::ATermList;
use mcrl3_aterm::Markable;
use mcrl3_aterm::ATerm;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::Marker;
use mcrl3_macros::mcrl3_derive_terms;
use mcrl3_macros::mcrl3_ignore;
use mcrl3_macros::mcrl3_term;

use crate::is_basic_sort;
use crate::is_bool_sort;
use crate::is_data_function_sort;
use crate::is_sort_expression;

// This module is only used internally to run the proc macro.
#[mcrl3_derive_terms]
mod inner {
    use super::*;

    #[mcrl3_term(is_bool_sort)]
    pub struct BoolSort {
        pub(crate) term: ATerm,
    }

    impl BoolSort {
        /// Returns the term representing true.
        pub fn true_term() -> DataExpression {
            DataExpression::from(ATerm::from(true_term()))
        }

        /// Returns the term representing false.
        pub fn false_term() -> DataExpression {
            DataExpression::from(ATerm::from(false_term()))
        }
    }

    // #[mcrl3_term(is_sort_expression)]
    pub struct SortExpression {
        term: ATerm,
    }

    impl SortExpression {
        /// Returns the name of the sort.
        pub fn name(&self) -> &str {
            // We only change the lifetime, but that is fine since it is derived from the current term.
            unsafe { std::mem::transmute(self.term.arg(0).symbol().name()) }
        }

        /// Returns true iff this is a basic sort
        pub fn is_basic_sort(&self) -> bool {
            is_basic_sort(&self.term)
        }

        /// Returns true iff this is a function sort
        pub fn is_function_sort(&self) -> bool {
            is_data_function_sort(&self.term)
        }
    }

    impl fmt::Display for SortExpression {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }

    #[mcrl3_term(is_basic_sort)]
    pub struct BasicSort {
        term: ATerm,
    }

    impl BasicSort {
        /// Returns the name of the sort.
        pub fn name(&self) -> &str {
            // We only change the lifetime, but that is fine since it is derived from the current term.
            unsafe { std::mem::transmute(self.term.arg(0).symbol().name()) }
        }
    }

    /// Derived from SortExpression
    #[mcrl3_term(is_data_function_sort)]
    pub struct FunctionSort {
        term: ATerm,
    }

    impl FunctionSort {
        /// Returns the name of the sort.
        pub fn domain(&self) -> ATermList<SortExpression> {
            ATermList::<SortExpression>::from(self.term.arg(0).protect())
        }

        /// Returns the name of the sort.
        pub fn codomain(&self) -> SortExpression {
            SortExpression::from(self.term.arg(1).protect())
        }
    }

    // #[mcrl3_ignore]
    impl From<SortExpression> for FunctionSort {
        fn from(sort: SortExpression) -> Self {
            Self { term: sort.term }
        }
    }

    // #[mcrl3_ignore]
    impl<'a> From<SortExpressionRef<'a>> for FunctionSortRef<'a> {
        fn from(sort: SortExpressionRef<'a>) -> Self {
            unsafe { std::mem::transmute(sort) }
        }
    }
}

pub use inner::*;