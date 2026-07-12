//! Data-specification type-checking tests.
//!
//! The first group is ported from mCRL2's
//! `libraries/data/test/typecheck_test.cpp` and `normalize_sorts_test.cpp`,
//! restricted to the cases that exercise the sort / alias / well-typedness
//! layer that `merc_typecheck` currently implements. The second group is a
//! randomized property test over acyclic alias graphs.

use std::collections::HashSet;

use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_typecheck::DataSpecification;
use merc_typecheck::WellTypedError;
use merc_utilities::random_test;
use rand::Rng;
use rand::RngExt;

/// Type checks `text`, asserting it is accepted (`expect_ok`) or rejected.
#[track_caller]
fn check(text: &str, expect_ok: bool) {
    let spec = UntypedDataSpecification::parse(text).expect("the specification should parse");
    let result = DataSpecification::from_untyped(spec);
    assert_eq!(
        result.is_ok(),
        expect_ok,
        "unexpected type-check result for:\n{text}\nerror: {:?}",
        result.err()
    );
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

#[test]
fn test_struct_with_reused_projection() {
    // A recursive structured sort whose projection `p` is reused across
    // constructors is well-formed.
    check("sort S = struct c(p: Bool) | d(p: Bool, q: S);\n", true);
}

#[test]
fn test_duplicate_sort_conflicting() {
    check("sort S = struct c;\n     S = Nat;\n", false);
}

#[test]
fn test_constructor_and_mapping_same_symbol() {
    // The same symbol `f: S` cannot be declared as both a constructor and a
    // mapping. (mCRL2 additionally rejects `cons f: S; map f: T;` on ambiguity
    // grounds; distinguishing different-sort overloads is overload resolution,
    // which is not implemented yet, so merc currently accepts that.)
    check("sort S;\ncons f: S;\nmap  f: S;\n", false);
}

#[test]
fn test_constructor_overloaded_by_signature() {
    // `f` as a constant of `S` and as a function `S -> T` is allowed.
    check("sort S;\n     T;\ncons f: S;\n     f: S -> T;\n", true);
}

#[test]
fn test_nested_inline_struct() {
    check("sort S = struct t(struct e(Nat));\n", true);
}

#[test]
fn test_cyclic_aliases_direct() {
    check("sort S = U;\n     U = S;\n", false);
}

#[test]
fn test_cyclic_aliases_indirect() {
    check("sort S = U;\n     U = T;\n     T = S;\n", false);
}

#[test]
fn test_function_alias() {
    check(
        "sort Array = Nat -> Nat;\n\
         map  update: Nat # Nat # Array -> Array;\n\
         var  i,n: Nat;\n     f: Array;\n\
         eqn  update(i, n, f)  =  lambda j: Nat. if(i == j, n, f(j));\n",
        true,
    );
}

#[test]
fn test_recursive_function_sort() {
    check("sort G;\n     F = F -> G;\n", false);
}

#[test]
fn test_recursive_function_sort_reverse() {
    check("sort G;\n     F = G -> F;\n", false);
}

// === Alias self-loop table (typecheck_test.cpp:1565-1636, test_sort_aliases) ===
// `alias.rs`'s existing tests already cover several rows of this table
// (direct/indirect cycles, List/FSet/FBag self-loops, struct-boxed
// recursion through List/Set/function-sort, mutual struct recursion); these
// add the rows that were not yet exercised.

#[test]
fn test_bare_set_self_alias_rejected() {
    // A *bare* (non-struct) self-alias through `Set`/`Bag` — unlike `List`,
    // `FSet` and `FBag`, which surface as `AliasCycle` — is caught by the
    // function-sort-loop checker instead, because Set/Bag "set the flag" the
    // same way a function sort does (they are infinite containers).
    match check_err("sort A3 = Set(A3);") {
        WellTypedError::RecursiveAliasThroughFunctionSort { sort } if sort == "A3" => {}
        other => panic!("unexpected error {other}"),
    }
}

#[test]
fn test_bare_bag_self_alias_rejected() {
    match check_err("sort A4 = Bag(A4);") {
        WellTypedError::RecursiveAliasThroughFunctionSort { sort } if sort == "A4" => {}
        other => panic!("unexpected error {other}"),
    }
}

#[test]
fn test_alias_loop_via_list_of_struct() {
    // `B`'s only reference to itself goes through both `List` (a finite,
    // inductively-safe container) and a struct constructor, so it is
    // accepted — a different shape than the existing `struct` wrapping a
    // `List` of itself.
    check("sort B = List(struct f(B)); map g: B; eqn g = [];", true);
}

#[test]
fn test_alias_loop_via_list_of_struct_with_extra_constant() {
    check("sort B = List(struct f(B) | c); map g: B; eqn g = [];", true);
}

#[test]
fn test_struct_constructor_named_like_containing_alias() {
    // A struct constructor sharing its name with the alias it belongs to.
    check("sort B; A11 = struct A11 | B;", true);
}

#[test]
fn test_struct_wrapping_fset_and_fbag_self_recursive() {
    // Struct-boxed recursion through the *finite* containers `FSet`/`FBag`,
    // distinct from the already-tested `Set`/function-sort cases (those hit
    // `RecursiveAliasThroughFunctionSort`; these do not, since FSet/FBag do
    // not set the flag).
    check("sort A14 = struct f(FSet(A14)) | c;", true);
    check("sort A15 = struct f(FSet(A15)) | g(FBag(A15)) | c;", true);
}

// === Non-emptiness fixpoint (Def. 15.1.7) ===

#[test]
fn test_recursive_struct_without_base_case_is_empty() {
    // A single self-recursive constructor with no base case has no finite
    // element — the fixpoint case Def. 15.1.7 exists for, distinct from the
    // already-tested "abstract sort" and "constant constructor" cases.
    // mCRL2: test_recursive_struct_no_base.
    match check_err("sort D = struct f(D);") {
        WellTypedError::EmptySort { sort } if sort == "D" => {}
        other => panic!("unexpected error {other}"),
    }
}

// === Known gaps (bug-candidates): duplicate/shadowed declaration names ===
// Ignored so the suite stays green; each documents a confirmed divergence
// from mCRL2 and encodes the *correct* (mCRL2-matching) behavior, so
// removing `#[ignore]` is the regression test once the gap is closed.

#[test]
#[ignore = "known gap: mCRL2 keys zero-arity constants by name only (add_constant), rejecting \
            any second declaration regardless of sort; merc's signature only dedupes identical \
            overloads and otherwise allows distinct-sort overloads, including nullary ones. \
            mCRL2: test_data_specification_constructor_same_signature"]
fn test_duplicate_constant_different_sort_rejected_cons_cons() {
    check("sort S; T; cons f: S; f: T;", false);
}

#[test]
#[ignore = "known gap: see test_duplicate_constant_different_sort_rejected_cons_cons; here the \
            second declaration is a `map` instead of a `cons`. \
            mCRL2: test_data_specification_constructor_map_same_signature"]
fn test_duplicate_constant_different_sort_rejected_cons_map() {
    check("sort S; T; cons f: S; map f: T;", false);
}

#[test]
#[ignore = "known gap: two different structs each declaring a nullary constructor of the same \
            name (`open`, `closed`) should be rejected for the same reason as \
            test_duplicate_constant_different_sort_rejected_* — merc currently allows it. \
            mCRL2: normalize_sorts_test.cpp test_loop_free_knuth_bendix_completion"]
fn test_cross_struct_duplicate_constant_name_rejected() {
    check(
        "sort front_doorstate = struct open | closed;
         rear_doorstate  = struct open | closed;",
        false,
    );
}

#[test]
#[ignore = "known gap: mCRL2's add_function rejects any user map/cons whose name collides with \
            a system function, regardless of sort (\"Attempt to redeclare a system function\"); \
            merc has no such check and accepts a verbatim redeclaration like this one. No direct \
            typecheck_test.cpp case; derived from mCRL2's typecheck.cpp add_function guard."]
fn test_user_declaration_shadowing_system_conversion_rejected() {
    check("map Nat2Pos: Nat -> Pos;", false);
}

#[test]
fn test_many_aliases_to_nat_and_struct() {
    // Ported from normalize_sorts_test.cpp: many aliases collapsing to `Nat`
    // plus a wide structured sort. mCRL2 used this to catch an exponential
    // normalization; it must stay fast and be accepted here.
    check(
        "sort A_t = Nat; B_t = Nat; C_t = Nat; D_t = Nat; E_t = Nat; F_t = Nat; G_t = Nat;\n\
         H_t = Nat; I_t = Nat; J_t = Nat; K_t = Nat; L_t = Nat; M_t = Nat; N_t = Nat; O_t = Nat;\n\
         S_t = struct s(a: A_t, b: B_t, c: C_t, d: D_t, e: E_t, f: F_t, g: G_t, h: H_t,\n\
                        i: I_t, j: J_t, k: K_t, l: L_t, m: M_t, n: N_t, o: O_t);\n",
        true,
    );
}

/// Picks either one of the already-declared sorts or a built-in sort.
fn random_leaf(rng: &mut impl Rng, earlier: &[String]) -> String {
    const BASICS: [&str; 5] = ["Bool", "Nat", "Pos", "Int", "Real"];
    if !earlier.is_empty() && rng.random_bool(0.6) {
        earlier[rng.random_range(0..earlier.len())].clone()
    } else {
        BASICS[rng.random_range(0..BASICS.len())].to_string()
    }
}

/// Builds a random (non-structured) sort expression over `earlier` sorts and the
/// built-in sorts, up to `depth` container/function nestings.
fn random_sort(rng: &mut impl Rng, earlier: &[String], depth: u32) -> String {
    if depth == 0 || rng.random_bool(0.4) {
        return random_leaf(rng, earlier);
    }
    match rng.random_range(0..3u32) {
        0 => {
            const CONTAINERS: [&str; 5] = ["List", "Set", "Bag", "FSet", "FBag"];
            let container = CONTAINERS[rng.random_range(0..CONTAINERS.len())];
            format!("{container}({})", random_sort(rng, earlier, depth - 1))
        }
        1 => format!(
            "({} -> {})",
            random_leaf(rng, earlier),
            random_sort(rng, earlier, depth - 1)
        ),
        _ => random_leaf(rng, earlier),
    }
}

/// Collects the names of every resolved (nominal) sort in `sort`.
fn collect_resolved_names(sort: &SortExpression, out: &mut Vec<String>) {
    match sort {
        SortExpression::Resolved(name, _) => out.push(name.clone()),
        SortExpression::Complex(_, subsort) => collect_resolved_names(subsort, out),
        SortExpression::Function { domain, range } => {
            collect_resolved_names(domain, out);
            collect_resolved_names(range, out);
        }
        SortExpression::FlattenedFunction { domain, range } => {
            for sort in domain {
                collect_resolved_names(sort, out);
            }
            collect_resolved_names(range, out);
        }
        SortExpression::Product { lhs, rhs } => {
            collect_resolved_names(lhs, out);
            collect_resolved_names(rhs, out);
        }
        SortExpression::Struct { inner } => {
            for constructor in inner {
                for (_, sort) in &constructor.args {
                    collect_resolved_names(sort, out);
                }
            }
        }
        SortExpression::Simple(_) | SortExpression::Reference(_) => {}
    }
}

/// Random acyclic alias graphs must type check, and normalization must fully
/// expand every non-structured alias — no normalized declaration may still
/// refer to one. Because every alias only refers to earlier sorts the graph is a
/// DAG, so there are no cycles and the specification is always well-typed.
#[test]
#[cfg_attr(miri, ignore)]
fn test_random_acyclic_aliases_are_normalized() {
    random_test(100, |rng| {
        let count = rng.random_range(2..8usize);
        let names: Vec<String> = (0..count).map(|i| format!("D{i}")).collect();
        let mut non_struct_aliases: HashSet<String> = HashSet::new();

        let mut sorts = String::from("sort ");
        for (i, name) in names.iter().enumerate() {
            let earlier = &names[0..i];
            match rng.random_range(0..3u32) {
                // Abstract sort.
                0 => sorts.push_str(&format!("{name};\n")),
                // Non-structured alias over earlier sorts.
                1 => {
                    sorts.push_str(&format!("{name} = {};\n", random_sort(rng, earlier, 3)));
                    non_struct_aliases.insert(name.clone());
                }
                // Structured-sort alias (a named representative, kept by normalization).
                _ => {
                    let argument = random_leaf(rng, earlier);
                    sorts.push_str(&format!("{name} = struct c{i}a({argument}) | c{i}b;\n"));
                }
            }
        }

        // Force every sort into a mapping so its normalized form is inspectable.
        let mut maps = String::from("map ");
        for (i, name) in names.iter().enumerate() {
            maps.push_str(&format!("g{i}: {name};\n"));
        }
        let text = format!("{sorts}{maps}");

        let spec = UntypedDataSpecification::parse(&text).unwrap_or_else(|e| panic!("should parse:\n{text}\n{e:?}"));
        let checked =
            DataSpecification::from_untyped(spec).unwrap_or_else(|e| panic!("should type check:\n{text}\n{e:?}"));

        for map in &checked.data_specification().map_declarations {
            let mut resolved = Vec::new();
            collect_resolved_names(&map.sort, &mut resolved);
            for name in resolved {
                assert!(
                    !non_struct_aliases.contains(&name),
                    "normalized sort of {} still refers to non-struct alias {name}:\n{text}",
                    map.identifier
                );
            }
        }
    });
}
