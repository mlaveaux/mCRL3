use log::debug;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;

use mcrl2::Pbes;
use merc_symbolic::SymbolicLps;
use merc_symbolic::reachability;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::explore_srf::PbesSrfLps;

/// Explore a PBES in SRF normal form using symbolic LDD-based reachability.
///
/// Returns the LDD encoding the set of reachable states (BES equations). State
/// vectors have layout `[equation_index, param_0, …, param_{n-1}]`. The summand
/// machinery (equation-index gating via `prepare`, condition enumeration,
/// read/write positions) is reused from the explicit [`PbesSrfLps`] through the
/// generic [`SymbolicLps`] adapter, shared with LPS symbolic exploration.
pub(crate) fn explore_pbes_symbolic(
    storage: &LDDManagerRef,
    pbes: &Pbes,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
    let lps = PbesSrfLps::new(pbes)?;
    let mut symbolic = SymbolicLps::new(storage, lps)?;

    debug!("{symbolic:?}");

    reachability(storage, &mut symbolic, timing)
}
