//! Data terms implementation for the mCRL3 toolset.
//!
//! This crate provides data expressions and related functionality.

#[forbid(unsafe_code)]

mod data_expression;
mod data_specification;
mod data_terms;
mod sort_terms;

pub use data_expression::*;
pub use data_specification::*;
pub use data_terms::*;
pub use sort_terms::*;
