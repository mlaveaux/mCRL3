//!
//! This module defines parity games themselves.
//!
//! Authors: Maurice Ter Beek, Maurice Laveaux, Sjef van Loo, Erik de Vink and Tim A.C. Willemse,
//!

#![forbid(unsafe_code)]

mod builder;
mod display_dot;
mod io;
mod io_pg;
mod io_vpg;
mod parity_game;
mod player;
mod predecessors;
mod random_game;
mod solitaire_game;
mod variability_make_total;
mod variability_parity_game;
mod variability_predecessors;

pub use builder::*;
pub use display_dot::*;
pub use io::*;
pub use io_pg::*;
pub use io_vpg::*;
pub use parity_game::*;
pub use player::*;
pub use predecessors::*;
pub use random_game::*;
pub use solitaire_game::*;
pub use variability_make_total::*;
pub use variability_parity_game::*;
pub use variability_predecessors::*;
