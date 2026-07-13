//! Phase-3 (equation-level) sort-inference tests, ported from mCRL2's
//! `libraries/data/test/typecheck_test.cpp` (the complete active suite; the
//! few cases mCRL2 itself keeps disabled are ported with a comment saying
//! so). mCRL2's cases are usually phrased as a bare data expression under a
//! variable context; each is adapted here into a full data specification
//! (`map .. ; var ..; eqn ..;`), since that is `merc_typecheck`'s only entry
//! point. A case whose sort is only determined *because* of that adaptation
//! (mCRL2 accepts the bare expression with a free sort) says so in a comment.
//!
//! Where a case checks *which* overload or upcast was picked rather than
//! plain accept/reject, the equation assigns the result to a `map` declared
//! with the exact expected sort (`map result: <expected>; eqn result = ..;`):
//! this only type checks if the right-hand side actually resolves to that
//! sort (or a sub-sort of it), so a bare `Ok` is sufficient evidence — no
//! need to reach into the crate's private inference tables.
//!
//! Confirmed divergences from mCRL2 come in two kinds, both marked in place:
//! `#[should_panic]` anchors encode mCRL2's behavior where merc has a *gap*
//! (they fail the moment the gap is fixed, forcing the flip into a plain
//! assertion — see the known-gaps section at the bottom), while divergences
//! in the *permissive* direction — merc's global constraint solver resolves
//! typings mCRL2's local algorithm rejects as ambiguous — assert merc's
//! behavior and cite the mCRL2 verdict in a comment (see "Known divergences"
//! in docs/typecheck.md §7a).

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

// === whr / function-update regressions ===

#[test]
fn test_eqn_set_where() {
    // Historical mCRL2 bug #787: a `whr` inside a set comprehension. Since
    // the binders commit this is genuinely inferred (`Set(Bool)` through the
    // `if` scheme with the `{}` widened `FSet <= Set`), no longer skipped.
    // mCRL2: test_eqn_set_where.
    check_ok(
        "map f_dot: Set(Bool);
         eqn f_dot = if(true, {}, { o: Bool | true whr z = true end });",
    );
}

#[test]
fn test_function_update_chain_without_lambda() {
    // The declared-mapping variant of mCRL2's test_function_updates (whose
    // lambda-base original is ported below), preserving the chained
    // single-argument update sort checking on a named base.
    check_ok(
        "map f: Bool -> Bool; g: Bool -> Bool;
         eqn g = f[true -> false][false -> true];",
    );
}

