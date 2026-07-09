//! Regression tests for defects in `aterm_int.rs` (`ATermInt` / `is_int_term`).

use merc_aterm::ATerm;
use merc_aterm::ATermInt;
use merc_aterm::Symbol;
use merc_aterm::is_int_term;

/// `is_int_term` only compares the head symbol against the reserved
/// `<aterm_int>` symbol, and `Symbol::new("<aterm_int>", 0)` hands out that
/// reserved symbol because the symbol pool shares symbols by name and arity.
/// A constant built from it is stored as a regular nullary term without the
/// annotation field, yet `is_int_term` claims it is an integer term, so the
/// safe `ATermInt::value` reads past the end of the term's allocation.
///
/// The postcondition: for every term `t` accepted as an integer term,
/// `ATermInt::new(t.value())` must be `t` again by maximal sharing.
#[test]
fn test_fake_int_term_value_roundtrip() {
    let fake = ATerm::constant(&Symbol::new("<aterm_int>", 0));
    assert!(
        is_int_term(&fake),
        "the reserved integer symbol is reachable through Symbol::new"
    );

    let fake_int: ATermInt = fake.clone().into();
    let value = fake_int.value();

    assert_eq!(
        ATerm::from(ATermInt::new(value)),
        fake,
        "an integer term must round-trip through its value"
    );
}
