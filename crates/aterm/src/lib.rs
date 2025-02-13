//! Implementation of the ATerm (Annotated Term) data structure.
//! 
//! ATerms are used to represent tree-like data structures with annotations.
//! This implementation provides:
//! - Maximal sharing of terms
//! - Garbage collection
//! - Binary storage format

mod aterm_builder;
mod aterm_container;
mod aterm_list;
mod aterm;
mod busy_forbidden;
mod global_aterm_pool;
mod symbol_pool;
mod symbol;
mod thread_aterm_pool;

pub use aterm_builder::*;
pub use aterm_container::*;
pub use aterm_list::*;
pub use aterm::*;
pub use busy_forbidden::*;
pub use global_aterm_pool::*;
pub use symbol_pool::*;
pub use symbol::*;
pub use thread_aterm_pool::*;