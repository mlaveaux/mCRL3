//! Phase-3 (equation-level) sort-inference tests, ported from mCRL2's
//! `libraries/data/test/typecheck_test.cpp`, restricted to the constructs
//! `merc_typecheck` currently implements: no binders (`lambda`,
//! `forall`/`exists`, `whr` as a free-standing expression) — see G8 in
//! `docs/typecheck.md`. mCRL2's cases are usually phrased as a bare data
//! expression under a variable context; each is adapted here into a full
//! data specification (`map .. ; var ..; eqn ..;`), since that is
//! `merc_typecheck`'s only entry point.
//!
//! Where a case checks *which* overload or upcast was picked rather than
//! plain accept/reject, the equation assigns the result to a `map` declared
//! with the exact expected sort (`map result: <expected>; eqn result = ..;`):
//! this only type checks if the right-hand side actually resolves to that
//! sort (or a sub-sort of it), so a bare `Ok` is sufficient evidence — no
//! need to reach into the crate's private inference tables.

use merc_syntax::UntypedDataSpecification;
use merc_typecheck::DataSpecification;
use merc_typecheck::InferenceError;
use merc_typecheck::WellTypedError;

/// Type checks `text`, asserting it is accepted.
#[track_caller]
fn check_ok(text: &str) {
    let spec = UntypedDataSpecification::parse(text).expect("the specification should parse");
    if let Err(err) = DataSpecification::from_untyped(spec) {
        panic!("expected the specification to type check, got {err}:\n{text}");
    }
}

/// Type checks `text`, returning the error for the caller to match on the
/// specific variant (never the message text, which may change).
#[track_caller]
fn check_err(text: &str) -> WellTypedError {
    let spec = UntypedDataSpecification::parse(text).expect("the specification should parse");
    match DataSpecification::from_untyped(spec) {
        Err(err) => err,
        Ok(_) => panic!("expected the specification to be rejected:\n{text}"),
    }
}

// === Overload disjunction corpus (typecheck_test.cpp:1209-1330) ===
// Unported before this: existing tests only exercise a two-overload
// disjunction over one argument; this corpus adds 4-way sort disjunctions
// over two arguments and arity disjunctions (0/1/2 arguments of the same
// name).

#[test]
fn test_ambiguous_function_picks_unique_zero_arity() {
    // `f` bare can only be the zero-arity overload; the others need arguments.
    // mCRL2: test_ambiguous_function.
    check_ok(
        "sort U; S; T;
         map f: Pos;
             f: Pos # Nat -> U;
             f: Pos # Pos -> S;
             f: Nat # Pos -> T;
             result: Pos;
         eqn result = f;",
    );
}

#[test]
fn test_ambiguous_function_application_picks_by_arg_sorts() {
    // With x: Pos, y: Nat, only one of the four overloads structurally
    // unifies with each argument pattern (Nat cannot downcast to Pos), so
    // each application has a unique resolution despite the shared name `f`.
    // mCRL2: test_ambiguous_function_application1/2/3.
    check_ok(
        "sort U; S; T;
         map f: Pos;
             f: Pos # Nat -> U;
             f: Pos # Pos -> S;
             f: Nat # Pos -> T;
             result: S;
         var x: Pos; y: Nat;
         eqn result = f(x, x);",
    );
    check_ok(
        "sort U; S; T;
         map f: Pos;
             f: Pos # Nat -> U;
             f: Pos # Pos -> S;
             f: Nat # Pos -> T;
             result: U;
         var x: Pos; y: Nat;
         eqn result = f(x, y);",
    );
    check_ok(
        "sort U; S; T;
         map f: Pos;
             f: Pos # Nat -> U;
             f: Pos # Pos -> S;
             f: Nat # Pos -> T;
             result: T;
         var x: Pos; y: Nat;
         eqn result = f(y, x);",
    );
}

#[test]
fn test_ambiguous_function_application_order_independent() {
    // Same overload set and call as `application1`, but declared in a
    // different order; the solver must pick the same overload (`S`)
    // regardless. mCRL2: test_ambiguous_function_application5.
    check_ok(
        "sort S; T; U;
         map f: Pos;
             f: Nat # Nat -> S;
             f: Nat # Pos -> T;
             f: Pos # Nat -> U;
             result: S;
         var x: Pos; y: Nat;
         eqn result = f(x, x);",
    );
}

