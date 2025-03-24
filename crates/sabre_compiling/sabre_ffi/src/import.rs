//!
//! See the export module for the function documentation.
//! 

#[link(name = "sabre-ffi")]
unsafe extern {
    fn make_term() -> usize;

    fn term_arg(term: usize, index: usize) -> usize;    
}