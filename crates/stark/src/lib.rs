mod ast;
mod consume;
mod diagnostics;
pub mod eval;
// `ir`/`value` are kept as their own public modules, rather than flattened
// like the rest of this crate's API, because `ir::BinaryOp` deliberately
// collides in name (not in meaning) with `ast::BinaryOp` — see `ir.rs`'s doc
// comment. Flattening both would be an ambiguous glob re-export.
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
