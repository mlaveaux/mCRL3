use log::debug;
use oxidd::ldd::LDDManagerRef;

use mcrl2::LinearProcessSpecification;
use merc_symbolic::ExplorationStrategy;
use merc_symbolic::ReachabilityOptions;
use merc_symbolic::ReachabilityResult;
use merc_symbolic::SymbolicLps;
use merc_symbolic::reachability_with_options;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::explore_explicit::ExplicitLinearProcessSpecification;

/// Explore the linear process specification using symbolic reachability.
///
/// The summand machinery (read/write positions, condition enumeration) is reused
/// from the explicit [`ExplicitLinearProcessSpecification`] via the generic
/// [`SymbolicLps`] adapter, so LPS and PBES symbolic exploration share one
/// implementation.
pub fn explore_lps_symbolic(
    storage: &LDDManagerRef,
    lps: &LinearProcessSpecification,
    strategy: ExplorationStrategy,
    detect_deadlocks: bool,
    timing: &Timing,
) -> Result<ReachabilityResult, MercError> {
    let lps = ExplicitLinearProcessSpecification::new(lps)?;
    let mut symbolic = SymbolicLps::new(storage, lps)?;

    debug!("{symbolic:?}");

    let options = ReachabilityOptions {
        strategy,
        detect_deadlocks,
    };
    reachability_with_options(storage, &mut symbolic, &options, timing)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::Path;
    use std::process::Command;

    use mcrl2::read_lps;
    use merc_io::traced_command;
    use merc_symbolic::ExplorationStrategy;
    use merc_symbolic::ReachabilityOptions;
    use merc_symbolic::SatCountCache;
    use merc_symbolic::SymbolicLtsBdd;
    use merc_symbolic::approx_satcount;
    use merc_symbolic::reachability_bdd;
    use merc_symbolic::reachability_with_options;
    use merc_symbolic::read_symbolic_lts;
    use merc_utilities::Timing;
    use merc_utilities::random_test;
    use oxidd::BooleanFunction;

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

        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../examples/mCRL2/academic/abp/abp.mcrl2");

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

        let states = explore_lps_symbolic(&storage, &lps, ExplorationStrategy::default(), false, &timing)
            .expect("Failed to explore LPS")
            .states;
        let num_of_states = states.len();

        assert_eq!(
            num_of_states, 74,
            "ABP should have 74 reachable states (see examples/lts/abp.aut)"
        );
    }

    /// Generates random LPS specs, runs `lpsreach` to produce `.sym` files, then compares three
    /// reachability counts against each other:
    ///
    /// 1. merc-lps LDD path: `explore_lps_symbolic` on the `.lps` directly.
    /// 2. lpsreach `.sym` read by merc, LDD reachability via `reachability_with_options`.
    /// 3. lpsreach `.sym` read by merc, BDD reachability via `reachability_bdd`.
    ///
    /// All three must agree on the number of reachable states.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_random_lps_lpsreach() {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let txt2lps = Path::new(&mcrl2_path).join("txt2lps");
        let lpsreach = Path::new(&mcrl2_path).join("lpsreach");

        let temp_dir = tempfile::tempdir().unwrap();
        let spec_path = temp_dir.path().join("spec.mcrl2");
        let lps_path = temp_dir.path().join("spec.lps");
        let sym_path = temp_dir.path().join("spec.sym");

        let storage = oxidd::ldd::new_manager(1 << 16, 1 << 16, 1);
        let timing = Timing::new();

        random_test(10, |rng| {
            let spec = merc_syntax::random_lps(rng, 4, 2, 0.4);
            std::fs::write(&spec_path, spec.to_string()).expect("Failed to write spec");

            // Compile the random spec to an LPS (no linearisation needed for the FSM-shaped output).
            let status = traced_command(Command::new(&txt2lps).arg(&spec_path).arg(&lps_path))
                .expect("Failed to execute txt2lps");
            assert!(status.success(), "txt2lps failed: {status}");

            // Run lpsreach to produce a .sym file.
            let status = traced_command(Command::new(&lpsreach).arg(&lps_path).arg(&sym_path))
                .expect("Failed to execute lpsreach");
            assert!(status.success(), "lpsreach failed: {status}");

            // 1. merc-lps LDD path: explore the .lps directly via merc's symbolic engine.
            let lps = read_lps(lps_path.to_str().unwrap()).expect("Failed to read LPS");
            let lps_ldd_count = explore_lps_symbolic(&storage, &lps, ExplorationStrategy::default(), false, &timing)
                .expect("Failed to explore LPS symbolically")
                .states
                .len();

            // 2. lpsreach .sym, LDD reachability.
            let mut sym_lts_ldd = read_symbolic_lts(&storage, File::open(&sym_path).expect("Failed to open .sym"))
                .expect("Failed to read .sym (LDD path)");
            let sym_ldd_count =
                reachability_with_options(&storage, &mut sym_lts_ldd, &ReachabilityOptions::default(), &timing)
                    .expect("Failed to run LDD reachability on .sym")
                    .states
                    .len();

            // 3. lpsreach .sym, BDD reachability (fresh read since reachability_with_options mutates the LTS).
            let sym_lts_bdd = read_symbolic_lts(&storage, File::open(&sym_path).expect("Failed to open .sym"))
                .expect("Failed to read .sym (BDD path)");
            let bdd_manager = oxidd::bdd::new_manager(1 << 16, 1 << 16, 1);
            let lts_bdd = SymbolicLtsBdd::from_symbolic_lts(&storage, &bdd_manager, &sym_lts_bdd)
                .expect("Failed to convert .sym to BDD");
            assert!(
                lts_bdd.initial_state().satisfiable(),
                "BDD conversion produced an unsatisfiable initial state"
            );
            let reach_bdd = reachability_bdd(&bdd_manager, &lts_bdd, false).expect("Failed to run BDD reachability");
            let sym_bdd_count = approx_satcount(
                &reach_bdd,
                lts_bdd.state_variables().len() as u32,
                &mut SatCountCache::new(),
            )
            .as_f64() as usize;

            assert_eq!(
                lps_ldd_count, sym_ldd_count,
                "merc-lps LDD ({lps_ldd_count}) and lpsreach .sym LDD ({sym_ldd_count}) counts disagree"
            );
            assert_eq!(
                sym_ldd_count, sym_bdd_count,
                "lpsreach .sym LDD ({sym_ldd_count}) and BDD ({sym_bdd_count}) counts disagree"
            );
        });
    }
}
