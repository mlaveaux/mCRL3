//! Integration tests for [`SymmetryAlgorithm`], exercising only its public API.

use mcrl2::Pbes;
use merc_utilities::test_logger;

use merc_pbes::Permutation;
use merc_pbes::SymmetryAlgorithm;

#[test]
fn test_symmetry_example_a() {
    test_logger();
    let pbes = Pbes::from_text(include_str!("../../../../../examples/pbes/a.text.pbes")).unwrap();

    let cliques = SymmetryAlgorithm::new(&pbes, false).unwrap().cliques();

    assert_eq!(cliques.len(), 0, "There should be no cliques in example a.text.pbes.");
}

#[test]
fn test_symmetry_examples_b() {
    test_logger();
    let pbes = Pbes::from_text(include_str!("../../../../../examples/pbes/b.text.pbes")).unwrap();

    let cliques = SymmetryAlgorithm::new(&pbes, false).unwrap().cliques();

    assert_eq!(cliques.len(), 0, "There should be no cliques in example b.text.pbes.");
}

#[test]
fn test_symmetry_examples_c() {
    test_logger();
    let pbes = Pbes::from_text(include_str!("../../../../../examples/pbes/c.text.pbes")).unwrap();

    let algorithm = SymmetryAlgorithm::new(&pbes, false).unwrap();
    let cliques = algorithm.cliques();

    assert_eq!(
        cliques.len(),
        1,
        "There should be exactly one clique in example c.text.pbes."
    );

    let mut symmetries: Vec<Permutation> = algorithm
        .candidates(false, false)
        .filter(|pi| algorithm.check_symmetry(pi))
        .collect();

    assert_eq!(
        symmetries.len(),
        2,
        "There should be exactly two symmetries in example c.text.pbes."
    );

    // Sort symmetries for consistent comparison
    symmetries.sort_by_key(|pi| pi.to_string());

    // Check that we have the identity permutation
    assert!(
        symmetries.iter().any(|pi| pi.is_identity()),
        "Expected to find the identity permutation"
    );

    // Check that we have the (1 3)(2 4) permutation
    assert!(
        symmetries
            .iter()
            .any(|pi| { pi.value(0) == 2 && pi.value(2) == 0 && pi.value(1) == 3 && pi.value(3) == 1 }),
        "Expected to find the (0 2)(1 3) permutation"
    );
}
