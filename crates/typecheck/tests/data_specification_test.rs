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
