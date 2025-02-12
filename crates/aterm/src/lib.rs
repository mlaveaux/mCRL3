//! Implementation of the ATerm (Annotated Term) data structure.
//! 
//! ATerms are used to represent tree-like data structures with annotations.
//! This implementation provides:
//! - Maximal sharing of terms
//! - Garbage collection
//! - Binary storage format

mod symbol_pool;
mod symbol;
mod global_aterm_pool;
mod aterm;
mod aterm_list;

pub use global_aterm_pool::*;
pub use symbol_pool::*;
pub use symbol::*;
pub use aterm::*;
pub use aterm_list::*;