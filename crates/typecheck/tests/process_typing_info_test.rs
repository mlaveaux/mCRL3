//! Tests for [`ProcessSpecification::typing_info`]: the span-keyed hover/go-to-definition
//! information accumulated over a checked process specification's action arguments,
//! process-instantiation arguments, conditions, time bounds, and `dist` weights.

use merc_syntax::Span;
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
        .as_ref()
        .unwrap_or_else(|| panic!("node at offset {offset} in '{text}' has no sort (e.g. an action/process name)"))
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
    let declaration = declaration
        .clone()
        .expect("a process parameter has a real declaration span");
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
    let declaration = declaration
        .clone()
        .expect("a sum-bound variable has a real declaration span");
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
    // Looked up directly by span rather than through `hover`/`at_offset`: `is_x(state)` packs its
    // identifier, `(`, argument and `)` with no gap, so every offset inside it also sits on the
    // *end* of a narrower sub-node (`is_x` or `state`) — `at_offset`'s inclusive-end tie-break
    // always prefers that narrower node, so no offset resolves to the guard's own application.
    let info = typing_for(text);
    for guard in ["is_a(state)", "is_b(state)", "is_c(state)"] {
        let start = text.find(guard).expect("guard text should occur verbatim");
        let span = Span { start, end: start + guard.len() };
        let node = info
            .nodes()
            .iter()
            .find(|node| node.span == span)
            .unwrap_or_else(|| panic!("no typed node spanning '{guard}'"));
        assert_eq!(
            node.sort.as_ref().expect("the guard is a Bool-checked application").to_string(),
            "Bool"
        );
    }
}

// ─── goto-def: action and process names ─────────────────────────────────────
//
// Unlike a variable occurrence, resolving *which* `act`/`proc` declaration a name refers to is
// overload/arity-based (`check_action_or_process`/`check_instantiation`, see
// `docs/name_resolution.md`), so it can only happen once type checking has picked the single
// matching candidate — these `ResolvedName::Action`/`Process` nodes are pushed directly at that
// point, not built from any `DataExpr` the way `Variable`/`Constructor`/`Mapping` are.

/// `a(n)`'s own name — not its argument — resolves to an `Action`, pointing at its `act`
/// declaration.
#[test]
fn test_action_name_goto_def_resolves_to_its_declaration() {
    let text = "act a: Nat; proc P(n: Nat) = a(n); init P(1);";
    let name = resolved_name_at(text, "a(n)");
    let ResolvedName::Action { name, declaration } = &name else {
        panic!("expected an Action resolution, got {name:?}");
    };
    assert_eq!(name, "a");
    let declaration = declaration.clone().expect("a plain `act` declaration has a real span");
    assert_eq!(&text[declaration.start..declaration.end], "a");
    assert_eq!(declaration.start, text.find("a: Nat").expect("'a: Nat' not found"));
}

/// A positional process instantiation (`P(1)`, parsed as the same `ProcessExprKind::Action` node
/// an action instance is) resolves to a `Process`, not an `Action`.
#[test]
fn test_positional_process_instantiation_name_goto_def_resolves_to_its_declaration() {
    let text = "proc P(n: Nat) = delta; init P(1);";
    let name = resolved_name_at(text, "P(1)");
    let ResolvedName::Process { name, declaration } = &name else {
        panic!("expected a Process resolution, got {name:?}");
    };
    assert_eq!(name, "P");
    let declaration = declaration.clone().expect("a plain `proc` declaration has a real span");
    assert_eq!(&text[declaration.start..declaration.end], "P");
    assert_eq!(
        declaration.start,
        text.find("P(n: Nat)").expect("'P(n: Nat)' not found")
    );
}

/// The assignment form of instantiation (`P(n = 1)`, a `ProcessExprKind::Id` node) resolves to a
/// `Process` the same way the positional form does.
#[test]
fn test_assignment_form_instantiation_name_goto_def_resolves_to_its_declaration() {
    let text = "proc P(n: Nat) = delta; init P(n = 1);";
    let name = resolved_name_at(text, "P(n = 1)");
    let ResolvedName::Process { name, declaration } = &name else {
        panic!("expected a Process resolution, got {name:?}");
    };
    assert_eq!(name, "P");
    let declaration = declaration.clone().expect("a plain `proc` declaration has a real span");
    assert_eq!(&text[declaration.start..declaration.end], "P");
}