#[test]
fn test_function_updates() {
    // Function updates on `lambda` bases, incl. a chained update and a
    // mismatched point sort. mCRL2: test_function_updates.
    check_ok("map f: Bool -> Bool; eqn f = (lambda x: Bool. x)[true -> false];");
    check_ok("map f: Bool -> Bool; eqn f = (lambda x: Bool. x)[true -> false][false -> true];");
    check_ok("map f: Nat -> Bool; eqn f = (lambda n: Nat. n mod 2 == 0)[0 -> false];");
    let err = check_err("map f: Bool -> Bool; eqn f = (lambda x: Bool. x)[0 -> false];");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
        "{err}"
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

// === Boolean and numeric literal basics (typecheck_test.cpp test_true..test_one_times_two_plus_three) ===

#[test]
fn test_boolean_operator_basics() {
    // mCRL2: test_true, test_if, test_not, test_and.
    check_ok("map b: Bool; eqn b = true;");
    check_ok("map b: Bool; eqn b = if(true, true, false);");
    check_ok("map b: Bool; eqn b = !true;");
    check_ok("map b: Bool; eqn b = true && false;");
}

#[test]
fn test_number_literal_operator_sorts() {
    // The declared result sort mirrors the sort mCRL2 infers for the bare
    // expression (`+` takes its heterogeneous overloads: a `Pos` on either
    // side yields `Pos`). mCRL2: test_zero, test_minus_one,
    // test_zero_plus_one, test_one_plus_zero, test_zero_plus_zero,
    // test_one_plus_one, test_one_times_two_plus_three.
    check_ok("map n: Nat; eqn n = 0;");
    check_ok("map i: Int; eqn i = -1;");
    check_ok("map p: Pos; eqn p = 0 + 1;");
    check_ok("map p: Pos; eqn p = 1 + 0;");
    check_ok("map n: Nat; eqn n = 0 + 0;");
    check_ok("map p: Pos; eqn p = 1 + 1;");
    check_ok("map p: Pos; eqn p = 1 * 2 + 3;");
}

// === List literals and operations (typecheck_test.cpp test_empty_list..test_head_list_zero_one) ===

#[test]
fn test_empty_list_takes_element_sort_from_use() {
    // mCRL2 accepts the bare `[]` with a free element sort; merc's equation
    // entry point determines it from the left-hand side (the never-determined
    // form is the known-gap anchor test_count_of_empty_list_is_nat below). mCRL2:
    // test_empty_list, test_empty_list_concat.
    check_ok("map l: List(Bool); eqn l = [];");
    check_ok("map l: List(Bool); eqn l = [] ++ [];");
}

#[test]
fn test_empty_list_membership() {
    // The member's sort determines the empty list's element sort through the
    // polymorphic `in` template. mCRL2: test_empty_list_in.
    check_ok("map b: Bool; eqn b = true in [];");
}

#[test]
fn test_list_literal_sorts() {
    // mCRL2: test_list_true_false, test_list_zero, test_list_one_two,
    // test_list_zero_concat_one_two.
    check_ok("map l: List(Bool); eqn l = [true, false];");
    check_ok("map l: List(Nat); eqn l = [0];");
    check_ok("map l: List(Pos); eqn l = [1, 2];");
    check_ok("map l: List(Nat); eqn l = [0] ++ [1, 2];");
}

#[test]
fn test_head_of_list_literal() {
    // mCRL2: test_head_list_zero, test_head_list_zero_one.
    check_ok("map n: Nat; eqn n = head([0]);");
    check_ok("map n: Nat; eqn n = head([0, 1]);");
}

// === Set/bag operations and comprehensions (typecheck_test.cpp test_emptyset..test_bag_comprehension) ===
// The bare `{}`/`{:}` literals are covered by the unit tests
// (test_empty_set_takes_element_sort_from_context and friends).

#[test]
fn test_emptyset_complement() {
    // `!` on sets comes from the polymorphic template; the equation context
    // supplies the element sort mCRL2 leaves free. mCRL2: test_emptyset_complement.
    check_ok("map s: Set(Bool); eqn s = !{};");
}

#[test]
fn test_set_complement_subset_with_context() {
    // The faithful `!{} <= {}` (either side empty) is the known-gap anchor
    // test_emptyset_complement_subset below; with the element sort supplied by a
    // variable, complement-under-subset itself types fine. mCRL2:
    // test_emptyset_complement_subset, test_emptyset_complement_subset_reverse.
    check_ok("map b: Bool; var s: Set(Nat); eqn b = !{} <= s;");
    check_ok("map b: Bool; var s: Set(Nat); eqn b = s <= !{};");
}

#[test]
fn test_emptybag_complement_rejected() {
    // Bags have no complement. mCRL2: test_emptybag_complement.
    let err = check_err("map b: Bag(Bool); eqn b = !{:};");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
        "{err}"
    );
}

#[test]
fn test_set_literal_sorts() {
    // A negative member joins the elements to `Int`. mCRL2:
    // test_set_true_false, test_set_numbers.
    check_ok("map s: FSet(Bool); eqn s = {true, false};");
    check_ok("map s: FSet(Int); eqn s = {1, 2, -7};");
}

#[test]
fn test_set_comprehension_with_mod_body() {
    // mCRL2: test_set_comprehension.
    check_ok("map s: Set(Nat); eqn s = { x: Nat | x mod 2 == 0 };");
}

#[test]
fn test_fset_count_and_pick() {
    // `#` and `pick` through the polymorphic container templates. mCRL2:
    // test_fset_count, test_fset_pick_bool, test_fset_pick_nat.
    check_ok("map n: Nat; eqn n = #{true, false};");
    check_ok("map b: Bool; eqn b = pick({true, false});");
    check_ok("map n: Nat; eqn n = pick({0, 1});");
}

#[test]
fn test_bag_literal_sorts() {
    // mCRL2: test_bag_true_false, test_bag_numbers.
    check_ok("map f: FBag(Bool); eqn f = {true: 1, false: 2};");
    check_ok("map f: FBag(Int); eqn f = {1: 1, 2: 2, -8: 8};");
}

