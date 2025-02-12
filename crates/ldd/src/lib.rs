//! # LDD
//!
//! A library to create and manipulate so called list decision diagrams,
//! abbreviated as LDDs. List decision diagrams are data structures that can
//! efficiently represent sets of vectors containing natural numbers \[Dijk18\]. 

//! # Representation
//!
//! An LDD is inductively defined as follows. First of all, constants 'true' and
//! 'false' are two distinct LDDs. Given LDDs x and y, then node(value, x, y) is
//! an LDD; where value is a natural number. As such, we observe that node(5,
//! true, node(4, true, false)) is an LDD and in general we obtain a tree-like
//! data structure.
//!
//! Next, we should explain how LDDs represent a set of vectors. Given an LDD n
//! then \[n\] is inductively defined as:
//!   - \[false\]                    = ∅
//!   - \[true\]                     = { <> }
//!   - \[node(value, down, right)\] = { value x | x in \[down\] } ∪ \[right\]
//!
//! Node that since 'true' and 'false' are not very insightful and clash with
//! Rust keywords we use 'empty vector' and 'empty set' for the constants 'true'
//! and 'false' respectively. This makes sense as they represent the empty
//! vector and empty set respectively.
//!
//! # Citations
//!
//! \[Dijk18\] --- Tom van Dijk, Jaco van de Pol. Sylvan: multi-core framework
//! for decision diagrams. International Journal on Software Tools for
//! Technology Transfer. 19(6):675-696, 2017.
//! 
extern crate static_assertions;

mod storage;
mod operations;
mod format;
pub mod iterators;

#[cfg(test)]
mod test_utility;

pub use storage::*;
pub use operations::*;
pub use format::*;