/// An action overloaded by argument sort (mirroring `abp.mcrl2`'s style) resolves each *use* to
/// the specific declaration its arguments actually match — the same disambiguation
/// `test_overloaded_name_resolves_to_the_matching_declaration_by_sort` (data crate) exercises for
/// constructors/mappings.
#[test]
fn test_overloaded_action_resolves_to_the_matching_declaration_by_arity() {
    let text = "act a: Nat; a: Bool; proc P = a(1) + a(true); init P;";
    let nat_decl = resolved_name_at(text, "a(1)");
    let ResolvedName::Action { declaration, .. } = &nat_decl else {
        panic!("expected an Action resolution, got {nat_decl:?}");
    };
    let declaration = declaration.clone().expect("a real declaration span");
    assert_eq!(declaration.start, text.find("a: Nat").expect("'a: Nat' not found"));

    let bool_decl = resolved_name_at(text, "a(true)");
    let ResolvedName::Action { declaration, .. } = &bool_decl else {
        panic!("expected an Action resolution, got {bool_decl:?}");
    };
    let declaration = declaration.clone().expect("a real declaration span");
    assert_eq!(declaration.start, text.find("a: Bool").expect("'a: Bool' not found"));
}

// ─── goto-def: hide/block/allow/comm/rename action-name sets ────────────────
//
// These name an action with no argument list to disambiguate an overload by, so
// `check_action_names` offers every same-named declaration as a `ResolvedName::ActionSet` instead
// of picking one — see `docs/action-name-set-goto-def-plan.md`. One test per construct/position
// covers that `typing` is actually threaded through every `check_action_names` call site, not just
// the first.

#[track_caller]
fn action_set_at(name: ResolvedName) -> (String, Vec<Span>) {
    let ResolvedName::ActionSet { name, declarations } = name else {
        panic!("expected an ActionSet resolution, got {name:?}");
    };
    (name, declarations)
}

#[test]
fn test_hide_action_name_resolves_to_action_set() {
    let text = "act a, b; init hide({b}, a);";
    let (name, declarations) = action_set_at(resolved_name_at(text, "b}"));
    assert_eq!(name, "b");
    let [declaration] = declarations.as_slice() else {
        panic!("expected exactly one declaration, got {declarations:?}");
    };
    assert_eq!(&text[declaration.start..declaration.end], "b");
}

#[test]
fn test_block_action_name_resolves_to_action_set() {
    let text = "act a; init block({a}, a);";
    let (name, declarations) = action_set_at(resolved_name_at(text, "a}"));
    assert_eq!(name, "a");
    assert_eq!(declarations.len(), 1);
}

#[test]
fn test_allow_action_name_resolves_to_action_set() {
    let text = "act a, b; init allow({b}, a);";
    let (name, declarations) = action_set_at(resolved_name_at(text, "b}"));
    assert_eq!(name, "b");
    assert_eq!(declarations.len(), 1);
}

#[test]
fn test_comm_from_and_to_action_names_resolve_to_action_sets() {
    // `comm`'s multi-action `from` side needs at least two actions (`Id "|" MultActId`, see the
    // grammar), unlike `rename`'s plain `Id -> Id`.
    let text = "act a, c, b; proc P = b; init comm({a|c -> b}, P);";
    let (from_name, from_declarations) = action_set_at(resolved_name_at(text, "a|c"));
    assert_eq!(from_name, "a");
    assert_eq!(from_declarations.len(), 1);

    let (to_name, to_declarations) = action_set_at(resolved_name_at(text, "b}"));
    assert_eq!(to_name, "b");
    assert_eq!(to_declarations.len(), 1);
}

#[test]
fn test_rename_from_and_to_action_names_resolve_to_action_sets() {
    let text = "act a, b; proc P = b; init rename({a -> b}, P);";
    let (from_name, from_declarations) = action_set_at(resolved_name_at(text, "a ->"));
    assert_eq!(from_name, "a");
    assert_eq!(from_declarations.len(), 1);

    let (to_name, to_declarations) = action_set_at(resolved_name_at(text, "b}"));
    assert_eq!(to_name, "b");
    assert_eq!(to_declarations.len(), 1);
}

/// An overloaded action named in an action-name set (no argument list to disambiguate by) offers
/// every declaration, in declaration order, rather than guessing one.
#[test]
fn test_overloaded_action_in_action_set_offers_every_declaration() {
    let text = "act b: Nat; b: Bool; a; init block({b}, a);";
    let (name, declarations) = action_set_at(resolved_name_at(text, "b}"));
    assert_eq!(name, "b");
    let [first, second] = declarations.as_slice() else {
        panic!("expected exactly two declarations, got {declarations:?}");
    };
    assert_eq!(first.start, text.find("b: Nat").expect("'b: Nat' not found"));
    assert_eq!(second.start, text.find("b: Bool").expect("'b: Bool' not found"));
}
