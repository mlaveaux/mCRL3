//! Tests for [`ProcessSpecification::typing_info`]: the span-keyed hover/go-to-definition
//! information accumulated over a checked process specification's action arguments,
//! process-instantiation arguments, conditions, time bounds, and `dist` weights.

use merc_syntax::UntypedProcessSpecification;
use merc_typecheck::ProcessSpecification;
use merc_typecheck::ResolvedName;
use merc_typecheck::TypingInfo;

/// Type checks `text` as a process specification, returning its full [`TypingInfo`] (every
/// process-body expression, merged).
#[track_caller]
fn typing_for(text: &str) -> TypingInfo {
    let spec = UntypedProcessSpecification::parse(text).expect("the specification should parse");
    let mut spec = ProcessSpecification::from_untyped(spec).expect("the specification should type check");
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

/// The reviewer's original repro: `a(n)`'s argument yields a non-empty, resolvable `TypingInfo`.
#[test]
fn test_action_argument_hover_reports_declared_sort() {
    assert_eq!(hover("act a: Nat; proc P(n: Nat) = a(n); init P(1);", "n);"), "Nat");
}

#[test]
fn test_action_argument_goto_def_resolves_to_process_parameter() {
    let name = resolved_name_at("act a: Nat; proc P(n: Nat) = a(n); init P(1);", "n);");
    assert!(
        matches!(&name, ResolvedName::Variable { name, .. } if name == "n"),
        "expected a Variable resolution to 'n', got {name:?}"
    );
}

/// The declaration span carried by a `Variable` resolution points at the process's own
/// parameter declaration, not the occurrence — the actual goto-definition target.
#[test]
fn test_action_argument_goto_def_declaration_points_at_process_parameter() {
    let text = "act a: Nat; proc P(n: Nat) = a(n); init P(1);";
    let name = resolved_name_at(text, "n);");
    let ResolvedName::Variable { declaration, .. } = &name else {
        panic!("expected a Variable resolution, got {name:?}");
    };
    let declaration = declaration.clone().expect("a process parameter has a real declaration span");
    assert_eq!(&text[declaration.start..declaration.end], "n");
}

/// A `sum`-bound variable's declaration span points at the `sum` binder itself, not the
/// process's parameter list.
#[test]
fn test_sum_bound_variable_goto_def_declaration_points_at_binder() {
    let text = "act a: Nat; proc P = sum x: Nat . a(x); init P;";
    let name = resolved_name_at(text, "x);");
    let ResolvedName::Variable { declaration, .. } = &name else {
        panic!("expected a Variable resolution, got {name:?}");
    };
    let declaration = declaration.clone().expect("a sum-bound variable has a real declaration span");
    assert_eq!(&text[declaration.start..declaration.end], "x");
}

/// A condition's guard is checked (and its typing recorded) too, not just action arguments.
#[test]
fn test_condition_guard_hover_reports_bool() {
    assert_eq!(hover("act a; proc P(b: Bool) = b -> a; init P(true);", "b ->"), "Bool");
}

/// A real-world-shaped repro: a `+`-joined chain of guarded `sum`-actions (mirroring a reported
/// LSP symptom where only the *final* branch's actions showed up in hover/goto-def). Every
/// branch's own bound variable must resolve, not just the last one.
#[test]
fn test_every_branch_of_a_choice_chain_contributes_typing() {
    let text = "act a: Nat; b: Nat; c: Nat; \
                proc P(n: Nat) = sum x: Nat. a(x) + sum y: Nat. b(y) + sum z: Nat. c(z); \
                init P(0);";
    assert_eq!(hover(text, "x)"), "Nat");
    assert_eq!(hover(text, "y)"), "Nat");
    assert_eq!(hover(text, "z)"), "Nat");

    assert!(matches!(&resolved_name_at(text, "x)"), ResolvedName::Variable { name, .. } if name == "x"));
    assert!(matches!(&resolved_name_at(text, "y)"), ResolvedName::Variable { name, .. } if name == "y"));
    assert!(matches!(&resolved_name_at(text, "z)"), ResolvedName::Variable { name, .. } if name == "z"));
}

/// As [`test_every_branch_of_a_choice_chain_contributes_typing`], for a chain of guarded
/// conditions (`is_x(state) -> ... + is_y(state) -> ... + ...`, the other real-world-shaped
/// repro): every guard's own condition must be checked and typed, not just the last.
#[test]
fn test_every_guard_of_a_condition_chain_contributes_typing() {
    let text = "act a, b, c; \
                map is_a, is_b, is_c: Nat -> Bool; \
                proc P(state: Nat) = is_a(state) -> a + is_b(state) -> b + is_c(state) -> c; \
                init P(0);";
    // The needle starts at `(`, inside the guard's `Application` span but outside its narrower
    // `is_x`/`state` sub-node spans, so `at_offset` resolves to the whole (`Bool`-checked) guard.
    assert_eq!(hover(text, "(state) -> a"), "Bool");
    assert_eq!(hover(text, "(state) -> b"), "Bool");
    assert_eq!(hover(text, "(state) -> c"), "Bool");
}
