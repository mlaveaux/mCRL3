#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use mcrl2::Pbes;
    use merc_explore::ExplorationStrategy;
    use merc_io::temp_dir;
    use merc_io::traced_command;
    use merc_vpg::PG;
    use merc_vpg::solve_zielonka;

    use crate::explore_pbes::explore_pbes;
    use crate::explore_srf::explore_srf_pbes;

    // -----------------------------------------------------------------------
    // Core comparison helper
    // -----------------------------------------------------------------------

    /// Explores `pbes` with both the SRF path and the general path, solves each
    /// parity game with Zielonka, and asserts the initial-vertex winner agrees.
    fn assert_general_matches_srf(pbes: &Pbes) {
        use merc_explore::CachingStrategy;

        // Normalise to positive normal form so the SRF converter accepts it.
        let mut normalised = Pbes::from_text(&pbes.to_string()).expect("re-parse failed");
        normalised.normalize();

        let game_srf =
            explore_srf_pbes(&normalised, ExplorationStrategy::Bfs, CachingStrategy::None)
                .expect("SRF exploration failed");
        let game_gen = explore_pbes(&normalised, ExplorationStrategy::Bfs)
            .expect("General exploration failed");

        let (sol_srf, _) = solve_zielonka(&game_srf, false);
        let (sol_gen, _) = solve_zielonka(&game_gen, false);

        assert_eq!(
            sol_srf[0][0],
            sol_gen[0][0],
            "SRF and general explorers disagree on initial-vertex winner"
        );
    }

    fn assert_general_matches_srf_from_text(text: &str) {
        let pbes = Pbes::from_text(text).expect("Failed to parse PBES");
        assert_general_matches_srf(&pbes);
    }

    /// Like [`assert_general_matches_srf_from_text`] but skips if the PBES fails
    /// to parse (e.g. ill-typed expressions from the random generator).
    fn try_assert_general_matches_srf_from_text(text: &str) -> bool {
        match Pbes::from_text(text) {
            Ok(pbes) => {
                assert_general_matches_srf(&pbes);
                true
            }
            Err(_) => false,
        }
    }

    // -----------------------------------------------------------------------
    // Inline unit tests — known formula structures
    // -----------------------------------------------------------------------

    #[test]
    fn test_simple_mu_true() {
        // mu X = val(true); solution: false (cannot prove true iteratively from bottom)
        // Actually mu X = val(true) fixpoint: X = true, so solution = true
        assert_general_matches_srf_from_text("pbes mu X = val(true); init X;");
    }

    #[test]
    fn test_simple_mu_false() {
        assert_general_matches_srf_from_text("pbes mu X = val(false); init X;");
    }

    #[test]
    fn test_simple_nu_true() {
        assert_general_matches_srf_from_text("pbes nu X = val(true); init X;");
    }

    #[test]
    fn test_simple_nu_false() {
        assert_general_matches_srf_from_text("pbes nu X = val(false); init X;");
    }

    #[test]
    fn test_mu_self_loop() {
        // mu X = X; init X — least fixpoint of X = X is false
        assert_general_matches_srf_from_text("pbes mu X = X; init X;");
    }

    #[test]
    fn test_nu_self_loop() {
        // nu X = X; init X — greatest fixpoint of X = X is true
        assert_general_matches_srf_from_text("pbes nu X = X; init X;");
    }

    #[test]
    fn test_and_formula() {
        // nu X = X && X — equivalent to nu X = X, solution true
        assert_general_matches_srf_from_text("pbes nu X = X && X; init X;");
    }

    #[test]
    fn test_or_formula() {
        // mu X = X || X — equivalent to mu X = X, solution false
        assert_general_matches_srf_from_text("pbes mu X = X || X; init X;");
    }

    #[test]
    fn test_two_equations_alternation() {
        // nu X = Y; mu Y = X — alternation depth 1
        assert_general_matches_srf_from_text("pbes nu X = Y; mu Y = X; init X;");
    }

    #[test]
    fn test_and_of_two_pvis() {
        // nu X = Y && Z; nu Y = val(true); nu Z = val(true);
        assert_general_matches_srf_from_text(
            "pbes nu X = Y && Z; nu Y = val(true); nu Z = val(true); init X;",
        );
    }

    #[test]
    fn test_or_of_two_pvis() {
        assert_general_matches_srf_from_text(
            "pbes mu X = Y || Z; mu Y = val(false); mu Z = val(false); init X;",
        );
    }

    #[test]
    fn test_nested_and_or() {
        // Nested formula: nu X = (Y && Z) || W
        assert_general_matches_srf_from_text(
            "pbes nu X = (Y && Z) || W; nu Y = val(true); nu Z = val(true); nu W = val(false); init X;",
        );
    }

    #[test]
    fn test_data_param_bool() {
        // PBES with a Bool parameter
        assert_general_matches_srf_from_text(
            "pbes nu X(b: Bool) = val(b); init X(true);",
        );
    }

    #[test]
    fn test_data_param_nat() {
        assert_general_matches_srf_from_text(
            "pbes mu X(n: Nat) = val(n == 0); init X(1);",
        );
    }

    #[test]
    fn test_data_param_with_pvi() {
        assert_general_matches_srf_from_text(
            "pbes nu X(n: Int) = val(n > 0) || X(n - 1); init X(3);",
        );
    }

    #[test]
    fn test_multiple_equations_data() {
        assert_general_matches_srf_from_text(
            "pbes mu X(n: Int) = val(n == 0) || Y(n); \
             nu Y(n: Int) = val(n > 0) && X(n - 1); \
             init X(2);",
        );
    }

    // -----------------------------------------------------------------------
    // Random PBES tests — use merc_syntax::random_pbes
    // -----------------------------------------------------------------------

    #[test]
    fn test_random_pbes_seeds() {
        use rand::SeedableRng;
        use merc_syntax::random_pbes;

        for seed in 0u64..50 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            // propositional only (no quantifiers, no integers)
            let pbes_ast = random_pbes(&mut rng, 3, 2, 3, false, false);
            let text = pbes_ast.to_string();
            assert_general_matches_srf_from_text(&text);
        }
    }

    #[test]
    fn test_random_pbes_with_integers() {
        use rand::SeedableRng;
        use merc_syntax::random_pbes;

        let mut valid = 0usize;
        for seed in 0u64..50 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let pbes_ast = random_pbes(&mut rng, 3, 2, 3, false, true);
            if try_assert_general_matches_srf_from_text(&pbes_ast.to_string()) {
                valid += 1;
            }
        }
        assert!(valid >= 10, "fewer than 10 well-typed integer PBESes in 50 seeds (got {valid})");
    }

    #[test]
    fn test_random_pbes_with_quantifiers() {
        use rand::SeedableRng;
        use merc_syntax::random_pbes;

        let mut valid = 0usize;
        for seed in 0u64..30 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let pbes_ast = random_pbes(&mut rng, 2, 2, 2, true, true);
            if try_assert_general_matches_srf_from_text(&pbes_ast.to_string()) {
                valid += 1;
            }
        }
        assert!(valid >= 5, "fewer than 5 well-typed quantifier PBESes in 30 seeds (got {valid})");
    }

    // -----------------------------------------------------------------------
    // File-based tests that require MCRL2_PATH
    // -----------------------------------------------------------------------

    fn convert_text_pbes_and_compare(text_pbes_path: &str) {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let txt2pbes = Path::new(&mcrl2_path).join("txt2pbes");
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(text_pbes_path);
        assert!(path.exists(), "file not found: {}", path.display());

        let temp = temp_dir("test_explore_pbes").unwrap();
        let pbes_path = temp.path().join("spec.pbes");

        let status = traced_command(Command::new(&txt2pbes).arg(&path).arg(&pbes_path))
            .expect("txt2pbes failed");
        assert!(status.success());

        let pbes = Pbes::from_file(pbes_path.to_str().unwrap()).expect("Failed to read PBES");
        assert_general_matches_srf(&pbes);
    }

    fn convert_mcrl2_and_compare(spec: &str, formula: &str) {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");
        let lps2pbes = Path::new(&mcrl2_path).join("lps2pbes");

        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(spec);
        let formula_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(formula);
        assert!(spec_path.exists(), "spec not found: {}", spec_path.display());
        assert!(
            formula_path.exists(),
            "formula not found: {}",
            formula_path.display()
        );

        let temp = temp_dir("test_explore_pbes").unwrap();
        let lps_path = temp.path().join("spec.lps");
        let pbes_path = temp.path().join("spec.pbes");

        let status = traced_command(Command::new(&mcrl22lps).arg(&spec_path).arg(&lps_path))
            .expect("mcrl22lps failed");
        assert!(status.success());

        let status = traced_command(
            Command::new(&lps2pbes)
                .arg("-f")
                .arg(&formula_path)
                .arg(&lps_path)
                .arg(&pbes_path),
        )
        .expect("lps2pbes failed");
        assert!(status.success());

        let pbes = Pbes::from_file(pbes_path.to_str().unwrap()).expect("Failed to read PBES");
        assert_general_matches_srf(&pbes);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_a_text_pbes() {
        convert_text_pbes_and_compare("../../../examples/pbes/a.text.pbes");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_b_text_pbes() {
        convert_text_pbes_and_compare("../../../examples/pbes/b.text.pbes");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_c_text_pbes() {
        convert_text_pbes_and_compare("../../../examples/pbes/c.text.pbes");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_par_nodeadlock() {
        convert_mcrl2_and_compare(
            "../../../examples/mCRL2/academic/par/par.mcrl2",
            "../../../examples/mCRL2/academic/par/nodeadlock.mcf",
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_abp() {
        convert_mcrl2_and_compare(
            "../../../examples/mCRL2/academic/abp/abp.mcrl2",
            "../../../examples/mCRL2/academic/abp/infinitely_often_enabled_then_infinitely_often_taken.mcf",
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_dining3_nodeadlock() {
        convert_mcrl2_and_compare(
            "../../../examples/mCRL2/academic/dining/dining3.mcrl2",
            "../../../examples/mCRL2/academic/dining/nodeadlock.mcf",
        );
    }
}
