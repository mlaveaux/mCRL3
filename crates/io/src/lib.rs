//!
//! A crate containing IO related functionality. This includes the reading of
//! .aut (Aldebaran) lts formats, and reading encoded integers.
//!
//! This crate does not use unsafe code.

#![forbid(unsafe_code)]

mod line_iterator;
mod progress;

pub mod io_aut;
pub mod u64_variablelength;
