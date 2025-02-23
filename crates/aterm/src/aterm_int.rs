use crate::{ATerm, THREAD_TERM_POOL};


pub struct ATermInt {
    term: ATerm,
}

impl ATermInt {
    pub fn new(value: usize) -> ATermInt {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            ATermInt {
                term: tp.create_int(value),
            }
        })
    }
}