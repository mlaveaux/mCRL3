#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use mcrl2::Pbes;
    use merc_explore::CachingStrategy;
    use merc_explore::ExplorationStrategy;
    use merc_io::temp_dir;
    use merc_io::traced_command;
    use merc_vpg::solve_zielonka;

    use crate::explore_srf::parity_game_from_pbes;

    fn pbessolve_result(pbessolve: &Path, pbes_path: &Path) -> bool {
        let output = Command::new(pbessolve)
            .arg(pbes_path)
            .output()
            .expect("Failed to execute pbessolve");
        assert!(
            output.status.success(),
            "pbessolve failed with status: {}",
            output.status
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        if combined.to_lowercase().contains("true") {
            true
        } else if combined.to_lowercase().contains("false") {
            false
        } else {
            panic!(
                "pbessolve produced unexpected output for {}: stdout={stdout:?} stderr={stderr:?}",
                pbes_path.display()
            )
        }
    }

    /// Converts a text PBES with `txt2pbes`, solves it with our parity game solver,
    /// and asserts the result matches `pbessolve`.
    fn compare_text_pbes_with_pbessolve(text_pbes_relative_path: &str) {
        compare_text_pbes_with_pbessolve_caching(text_pbes_relative_path, CachingStrategy::None);
    }

    /// Like [`compare_text_pbes_with_pbessolve`] but explores the PBES with the
    /// given [`CachingStrategy`].
    fn compare_text_pbes_with_pbessolve_caching(text_pbes_relative_path: &str, caching: CachingStrategy) {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let txt2pbes = Path::new(&mcrl2_path).join("txt2pbes");
        let pbessolve = Path::new(&mcrl2_path).join("pbessolve");

        let text_pbes_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(text_pbes_relative_path);
        assert!(
            text_pbes_path.exists(),
            "Text PBES file not found: {}",
            text_pbes_path.display()
        );

        let temp = temp_dir("test_explore_srf").unwrap();
        let pbes_path = temp.path().join("spec.pbes");

        let status = traced_command(Command::new(&txt2pbes).arg(&text_pbes_path).arg(&pbes_path))
            .expect("Failed to execute txt2pbes");
        assert!(status.success(), "txt2pbes failed with status: {status}");

        let reference = pbessolve_result(&pbessolve, &pbes_path);

        let pbes = Pbes::from_file(pbes_path.to_str().unwrap()).expect("Failed to read PBES");
        let game =
            parity_game_from_pbes(&pbes, ExplorationStrategy::Bfs, caching).expect("Failed to build parity game");
        let (solution, _) = solve_zielonka(&game, false);
        let result = solution[0][0];

        assert_eq!(
            result, reference,
            "PBES solution mismatch for {text_pbes_relative_path}: our solver says {result}, pbessolve says {reference}"
        );
    }

    /// Generates a PBES from an mCRL2 spec and a modal formula using `mcrl22lps`
    /// and `lps2pbes`, then compares our solver's result with `pbessolve`.
    fn compare_mcrl2_spec_with_pbessolve(spec_relative_path: &str, formula_relative_path: &str) {
        compare_mcrl2_spec_with_pbessolve_caching(spec_relative_path, formula_relative_path, CachingStrategy::None);
    }

    /// Like [`compare_mcrl2_spec_with_pbessolve`] but explores the PBES with the
    /// given [`CachingStrategy`].
    fn compare_mcrl2_spec_with_pbessolve_caching(
        spec_relative_path: &str,
        formula_relative_path: &str,
        caching: CachingStrategy,
    ) {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");
        let lps2pbes = Path::new(&mcrl2_path).join("lps2pbes");
        let pbessolve = Path::new(&mcrl2_path).join("pbessolve");

        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(spec_relative_path);
        let formula_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(formula_relative_path);
        assert!(spec_path.exists(), "Spec file not found: {}", spec_path.display());
        assert!(
            formula_path.exists(),
            "Formula file not found: {}",
            formula_path.display()
        );

        let temp = temp_dir("test_explore_srf").unwrap();
        let lps_path = temp.path().join("spec.lps");
        let pbes_path = temp.path().join("spec.pbes");

        let status = traced_command(Command::new(&mcrl22lps).arg(&spec_path).arg(&lps_path))
            .expect("Failed to execute mcrl22lps");
        assert!(status.success(), "mcrl22lps failed with status: {status}");

        let status = traced_command(
            Command::new(&lps2pbes)
                .arg("-f")
                .arg(&formula_path)
                .arg(&lps_path)
                .arg(&pbes_path),
        )
        .expect("Failed to execute lps2pbes");
        assert!(status.success(), "lps2pbes failed with status: {status}");

        let reference = pbessolve_result(&pbessolve, &pbes_path);

        let pbes = Pbes::from_file(pbes_path.to_str().unwrap()).expect("Failed to read PBES");
        let game =
            parity_game_from_pbes(&pbes, ExplorationStrategy::Bfs, caching).expect("Failed to build parity game");
        let (solution, _) = solve_zielonka(&game, false);
        let result = solution[0][0];

        assert_eq!(
            result, reference,
            "PBES solution mismatch for {spec_relative_path} with {formula_relative_path}: \
             our solver says {result}, pbessolve says {reference}"
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_a_text_pbes() {
        compare_text_pbes_with_pbessolve("../../../examples/pbes/a.text.pbes");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_b_text_pbes() {
        compare_text_pbes_with_pbessolve("../../../examples/pbes/b.text.pbes");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_c_text_pbes() {
        compare_text_pbes_with_pbessolve("../../../examples/pbes/c.text.pbes");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_par_nodeadlock() {
        compare_mcrl2_spec_with_pbessolve(
            "../../../examples/mCRL2/academic/par/par.mcrl2",
            "../../../examples/mCRL2/academic/par/nodeadlock.mcf",
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_allow_nodeadlock() {
        compare_mcrl2_spec_with_pbessolve(
            "../../../examples/mCRL2/academic/allow/allow.mcrl2",
            "../../../examples/mCRL2/academic/allow/nodeadlock.mcf",
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_block_nodeadlock() {
        compare_mcrl2_spec_with_pbessolve(
            "../../../examples/mCRL2/academic/block/block.mcrl2",
            "../../../examples/mCRL2/academic/block/nodeadlock.mcf",
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_abp() {
        compare_mcrl2_spec_with_pbessolve(
            "../../../examples/mCRL2/academic/abp/abp.mcrl2",
            "../../../examples/mCRL2/academic/abp/infinitely_often_enabled_then_infinitely_often_taken.mcf",
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_cabp() {
        compare_mcrl2_spec_with_pbessolve(
            "../../../examples/mCRL2/academic/cabp/cabp.mcrl2",
            "../../../examples/mCRL2/academic/cabp/infinitely_often_enabled_then_infinitely_often_taken.mcf",
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_abp_local_cache() {
        compare_mcrl2_spec_with_pbessolve_caching(
            "../../../examples/mCRL2/academic/abp/abp.mcrl2",
            "../../../examples/mCRL2/academic/abp/infinitely_often_enabled_then_infinitely_often_taken.mcf",
            CachingStrategy::Local,
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mcrl2_dining3_nodeadlock() {
        compare_mcrl2_spec_with_pbessolve(
            "../../../examples/mCRL2/academic/dining/dining3.mcrl2",
            "../../../examples/mCRL2/academic/dining/nodeadlock.mcf",
        );
    }
}
