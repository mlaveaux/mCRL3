#![doc = include_str!("../README.md")]

mod ast;
mod consume;
mod diagnostics;
pub mod eval;
pub mod ir;
mod lower;
mod parse;
mod precedence;
mod resolve;
mod specification;
mod typecheck;
mod types;
pub mod value;

pub use ast::*;
pub use consume::*;
pub use diagnostics::*;
pub use lower::lower;
pub use parse::*;
pub use precedence::*;
pub use resolve::*;
pub use specification::*;
pub use typecheck::*;
pub use types::*;
