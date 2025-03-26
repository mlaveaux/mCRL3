use mcrl3_aterm::ATermRef;
use mcrl3_aterm::THREAD_TERM_POOL;

use crate::ATermFFI;
use crate::ATermRefFFI;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn make_term() -> ATermFFI {
    unimplemented!("rewrite");
}

pub unsafe extern "C" fn term_arg(term: &ATermFFI, index: usize) -> usize {
    unimplemented!("rewrite");
    //ATermRef::new(term.index).arg(index)
}