#[test]
fn test_three_way_arity_overload_nested_application() {
    // `f` is overloaded 0/1/2-ary; a 3-way *arity* disjunction, distinct from
    // the 4-way *sort* disjunction above. mCRL2:
    // test_duplicate_function_different_arity_horrible[_app1/_app2].
    check_ok(
        "map f: Nat -> Bool;
             f: Nat # Nat -> Bool;
             f: Nat;
             result: Nat;
         eqn result = f;",
    );
    check_ok(
        "map f: Nat -> Bool;
             f: Nat # Nat -> Bool;
             f: Nat;
             result: Bool;
         eqn result = f(f);",
    );
    check_ok(
        "map f: Nat -> Bool;
             f: Nat # Nat -> Bool;
             f: Nat;
             result: Bool;
         eqn result = f(f, f);",
    );
}

#[test]
fn test_self_application_through_constant_and_function_overload() {
    // `f` overloaded as a constant `S` and as `S -> T`; applying the constant
    // overload to itself resolves to `T`. mCRL2:
    // test_data_expressions_different_signature.
    check_ok(
        "sort S; T;
         cons f: S;
              f: S -> T;
         map result: T;
         eqn result = f(f);",
    );
}

// === Numeric upcast / list literal join ===

#[test]
fn test_upcast_pos_plus_nat_via_variables() {
    // `+` and `==` over declared *variables* rather than a literal on one
    // side, as the existing literal-focused tests use. `Pos # Nat -> Pos` is
    // a direct Appendix-B overload here, no upcast needed. mCRL2:
    // test_upcast_pos2nat.
    check_ok(
        "map result: Pos;
         var x: Pos; y: Nat;
         eqn result = x + y;",
    );
    check_ok(
        "map result: Bool;
         var x: Pos; y: Nat;
         eqn result = (x == y);",
    );
}

#[test]
fn test_list_literal_mixed_nat_pos_joins_to_nat() {
    // mCRL2: test_list_nat_pos, test_list_pos_nat.
    check_ok("map l: List(Nat); eqn l = [0, 1, 2];");
    check_ok("map l: List(Nat); eqn l = [1, 0, 2];");
}

#[test]
fn test_list_concat_variable_upcast() {
    // A declared `List(Nat)`/`List(Pos)` variable concatenated with a
    // literal list stays at the variable's sort. mCRL2:
    // test_list_nat_concat_one_two, test_list_pos_concat_one_two.
    check_ok("map r: List(Nat); var l: List(Nat); eqn r = l ++ [1, 2];");
    check_ok("map r: List(Pos); var l: List(Pos); eqn r = l ++ [1, 2];");
}

#[test]
fn test_list_concat_asymmetric_upcast() {
    // `[0] ++ l` succeeds when `l: List(Nat)` (the literal upcasts), but not
    // when `l: List(Pos)` (the literal `0` cannot downcast). mCRL2:
    // test_list_zero_concat_list_nat, test_list_zero_concat_list_pos.
    check_ok("map r: List(Nat); var l: List(Nat); eqn r = [0] ++ l;");
    let err = check_err("map r: List(Pos); var l: List(Pos); eqn r = [0] ++ l;");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
        "{err}"
    );
}

#[test]
fn test_list_mismatched_variable_sorts_rejected() {
    // `List` has no sub-sort relation between element sorts (unlike
    // `FSet(S) <= Set(S)`), so `List(Pos)` and `List(Nat)` are simply
    // incomparable, both under `++` and `==`. mCRL2:
    // test_list_pos_concat_list_nat, test_list_is_list_nat.
    let err = check_err("map r: List(Nat); var x: List(Pos); y: List(Nat); eqn r = x ++ y;");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
        "{err}"
    );
    let err = check_err("map b: Bool; var x: List(Pos); y: List(Nat); eqn b = (x == y);");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
        "{err}"
    );
}

// === Appendix-B boundary cases (book-derived, no direct typecheck_test.cpp line) ===

#[test]
fn test_fbag_literal_widens_to_bag_at_use() {
    // The `Bag` analogue of the already-tested `FSet <= Set` widening; `Bag`
    // members are (value, multiplicity) pairs, a different code path.
    check_ok("map b: Bag(Nat); eqn b = {0: 2, 1: 3};");
}

#[test]
fn test_exp_operator_sort() {
    // `exp: Pos # Nat -> Pos` needs one upcast (the exponent); `exp: Nat #
    // Nat -> Nat` would need two, so the ranked solver prefers the former.
    check_ok("map p: Pos; eqn p = exp(2, 3);");
}

#[test]
fn test_mod_upcasts_positive_dividend_to_nat() {
    // `mod: Nat # Pos -> Nat` is the only overload; a `Pos` dividend upcasts.
    check_ok("map n: Nat; var x: Pos; eqn n = x mod 2;");
}

#[test]
fn test_div_over_int_stays_int() {
    check_ok("map r: Int; var x: Int; eqn r = x div 2;");
}

