
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::Path;
    use std::process::Command;

    use mcrl2::read_lps;
    use merc_explore::CachingStrategy;
    use merc_lts::LTS;
    use merc_lts::LtsBuilderFast;
    use merc_lts::StateIndex;
    use merc_lts::read_mcrl2_aut;
    use merc_reduction::Equivalence;
    use merc_reduction::compare_lts;
    use merc_utilities::Timing;

    use crate::explore_lps_explicit;

    /// Runs `mcrl22lps` and `lps2lts` on a `.mcrl2` specification, explores the
    /// LPS with `explore_lps_explicit`, and asserts strong bisimilarity between
    /// the two resulting LTSs.
    fn compare_with_lps2lts(spec_relative_path: &str) {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");
        let lps2lts = Path::new(&mcrl2_path).join("lps2lts");

        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(spec_relative_path);
        assert!(spec_path.exists(), "Spec file not found: {}", spec_path.display());

        let temp_dir = tempfile::tempdir().unwrap();
        let lps_path = temp_dir.path().join("spec.lps");
        let aut_path = temp_dir.path().join("reference.aut");

        let status = Command::new(&mcrl22lps)
            .arg(&spec_path)
            .arg(&lps_path)
            .status()
            .expect("Failed to execute mcrl22lps");
        assert!(status.success(), "mcrl22lps failed with status: {status}");

        let status = Command::new(&lps2lts)
            .arg(&lps_path)
            .arg(&aut_path)
            .status()
            .expect("Failed to execute lps2lts");
        assert!(status.success(), "lps2lts failed with status: {status}");

        let reference_lts = read_mcrl2_aut(File::open(&aut_path).unwrap())
            .expect("Failed to read reference .aut");

        let lps = read_lps(lps_path.to_str().unwrap()).expect("Failed to read LPS");
        let mut builder: LtsBuilderFast<String> = LtsBuilderFast::new(Vec::new(), Vec::new());
        explore_lps_explicit(&mut builder, &lps, CachingStrategy::None, &Timing::new()).expect("Failed to explore LPS");
        let result_lts = builder.finish(StateIndex::new(0), false);

        assert_eq!(
            reference_lts.num_of_states(),
            result_lts.num_of_states(),
            "State count mismatch for {spec_relative_path}"
        );
        assert_eq!(
            reference_lts.num_of_transitions(),
            result_lts.num_of_transitions(),
            "Transition count mismatch for {spec_relative_path}"
        );
        assert!(
            compare_lts(
                Equivalence::StrongBisim,
                reference_lts,
                result_lts,
                false,
                &mut Timing::new(),
            ),
            "LTSs are not strongly bisimilar for {spec_relative_path}"
        );
    }

    #[test]
    fn test_explore_abp() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/abp/abp.mcrl2");
    }

    #[test]
    fn test_explore_cabp() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/cabp/cabp.mcrl2");
    }

    #[test]
    fn test_explore_allow() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/allow/allow.mcrl2");
    }

    #[test]
    fn test_explore_block() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/block/block.mcrl2");
    }

    #[test]
    fn test_explore_dining3() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/dining/dining3.mcrl2");
    }

    #[test]
    fn test_explore_par() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/par/par.mcrl2");
    }

    #[test]
    fn test_explore_onebit() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/onebit/onebit.mcrl2");
    }
}
