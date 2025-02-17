use crate::ATerm;


pub struct ATermInt {
    term: ATerm,
}

impl ATermInt {
    pub fn new(value: i32) -> ATermInt {
        ATermInt {
            term: ATerm::default(),
        }
    }
}