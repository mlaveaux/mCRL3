//! Implementation of the ATerm (Annotated Term) data structure.
//!
//! ATerms are used to represent tree-like data structures with annotations.
//! This implementation provides:
//! - Maximal sharing of terms
//! - Garbage collection
//! - Binary storage format

mod aterm;
mod aterm_builder;
mod aterm_container;
mod aterm_list;
mod busy_forbidden;
mod global_aterm_pool;
mod symbol;
mod symbol_pool;
mod thread_aterm_pool;
mod parse_term;
mod binary_io;

pub use aterm::*;
pub use aterm_builder::*;
pub use aterm_container::*;
pub use aterm_list::*;
pub use busy_forbidden::*;
pub use global_aterm_pool::*;
pub use symbol::*;
pub use symbol_pool::*;
pub use thread_aterm_pool::*;
pub use parse_term::*;
pub use binary_io::*;