#[test]
fn test_int2pos_conversion_family() {
    // Exercises every downcast conversion name at once, a regression net for
    // the basic-sort system signature. mCRL2: test_proper_use_of_int2pos1.
    check_ok(
        "map fpos: Pos -> Bool;
             fnat: Nat -> Bool;
             fint: Int -> Bool;
             result: Bool;
         eqn result = fpos(Nat2Pos(0)) && fpos(Int2Pos(-1)) && fpos(Real2Pos(1 / 2)) &&
                      fnat(Int2Nat(-1)) && fnat(Real2Nat(1 / 2)) &&
                      fint(Real2Int(1 / 2));",
    );
}

#[test]
fn test_avoidance_of_possible_types_regression() {
    // Historical mCRL2 regression: a stale "PossibleTypes([Nat,Int,Real])"
    // sort for `#` used to leak past the `==` scheme. mCRL2:
    // test_avoidance_of_possible_types.
    check_ok("map result: Bool; eqn result = (#[0, 1] == -1);");
}

// === Skipped-construct regression anchors ===
// The overall specification must stay accepted only because the construct
// inside is deferred (`EquationTyping::Skipped`, G8); a bare `Ok` does not
// distinguish "correctly skipped" from "wrongly skipped", but it does catch
// a regression where the equation became checked and rejected.

#[test]
fn test_eqn_set_where_is_skipped_not_rejected() {
    // Historical mCRL2 bug #787. mCRL2: test_eqn_set_where.
    check_ok(
        "map f_dot: Set(Bool);
         eqn f_dot = if(true, {}, { o: Bool | true whr z = true end });",
    );
}

#[test]
fn test_function_update_chain_without_lambda() {
    // Reworks mCRL2's test_function_updates (which chains updates on a
    // `lambda` base, out of scope here) with a declared mapping instead,
    // preserving the chained single-argument update sort checking.
    check_ok(
        "map f: Bool -> Bool; g: Bool -> Bool;
         eqn g = f[true -> false][false -> true];",
    );
}

// === Product-domain / multi-parameter matching ===

#[test]
fn test_matching_multi_param_distinct_sorts() {
    // Two-parameter declaration-sort matching over a real product domain,
    // distinct from the single-parameter cases already tested. mCRL2:
    // test_matching.
    check_ok(
        "map f: Pos # Nat -> Bool;
         var x: Pos; y: Nat;
         eqn f(x, y) = true;",
    );
}

#[test]
fn test_matching_repeated_variable_non_strict() {
    // The same variable filling two parameter positions of different
    // declared sorts; `x` upcasts into the `Nat` slot. mCRL2:
    // test_matching_non_strict.
    check_ok(
        "map f: Pos # Nat -> Bool;
         var x: Pos;
         eqn f(x, x) = true;",
    );
}

#[test]
fn test_aliased_list_of_list_equality() {
    // Alias normalization reaching through two nested container levels
    // feeding the `==` scheme. mCRL2: test_aliases.
    check_ok(
        "sort B; A = List(List(B)); C = List(B);
         map result: Bool;
         var f: A; g: List(C);
         eqn result = (f == g);",
    );
}

#[test]
fn test_ambiguous_projection_function_resolves() {
    // mCRL2's own checker rejects this (comment: \"shows an ambiguous
    // projection function that cannot be resolved with the current
    // typechecker ... should be enabled with a new typechecker\") — merc's
    // constraint-based solver is that new typechecker: `pi_1` is overloaded
    // across two struct alternatives (`T1(pi_1: T)`, `T2(pi_1: S)`), and
    // `IS_T1(p)` in the same conjunct disambiguates which one applies.
    // mCRL2: test_ambiguous_projection_function (typecheck_test.cpp).
    check_ok(
        "sort S;
             T = struct T0 | T1(pi_1: T)?IS_T1 | T2(pi_1: S)?IS_T2;
         map R: T -> Bool;
             result: Bool;
         var p: T;
         eqn result = R(pi_1(p)) && IS_T1(p);",
    );
}

// === Known gaps (bug-candidates) ===
// Ignored so the suite stays green; each documents a confirmed divergence
// from mCRL2 and encodes the *correct* (mCRL2-matching) behavior, so
// removing `#[ignore]` is the regression test once the gap is closed.

#[test]
#[ignore = "known gap: the element sort of an empty container is never constrained, so `#[]` \
            reports UnderdeterminedSort instead of Nat; mCRL2 test_empty_list_size accepts it \
            because `#` (List(S) -> Nat) does not need S resolved to compute the result"]
fn test_count_of_empty_list_is_nat() {
    check_ok("map n: Nat; eqn n = #[];");
}
