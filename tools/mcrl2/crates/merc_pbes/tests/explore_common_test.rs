//! Integration tests for the symmetry parameter basis helpers, moved out of
//! `src/explore_common.rs` since they only exercise the crate's public API.

use mcrl2::Pbes;

use merc_pbes::check_parameter_basis;
use merc_pbes::symmetry_parameter_basis;

/// The basis merges the parameters of every equation, and is the same vector
/// each time it is asked for within one process.
///
/// The *order* is deliberately not asserted: `unify_parameters` derives it
/// from term addresses, so it varies between processes (the same PBES yields
/// `[m, n]` in one run and `[n, m]` in another). That is what makes the
/// per-run [`check_parameter_basis`] necessary rather than a length check,
/// and why a `--quotient` generator index only means something within a run.
#[test]
#[cfg_attr(miri, ignore)]
fn parameter_basis_merges_all_equations_and_is_stable() {
    let pbes = Pbes::from_text(
        "pbes nu X(m: Nat) = Y(m) && X(m);
              mu Y(n: Nat) = X(n) || Y(n);
         init X(0);",
    )
    .unwrap();

    let basis = symmetry_parameter_basis(&pbes).unwrap();
    let mut names: Vec<String> = basis.iter().map(|v| v.to_string()).collect();
    names.sort();
    assert_eq!(names, ["m: Nat", "n: Nat"]);

    assert_eq!(
        basis,
        symmetry_parameter_basis(&pbes).unwrap(),
        "the basis must not change between calls, or the generators would index into \
         a different vector than the one they were checked against"
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn parameter_basis_accepts_itself_and_rejects_a_different_vector() {
    let pbes = Pbes::from_text("pbes nu X(m: Nat, n: Nat) = X(n, m);\ninit X(0, 1);").unwrap();
    let basis = symmetry_parameter_basis(&pbes).unwrap();
    assert_eq!(basis.len(), 2);

    check_parameter_basis(&basis, &basis, "test").expect("a vector must be its own basis");

    // Reordered: same parameters, but position k now means a different one,
    // which is exactly what makes a permutation of positions meaningless.
    let swapped: Vec<_> = basis.iter().rev().cloned().collect();
    let error = check_parameter_basis(&basis, &swapped, "test")
        .expect_err("a reordered vector is not the basis")
        .to_string();
    assert!(error.contains("test"), "the error must name the backend: {error}");

    // Truncated, the case a bare length check would also catch.
    assert!(check_parameter_basis(&basis, &basis[..1], "test").is_err());
}
