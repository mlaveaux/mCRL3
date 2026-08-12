use std::fmt;

use mcrl2_macros::mcrl2_derive_terms;

use crate::ATermRef;

pub(crate) fn is_aterm_string(term: &ATermRef<'_>) -> bool {
    term.get_head_symbol().arity() == 0
}

#[mcrl2_derive_terms]
mod inner {
    use mcrl2_macros::mcrl2_term;

    use crate::ATerm;
    use crate::ATermRef;
    use crate::Markable;
    use crate::Todo;
    use crate::is_aterm_string;

    /// Represents an atermpp::aterm_string from the mCRL2 toolset.
    #[mcrl2_term(is_aterm_string)]
    pub struct ATermString {
        term: ATerm,
    }

    impl ATermString {
        /// Returns the string value.
        pub fn str(&self) -> String {
            // The Rust::Str should ensure that this is a valid string.
            self.term.get_head_symbol().name().to_string()
        }
    }
}

pub use inner::ATermString;
pub use inner::ATermStringRef;

impl ATermStringRef<'static> {
    /// Creates a reference to the maximally shared aterm_string at `term`.
    ///
    /// Two occurrences of the same name are the same term, so the resulting
    /// reference can be used as a hash key that identifies a name without
    /// rendering it to a `String`.
    ///
    /// # Safety
    ///
    /// The term at `term` must stay live for the whole of `'static`, i.e. for as
    /// long as the returned reference is reachable; see [`crate::ATermRef`].
    pub unsafe fn from_address(term: *const crate::_aterm) -> ATermStringRef<'static> {
        // SAFETY: the caller upholds that the term stays live for `'static`.
        ATermStringRef::new(unsafe { ATermRef::new(term) })
    }
}

impl fmt::Display for ATermString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl fmt::Display for ATermStringRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}
