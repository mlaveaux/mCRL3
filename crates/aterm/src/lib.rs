//! Implementation of the [ATerm] related data structure.
//!
//! An aterm is a first-order term of the following form:
//!
//! t := c | f(t1, ..., tn) | u64
//!
//! where `f` is a function symbol with arity `n > 0` and a unique name, `c` is a constant and `u64` is a numerical term.
//!
//! Terms are stored maximally shared in the global aterm pool, meaning that T1,
//! Tn are shared between all terms and the term is immutable. This global aterm
//! pool performs garbage collection to remove terms that are no longer
//! reachable. This is kept track of by the thread-local aterm pool.

#![cfg_attr(feature = "mcrl3_miri", feature(reentrant_lock))]

mod aterm_builder;
mod aterm_int;
mod aterm_list;
mod aterm_binary_stream;
mod aterm_string;
mod aterm;
mod gc_mutex;
mod global_aterm_pool;
mod markable;
mod parse_term;
mod protected;
mod random_term;
mod shared_term;
mod symbol_pool;
mod symbol;
mod thread_aterm_pool;
mod transmutable;

pub use aterm_builder::*;
pub use aterm_int::*;
pub use aterm_list::*;
pub use aterm_binary_stream::*;
pub use aterm_string::*;
pub use aterm::*;
pub use global_aterm_pool::*;
pub use markable::*;
pub use parse_term::*;
pub use protected::*;
pub use random_term::*;
pub use shared_term::*;
pub use symbol_pool::*;
pub use symbol::*;
pub use thread_aterm_pool::*;
pub use transmutable::*;