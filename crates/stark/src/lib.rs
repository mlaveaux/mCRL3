#![doc = include_str!("../README.md")]
// The crate documents its private items (`cargo doc --document-private-items`),
// so the design-rationale module docs may point at the private helpers and
// fields they describe.
#![allow(rustdoc::private_intra_doc_links)]

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

pub(crate) use parse::*;

pub use ast::DefId;
pub use ast::UntypedStarkSpecification;
pub use diagnostics::Diagnostics;
pub use ir::IrProgram;
pub use resolve::DefKind;
pub use specification::StarkSpecification;
