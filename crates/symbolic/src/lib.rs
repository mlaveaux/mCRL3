#![forbid(unsafe_code)]

mod bdd;
mod dependency_graph;
mod io;
mod ldd;
mod mince_variable_order;
mod symbolic_lps;
mod symbolic_lps_explore;
mod util;

#[cfg(test)]
mod random_symbolic_lts;
#[cfg(test)]
mod random_vector_set;

pub(crate) use bdd::*;
pub(crate) use dependency_graph::*;
pub(crate) use ldd::*;
pub(crate) use symbolic_lps::*;
pub(crate) use util::*;

#[cfg(test)]
pub(crate) use random_symbolic_lts::*;
#[cfg(test)]
pub(crate) use random_vector_set::*;

pub use bdd::CubeIterAll;
pub use bdd::FormatConfig;
pub use bdd::FormatConfigSet;
pub use bdd::SymbolicLtsBdd;
pub use bdd::bits_to_bdd;
pub use bdd::convert_symbolic_lts_bdd;
pub use bdd::minus;
pub use bdd::minus_edge;
pub use bdd::quotient_symbolic;
pub use bdd::random_bdd;
pub use bdd::reachability_bdd;
pub use bdd::refine_bisimulation;
pub use bdd::sigref_symbolic;
pub use dependency_graph::parse_compacted_dependency_graph;
pub use io::SymFormat;
pub use io::guess_format_from_extension;
pub use ldd::ExplorationStrategy;
pub use ldd::ReachabilityOptions;
pub use ldd::ReachabilityResult;
pub use ldd::SymbolicLTS;
pub use ldd::convert_symbolic_lts;
pub use ldd::reachability;
pub use ldd::reachability_with_options;
pub use ldd::read_sylvan;
pub use ldd::read_symbolic_lts;
pub use ldd::write_symbolic_lts;
pub use mince_variable_order::reorder;
pub use symbolic_lps::SymbolicLPS;
pub use symbolic_lps_explore::SymbolicLps;
pub use util::SatCountCache;
pub use util::approx_satcount;
