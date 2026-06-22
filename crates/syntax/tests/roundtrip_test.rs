//! Regression tests for parser bugs fixed during the crate review, plus
//! randomized print/parse round-trip (string-fixpoint) property tests.

use std::ops::ControlFlow;

use rand::RngExt;

use merc_syntax::Bound;
use merc_syntax::PbesExpr;
use merc_syntax::ProcExprBinaryOp;
use merc_syntax::ProcessExpr;
use merc_syntax::Span;
use merc_syntax::StateFrm;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::UntypedPbes;
use merc_syntax::UntypedPres;
use merc_syntax::UntypedProcessSpecification;
use merc_syntax::UntypedStateFrmSpec;
use merc_syntax::line_column;
use merc_syntax::make_process_specification;
use merc_syntax::random_lps;
use merc_syntax::random_pbes;
use merc_syntax::visit_statefrm;
use merc_utilities::random_test;

// --- Regression tests for the review fixes -------------------------------------------------------

/// PBES quantifiers used to panic because `forall`/`exists` were registered as
/// prefix operators but only handled in the postfix closure.
#[test]
fn pbes_quantifiers_parse() {
    let pbes = UntypedPbes::parse("pbes mu X = forall n: Nat . val(n < 3) => exists m: Nat . val(m < n); init X;")
        .expect("PBES with quantifiers should parse");

    let formula = &pbes.equations[0].formula;
    assert!(
        matches!(formula, PbesExpr::Quantifier { .. } | PbesExpr::Binary { .. }),
        "unexpected formula: {formula:?}"
    );
}

/// `sum`/`inf`/`sup` state-formula operators used to all collapse to `Bound::Sup`.
#[test]
fn state_formula_bounds_are_distinct() {
    for (input, expected) in [
        ("sum n: Nat . val(n < 3)", Bound::Sum),
        ("inf n: Nat . val(n < 3)", Bound::Inf),
        ("sup n: Nat . val(n < 3)", Bound::Sup),
    ] {
        let spec = UntypedStateFrmSpec::parse(input).expect("state formula should parse");
        match spec.formula {
            StateFrm::Bound { bound, .. } => assert_eq!(bound, expected, "for input {input:?}"),
            other => panic!("expected a Bound for {input:?}, got {other:?}"),
        }
    }
}

/// The process `<<` (until) operator used to hit `unimplemented!`.
#[test]
fn process_until_operator_parses() {
    let spec = UntypedProcessSpecification::parse("init a << b;").expect("`<<` should parse");
    match spec.init.expect("init present") {
        ProcessExpr::Binary { op, .. } => assert_eq!(op, ProcExprBinaryOp::Until),
        other => panic!("expected a binary Until, got {other:?}"),
    }
}

/// A bare `delay` / `yaled` (without `@time`) used to fail to parse.
#[test]
fn bare_delay_and_yaled_parse() {
    assert!(matches!(
        UntypedStateFrmSpec::parse("delay").unwrap().formula,
        StateFrm::Delay(None)
    ));
    assert!(matches!(
        UntypedStateFrmSpec::parse("yaled").unwrap().formula,
        StateFrm::Yaled(None)
    ));
    assert!(matches!(
        UntypedStateFrmSpec::parse("delay@(3)").unwrap().formula,
        StateFrm::Delay(Some(_))
    ));
}

/// The left-merge operator must print as `||_` so that the output reparses.
#[test]
fn left_merge_round_trips() {
    let printed = format!("{}", UntypedProcessSpecification::parse("init a ||_ b;").unwrap());
    assert!(printed.contains("||_"), "left merge should print as ||_: {printed}");
    UntypedProcessSpecification::parse(&printed).expect("printed left merge should reparse");
}

/// A PRES used the `pbes` keyword, dropped infix operators, and mismatched the
/// constant-multiply rules. Exercise a spec that touches all of those.
#[test]
fn pres_specification_parses() {
    let pres = UntypedPres::parse(
        "pres \
           mu X(n: Nat) = (val(n < 3) => X(n)) && eqinf(X(n)) + val(2) * X(n); \
           nu Y = sup m: Nat . (Y + condsm(Y, Y, Y)); \
         init X(0);",
    )
    .expect("PRES should parse");
    assert_eq!(pres.equations.len(), 2);
}

/// `visit_*` must return a `Break` value produced by a nested (non-root) node.
#[test]
fn visitor_breaks_from_nested_node() {
    // The `Y` identifier only appears below the top-level conjunction.
    let spec = UntypedStateFrmSpec::parse("true && (mu X. (X && Y))").unwrap();

    let found = visit_statefrm(&spec.formula, |frm| {
        if let StateFrm::Id(name, _) = frm
            && name == "Y"
        {
            return Ok(ControlFlow::Break(name.clone()));
        }
        Ok(ControlFlow::Continue(()))
    })
    .unwrap();

    assert_eq!(found.as_deref(), Some("Y"), "Break value from a nested node was lost");
}

