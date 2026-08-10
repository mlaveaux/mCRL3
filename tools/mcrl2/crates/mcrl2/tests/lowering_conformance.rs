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
//!
//! The tests come in two layers. The section tests below check one
//! `user_defined_*` section of a minimal spec in isolation, so a regression
//! points straight at the section that broke. [`assert_round_trips`] then
//! checks *every* section of a spec at once, and the round-trip cases at the
//! bottom of the file run it over the language features a real specification
//! mixes (structs, containers, binders, coercions, conditions, …).

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
    let typed =
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

/// The printed form of every element of an mCRL2 oracle `ATermList`, keyed by
/// address, so a mismatch can name the missing term instead of its pointer.
fn oracle_texts(list: &ATermList<Mcrl2ATerm>) -> Vec<(usize, String)> {
    list.iter().map(|t| (t.address() as usize, t.to_string())).collect()
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

/// As [assert_oracle_subset], but names the offending term in the failure.
#[track_caller]
fn assert_oracle_subset_named(section: &str, spec: &str, merc: &HashSet<usize>, oracle: &[(usize, String)]) {
    let missing: Vec<&str> = oracle
        .iter()
        .filter(|(addr, _)| !merc.contains(addr))
        .map(|(_, text)| text.as_str())
        .collect();

    assert!(
        missing.is_empty(),
        "{section}: {} of {} oracle term(s) are not structurally present in merc's lowered output \
         for the specification:\n{spec}\nmissing:\n  {}",
        missing.len(),
        oracle.len(),
        missing.join("\n  ")
    );
}

/// Type checks and lowers `text` with both merc and the mCRL2 toolset and
/// asserts that *every* user-defined section of the oracle round-trips: each
/// term the toolset produces is structurally present in merc's lowered output,
/// with the sorts section additionally required to match exactly (no
/// system-defined sorts are appended there).
///
/// This is the whole-specification counterpart of the per-section tests above;
/// a spec that passes it is one merc lowers to the same terms the toolset's own
/// binary form holds.
#[track_caller]
fn assert_round_trips(text: &str) {
    let lowered = lower(text);
    let oracle = DataSpecification::from_string(text);

    let merc: HashSet<usize> = lowered.sorts().iter().cloned().map(merc_addr).collect();
    let oracle_sorts = oracle.user_defined_sorts();
    assert_eq!(
        merc.len(),
        oracle_sorts.iter().count(),
        "sorts: count mismatch for the specification:\n{text}"
    );
    assert_oracle_subset_named("sorts", text, &merc, &oracle_texts(&oracle_sorts));

    let merc: HashSet<usize> = lowered.aliases().iter().cloned().map(merc_addr).collect();
    assert_oracle_subset_named("aliases", text, &merc, &oracle_texts(&oracle.user_defined_aliases()));

    let merc: HashSet<usize> = lowered.constructors().iter().cloned().map(merc_addr).collect();
    assert_oracle_subset_named(
        "constructors",
        text,
        &merc,
        &oracle_texts(&oracle.user_defined_constructors()),
    );

    let merc: HashSet<usize> = lowered.mappings().iter().cloned().map(merc_addr).collect();
    assert_oracle_subset_named("mappings", text, &merc, &oracle_texts(&oracle.user_defined_mappings()));

    let merc: HashSet<usize> = lowered.equations().iter().cloned().map(merc_addr).collect();
    assert_oracle_subset_named("equations", text, &merc, &oracle_texts(&oracle.user_defined_equations()));
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

// ─── whole-specification round trips ────────────────────────────────────────
//
// Each case below runs every section of one specification through
// `assert_round_trips`, so a term that merc lowers differently from the toolset
// fails whichever section it belongs to. The cases are grouped by the language
// feature they exercise.

#[test]
fn test_round_trip_abstract_sorts_and_declarations() {
    assert_round_trips(
        "sort S; T;\n\
         cons c: S; d: Bool # S -> S;\n\
         map  f: S -> T; g: S # T -> Bool;\n",
    );
}

#[test]
fn test_round_trip_equation_with_condition() {
    // The condition is a separate `DataEqn` argument, so it round-trips only
    // if merc puts it in the same position the toolset does.
    assert_round_trips(
        "map f: Nat -> Bool; g: Nat -> Bool;\n\
         var x: Nat;\n\
         eqn f(x) -> g(x) = true;\n\
             !f(x) -> g(x) = false;\n",
    );
}

#[test]
fn test_round_trip_boolean_operators() {
    assert_round_trips(
        "map f: Bool # Bool -> Bool;\n\
         var b: Bool; c: Bool;\n\
         eqn f(b, c) = (b && c) || (!b => c);\n",
    );
}

#[test]
fn test_round_trip_number_literals_and_coercions() {
    // `1` infers at `Pos` and is widened to each of `Nat`/`Int`/`Real` by the
    // declared parameter sort, so this pins down every step of the numeric
    // coercion chain against the toolset's own.
    assert_round_trips(
        "map p: Pos -> Bool; n: Nat -> Bool; i: Int -> Bool; r: Real -> Bool;\n\
         map q: Bool;\n\
         eqn q = p(1) && n(1) && i(1) && r(1);\n",
    );
}

#[test]
fn test_round_trip_large_number_literal() {
    // Larger than a machine word, so the literal is a multi-digit
    // `@concat_digit` chain rather than a single `@most_significant_digit`.
    assert_round_trips(
        "map f: Nat -> Bool;\n\
         map q: Bool;\n\
         eqn q = f(18446744073709551621);\n",
    );
}

#[test]
fn test_round_trip_arithmetic() {
    // Kept within `Nat` throughout: `-` on two `Nat`s yields `Int`, which the
    // toolset rejects as the right-hand side of a `Nat` equation, so a
    // subtraction here would test the oracle's tolerance rather than merc's
    // lowering.
    assert_round_trips(
        "map f: Nat # Nat -> Nat;\n\
         var x: Nat; y: Nat;\n\
         eqn f(x, y) = x + y * 2 + x div 2 + x mod 3;\n",
    );
}

#[test]
fn test_round_trip_structured_sort() {
    // Struct desugaring declares the constructors, recognisers and projections
    // on merc's side; the toolset declares the same symbols from the struct.
    assert_round_trips(
        "sort D = struct c1(pr1: Nat, pr2: Bool)?is_c1 | c2?is_c2;\n\
         map f: D -> Bool;\n\
         var d: D;\n\
         eqn f(d) = is_c1(d);\n",
    );
}

#[test]
fn test_round_trip_recursive_structured_sort() {
    assert_round_trips(
        "sort Tree = struct leaf | node(left: Tree, right: Tree);\n\
         map size: Tree -> Nat;\n\
         var l: Tree; r: Tree;\n\
         eqn size(leaf) = 1;\n\
             size(node(l, r)) = size(l) + size(r);\n",
    );
}

#[test]
fn test_round_trip_alias_chain() {
    assert_round_trips(
        "sort A = Nat; B = A; C = List(B);\n\
         map f: C -> Bool;\n",
    );
}

#[test]
fn test_round_trip_lists() {
    assert_round_trips(
        "map f: List(Nat) -> Nat;\n\
         var l: List(Nat); x: Nat;\n\
         eqn f([]) = 0;\n\
             f(x |> l) = x + f(l);\n\
             f([1, 2, 3]) = 6;\n",
    );
}

#[test]
fn test_round_trip_sets_and_bags() {
    assert_round_trips(
        "map f: Set(Nat) -> Bool; g: Bag(Nat) -> Nat;\n\
         var s: Set(Nat); b: Bag(Nat);\n\
         eqn f(s) = 1 in s;\n\
             g(b) = count(1, b);\n",
    );
}

#[test]
fn test_round_trip_finite_set_and_bag_literals() {
    assert_round_trips(
        "map f: FSet(Nat) -> Bool; g: FBag(Nat) -> Bool;\n\
         map q: Bool;\n\
         eqn q = f({1, 2}) && g({1: 2, 3: 4});\n",
    );
}

#[test]
fn test_round_trip_set_comprehension() {
    assert_round_trips(
        "map evens: Set(Nat);\n\
         eqn evens = { x: Nat | x mod 2 == 0 };\n",
    );
}

#[test]
fn test_round_trip_quantifiers() {
    assert_round_trips(
        "map f: List(Nat) -> Bool; g: List(Nat) -> Bool;\n\
         var l: List(Nat);\n\
         eqn f(l) = forall x: Nat. x in l => x > 0;\n\
             g(l) = exists x: Nat. x in l && x == 0;\n",
    );
}

#[test]
fn test_round_trip_lambda_and_higher_order() {
    assert_round_trips(
        "map apply: (Nat -> Nat) # Nat -> Nat;\n\
         map inc: Nat -> Nat;\n\
         var f: Nat -> Nat; x: Nat;\n\
         eqn apply(f, x) = f(x);\n\
             inc = lambda y: Nat. y + 1;\n",
    );
}

#[test]
fn test_round_trip_where_clause() {
    assert_round_trips(
        "map f: Nat -> Nat;\n\
         var x: Nat;\n\
         eqn f(x) = y + y whr y = x + 1 end;\n",
    );
}

#[test]
fn test_round_trip_function_update() {
    assert_round_trips(
        "map f: Nat -> Nat; g: Nat -> Nat;\n\
         eqn g = f[0 -> 1];\n",
    );
}

#[test]
fn test_round_trip_if_and_comparisons() {
    assert_round_trips(
        "sort D;\n\
         cons d: D; e: D;\n\
         map f: D # D -> D;\n\
         var x: D; y: D;\n\
         eqn f(x, y) = if(x == y, x, if(x != y, y, d));\n",
    );
}

#[test]
fn test_round_trip_overloaded_mapping() {
    // One name with several declared sorts: the lowered `OpIdNoIndex` embeds
    // the resolved overload's sort, so picking a different overload than the
    // toolset would show up as a missing equation term.
    assert_round_trips(
        "sort D;\n\
         cons d: D;\n\
         map f: D -> Bool; f: Nat -> Bool; f: D # Nat -> Bool;\n\
         map q: Bool;\n\
         eqn q = f(d) && f(0) && f(d, 0);\n",
    );
}
