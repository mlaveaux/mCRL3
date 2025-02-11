//! Implementation of the ATerm (Annotated Term) data structure.
//! 
//! ATerms are used to represent tree-like data structures with annotations.
//! This implementation provides:
//! - Maximal sharing of terms
//! - Garbage collection
//! - Binary storage format

mod global_symbol_pool;
mod symbol;
mod global_aterm_pool;
mod aterm;

pub use global_aterm_pool::*;
pub use global_symbol_pool::*;
pub use symbol::*;
pub use aterm::*;