#[test]
fn test_fbag_count_and_pick() {
    // mCRL2: test_fbag_count_numbers, test_fbag_pick_numbers.
    check_ok("map n: Nat; eqn n = #{1: 1, 2: 2, -8: 8};");
    check_ok("map i: Int; eqn i = pick({1: 1, 2: 2, -8: 8});");
}

#[test]
fn test_bag_comprehension_with_lambda_body() {
    // A lambda applied inside the multiplicity body. mCRL2: test_bag_comprehension.
    check_ok("map b: Bag(Nat); eqn b = { x: Nat | (lambda y: Nat. y * y)(x) };");
}

#[test]
fn test_bag_comprehension_body_sorts() {
    // A `Pos` body (`n + 1`) and a `Nat` literal body read as bags; a `Real`
    // body is rejected. mCRL2: test_bag_with_pos_as_argument,
    // test_bag_with_nat_as_argument1, test_bag_with_real_as_argument
    // (test_bag_with_nat_as_argument2 is the unit test
    // test_bag_comprehension_from_numeric_body).
    check_ok("map b: Bag(Pos); eqn b = { n: Pos | n + 1 };");
    check_ok("map b: Bag(Pos); eqn b = { n: Pos | 0 };");
    let err = check_err("map b: Bag(Pos); eqn b = { n: Pos | 2 / 3 };");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
        "{err}"
    );
}

// === Binders: lambda, forall/exists (typecheck_test.cpp test_inline_struct..test_exists_simple) ===

#[test]
fn test_lambda_term_with_wrong_number_of_arguments() {
    // A 2012 mCRL2 core-dump regression: a unary lambda applied to two
    // arguments. mCRL2: test_lambda_term_with_wrong_number_of_arguments.
    let err = check_err("map b: Bool; eqn b = (lambda x: Nat. x)(1, 2) > 0;");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NotAFunction { .. })),
        "{err}"
    );
}

#[test]
fn test_lambda_aliasing() {
    // The inner `f` shadows the outer for the body, so `f(f)` must apply the
    // function to itself, which cannot unify. mCRL2: test_lambda_aliasing.
    let err = check_err(
        "map g: Nat -> (Nat -> Bool) -> Bool;
         eqn g = lambda f: Nat. lambda f: Nat -> Bool. f(f);",
    );
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
        "{err}"
    );
}

#[test]
fn test_lambda_variable_aliasing() {
    // The lambda's `x: S` shadows the declared `x: S -> T`, so `x(x)`
    // applies a non-function. mCRL2: test_lambda_variable_aliasing.
    let err = check_err("sort S; T; map h: S -> Bool; var x: S -> T; eqn h = lambda x: S. x(x);");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NotAFunction { .. })),
        "{err}"
    );
}

#[test]
fn test_forall_nat_vs_int_body() {
    // The body compares a `Nat` variable with a negative literal, joining at
    // `Int`. mCRL2: test_forall_simple_nat_vs_int (test_forall_simple is the
    // unit test test_quantifier_infers_bool_sort).
    check_ok("map b: Bool; eqn b = forall n: Nat. n > -1;");
}

#[test]
fn test_exists_simple() {
    // mCRL2: test_exists_simple.
    check_ok("map b: Bool; eqn b = exists n: Nat. n > 481;");
}

#[test]
fn test_binders_over_anonymous_structs_accepted() {
    // Anonymous `struct` binder sorts defer the whole equation
    // (`EquationTyping::Skipped`), so these stay accepted; the variants that
    // mCRL2 *rejects* are the known-gap anchors below. mCRL2:
    // test_inline_structs_compare, test_forall_structs_compare,
    // test_exists_structs_compare, test_lambda_anonymous_struct.
    check_ok("map b: (struct t) # (struct t) -> Bool; eqn b = lambda x,y: struct t. x == y;");
    check_ok("map b: Bool; eqn b = forall x,y: struct t. x == y;");
    check_ok("map b: Bool; eqn b = exists x,y: struct t. x == y;");
    check_ok("map f: (struct t) -> Bool; g: (struct t) -> Bool; eqn g = lambda x: struct t. f(x);");
}

