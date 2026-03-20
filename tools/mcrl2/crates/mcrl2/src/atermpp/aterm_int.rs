use std::fmt;

use mcrl2_macros::mcrl2_derive_terms;
use mcrl2_sys::atermpp::ffi::mcrl2_aterm_is_int;

use crate::ATermRef;

pub fn is_aterm_int(term: &ATermRef<'_>) -> bool {
    mcrl2_aterm_is_int(term.get())
}

#[mcrl2_derive_terms]
mod inner {
    use mcrl2_macros::mcrl2_ignore;
    use mcrl2_macros::mcrl2_term;

    use crate::ATerm;
    use crate::ATermRef;
    use crate::Markable;
    use crate::THREAD_TERM_POOL;
    use crate::Todo;
    use crate::is_aterm_int;

    /// Represents an atermpp::aterm_int from the mCRL2 toolset.
    #[mcrl2_term(is_aterm_int)]
    pub struct ATermInt {
        term: ATerm,
    }

    impl ATermInt {
        /// Creates a new ATermInt from the given value.
        #[mcrl2_ignore]
        pub fn with_value(value: u64) -> Self {
            Self {
                term: THREAD_TERM_POOL.with_borrow(|tp| tp.create_int(value))   
            }                     
        }

        /// Returns the integer value.
        ///
        /// TODO: Implement via FFI once mcrl2-sys exposes a function to retrieve the
        /// integer value from an aterm_int (e.g. `mcrl2_aterm_int_value`).
        pub fn value(&self) -> u64 {
            unimplemented!("Cannot retrieve integer value yet; mcrl2-sys FFI binding required")
        }
    }
}

pub use inner::*;

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

