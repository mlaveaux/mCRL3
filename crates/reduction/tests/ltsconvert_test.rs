use merc_lts::LTS;
use merc_lts::random_lts;
use merc_lts::reachable_lts;
use merc_lts::read_mcrl2_aut;
use merc_lts::write_mcrl2_aut;
use merc_reduction::Equivalence;
use merc_reduction::compare_lts;
use merc_reduction::reduce_lts;

/// Compares our reduction against mCRL2's `ltsconvert` on random inputs.
pub(crate) fn test_mcrl2_sigref_vs_ltsconvert_impl(name: &str, equivalence: Equivalence, argument: &str) {
    let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
        println!("Skipping test: MCRL2_PATH not set");
        return;
    };

    let ltsconvert = std::path::Path::new(&mcrl2_path).join("ltsconvert");

    // Write the random LTS to a temp file for ltsconvert to process.
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("input.aut");
    let output_path = temp_dir.path().join("output.aut");

    merc_utilities::random_test(100, |rng| {
        let mut files = merc_io::DumpFiles::new(name);

        // ltsconvert only works on the reachable part, so restrict our random LTS as well.
        let lts = merc_lts::reachable_lts(&merc_lts::random_lts::<String, _>(rng, 1000, 3));
        log::info!(
            "Generated random LTS with {} states and {} transitions",
            lts.num_of_states(),
            lts.num_of_transitions()
        );

        {
            let mut input_file = std::fs::File::create(&input_path).unwrap();
            merc_lts::write_mcrl2_aut(&mut input_file, &lts).unwrap();
        }
        files
            .dump("input.aut", |writer| write_mcrl2_aut(writer, &lts))
            .unwrap();

        let status = std::process::Command::new(&ltsconvert)
            .arg(format!("-e{}", argument))
            .arg(&input_path)
            .arg(&output_path)
            .status()
            .expect("Failed to run ltsconvert");
        assert!(status.success(), "ltsconvert failed with status: {status}");

        let ltsconvert_reduced = read_mcrl2_aut(std::fs::File::open(&output_path).unwrap()).unwrap();

        let mut timing = merc_utilities::Timing::new();
        let our_reduced = reduce_lts(lts, equivalence, false, &timing);

        files
            .dump("reduced.aut", |writer| write_mcrl2_aut(writer, &our_reduced))
            .unwrap();
        files
            .dump("ltsconvert_reduced.aut", |writer| {
                write_mcrl2_aut(writer, &ltsconvert_reduced)
            })
            .unwrap();

        assert_eq!(
            our_reduced.num_of_states(),
            ltsconvert_reduced.num_of_states(),
            "Number of states differs",
        );
        assert_eq!(
            our_reduced.num_of_transitions(),
            ltsconvert_reduced.num_of_transitions(),
            "Number of transitions differs",
        );

        assert!(
            compare_lts(
                Equivalence::StrongBisim,
                our_reduced,
                ltsconvert_reduced,
                false,
                &mut timing
            ),
            "The reduced LTSs are not strongly bisimilar"
        );
    });
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_mcrl2_branching_bisim_sigref_vs_ltsconvert() {
    test_mcrl2_sigref_vs_ltsconvert_impl(
        "test_mcrl2_branching_bisim_sigref_vs_ltsconvert",
        Equivalence::BranchingBisim,
        "branching-bisim",
    );
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_mcrl2_divergence_preserving_branching_bisim_sigref_vs_ltsconvert() {
    test_mcrl2_sigref_vs_ltsconvert_impl(
        "test_mcrl2_divergence_preserving_branching_bisim_sigref_vs_ltsconvert",
        Equivalence::BranchingBisimDivergencePreserving,
        "dpbranching-bisim",
    );
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_mcrl2_strong_bisim_sigref_vs_ltsconvert() {
    test_mcrl2_sigref_vs_ltsconvert_impl(
        "test_mcrl2_strong_bisim_sigref_vs_ltsconvert",
        Equivalence::StrongBisim,
        "bisim",
    );
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_mcrl2_weak_bisim_sigref_vs_ltsconvert() {
    test_mcrl2_sigref_vs_ltsconvert_impl(
        "test_mcrl2_weak_bisim_sigref_vs_ltsconvert",
        Equivalence::WeakBisim,
        "weak-bisim",
    );
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_mcrl2_divergence_preserving_weak_bisim_sigref_vs_ltsconvert() {
    test_mcrl2_sigref_vs_ltsconvert_impl(
        "test_mcrl2_divergence_preserving_weak_bisim_sigref_vs_ltsconvert",
        Equivalence::WeakBisimDivergencePreserving,
        "dpweak-bisim",
    );
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_mcrl2_weak_bisimulation_vs_ltsconvert() {
    test_mcrl2_sigref_vs_ltsconvert_impl(
        "test_mcrl2_weak_bisimulation_vs_ltsconvert",
        Equivalence::WeakBisim,
        "weak-bisim",
    );
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn test_mcrl2_divergence_preserving_weak_bisimulation_vs_ltsconvert() {
    test_mcrl2_sigref_vs_ltsconvert_impl(
        "test_mcrl2_divergence_preserving_weak_bisimulation_vs_ltsconvert",
        Equivalence::WeakBisimDivergencePreserving,
        "dpweak-bisim",
    );
}
