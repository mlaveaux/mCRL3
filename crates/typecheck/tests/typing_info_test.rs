//! Tests for [`TypingInfo`]: the span-keyed hover/go-to-definition information built from a
//! checked [`DataSpecification`].

use merc_syntax::DataExpr;
use merc_syntax::UntypedDataSpecification;
use merc_typecheck::DataSpecification;
use merc_typecheck::ResolvedName;
use merc_typecheck::TypingInfo;

/// Type checks `text`, returning its full [`TypingInfo`] (every user equation, merged).
#[track_caller]
fn typing_for(text: &str) -> TypingInfo {
    let spec = UntypedDataSpecification::parse(text).expect("the specification should parse");
    let mut spec = DataSpecification::from_untyped(spec).expect("the specification should type check");
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

// ─── hover: sorts ───────────────────────────────────────────────────────────

#[test]
fn test_variable_hover_reports_declared_sort() {
    assert_eq!(hover("map f: Nat -> Bool; var n: Nat; eqn f(n) = true;", "n)"), "Nat");
}

#[test]
fn test_literal_hover_reports_its_own_minimal_sort() {
    // `f`'s declared `Nat` result widens the *equation's* join, not `1`'s own recorded sort:
    // hovering the literal itself shows `Pos`, its minimal sort, same as a standalone expression.
    assert_eq!(hover("map f: Nat; eqn f = 1;", "1;"), "Pos");
}

#[test]
fn test_hovering_each_side_of_an_upcast_equation_shows_its_own_sort() {
    // `f: Real` and `1: Pos` widen to a common `Real` for the equation to type check, but neither
    // side's own node is *forced* to report that shared value: `f` still shows its declared
    // `Real`, and `1` still shows its own minimal `Pos`.
    let text = "map f: Real; eqn f = 1;";
    assert_eq!(hover(text, "f ="), "Real");
    assert_eq!(hover(text, "1;"), "Pos");
}

// ─── hover: operator tie-break ──────────────────────────────────────────────
//
// `x + y` lowers to `Application { function: Id("+"), arguments: [x, y] }` before inference ever
// runs, and the synthesized `Id("+")`/`Application` nodes both inherit the whole surface
// expression's span (neither existed in the source on its own). `TypingInfo::at_offset`'s
// documented tie-break (last node generated wins) is what picks the operator's own resolution
// over the application's result sort when hovering anywhere in the expression.

#[test]
fn test_hovering_a_numeric_operator_resolves_to_its_system_mapping() {
    // `+` on a concrete numeric sort resolves to a real Appendix-B mapping overload (`+: Pos #
    // Pos -> Pos`, here), not the generic polymorphic scheme -- `+`/`-`/`*` are also overloaded
    // as Set/Bag union/difference/intersection through that scheme, but a concrete numeric use
    // like this one resolves to the more specific system-defined declaration instead.
    match resolved_name_at("map f: Nat; eqn f = 1 + 1;", "+") {
        ResolvedName::SystemDefined { name } => assert_eq!(name, "+"),
        other => panic!("expected a SystemDefined resolution for '+', got {other:?}"),
    }
}

#[test]
fn test_hovering_a_comparison_operator_resolves_to_the_polymorphic_builtin() {
    // Unlike `+`, `==` has no concrete per-sort declaration anywhere to prefer: it is only ever
    // the polymorphic scheme, so it resolves to `Builtin`.
    match resolved_name_at("map f: Bool; eqn f = 1 == 1;", "==") {
        ResolvedName::Builtin { name } => assert_eq!(name, "=="),
        other => panic!("expected a Builtin resolution for '==', got {other:?}"),
    }
}

// ─── goto-def: constructors and mappings ────────────────────────────────────

#[test]
fn test_constructor_goto_def_points_at_its_declaration() {
    let text = "sort D; cons c: D; map f: D -> Bool; eqn f(c) = true;";
    match resolved_name_at(text, "c)") {
        ResolvedName::Constructor { name, declaration, .. } => {
            assert_eq!(name, "c");
            let span = declaration.expect("a plain `cons` declaration has a real span");
            assert_eq!(
                &text[span.start..span.end],
                "c",
                "the declaration span is precisely the identifier, not `c: D`"
            );
        }
        other => panic!("expected a Constructor resolution, got {other:?}"),
    }
}

#[test]
fn test_mapping_goto_def_points_at_its_declaration() {
    let text = "map f: Bool; eqn f = true;";
    match resolved_name_at(text, "f =") {
        ResolvedName::Mapping { name, declaration, .. } => {
            assert_eq!(name, "f");
            let span = declaration.expect("a plain `map` declaration has a real span");
            assert_eq!(
                &text[span.start..span.end],
                "f",
                "the declaration span is precisely the identifier, not `f: Bool`"
            );
        }
        other => panic!("expected a Mapping resolution, got {other:?}"),
    }
}

#[test]
fn test_struct_constructor_goto_def_points_at_its_own_name_in_the_struct_declaration() {
    // Unlike a plain `cons` declaration, a struct's `c1`/`c2` have no declaration of their own to
    // point at other than the `struct` sort expression itself — this confirms `ConstructorDecl`'s
    // own span (see `merc_syntax`) reaches goto-definition rather than falling back to `None`.
    let text = "sort D = struct c1(a: Bool) | c2; map f: D -> Bool; eqn f(c1(true)) = true;";
    match resolved_name_at(text, "c1(true)") {
        ResolvedName::Constructor { name, declaration, .. } => {
            assert_eq!(name, "c1");
            let span = declaration.expect("a struct constructor now has a real declaration span");
            assert_eq!(
                &text[span.start..span.end],
                "c1",
                "must point at just the constructor's own name"
            );
        }
        other => panic!("expected a Constructor resolution, got {other:?}"),
    }
}

#[test]
fn test_struct_projection_and_recogniser_goto_def_point_at_their_own_names() {
    let text = "sort D = struct c1(a: Bool)?is_c1 | c2; eqn true = is_c1(c1(true)) && a(c1(true));";

    match resolved_name_at(text, "is_c1(c1(true))") {
        ResolvedName::Mapping { name, declaration, .. } => {
            assert_eq!(name, "is_c1");
            let span = declaration.expect("a struct's recogniser now has a real declaration span");
            assert_eq!(&text[span.start..span.end], "is_c1");
        }
        other => panic!("expected a Mapping resolution for the recogniser, got {other:?}"),
    }

    match resolved_name_at(text, "a(c1(true))") {
        ResolvedName::Mapping { name, declaration, .. } => {
            assert_eq!(name, "a");
            let span = declaration.expect("a struct's projection now has a real declaration span");
            assert_eq!(&text[span.start..span.end], "a");
        }
        other => panic!("expected a Mapping resolution for the projection, got {other:?}"),
    }
}

#[test]
fn test_overloaded_name_resolves_to_the_matching_declaration_by_sort() {
    // `c` is declared twice with different sorts: a nullary `Bool` constructor and a `Nat -> Bool`
    // mapping. Each *use* must resolve to whichever declaration actually matches its context.
    let text = "sort D; cons c: D; map c: Nat -> D; map h: D; map g: D -> Bool; eqn g(c) = true; eqn h = c(1);";

    match resolved_name_at(text, "c) = true") {
        ResolvedName::Constructor { name, .. } => assert_eq!(name, "c"),
        other => panic!("expected the nullary use to resolve to the Constructor, got {other:?}"),
    }
    match resolved_name_at(text, "c(1)") {
        ResolvedName::Mapping { name, .. } => assert_eq!(name, "c"),
        other => panic!("expected the applied use to resolve to the Mapping, got {other:?}"),
    }
}

#[test]
fn test_duplicate_declaration_resolves_to_the_first() {
    // A literal duplicate (legal in mCRL2) is the one case `(name, sort)` doesn't disambiguate;
    // it resolves to the first declaration in source order.
    let text = "map f: Nat; f: Nat; eqn f = 1;";
    match resolved_name_at(text, "f =") {
        ResolvedName::Mapping { declaration, .. } => {
            let span = declaration.expect("a plain `map` declaration has a real span");
            assert_eq!(
                &text[span.start..span.end],
                "f",
                "should resolve to the first declaration, and be precisely the identifier"
            );
        }
        other => panic!("expected a Mapping resolution, got {other:?}"),
    }
}

#[test]
fn test_system_defined_symbol_reports_no_user_declaration() {
    // `succ` is an Appendix-B mapping with no user declaration to point at.
    match resolved_name_at("map f: Nat; eqn f = succ(0);", "succ") {
        ResolvedName::SystemDefined { name } => assert_eq!(name, "succ"),
        other => panic!("expected a SystemDefined resolution, got {other:?}"),
    }
}

// ─── typecheck_expression_with_typing ───────────────────────────────────────

#[test]
fn test_typecheck_expression_with_typing_returns_a_typing_over_the_expression() {
    let untyped = UntypedDataSpecification::parse("map f: Nat;").expect("the specification should parse");
    let mut spec = DataSpecification::from_untyped(untyped).expect("the specification should type check");
    let expr = DataExpr::parse("1 + 1").expect("the expression should parse");

    let (lowered, info) = spec
        .typecheck_expression_with_typing(&expr)
        .expect("'1 + 1' should type check");
    // The lowered aterm form prints in prefix notation, already covered by expression_test.rs;
    // just confirm this API path lowers the same way.
    assert_eq!(lowered.to_string(), "+(@c1, @c1)");

    // Same tie-break as the equation case: hovering the operator resolves to its concrete
    // system-defined overload (see test_hovering_a_numeric_operator_resolves_to_its_system_mapping).
    let offset = "1 + 1".find('+').unwrap();
    match info.at_offset(offset).and_then(|node| node.name.clone()) {
        Some(ResolvedName::SystemDefined { name }) => assert_eq!(name, "+"),
        other => panic!("expected a SystemDefined resolution for '+', got {other:?}"),
    }
}

// ─── structural invariants ──────────────────────────────────────────────────

#[test]
fn test_every_node_has_a_well_formed_non_degenerate_span_into_the_source() {
    let text = "sort D;\n\
                cons c: D;\n\
                map f: D -> Bool;\n\
                g: List(Nat) -> Bool;\n\
                var d: D;\n\
                eqn f(d) = true;\n\
                eqn g([1, 2, 3]) = 1 in [1, 2, 3];";
    let info = typing_for(text);

    assert!(
        !info.nodes().is_empty(),
        "a specification with equations should produce typed nodes"
    );
    for node in info.nodes() {
        assert!(
            node.span.start <= node.span.end,
            "span {:?} has start after end",
            node.span
        );
        assert!(
            node.span.end <= text.len(),
            "span {:?} runs past the end of the source",
            node.span
        );
    }
}
