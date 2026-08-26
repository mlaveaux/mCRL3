//!
//! Symbolic (LDD-based) parity games and their Zielonka solver.
//!
//! Lives inside `merc_vpg` rather than `merc_symbolic` or a new crate — see
//! `docs/symbolic-parity-game-plan.md` §1 for why: `merc_vpg` already depends on `merc_symbolic`,
//! so the dependency edge cannot be reversed, and this needs [`crate::Player`]/[`crate::Priority`]
//! from `merc_vpg` to stay cross-checkable against [`crate::solve_zielonka`].
//!

mod convert_symbolic_game;
mod partial_solve;
mod random_symbolic_game;
mod symbolic_parity_game;
mod symbolic_zielonka;
mod verify_symbolic;

pub use convert_symbolic_game::convert_symbolic_parity_game;
pub use partial_solve::detect_fatal_attractors;
pub use partial_solve::detect_fatal_attractors_within_safe_vertices;
pub use partial_solve::detect_forced_cycles;
pub use partial_solve::detect_forced_cycles_within_safe_vertices;
pub use partial_solve::detect_solitair_cycles;
pub use partial_solve::detect_solitair_cycles_within_safe_vertices;
pub use partial_solve::partial_solve;
pub use random_symbolic_game::encode_parity_game;
pub use symbolic_parity_game::AttractorProgress;
pub use symbolic_parity_game::SymbolicParityGame;
pub use symbolic_zielonka::ExtendedParityGame;
pub use symbolic_zielonka::RecursionProgress;
pub use symbolic_zielonka::SymbolicSolution;
pub use symbolic_zielonka::solve_symbolic_zielonka;
pub use verify_symbolic::verify_symbolic_strategy;
