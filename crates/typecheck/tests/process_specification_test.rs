//! Whole-process-specification type-checking tests: actions, process bodies, and `init`, on top
//! of the data-specification checking `data_specification_test.rs` already covers.

use merc_syntax::UntypedProcessSpecification;
use merc_typecheck::ProcessError;
use merc_typecheck::ProcessSpecification;

/// Type checks `text`, asserting it is accepted.
#[track_caller]
fn check_ok(text: &str) {
    let spec = UntypedProcessSpecification::parse(text).expect("the specification should parse");
    if let Err(error) = ProcessSpecification::from_untyped(spec) {
        panic!("expected the specification to type check:\n{text}\nerror: {error}");
    }
}

/// Type checks `text`, returning the error for the caller to match on the specific variant.
#[track_caller]
fn check_err(text: &str) -> ProcessError {
    let spec = UntypedProcessSpecification::parse(text).expect("the specification should parse");
    match ProcessSpecification::from_untyped(spec) {
        Err(error) => error,
        Ok(_) => panic!("expected the specification to be rejected:\n{text}"),
    }
}

// ─── basic acceptance ───────────────────────────────────────────────────────

#[test]
fn test_action_with_matching_argument_sort_is_accepted() {
    // `0: Nat` directly, unlike `1`, which is `Pos` and only reaches `Nat` via upcast.
    check_ok("act a: Nat; init a(0);");
}

#[test]
fn test_action_argument_upcasts_like_any_other_expression() {
    // `1: Pos` upcasts to `Nat`, same as anywhere else a `Pos` is used where a `Nat` is expected.
    check_ok("act a: Nat; init a(1);");
}

#[test]
fn test_process_instantiation_is_accepted() {
    check_ok("proc P = delta; init P;");
}

#[test]
fn test_specification_with_no_init_is_accepted() {
    // A library-only specification is legitimate; absent `init` is not itself an error.
    check_ok("act a; proc P = a . delta;");
}

// ─── undeclared / mismatched ────────────────────────────────────────────────

