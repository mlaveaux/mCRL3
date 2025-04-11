use std::num::NonZero;

use mcrl3_aterm::ATermRef;
use mcrl3_aterm::Term;

use crate::ATermFFI;

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn make_term() -> ATermFFI {
    unimplemented!("rewrite");
}

/// Returns the term index of arguments of the term.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_arg(term: NonZero<usize>, index: usize) -> NonZero<usize> {
    unsafe { ATermRef::from_index(term).arg(index).index() }
}
