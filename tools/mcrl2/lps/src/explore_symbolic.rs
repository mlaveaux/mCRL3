use log::debug;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;

use mcrl2::LinearProcessSpecification;
use merc_symbolic::SymbolicLps;
use merc_symbolic::reachability;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::explore_explicit::ExplicitLinearProcessSpecification;

/// Explore the linear process specification using symbolic reachability.
///
/// The summand machinery (read/write positions, condition enumeration) is reused
/// from the explicit [`ExplicitLinearProcessSpecification`] via the generic
/// [`SymbolicLps`] adapter, so LPS and PBES symbolic exploration share one
/// implementation.
pub(crate) fn explore_lps_symbolic(
    storage: &LDDManagerRef,
    lps: &LinearProcessSpecification,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
    let lps = ExplicitLinearProcessSpecification::new(lps)?;
    let mut symbolic = SymbolicLps::new(storage, lps)?;

    debug!("{symbolic:?}");

    reachability(storage, &mut symbolic, timing)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use mcrl2::read_lps;
    use merc_utilities::Timing;

    use super::explore_lps_symbolic;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_symbolic_abp() {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");

        let temp_dir = tempfile::tempdir().unwrap();
        let lps_path = temp_dir.path().join("abp.lps");

        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/mCRL2/academic/abp/abp.mcrl2");

        // Run mcrl22lps on the ABP example to get an LPS file.
        let status = Command::new(&mcrl22lps)
            .arg(&spec_path)
            .arg(&lps_path)
            .status()
            .expect("Failed to execute mcrl22lps");
        assert!(status.success(), "mcrl22lps failed with status: {status}");

        let lps = read_lps(lps_path.to_str().expect("LPS path is valid UTF-8")).expect("Failed to read LPS");

        let storage = oxidd::ldd::new_manager(1 << 20, 1 << 20, 1);
        let timing = Timing::new();

        let states = explore_lps_symbolic(&storage, &lps, &timing).expect("Failed to explore LPS");
        let num_of_states = states.len();

        assert_eq!(
            num_of_states, 74,
            "ABP should have 74 reachable states (see examples/lts/abp.aut)"
        );
    }
}