#[test]
fn test_undeclared_action_is_rejected() {
    let error = check_err("init a;");
    assert!(
        matches!(error, ProcessError::UndeclaredActionOrProcess { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_action_with_wrong_argument_arity_is_rejected() {
    let error = check_err("act a: Nat; init a(1, 2);");
    assert!(
        matches!(error, ProcessError::UndeclaredActionOrProcess { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_action_with_mismatched_argument_sort_is_rejected() {
    let error = check_err("act a: Nat; init a(true);");
    assert!(
        matches!(error, ProcessError::NoMatchingOverload { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_undeclared_process_instantiation_is_rejected() {
    let error = check_err("init P;");
    assert!(
        matches!(error, ProcessError::UndeclaredActionOrProcess { .. }),
        "got {error:?}"
    );
}

// ─── overload resolution ────────────────────────────────────────────────────

#[test]
fn test_action_overloaded_by_argument_sort_resolves_per_use() {
    // Mirrors abp.mcrl2's `s3,r3,c3: D # Bool; s3,r3,c3: Error;` shape.
    check_ok(
        "sort D = Bool; sort Error = struct e;\
         act c: D # Bool; act c: Error;\
         init c(true, false) . c(e);",
    );
}

#[test]
fn test_process_overloaded_by_parameter_arity_resolves_per_use() {
    // Mirrors abp_bw.mcrl2's `S`, `S(b:Bit)`, `S(d:D,b:Bit)` shape.
    check_ok(
        "sort D = Bool; sort Bit = struct b0 | b1;\
         proc S = delta;\
         proc S(b: Bit) = delta;\
         proc S(d: D, b: Bit) = delta;\
         init S . S(b0) . S(true, b0);",
    );
}

#[test]
fn test_ambiguous_action_use_is_rejected() {
    // Two overloads of `c` both accept a `Nat` argument (`Nat <= Int` widens either way), so a
    // plain `Nat`-sorted argument doesn't disambiguate between them.
    let error = check_err("act c: Nat; act c: Int; init c(1);");
    assert!(
        matches!(error, ProcessError::AmbiguousActionOrProcess { .. }),
        "got {error:?}"
    );
}

// ─── scoping ─────────────────────────────────────────────────────────────────

#[test]
fn test_sum_bound_variable_is_in_scope_for_the_action_argument() {
    check_ok("act a: Nat; init sum n: Nat . a(n);");
}

#[test]
fn test_sum_bound_variable_is_out_of_scope_outside_the_sum() {
    let error = check_err("act a: Nat; init (sum n: Nat . a(n)) . a(n);");
    // `a` has a single overload, but even a single candidate's failure is still reported through
    // `NoMatchingOverload` (consistent with the multi-candidate case) rather than surfaced as a
    // bare `Inference` error.
    let ProcessError::NoMatchingOverload { cause, .. } = error else {
        panic!("expected a NoMatchingOverload, got {error:?}");
    };
    assert!(
        matches!(
            *cause,
            ProcessError::Inference(merc_typecheck::InferenceError::UndeclaredName { .. })
        ),
        "got {cause:?}"
    );
}

#[test]
fn test_process_parameter_is_in_scope_for_its_own_body() {
    check_ok("act a: Nat; proc P(n: Nat) = a(n); init P(1);");
}

#[test]
fn test_global_variable_is_in_scope_in_a_process_body_and_init() {
    check_ok("glob n: Nat; act a: Nat; proc P = a(n); init P . a(n);");
}

#[test]
fn test_process_parameter_shadows_a_global_of_the_same_name() {
    // `P`'s own `n: Bool` parameter shadows the global `n: Nat`; `a`'s declared `Bool` argument
    // sort only accepts the shadowed (parameter) binding.
    check_ok("glob n: Nat; act a: Bool; proc P(n: Bool) = a(n); init P(true);");
}

// ─── conditions, time, distributions ────────────────────────────────────────

#[test]
fn test_non_boolean_condition_is_rejected() {
    let error = check_err("act a; init 1 -> a <> delta;");
    assert!(matches!(error, ProcessError::Inference(_)), "got {error:?}");
}

#[test]
fn test_non_real_time_bound_is_rejected() {
    let error = check_err("act a; init a @ true;");
    assert!(matches!(error, ProcessError::Inference(_)), "got {error:?}");
}

#[test]
fn test_dist_weight_is_checked_against_real() {
    check_ok("act a: Nat; init dist n: Nat[1/2] . a(n);");
}

// ─── assignment-form instantiation ──────────────────────────────────────────

#[test]
fn test_assignment_form_instantiation_is_accepted() {
    check_ok("proc P(n: Nat) = delta; init P(n = 1);");
}

#[test]
fn test_assignment_form_instantiation_may_omit_parameters() {
    check_ok("proc P(n: Nat, b: Bool) = delta; init P(n = 1);");
}

#[test]
fn test_assignment_to_an_unknown_parameter_is_rejected() {
    let error = check_err("proc P(n: Nat) = delta; init P(m = 1);");
    assert!(
        matches!(error, ProcessError::UnknownProcessParameter { .. }),
        "got {error:?}"
    );
}

// ─── declaration-level errors ───────────────────────────────────────────────

#[test]
fn test_anonymous_struct_in_action_declaration_is_rejected() {
    let error = check_err("act a: struct x | y; init a(x);");
    assert!(
        matches!(error, ProcessError::AnonymousStructInDeclaration { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_action_and_process_sharing_a_name_is_rejected() {
    let error = check_err("act P: Nat; proc P = delta; init delta;");
    assert!(
        matches!(error, ProcessError::ActionAndProcessConflict { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_duplicate_process_parameter_is_rejected() {
    let error = check_err("proc P(n: Nat, n: Bool) = delta; init P(1, true);");
    assert!(
        matches!(error, ProcessError::DuplicateProcessParameter { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_duplicate_global_variable_is_rejected() {
    let error = check_err("glob n: Nat, n: Bool; init delta;");
    assert!(
        matches!(error, ProcessError::DuplicateGlobalVariable { .. }),
        "got {error:?}"
    );
}

// ─── action-set checks (hide/block/allow/comm/rename) ───────────────────────

#[test]
fn test_hiding_an_undeclared_action_is_rejected() {
    let error = check_err("act a; init hide({b}, a);");
    assert!(matches!(error, ProcessError::UndeclaredAction { .. }), "got {error:?}");
}

#[test]
fn test_blocking_a_declared_action_is_accepted() {
    check_ok("act a; init block({a}, a);");
}

#[test]
fn test_allowing_an_undeclared_multi_action_is_rejected() {
    let error = check_err("act a; init allow({b}, a);");
    assert!(matches!(error, ProcessError::UndeclaredAction { .. }), "got {error:?}");
}

#[test]
fn test_comm_with_identical_action_sorts_is_accepted() {
    check_ok("act a, b, c: Nat; init comm({a|b -> c}, delta);");
}

#[test]
fn test_comm_with_no_argument_actions_is_accepted() {
    check_ok("act a, b, c; init comm({a|b -> c}, delta);");
}

#[test]
fn test_comm_widens_combined_actions_like_any_other_position() {
    // `a: Pos` and `b: Nat` join to `Nat`, same as `Pos <= Nat` upcasts anywhere else; `c: Nat`
    // then accepts that joined sort directly.
    check_ok("act a: Pos; act b: Nat; act c: Nat; init comm({a|b -> c}, delta);");
}

#[test]
fn test_comm_widens_the_joined_sort_into_the_result_action() {
    // `a`/`b: Pos` join to `Pos`, which upcasts into `c: Nat`, exactly like passing a `Pos`
    // argument where a `Nat` parameter is declared.
    check_ok("act a, b: Pos; act c: Nat; init comm({a|b -> c}, delta);");
}

#[test]
fn test_comm_picks_the_compatible_overload_among_several() {
    // `a: Bool` cannot combine with `b: Nat`, but `a`'s other overload (`Nat`) can.
    check_ok("act a: Bool; act a: Nat; act b, c: Nat; init comm({a|b -> c}, delta);");
}

#[test]
fn test_comm_with_incompatible_action_sorts_is_rejected() {
    let error = check_err("act a: Bool; act b: Nat; act c: Nat; init comm({a|b -> c}, delta);");
    assert!(
        matches!(error, ProcessError::IncompatibleCommunication { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_comm_with_mismatched_arity_is_rejected() {
    let error = check_err("act a: Nat; act b: Nat # Nat; act c: Nat; init comm({a|b -> c}, delta);");
    assert!(
        matches!(error, ProcessError::IncompatibleCommunication { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_comm_result_sort_too_narrow_is_rejected() {
    // `a`/`b: Nat` join to `Nat`, which does not downcast into `c: Pos`.
    let error = check_err("act a, b: Nat; act c: Pos; init comm({a|b -> c}, delta);");
    assert!(
        matches!(error, ProcessError::IncompatibleCommunication { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_rename_with_identical_action_sorts_is_accepted() {
    check_ok("act a, b: Nat; init rename({a -> b}, delta);");
}

#[test]
fn test_rename_widens_like_any_other_position() {
    check_ok("act a: Pos; act b: Nat; init rename({a -> b}, delta);");
}

#[test]
fn test_rename_with_incompatible_action_sorts_is_rejected() {
    let error = check_err("act a: Bool; act b: Nat; init rename({a -> b}, delta);");
    assert!(
        matches!(error, ProcessError::IncompatibleRename { .. }),
        "got {error:?}"
    );
}

#[test]
fn test_rename_with_mismatched_arity_is_rejected() {
    let error = check_err("act a: Nat; act b: Nat # Nat; init rename({a -> b}, delta);");
    assert!(
        matches!(error, ProcessError::IncompatibleRename { .. }),
        "got {error:?}"
    );
}