#[test]
fn test_anonymous_struct_variable_sorts() {
    // Anonymous structs in a `var` block are hoisted, and structurally
    // identical ones share one hoisted declaration, so equal binder sorts
    // compare while a recogniser makes the sorts distinct. mCRL2:
    // test_equal_context, test_not_equal_context.
    check_ok("map b: Bool; var x: struct t?is_t; y: struct t?is_t; eqn b = (x == y);");
    let err = check_err("map b: Bool; var x: struct t; y: struct t?is_t; eqn b = (x == y);");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
        "{err}"
    );
}

// === where clauses (typecheck_test.cpp test_where..test_where_mix_nat_list) ===

#[test]
fn test_where_basic() {
    // mCRL2: test_where.
    check_ok("map p: Pos; eqn p = x + y whr x = 3, y = 10 end;");
}

#[test]
fn test_where_bindings_use_outer_scope_only() {
    // A sibling binding's name is not in scope for a right-hand side, so
    // without an outer declaration the reference is undeclared. mCRL2:
    // test_where_var_one_occurs_in_two, test_where_var_one_and_two_occur_in_two,
    // test_where_var_two_occurs_in_one,
    // test_where_var_one_occurs_in_two_and_vice_versa.
    for spec in [
        "map p: Pos; eqn p = x + y whr x = 3, y = x + 10 end;",
        "map p: Pos; eqn p = x + y whr x = 3, y = x + y + 10 end;",
        "map p: Pos; eqn p = x + y whr x = y + 10, y = 3 end;",
        "map p: Pos; eqn p = x + y whr x = y + 10, y = x + 3 end;",
    ] {
        let err = check_err(spec);
        assert!(
            matches!(err, WellTypedError::Inference(InferenceError::UndeclaredName { .. })),
            "{err}"
        );
    }
}

#[test]
fn test_where_bindings_resolve_against_declared_variables() {
    // With outer declarations, every right-hand side types against the
    // declared variables (not the sibling bindings). mCRL2:
    // test_where_in_context and its four *_in_context variants.
    check_ok("map p: Pos; var x: Pos; y: Nat; eqn p = x + y whr x = 3, y = 0 end;");
    check_ok("map p: Pos; var x: Pos; y: Pos; eqn p = x + y whr x = 3, y = x + 10 end;");
    check_ok("map p: Pos; var x: Pos; y: Pos; eqn p = x + y whr x = 3, y = x + y + 10 end;");
    check_ok("map p: Pos; var x: Pos; y: Nat; eqn p = x + y whr x = y + 10, y = 0 end;");
    check_ok("map p: Pos; var x: Pos; y: Pos; eqn p = x + y whr x = y + 10, y = x + 3 end;");
}

#[test]
fn test_where_mix_nat_list() {
    // mCRL2: test_where_mix_nat_list.
    check_ok("map l: List(Nat); var x: Nat; z: Nat; eqn l = x1 ++ y whr x1 = [0, z], y = [x] end;");
}

#[test]
fn test_where_mix_nat_pos_list_types_globally() {
    // DIVERGES from mCRL2 (permissive direction): mCRL2 types each binding
    // at its minimal sort (x = [0, y]: List(Nat), y = [x]: List(Pos)) and
    // then cannot concatenate them; merc's solver types both bindings at
    // List(Nat) — the `[x]` element upcasts Pos <= Nat — which is a coherent
    // assignment, so the equation is accepted. See "Known divergences" in
    // docs/typecheck.md §7a. mCRL2: test_where_mix_nat_pos_list (rejected).
    check_ok("map l: List(Nat); var x: Pos; y: Nat; eqn l = x ++ y whr x = [0, y], y = [x] end;");
}

// === Bare overloaded names and sort-directed applications (typecheck_test.cpp test_duplicate_function_*) ===

