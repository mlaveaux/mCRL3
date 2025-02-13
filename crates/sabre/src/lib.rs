//!
//! The Set Automaton Based Rewrite Engine (Sabre) implements a rewriter for
//! conditional first-order non-linear rewrite rules.
//!
//! This crate does not use unsafe code.

//#![forbid(unsafe_code)]

pub mod innermost_rewriter;
pub mod matching;
pub mod rewrite_specification;
pub mod sabre_rewriter;
pub mod set_automaton;
pub mod utilities;

#[cfg(test)]
pub mod test_utility;

pub use innermost_rewriter::*;
pub use rewrite_specification::*;
pub use sabre_rewriter::*;
