//!
//! Symbolic (LDD-based) parity games and their Zielonka solver.
//!
//! Lives inside `merc_vpg` rather than `merc_symbolic` or a new crate — see
//! `docs/symbolic-parity-game-plan.md` §1 for why: `merc_vpg` already depends on `merc_symbolic`,
//! so the dependency edge cannot be reversed, and this needs [`crate::Player`]/[`crate::Priority`]
//! from `merc_vpg` to stay cross-checkable against [`crate::solve_zielonka`].
//!

mod convert_symbolic_game;
mod random_symbolic_game;
mod symbolic_parity_game;
mod symbolic_zielonka;

pub use convert_symbolic_game::*;
pub use random_symbolic_game::*;
pub use symbolic_parity_game::*;
pub use symbolic_zielonka::*;
