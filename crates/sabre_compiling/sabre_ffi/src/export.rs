use mcrl3_aterm::{ATermRef, THREAD_TERM_POOL};

use crate::{ATermFFI, ATermRefFFI};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn make_term() -> ATermFFI {
    unimplemented!("rewrite");
}

pub unsafe extern "C" fn term_arg(term: &ATermFFI, index: usize) -> usize {
    unimplemented!("rewrite");
    //ATermRef::new(term.index).arg(index)
}