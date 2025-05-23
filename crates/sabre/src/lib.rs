//!
//! The Set Automaton Based Rewrite Engine (Sabre) implements a rewriter for
//! conditional first-order non-linear rewrite rules.
//!
//! This crate does not use unsafe code.

//#![forbid(unsafe_code)]

mod innermost_rewriter;
mod matching;
mod naive_rewriter;
mod rewrite_specification;
mod sabre_rewriter;
mod set_automaton;
pub mod utilities;

#[cfg(test)]
pub mod test_utility;

pub use innermost_rewriter::*;
pub use naive_rewriter::*;
pub use rewrite_specification::*;
pub use sabre_rewriter::*;
pub use set_automaton::*;
