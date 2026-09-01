//! Tests for [`PbesSpecification::typing_info`]: the span-keyed hover/go-to-definition
//! information accumulated over a checked PBES's `val(...)` expressions, `PropVarInst` arguments,
//! and quantifier binders.

use merc_syntax::UntypedPbes;
use merc_typecheck::PbesSpecification;
use merc_typecheck::ResolvedName;
use merc_typecheck::TypingInfo;

/// Type checks `text` as a PBES, returning its full [`TypingInfo`] (every checked expression,
/// merged).
#[track_caller]
fn typing_for(text: &str) -> TypingInfo {
    let spec = UntypedPbes::parse(text).expect("the specification should parse");
    let mut spec = PbesSpecification::from_untyped(spec).expect("the specification should type check");
    spec.typing_info()
}

/// Finds `needle`'s byte offset in `text` and returns the sort (as displayed text) of the most
/// specific typed node there.
#[track_caller]
fn hover(text: &str, needle: &str) -> String {
    let offset = text
        .find(needle)
        .unwrap_or_else(|| panic!("'{needle}' not found in '{text}'"));
    typing_for(text)
        .at_offset(offset)
        .unwrap_or_else(|| panic!("no typed node at offset {offset} in '{text}'"))
        .sort
        .as_ref()
        .unwrap_or_else(|| panic!("node at offset {offset} in '{text}' has no sort"))
        .to_string()
}

/// As [`hover`], but returns the resolved name instead of the sort.
#[track_caller]
fn resolved_name_at(text: &str, needle: &str) -> ResolvedName {
    let offset = text
        .find(needle)
        .unwrap_or_else(|| panic!("'{needle}' not found in '{text}'"));
    typing_for(text)
        .at_offset(offset)
        .unwrap_or_else(|| panic!("no typed node at offset {offset} in '{text}'"))
        .name
        .clone()
        .unwrap_or_else(|| panic!("node at offset {offset} in '{text}' has no resolved name"))
}

/// The reviewer's shape for the process crate, carried over to PBES: a `PropVarInst` argument
/// yields a non-empty, resolvable `TypingInfo`.
#[test]
fn test_prop_var_inst_argument_hover_reports_declared_sort() {
    assert_eq!(hover("pbes mu X(n: Nat) = val(n == n); init X(1);", "1);"), "Pos");
}

#[test]
fn test_prop_var_inst_self_recursive_argument_goto_def_resolves_to_parameter() {
    let text = "pbes nu X(n: Nat) = val(n == 0) || X(n); init X(0);";
    let name = resolved_name_at(text, "n);");
    assert!(
        matches!(&name, ResolvedName::Variable { name, .. } if name == "n"),
        "expected a Variable resolution to 'n', got {name:?}"
    );
}

/// The declaration span carried by a `Variable` resolution points at the equation's own
/// parameter declaration, not the (self-recursive) occurrence.
#[test]
fn test_prop_var_inst_self_recursive_argument_goto_def_declaration_points_at_parameter() {
    let text = "pbes nu X(n: Nat) = val(n == 0) || X(n); init X(0);";
    let name = resolved_name_at(text, "n);");
    let ResolvedName::Variable { declaration, .. } = &name else {
        panic!("expected a Variable resolution, got {name:?}");
    };
    let declaration = declaration.clone().expect("an equation parameter has a real declaration span");
    assert_eq!(&text[declaration.start..declaration.end], "n");
}

/// A quantifier-bound variable's declaration span points at the `forall`/`exists` binder
/// itself, not the equation's own parameter list.
#[test]
fn test_quantifier_bound_variable_goto_def_declaration_points_at_binder() {
    let text = "pbes mu X = forall n: Nat . val(n == n); init X;";
    let name = resolved_name_at(text, "n == n");
    let ResolvedName::Variable { declaration, .. } = &name else {
        panic!("expected a Variable resolution, got {name:?}");
    };
    let declaration = declaration.clone().expect("a quantifier-bound variable has a real declaration span");
    assert_eq!(&text[declaration.start..declaration.end], "n");
}

/// A quantifier's bound variable is checked (and its typing recorded) inside its own body.
#[test]
fn test_quantifier_bound_variable_hover_reports_declared_sort() {
    assert_eq!(
        hover("pbes mu X = forall n: Nat . val(n == n); init X;", "n == n"),
        "Nat"
    );
}

/// Every branch of a conjunction/disjunction chain contributes its own `val(...)` typing — not
/// just the last one (mirrors the process crate's analogous choice-chain regression test).
#[test]
fn test_every_branch_of_a_conjunction_chain_contributes_typing() {
    let text = "pbes mu X(a: Nat, b: Nat, c: Nat) = val(a == a) && val(b == b) && val(c == c); init X(0, 0, 0);";
    assert_eq!(hover(text, "a == a"), "Nat");
    assert_eq!(hover(text, "b == b"), "Nat");
    assert_eq!(hover(text, "c == c"), "Nat");
}

/// `typing_info` merges in the data specification's own `eqn` typing, not just the PBES
/// expressions' — mirrors [`crate::ProcessSpecification::typing_info`]'s equivalent regression
/// test (a data-`eqn` right-hand side has no PBES formula wrapping it, so only the merge itself
/// can surface it here).
#[test]
fn test_typing_info_merges_the_data_specification_eqn_typing() {
    let text = "map f: Nat; eqn f = 1; pbes mu X = val(f == f); init X;";
    assert_eq!(hover(text, "1;"), "Pos");
}
