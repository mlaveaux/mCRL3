use log::debug;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;

use mcrl2::Pbes;
use merc_symbolic::ReachabilityOptions;
use merc_symbolic::SymbolicLPS;
use merc_symbolic::SymbolicLps;
use merc_symbolic::SymbolicLpsOptions;
use merc_symbolic::reachability_with_options;
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
///
/// The `encoding` decides how the equations are distributed over the transition
/// groups and in which order their parameters are stored, mirroring the
/// `--groups` and `--reorder` options of mCRL2's `pbessolvesymbolic`, and
/// `cached` its `--cached` option: every group then remembers the parameter
/// values it has already learned successors for, instead of re-enumerating them.
pub fn explore_pbes_symbolic(
    storage: &LDDManagerRef,
    pbes: &Pbes,
    encoding: &SymbolicLpsOptions,
    cached: bool,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
    let lps = PbesSrfLps::new(pbes)?;
    let mut symbolic = SymbolicLps::with_options(storage, lps, encoding)?;

    debug!("{symbolic:?}");

    let options = ReachabilityOptions {
        cached,
        ..ReachabilityOptions::default()
    };
    let mut context = symbolic.create_context();
    Ok(reachability_with_options(storage, &mut symbolic, &mut context, &options, timing)?.states)
}
