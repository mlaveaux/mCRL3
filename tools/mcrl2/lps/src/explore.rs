use merc_ldd::Ldd;
use merc_symbolic::SymbolicLTS;

use mcrl2::{LinearProcessSpecification, Symbol};

/// Exploration of linear process specifications.
pub fn explore_lps(lps: &LinearProcessSpecification) -> Result<(), String> {
    unimplemented!()
}

/// This struct provides a [merc_symbolic::SymbolicLTS] interface to a [mcrl2::LinearProcessSpecification].
struct SymbolicLinearProcessSpecification {

}

/// Represents a symbolic summand of a [mcrl2::LinearProcessSpecification].
struct SymbolicSummand {

    /// Re
    project_ldd: Ldd,
}

impl SymbolicSummand {

}

impl SymbolicLTS for SymbolicLinearProcessSpecification {

}

impl TransitionGroup for SymbolicSummand {

}


