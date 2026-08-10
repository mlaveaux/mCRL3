//! Lowering conformance tests.
//!
//! For each test spec the pipeline is:
//!   1. Parse with `merc_typecheck` → lower → `Mcrl2DataSpecification`
//!      (pure-Rust `merc_aterm` terms, serialised `OpIdNoIndex` form).
//!   2. Parse the same text with mCRL2's own C++ type-checker via
//!      `DataSpecification::from_string`; its `user_defined_*` accessors return
//!      the serialised form (index stripped: `OpId` → `OpIdNoIndex`).
//!   3. Convert merc's lowered aterms into the shared C++ aterm pool with
//!      `merc_aterm_to_mcrl2`.
//!
//! Because the C++ pool is maximally shared, two structurally identical terms
//! have the *same* address, so conformance is checked by pure structural
//! (address) equality — never by comparing pretty-printed strings.

use std::collections::HashSet;

use mcrl2::ATerm as Mcrl2ATerm;
use mcrl2::ATermList;
use mcrl2::DataSpecification;
use mcrl2::merc_aterm_to_mcrl2;
use merc_aterm::Term as MercTerm;
use merc_data::Mcrl2DataSpecification;
use merc_syntax::UntypedDataSpecification;
use merc_typecheck::DataSpecification as TypecheckedSpec;
use merc_typecheck::NumberEncoding;

// ─── helpers ────────────────────────────────────────────────────────────────

/// Run the full merc typecheck + lowering pipeline on `text`.
///
/// The oracle is built with machine numbers enabled, so number literals are
/// digit chains (`@most_significant_digitNat(0)`) rather than the Appendix-B
/// binary constructors (`@c0`). merc must be asked for the same encoding or
/// the two sides are not comparable.
fn lower(text: &str) -> Mcrl2DataSpecification {
    let untyped = UntypedDataSpecification::parse(text).expect("merc parse failed");
    let mut typed =
        TypecheckedSpec::from_untyped_with(untyped, NumberEncoding::MachineWord).expect("merc typecheck failed");
    typed.lower_data_specification()
}

/// Convert a merc term into the shared C++ aterm pool and return its address.
fn merc_addr<T: Into<merc_aterm::ATerm>>(term: T) -> usize {
    let merc_term = term.into();
    let mcrl2_term = merc_aterm_to_mcrl2(&merc_term.copy());
    mcrl2_term.address() as usize
}

/// The addresses of every element of an mCRL2 oracle `ATermList`.
fn oracle_addrs(list: ATermList<Mcrl2ATerm>) -> Vec<usize> {
    list.iter().map(|t| t.address() as usize).collect()
}

/// Asserts every oracle term is structurally present in merc's lowered output.
///
/// merc appends system-defined declarations after the user ones, so the merc
/// set is a superset of the oracle's user-only set — the correct relation is
/// `oracle ⊆ merc`.
#[track_caller]
fn assert_oracle_subset(section: &str, merc: &HashSet<usize>, oracle: &[usize]) {
    for (i, addr) in oracle.iter().enumerate() {
        assert!(
            merc.contains(addr),
            "{section}: oracle term #{i} is not structurally present in merc's lowered output"
        );
    }
}

// ─── sorts ──────────────────────────────────────────────────────────────────

#[test]
fn test_user_defined_sorts_match_oracle() {
    let text = "sort S;\n     T;\n";

    let lowered = lower(text);
    let oracle = DataSpecification::from_string(text);

    let merc: HashSet<usize> = lowered.sorts().iter().cloned().map(merc_addr).collect();
    let oracle = oracle_addrs(oracle.user_defined_sorts());

    // No system sorts are added to `sorts()`, so this is exact set equality.
    assert_eq!(merc.len(), oracle.len(), "sorts: count mismatch");
    assert_oracle_subset("sorts", &merc, &oracle);
}

// ─── aliases ────────────────────────────────────────────────────────────────

#[test]
fn test_user_defined_aliases_match_oracle() {
    let text = "sort MyNat = Nat;\n";

    let lowered = lower(text);
    let oracle = DataSpecification::from_string(text);

    let merc: HashSet<usize> = lowered.aliases().iter().cloned().map(merc_addr).collect();
    let oracle = oracle_addrs(oracle.user_defined_aliases());

    assert_oracle_subset("aliases", &merc, &oracle);
}

// ─── constructors ───────────────────────────────────────────────────────────

#[test]
fn test_user_defined_constructors_match_oracle() {
    let text = "sort S;\ncons c: S;\n     d: Bool -> S;\n";

    let lowered = lower(text);
    let oracle = DataSpecification::from_string(text);

    let merc: HashSet<usize> = lowered.constructors().iter().cloned().map(merc_addr).collect();
    let oracle = oracle_addrs(oracle.user_defined_constructors());

    assert_oracle_subset("constructors", &merc, &oracle);
}

// ─── mappings ───────────────────────────────────────────────────────────────

#[test]
fn test_user_defined_mappings_match_oracle() {
    let text = "sort S;\ncons c: S;\nmap  f: S -> Bool;\n     g: S # S -> S;\n";

    let lowered = lower(text);
    let oracle = DataSpecification::from_string(text);

    let merc: HashSet<usize> = lowered.mappings().iter().cloned().map(merc_addr).collect();
    let oracle = oracle_addrs(oracle.user_defined_mappings());

    assert_oracle_subset("mappings", &merc, &oracle);
}

// ─── equations (with number-literal and operator lowering) ──────────────────

#[test]
fn test_user_defined_equations_match_oracle() {
    // Exercises operator lowering (`==`) and a number literal (`0 : Nat`).
    let text = "map f: Nat -> Bool;\nvar x: Nat;\neqn f(x) = (x == 0);\n";

    let lowered = lower(text);
    let oracle = DataSpecification::from_string(text);

    let merc: HashSet<usize> = lowered.equations().iter().cloned().map(merc_addr).collect();
    let oracle = oracle_addrs(oracle.user_defined_equations());

    assert_oracle_subset("equations", &merc, &oracle);
}
