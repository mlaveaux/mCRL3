//! Implementation of the `ATerm` data structure.
//!
//! An aterm is a first-order term of the following form:
//!
//! T := a | f_n(T1, ..., Tn) | u64
//!
//! where f_n is a function symbol with arity n and a unique name.
//!
//! Terms are stored maximally shared in the global aterm pool, meaning that T1,
//! Tn are shared between all terms and the term is immutable. This global aterm
//! pool performs garbage collection to remove terms that are no longer
//! reachable. This is kept track of by the thread-local aterm pool.

#![cfg_attr(feature = "mcrl3_miri", feature(reentrant_lock))]

mod aterm;
mod aterm_builder;
mod aterm_container;
mod aterm_int;
mod aterm_list;
mod aterm_string;
mod gc_mutex;
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
pub use global_aterm_pool::*;
pub use parse_term::*;
pub use str_ref::*;
pub use symbol::*;
pub use symbol_pool::*;
pub use thread_aterm_pool::*;
