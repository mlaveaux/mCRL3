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
mod aterm_int;
mod aterm_list;
mod aterm_string;
mod busy_forbidden;
mod default_symbols;
mod global_aterm_pool;
mod parse_term;
mod str_ref;
mod symbol;
mod symbol_pool;
mod thread_aterm_pool;

pub use aterm::*;
pub use aterm_builder::*;
pub use aterm_container::*;
pub use aterm_int::*;
pub use aterm_list::*;
pub use aterm_string::*;
pub use busy_forbidden::*;
pub use default_symbols::*;
pub use global_aterm_pool::*;
pub use parse_term::*;
pub use str_ref::*;
pub use symbol::*;
pub use symbol_pool::*;
pub use thread_aterm_pool::*;