#[test]
fn test_bare_overloaded_name_is_ambiguous() {
    // A bare `f` with several overloads has no unique sort. mCRL2 phrases
    // these as bare expressions with an unknown expected sort; comparing `f`
    // with itself recreates that here (any consistent overload pair ties).
    // mCRL2: test_duplicate_function_different_arity_larger,
    // test_duplicate_function_different_arity_functional,
    // test_duplicate_function_same_arity.
    for spec in [
        "map f: Nat -> Bool; f: Nat # Nat -> Bool; b: Bool; eqn b = (f == f);",
        "map f: Nat -> Nat -> Bool; f: Nat -> Bool; b: Bool; eqn b = (f == f);",
        "map f: Pos -> Nat; f: Nat -> Pos; b: Bool; eqn b = (f == f);",
    ] {
        let err = check_err(spec);
        assert!(
            matches!(
                err,
                WellTypedError::Inference(InferenceError::AmbiguousExpression { .. })
            ),
            "{err}"
        );
    }
}

#[test]
fn test_zero_arity_overload_resolution() {
    // A bare `f` with exactly one zero-arity overload resolves regardless of
    // declaration order, and `f(f)` threads the constant through the unary
    // overload. mCRL2: test_duplicate_function_different_arity (and its
    // _reverse ordering), test_duplicate_function_application (the
    // three-overload variants are
    // test_three_way_arity_overload_nested_application above).
    check_ok("map f: Nat -> Bool; f: Nat; r: Nat; eqn r = f;");
    check_ok("map f: Nat; f: Nat -> Bool; r: Nat; eqn r = f;");
    check_ok("map f: Nat -> Bool; f: Nat; b: Bool; eqn b = f(f);");
}

#[test]
fn test_arity_overloads_through_lambda_application() {
    // The 0/1/2-ary `f` family threaded through an applied lambda. mCRL2:
    // test_duplicate_function_different_arity_horrible_abs.
    check_ok(
        "map f: Nat -> Bool; f: Nat # Nat -> Bool; f: Nat; b: Bool;
         eqn b = f((lambda x: Bool. f)(f(f, f)));",
    );
}

#[test]
fn test_same_arity_overloads_resolved_by_argument() {
    // `f: Pos -> Nat` vs `f: Nat -> Pos`: a `Nat` argument cannot downcast
    // to `Pos`, and for a `Pos` argument the ranked solver prefers the exact
    // application over the upcast one. mCRL2:
    // test_duplicate_function_same_arity_application_{nat,pos}_{constant,variable}.
    check_ok("map f: Pos -> Nat; f: Nat -> Pos; r: Pos; eqn r = f(0);");
    check_ok("map f: Pos -> Nat; f: Nat -> Pos; r: Nat; eqn r = f(1);");
    check_ok("map f: Pos -> Nat; f: Nat -> Pos; r: Pos; var x: Nat; eqn r = f(x);");
    check_ok("map f: Pos -> Nat; f: Nat -> Pos; r: Nat; var x: Pos; eqn r = f(x);");
}

#[test]
fn test_function_application_argument_upcasts() {
    // `f: Nat -> Bool` accepts `Pos` arguments (upcast) and rejects `Int`
    // ones (no downcast), for literals and variables alike. mCRL2:
    // test_function_symbol, test_function_application_{pos,nat,int}_constant,
    // test_function_application_{pos,nat,int}_variable.
    check_ok("map f: Nat -> Bool; g: Nat -> Bool; eqn g = f;");
    check_ok("map f: Nat -> Bool; b: Bool; eqn b = f(1);");
    check_ok("map f: Nat -> Bool; b: Bool; eqn b = f(0);");
    check_ok("map f: Nat -> Bool; b: Bool; var x: Pos; eqn b = f(x);");
    check_ok("map f: Nat -> Bool; b: Bool; var x: Nat; eqn b = f(x);");
    for spec in [
        "map f: Nat -> Bool; b: Bool; eqn b = f(-1);",
        "map f: Nat -> Bool; b: Bool; var x: Int; eqn b = f(x);",
    ] {
        let err = check_err(spec);
        assert!(
            matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
            "{err}"
        );
    }
}

#[test]
fn test_struct_constructor_applications() {
    // A struct constructor `c: Nat -> S` behaves like any mapping under
    // application and upcasting. mCRL2: test_struct_constructor and the five
    // test_struct_constructor_application_* cases.
    check_ok("sort S = struct c(Nat); map g: Nat -> S; eqn g = c;");
    check_ok("sort S = struct c(Nat); map r: S; eqn r = c(1);");
    check_ok("sort S = struct c(Nat); map r: S; eqn r = c(0);");
    check_ok("sort S = struct c(Nat); map r: S; var x: Pos; eqn r = c(x);");
    check_ok("sort S = struct c(Nat); map r: S; var x: Nat; eqn r = c(x);");
    for spec in [
        "sort S = struct c(Nat); map r: S; eqn r = c(-1);",
        "sort S = struct c(Nat); map r: S; var x: Int; eqn r = c(x);",
    ] {
        let err = check_err(spec);
        assert!(
            matches!(err, WellTypedError::Inference(InferenceError::NoTyping { .. })),
            "{err}"
        );
    }
}

