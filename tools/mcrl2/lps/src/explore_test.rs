#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Cursor;
    use std::path::Path;
    use std::process::Command;

    use mcrl2::read_lps;
    use merc_explore::CachingStrategy;
    use merc_explore::ExplorationStrategy;
    use merc_io::temp_dir;
    use merc_io::traced_command;
    use merc_lts::AutStream;
    use merc_lts::LTS;
    use merc_lts::LtsBuilderFast;
    use merc_lts::MutexLtsBuilder;
    use merc_lts::StateIndex;
    use merc_lts::read_mcrl2_aut;
    use merc_lts::write_mcrl2_aut;
    use merc_reduction::Equivalence;
    use merc_reduction::compare_lts;
    use merc_syntax::random_lps;
    use merc_utilities::Timing;
    use merc_utilities::random_test;

    use crate::explore_explicit::Mcrl2MultiActionLabel;
    use crate::explore_explicit::explore_lps_explicit_parallel;
    use crate::explore_lps_explicit;

    /// Runs `mcrl22lps` and `lps2lts` on a `.mcrl2` specification, explores the
    /// LPS with `explore_lps_explicit`, and asserts strong bisimilarity between
    /// the two resulting LTSs.
    fn compare_with_lps2lts(spec_relative_path: &str) {
        compare_with_lps2lts_caching(spec_relative_path, CachingStrategy::None);
    }

    /// Like [`compare_with_lps2lts`] but explores the LPS with the given
    /// [`CachingStrategy`].
    fn compare_with_lps2lts_caching(spec_relative_path: &str, strategy: CachingStrategy) {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");
        let lps2lts = Path::new(&mcrl2_path).join("lps2lts");

        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(spec_relative_path);
        assert!(spec_path.exists(), "Spec file not found: {}", spec_path.display());

        let temp_dir = temp_dir("test_explore_lps").unwrap();
        let lps_path = temp_dir.path().join("spec.lps");
        let aut_path = temp_dir.path().join("reference.aut");

        let status = traced_command(Command::new(&mcrl22lps).arg(&spec_path).arg(&lps_path))
            .expect("Failed to execute mcrl22lps");
        assert!(status.success(), "mcrl22lps failed with status: {status}");

        let status =
            traced_command(Command::new(&lps2lts).arg(&lps_path).arg(&aut_path)).expect("Failed to execute lps2lts");
        assert!(status.success(), "lps2lts failed with status: {status}");

        let reference_lts = read_mcrl2_aut(File::open(&aut_path).unwrap()).expect("Failed to read reference .aut");

        let lps = read_lps(lps_path.to_str().unwrap()).expect("Failed to read LPS");
        let mut builder: LtsBuilderFast<Mcrl2MultiActionLabel> = LtsBuilderFast::new(Vec::new(), Vec::new());
        explore_lps_explicit(
            &mut builder,
            &lps,
            strategy,
            ExplorationStrategy::Dfs,
            false,
            &Timing::new(),
        )
        .expect("Failed to explore LPS");
        let result_lts = builder.finish(StateIndex::new(0), false);

        write_mcrl2_aut(
            &mut File::create(temp_dir.path().join("result.aut")).unwrap(),
            &result_lts,
        )
        .expect("Failed to write result .aut");

        assert_eq!(
            reference_lts.num_of_states(),
            result_lts.num_of_states(),
            "State count mismatch for {spec_relative_path} with {strategy:?}"
        );
        assert_eq!(
            reference_lts.num_of_transitions(),
            result_lts.num_of_transitions(),
            "Transition count mismatch for {spec_relative_path} with {strategy:?}"
        );
        assert!(
            compare_lts(
                Equivalence::StrongBisim,
                reference_lts,
                result_lts.relabel(|label| { Ok(label.to_string()) }).unwrap(),
                false,
                &Timing::new(),
            ),
            "LTSs are not strongly bisimilar for {spec_relative_path} with {strategy:?}"
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_abp() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/abp/abp.mcrl2");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_cabp() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/cabp/cabp.mcrl2");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_allow() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/allow/allow.mcrl2");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_block() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/block/block.mcrl2");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_dining3() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/dining/dining3.mcrl2");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_par() {
        compare_with_lps2lts("../../../examples/mCRL2/academic/par/par.mcrl2");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_abp_local_cache() {
        compare_with_lps2lts_caching("../../../examples/mCRL2/academic/abp/abp.mcrl2", CachingStrategy::Local);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_cabp_local_cache() {
        compare_with_lps2lts_caching(
            "../../../examples/mCRL2/academic/cabp/cabp.mcrl2",
            CachingStrategy::Local,
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_dining3_local_cache() {
        compare_with_lps2lts_caching(
            "../../../examples/mCRL2/academic/dining/dining3.mcrl2",
            CachingStrategy::Local,
        );
    }

    /// Generates random LPS specs using [`random_lps`], explores them with
    /// [`explore_lps_explicit`], and asserts strong bisimilarity against `lps2lts`.
    ///
    /// Random specs are written to a temp file and converted with `txt2lps` (no
    /// linearisation step required for the FSM-shaped output of [`random_lps`]).
    fn compare_random_lps_with_lps2lts(strategy: CachingStrategy) {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let txt2lps = Path::new(&mcrl2_path).join("txt2lps");
        let lps2lts = Path::new(&mcrl2_path).join("lps2lts");

        let temp_dir = temp_dir("test_explore_random_lps").unwrap();
        let spec_path = temp_dir.path().join("spec.mcrl2");
        let lps_path = temp_dir.path().join("spec.lps");
        let aut_path = temp_dir.path().join("reference.aut");

        random_test(20, |rng| {
            let spec = random_lps(rng, 5, 3, 0.4);
            std::fs::write(&spec_path, spec.to_string()).expect("Failed to write random LPS spec");

            let status = traced_command(Command::new(&txt2lps).arg(&spec_path).arg(&lps_path))
                .expect("Failed to execute txt2lps");
            assert!(status.success(), "txt2lps failed with status: {status}");

            let status = traced_command(Command::new(&lps2lts).arg(&lps_path).arg(&aut_path))
                .expect("Failed to execute lps2lts");
            assert!(status.success(), "lps2lts failed with status: {status}");

            let reference_lts = read_mcrl2_aut(File::open(&aut_path).unwrap()).expect("Failed to read reference .aut");

            let lps = read_lps(lps_path.to_str().unwrap()).expect("Failed to read LPS");
            let mut builder: LtsBuilderFast<Mcrl2MultiActionLabel> = LtsBuilderFast::new(Vec::new(), Vec::new());
            explore_lps_explicit(
                &mut builder,
                &lps,
                strategy,
                ExplorationStrategy::Dfs,
                false,
                &Timing::new(),
            )
            .expect("Failed to explore LPS");
            let result_lts = builder.finish(StateIndex::new(0), false);

            assert_eq!(
                reference_lts.num_of_states(),
                result_lts.num_of_states(),
                "State count mismatch with {strategy:?}"
            );
            assert_eq!(
                reference_lts.num_of_transitions(),
                result_lts.num_of_transitions(),
                "Transition count mismatch with {strategy:?}"
            );
            assert!(
                compare_lts(
                    Equivalence::StrongBisim,
                    reference_lts,
                    result_lts.relabel(|label| { Ok(label.to_string()) }).unwrap(),
                    false,
                    &Timing::new(),
                ),
                "LTSs are not strongly bisimilar with {strategy:?}"
            );
        });
    }

    /// Explores `lps_path` both sequentially and with the parallel level-
    /// synchronised BFS on several threads, and asserts the two LTSs have equal
    /// state/transition counts and are strongly bisimilar. State numbering is
    /// nondeterministic under parallelism, so the comparison is up to bisimulation.
    fn assert_parallel_matches_sequential(lps_path: &Path) {
        let lps = read_lps(lps_path.to_str().unwrap()).expect("Failed to read LPS");

        // Sequential reference, relabelled to strings for comparison.
        let mut builder: LtsBuilderFast<Mcrl2MultiActionLabel> = LtsBuilderFast::new(Vec::new(), Vec::new());
        explore_lps_explicit(
            &mut builder,
            &lps,
            CachingStrategy::None,
            ExplorationStrategy::Bfs,
            false,
            &Timing::new(),
        )
        .expect("Sequential exploration failed");
        let sequential = builder
            .finish(StateIndex::new(0), false)
            .relabel(|label| Ok(label.to_string()))
            .unwrap();

        // Parallel exploration across several threads, streamed into an
        // in-memory AUT buffer (guarded by a `MutexLtsBuilder`) and read back as
        // a string-labelled LTS.
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut builder = MutexLtsBuilder::new(AutStream::new_mcrl2(&mut buffer).unwrap());
            explore_lps_explicit_parallel(
                &mut builder,
                &lps,
                CachingStrategy::None,
                4,
                false,
                false,
                &Timing::new(),
            )
            .expect("Parallel exploration failed");
        }
        buffer.set_position(0);
        let parallel = read_mcrl2_aut(&mut buffer).expect("Failed to read parallel AUT output");

        assert_eq!(
            parallel.num_of_states(),
            sequential.num_of_states(),
            "Parallel and sequential state counts differ for {}",
            lps_path.display()
        );
        assert_eq!(
            parallel.num_of_transitions(),
            sequential.num_of_transitions(),
            "Parallel and sequential transition counts differ for {}",
            lps_path.display()
        );
        assert!(
            compare_lts(Equivalence::StrongBisim, parallel, sequential, false, &Timing::new()),
            "Parallel and sequential LTSs are not strongly bisimilar for {}",
            lps_path.display()
        );
    }

    /// Runs `mcrl22lps` on a `.mcrl2` specification and asserts that parallel and
    /// sequential explicit exploration agree.
    fn compare_parallel_with_sequential(spec_relative_path: &str) {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");
        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(spec_relative_path);
        assert!(spec_path.exists(), "Spec file not found: {}", spec_path.display());

        let temp_dir = temp_dir("test_explore_lps_parallel").unwrap();
        let lps_path = temp_dir.path().join("spec.lps");

        let status = traced_command(Command::new(&mcrl22lps).arg(&spec_path).arg(&lps_path))
            .expect("Failed to execute mcrl22lps");
        assert!(status.success(), "mcrl22lps failed with status: {status}");

        assert_parallel_matches_sequential(&lps_path);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_parallel_abp() {
        compare_parallel_with_sequential("../../../examples/mCRL2/academic/abp/abp.mcrl2");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_parallel_cabp() {
        compare_parallel_with_sequential("../../../examples/mCRL2/academic/cabp/cabp.mcrl2");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_parallel_dining3() {
        compare_parallel_with_sequential("../../../examples/mCRL2/academic/dining/dining3.mcrl2");
    }

    /// Generates random LPS specs with [`random_lps`] and asserts that parallel
    /// and sequential exploration agree on each.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_explore_parallel_random_lps() {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let txt2lps = Path::new(&mcrl2_path).join("txt2lps");

        let temp_dir = temp_dir("test_explore_random_lps_parallel").unwrap();
        let spec_path = temp_dir.path().join("spec.mcrl2");
        let lps_path = temp_dir.path().join("spec.lps");

        random_test(20, |rng| {
            let spec = random_lps(rng, 5, 3, 0.4);
            std::fs::write(&spec_path, spec.to_string()).expect("Failed to write random LPS spec");

            let status = traced_command(Command::new(&txt2lps).arg(&spec_path).arg(&lps_path))
                .expect("Failed to execute txt2lps");
            assert!(status.success(), "txt2lps failed with status: {status}");

            assert_parallel_matches_sequential(&lps_path);
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_random_lps() {
        compare_random_lps_with_lps2lts(CachingStrategy::None);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_random_lps_local_cache() {
        compare_random_lps_with_lps2lts(CachingStrategy::Local);
    }
}
