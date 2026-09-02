//! Tests for [`PresSpecification::typing_info`]: the span-keyed hover/go-to-definition
//! information accumulated over a checked PRES's `val(...)` expressions, `PropVarInst` arguments,
//! constant multipliers, and `inf`/`sup`/`sum` binders. Mirrors `pbes_typing_info_test.rs`.

use merc_syntax::UntypedPres;
use merc_typecheck::PresSpecification;
use merc_typecheck::ResolvedName;
use merc_typecheck::TypingInfo;

/// Type checks `text` as a PRES, returning its full [`TypingInfo`] (every checked expression,
/// merged).
#[track_caller]
fn typing_for(text: &str) -> TypingInfo {
    let spec = UntypedPres::parse(text).expect("the specification should parse");
    let mut spec = PresSpecification::from_untyped(spec).expect("the specification should type check");
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

#[test]
fn test_prop_var_inst_argument_hover_reports_declared_sort() {
    assert_eq!(hover("pres mu X(n: Nat) = val(n); init X(1);", "1);"), "Pos");
}

#[test]
fn test_prop_var_inst_self_recursive_argument_goto_def_resolves_to_parameter() {
    let text = "pres nu X(n: Nat) = val(n) || X(n); init X(0);";
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
    let text = "pres nu X(n: Nat) = val(n) || X(n); init X(0);";
    let name = resolved_name_at(text, "n);");
    let ResolvedName::Variable { declaration, .. } = &name else {
        panic!("expected a Variable resolution, got {name:?}");
    };
    let declaration = declaration
        .clone()
        .expect("an equation parameter has a real declaration span");
    assert_eq!(&text[declaration.start..declaration.end], "n");
}

/// A `sum`-bound variable's declaration span points at the `sum` binder itself, not the
/// equation's own parameter list.
#[test]
fn test_bound_variable_goto_def_declaration_points_at_binder() {
    let text = "pres mu X = sum n: Nat . val(n); init X;";
    let name = resolved_name_at(text, "n); init");
    let ResolvedName::Variable { declaration, .. } = &name else {
        panic!("expected a Variable resolution, got {name:?}");
    };
    let declaration = declaration
        .clone()
        .expect("a sum-bound variable has a real declaration span");
    assert_eq!(&text[declaration.start..declaration.end], "n");
}

/// A `sum` binder's bound variable is checked (and its typing recorded) inside its own body.
#[test]
fn test_bound_variable_hover_reports_declared_sort() {
    assert_eq!(hover("pres mu X = sum n: Nat . val(n); init X;", "n); init"), "Nat");
}

/// A constant multiplier's own `val(...)` expression is checked (and its typing recorded)
/// against `Real`.
#[test]
fn test_constant_multiply_hover_reports_declared_sort() {
    assert_eq!(hover("pres mu X(n: Nat) = val(n) * X(n); init X(1);", "n) * X"), "Nat");
}

/// `typing_info` merges in the data specification's own `eqn` typing, not just the PRES
/// expressions' — mirrors [`crate::PbesSpecification::typing_info`]'s equivalent regression test.
#[test]
fn test_typing_info_merges_the_data_specification_eqn_typing() {
    let text = "map f: Nat; eqn f = 1; pres mu X = val(f); init X;";
    assert_eq!(hover(text, "1;"), "Pos");
}

// ─── goto-def: propositional-variable names ─────────────────────────────────
//
// Mirrors `pbes_typing_info_test.rs`'s equivalent section: a PRES equation is never overloaded, so
// `check_prop_var_inst` resolves every `PropVarInst` to exactly one declaration, unconditionally.

/// `init X(1);`'s own name resolves to a `PropositionalVariable`, pointing at its `pres`
/// declaration.
#[test]
fn test_prop_var_inst_name_goto_def_resolves_to_its_declaration() {
    let text = "pres mu X(n: Nat) = val(n); init X(1);";
    let name = resolved_name_at(text, "X(1)");
    let ResolvedName::PropositionalVariable { name, declaration } = &name else {
        panic!("expected a PropositionalVariable resolution, got {name:?}");
    };
    assert_eq!(name, "X");
    let declaration = declaration.clone().expect("a plain `pres` equation has a real span");
    assert_eq!(&text[declaration.start..declaration.end], "X(n: Nat)");
}

/// A self-recursive `PropVarInst` inside the equation's own formula resolves the same way as a use
/// from `init`.
#[test]
fn test_self_recursive_prop_var_inst_name_goto_def_resolves_to_its_declaration() {
    let text = "pres nu X(n: Nat) = val(n) || X(n); init X(0);";
    let name = resolved_name_at(text, "X(n)");
    let ResolvedName::PropositionalVariable { name, declaration } = &name else {
        panic!("expected a PropositionalVariable resolution, got {name:?}");
    };
    assert_eq!(name, "X");
    let declaration = declaration.clone().expect("a plain `pres` equation has a real span");
    assert_eq!(&text[declaration.start..declaration.end], "X(n: Nat)");
}