#[test]
fn test_data_expressions_struct() {
    // Constructor application through a nested anonymous struct declaration.
    // mCRL2: test_data_expressions_struct.
    check_ok("sort S = struct t(struct e(Nat)); map b: Bool; var x: S; eqn b = (x == t(e(3)));");
}

#[test]
fn test_proper_use_of_int2pos() {
    // mCRL2: test_proper_use_of_int2pos (the whole conversion family is
    // test_int2pos_conversion_family above).
    check_ok("map f: Pos -> Bool; b: Bool; eqn b = f(Int2Pos(-1));");
}

// === Ranked resolution of overloads mCRL2 reports as ambiguous ===
// All four DIVERGE from mCRL2 in the permissive direction: mCRL2 collects
// the possible result sorts of the inner `f` and rejects as ambiguous when
// more than one candidate remains, without ranking; merc's solver ranks the
// exact match above the upcast (and filters through the equation's expected
// sort), leaving a unique best solution. See "Known divergences" in
// docs/typecheck.md §7a.

#[test]
fn test_ambiguous_function_application_recursive() {
    // Resolves with f: Pos -> Int (exact into g) over f: Pos -> Nat (one
    // upcast). mCRL2: test_ambiguous_function_application_recursive (rejected).
    check_ok("map g: Int -> Bool; f: Pos -> Nat; f: Pos -> Int; b: Bool; var x: Pos; eqn b = g(f(x));");
}

#[test]
fn test_ambiguous_function_application_recursive2() {
    // The added g: Int -> Int is filtered out by the equation's Bool
    // left-hand side. mCRL2: test_ambiguous_function_application_recursive2 (rejected).
    check_ok(
        "map g: Int -> Bool; f: Pos -> Nat; f: Pos -> Int; g: Int -> Int; b: Bool; var x: Pos;
         eqn b = g(f(x));",
    );
}

#[test]
fn test_ambiguous_function_application_recursive3() {
    // Resolves with f: Pos -> Nat (argument exact, result upcast) over
    // f: Int -> Int (argument upcast by two). mCRL2:
    // test_ambiguous_function_application_recursive3 (rejected).
    check_ok(
        "map g: Int -> Bool; f: Pos -> Nat; f,g: Int -> Int; b: Bool; var x: Pos;
         eqn b = g(f(x));",
    );
}

#[test]
fn test_ambiguous_function_application_recursive4() {
    // g: Nat -> Int is filtered by the Bool left-hand side; f resolves as in
    // the first case. mCRL2: test_ambiguous_function_application_recursive4 (rejected).
    check_ok(
        "map g: Int -> Bool; f: Pos -> Nat; f: Pos -> Int; g: Nat -> Int; b: Bool; var x: Pos;
         eqn b = g(f(x));",
    );
}

// === Upstream-disabled cases (typecheck_test.cpp keeps these commented out) ===

#[test]
fn test_matching_ambiguous() {
    // Upstream expected accept but keeps the case disabled over
    // pretty-printer reordering (not a typechecking issue): the exact
    // `Pos # Nat` overload outranks `Nat # Nat` for `f(x, y)`, and `f(y, y)`
    // only fits `Nat # Nat`. mCRL2: test_matching_ambiguous (disabled).
    check_ok(
        "map f: Pos # Nat -> Bool; f: Nat # Nat -> Bool;
         var x: Pos; y: Nat; eqn f(x, y) = false;
         var x: Pos; y: Nat; eqn f(y, y) = true;",
    );
}

#[test]
fn test_matching_ambiguous_rhs() {
    // A constant `f: Int` applied to an argument on an equation left-hand
    // side; the original's second equation (`f(x) = 3;`) is dropped since
    // the rejection already fires on the first. mCRL2:
    // test_matching_ambiguous_rhs (disabled, expected reject).
    let err = check_err("map f: Int; var x: Pos; eqn f(x) = -5;");
    assert!(
        matches!(err, WellTypedError::Inference(InferenceError::NotAFunction { .. })),
        "{err}"
    );
}

