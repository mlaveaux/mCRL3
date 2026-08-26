//!
//! State space exploration for mCRL2 linear process specifications (LPSs).
//!
//! The [`merc-lps`] binary is a thin command line wrapper around this crate: it
//! parses the arguments, reads and preprocesses the LPS and then calls into one
//! of the explorers defined here. The explorers take the LPS as given, so that
//! every one of them sees the same summands as the rest of the tool.
//!
//! - [`explore_lps_explicit`] and [`explore_lps_explicit_parallel`] enumerate the
//!   state space explicitly, feeding the discovered transitions into an
//!   [`merc_lts::LtsBuilder`].
//! - [`explore_lps_symbolic`] performs LDD-based symbolic reachability.
//! - [`CfgLinearProcessSpecification`] wraps an LPS with a control flow analysis
//!   ([`mcrl2::ControlFlowGraph`], shared with `merc_pbes`'s SRF explorer)
//!   that prunes summands whose control flow guard cannot hold in the current
//!   state.
//!
//! [`merc-lps`]: https://mercorg.github.io/merc-website/

pub mod cfg_lps;
pub mod convert_data_specification;
pub mod explore_explicit;
pub mod explore_symbolic;
pub mod io;

pub use cfg_lps::CfgLinearProcessSpecification;
pub use cfg_lps::CfgMetrics;
pub use cfg_lps::SummandCfgMetrics;
pub use convert_data_specification::convert_data_specification;
pub use explore_explicit::ExplicitLinearProcessSpecification;
pub use explore_explicit::LtsMultiActionAdapter;
pub use explore_explicit::Mcrl2MultiActionLabel;
pub use explore_explicit::explore_lps_explicit;
pub use explore_explicit::explore_lps_explicit_parallel;
pub use explore_symbolic::explore_lps_symbolic;
pub use explore_symbolic::explore_lps_symbolic_to_sym;
pub use io::LpsFormat;

/// A symbolic LTS with mCRL2 multi-action transition labels, as produced by
/// [`explore_lps_symbolic_to_sym`] and consumed by [`merc_symbolic::write_symbolic_lts`].
pub type Mcrl2SymbolicLts = merc_symbolic::SymbolicLts<merc_lts::LtsMultiAction<merc_lts::LtsAction>>;