/// `line_column` used to be `print_location`, which computed the next accumulator
/// value as `current - line.len()`. When `span.start` fell inside the first line
/// (e.g. offset 0), `current - line.len()` underflowed for `usize`, causing a
/// panic in debug builds and silent wrap-around in release.
#[test]
fn line_column_no_underflow() {
    // Single-line: offset 0 is the first character — the old code would attempt
    // `0usize - "hello".len()` = underflow.
    assert_eq!(
        line_column("hello", &Span { start: 0, end: 1 }),
        (1, 1),
        "first character of a single-line string"
    );
    // Interior character on the first line.
    assert_eq!(
        line_column("hello", &Span { start: 4, end: 5 }),
        (1, 5),
        "last character of a single-line string"
    );
}

#[test]
fn line_column_multi_line() {
    // Second line: offset 6 is 'w' (the first character after the '\n' in "hello\n").
    assert_eq!(
        line_column("hello\nworld", &Span { start: 6, end: 7 }),
        (2, 1),
        "first character of second line"
    );
    // Interior character on the second line.
    assert_eq!(
        line_column("hello\nworld", &Span { start: 8, end: 9 }),
        (2, 3),
        "third character of second line"
    );
}

#[test]
fn line_column_past_end_does_not_panic() {
    // Offset past the end of input must not panic (saturating_sub guards this).
    let (line, _col) = line_column("hi", &Span { start: 100, end: 101 });
    assert_eq!(line, 1, "past-end offset resolves to the last line");
}

/// An `EqnSpec` without a `var` declaration section must not emit an empty
/// `var` section when printed. The grammar requires at least one declaration
/// after `var`, so printing an empty `var\n` block produces output that cannot
/// be parsed back.
#[test]
fn eqn_spec_without_variables_no_empty_var_section() {
    let spec = UntypedDataSpecification::parse("eqn true = false;").expect("eqn without var should parse");
    let printed = format!("{spec}");
    assert!(
        !printed.contains("var\n"),
        "empty var section must not be emitted:\n{printed}"
    );
    UntypedDataSpecification::parse(&printed).expect("re-printed form must parse");
}

/// Action declarations with sort arguments used to be printed as `id(s1, s2)`
/// instead of the correct `id: s1 # s2` form. The incorrect form would fail to
/// reparse because the grammar expects a colon-separated sort product.
#[test]
fn act_decl_with_args_prints_colon_hash() {
    let spec = UntypedProcessSpecification::parse("act a: Bool # Nat;")
        .expect("action declaration with sort args should parse");
    let printed = format!("{spec}");
    assert!(
        printed.contains("a: Bool # Nat"),
        "ActDecl with args must use ':' and '#':\n{printed}"
    );
    UntypedProcessSpecification::parse(&printed).expect("printed form must reparse");
}

// --- Randomized print/parse round-trip properties ------------------------------------------------

/// Property: for every generated AST, the printed form parses, and printing the
/// reparsed AST yields exactly the same string (a fixpoint of `parse ∘ display`).
/// This catches Display/grammar mismatches without depending on `PartialEq`
/// surviving parenthesization.

#[test]
#[cfg_attr(miri, ignore)] // Test is too slow under miri
fn random_lps_print_parse_fixpoint() {
    random_test(100, |rng| {
        let spec = random_lps(rng, 4, 3, 0.6);
        let printed = format!("{spec}");
        let reparsed = UntypedProcessSpecification::parse(&printed)
            .unwrap_or_else(|e| panic!("failed to reparse generated LPS:\n{printed}\nerror: {e}"));
        assert_eq!(printed, format!("{reparsed}"), "LPS print/parse not a fixpoint");
    });
}

#[test]
#[cfg_attr(miri, ignore)] // Test is too slow under miri
fn random_process_spec_print_parse_fixpoint() {
    random_test(100, |rng| {
        let use_integers = rng.random_bool(0.5);
        let spec = make_process_specification(rng, 3, 4, use_integers);
        let printed = format!("{spec}");
        let reparsed = UntypedProcessSpecification::parse(&printed)
            .unwrap_or_else(|e| panic!("failed to reparse generated process spec:\n{printed}\nerror: {e}"));
        assert_eq!(
            printed,
            format!("{reparsed}"),
            "process spec print/parse not a fixpoint"
        );
    });
}

#[test]
#[cfg_attr(miri, ignore)] // Test is too slow under miri
fn random_pbes_print_parse_fixpoint() {
    random_test(100, |rng| {
        let use_quantifiers = rng.random_bool(0.5);
        let use_integers = rng.random_bool(0.5);
        let pbes = random_pbes(rng, 3, 4, 4, use_quantifiers, use_integers);
        let printed = format!("{pbes}");
        let reparsed = UntypedPbes::parse(&printed)
            .unwrap_or_else(|e| panic!("failed to reparse generated PBES:\n{printed}\nerror: {e}"));
        assert_eq!(printed, format!("{reparsed}"), "PBES print/parse not a fixpoint");
    });
}