#[test]
fn test_ambiguous_function_application4_with_expected_sort() {
    // Upstream (disabled) expected `f(x, x)` under an *unknown* expected
    // sort to resolve to `Nat # Nat -> S`, i.e. expand-all-arguments
    // semantics. merc's equation entry always has an expected sort, which
    // determines the overload either way; the unknown-expected reading
    // (lexicographic-nearest would pick `U`, mCRL2 intended `S`) is recorded
    // under G5 in docs/typecheck.md. mCRL2:
    // test_ambiguous_function_application4/4a (disabled).
    check_ok(
        "sort S; T; U; map f: Pos; f: Pos # Nat -> U; f: Nat # Nat -> S; f: Nat # Pos -> T; result: U;
         var x: Pos; y: Nat; eqn result = f(x, x);",
    );
    check_ok(
        "sort S; T; U; map f: Pos; f: Pos # Nat -> U; f: Nat # Nat -> S; f: Nat # Pos -> T; result: S;
         var x: Pos; y: Nat; eqn result = f(x, x);",
    );
}

// === Known gaps (bug-candidates) ===
// Each anchor asserts the *correct* (mCRL2-matching) behavior inside a
// `#[should_panic]` (CI runs with --include-ignored, so `#[ignore]` cannot
// keep the suite green): today the assertion panics because merc diverges,
// and the moment the gap is fixed the test fails, forcing the attribute's
// removal — which turns it into the fix's plain regression test. The
// `expected` substring pins the panic to the intended assertion.

#[test]
#[should_panic(expected = "expected the specification to type check")]
// Known gap: the element sort of an empty container is never constrained, so
// `#[]` reports UnderdeterminedSort instead of Nat; mCRL2
// test_empty_list_size accepts it because `#` (List(S) -> Nat) does not need
// S resolved to compute the result.
fn test_count_of_empty_list_is_nat() {
    check_ok("map n: Nat; eqn n = #[];");
}

#[test]
#[should_panic(expected = "expected the specification to type check")]
// Known gap: same empty-container family as test_count_of_empty_list_is_nat
// — comparing two empty sets never constrains the shared element sort, so
// merc reports UnderdeterminedSort where mCRL2 accepts. mCRL2:
// test_emptyset_complement_subset.
fn test_emptyset_complement_subset() {
    check_ok("map b: Bool; eqn b = !{} <= {};");
}

#[test]
#[should_panic(expected = "expected the specification to type check")]
// Known gap: the reverse form of test_emptyset_complement_subset. mCRL2:
// test_emptyset_complement_subset_reverse.
fn test_emptyset_complement_subset_reverse() {
    check_ok("map b: Bool; eqn b = {} <= !{};");
}

// Known gap behind the next three anchors: an anonymous-struct binder sort
// defers the whole equation (EquationTyping::Skipped), so these
// specifications are accepted unchecked where mCRL2 rejects them (the inline
// struct's constructor `t` is not usable in the body, and `struct t?is_t` is
// a different sort than `struct t`). Rejecting them requires hoisting binder
// structs (G8/Phase 4).

#[test]
#[should_panic(expected = "expected the specification to be rejected")]
// mCRL2: test_inline_struct.
fn test_inline_struct_rejected() {
    check_err("map b: (struct t) -> Bool; eqn b = lambda x: struct t. x == t;");
}

#[test]
#[should_panic(expected = "expected the specification to be rejected")]
// mCRL2: test_inline_struct_recogniser.
fn test_inline_struct_recogniser_rejected() {
    check_err("map b: (struct t?is_t) -> Bool; eqn b = lambda x: struct t?is_t. x == t;");
}

#[test]
#[should_panic(expected = "expected the specification to be rejected")]
// mCRL2: test_inline_structs_compare_recogniser.
fn test_inline_structs_compare_recogniser_rejected() {
    check_err(
        "map b: (struct t?is_t) # (struct t) -> Bool;
         eqn b = lambda x: struct t?is_t, y: struct t. x == y;",
    );
}

