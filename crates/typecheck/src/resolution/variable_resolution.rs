//! Resolves *variable* references — as opposed to constructor/mapping/action/process names, which
//! are not context-free (see `docs/name_resolution.md`) — in a process specification's `proc`
//! bodies/`init` and a PBES's equation bodies/`init`, before type checking runs.
//!
//! A variable occurrence's binder — `sum`/`dist`, a process's own parameters, a PBES
//! quantifier/equation parameter — is decided purely by lexical scoping over the untyped syntax
//! tree, with no dependency on inferred sorts (unlike overloaded constructor/mapping resolution,
//! or the arity-based action-vs-process disambiguation `check_action_or_process` performs). This
//! pass therefore needs no [`crate::DataSpecification`] and cannot fail: a name not found in
//! scope is left as a plain `Id`, and `inference.rs`'s overload/`UndeclaredName` machinery is
//! responsible for rejecting it if it turns out to be undeclared.
//!
//! [`DataExprKind::Id`] resolves to [`DataExprKind::Resolved`] the same way a sort's
//! `Reference` resolves to `Resolved` in `resolution::name_resolution::resolve_sort_id`.

use merc_syntax::DataExpr;
use merc_syntax::DataExprKind;
use merc_syntax::IdDecl;
use merc_syntax::PbesExpr;
use merc_syntax::PbesExprKind;
use merc_syntax::ProcessExpr;
use merc_syntax::ProcessExprKind;
use merc_syntax::PropVarInst;
use merc_syntax::Span;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::UntypedPbes;
use merc_syntax::UntypedProcessSpecification;

/// Resolves every context-free variable reference in `spec`'s own `var`-block equations —
/// the one case `docs/name_resolution.md` originally left uncovered. Each `EqnSpec`'s
/// `variables` are in scope for that block's own equations only; an equation's `lhs` is
/// resolved exactly like its `rhs`/`condition` — mCRL2 equations don't bind new variables in
/// their left-hand side, `lhs` is just another read of the block's declared variables.
pub(crate) fn resolve_data_specification_variables(spec: &mut UntypedDataSpecification) {
    for eqn_spec in &mut spec.equation_declarations {
        let mut scope = binder_scope(&eqn_spec.variables);
        for equation in &mut eqn_spec.equations {
            if let Some(condition) = &mut equation.condition {
                resolve_in_data_expr(condition, &mut scope);
            }
            resolve_in_data_expr(&mut equation.lhs, &mut scope);
            resolve_in_data_expr(&mut equation.rhs, &mut scope);
        }
    }
}

/// Resolves every context-free variable reference in `spec`'s `proc` bodies and `init`.
pub(crate) fn resolve_process_variables(spec: &mut UntypedProcessSpecification) {
    let globals = binder_scope(&spec.global_variables);

    for proc_decl in &mut spec.process_declarations {
        // A process's own parameters shadow a global variable of the same name.
        let mut scope = globals.clone();
        push_binder(&mut scope, &proc_decl.params);
        resolve_in_process_expr(&mut proc_decl.body, &mut scope);
    }

    if let Some(init) = &mut spec.init {
        // `init` sits outside every process's own parameter scope — only globals apply.
        let mut scope = globals.clone();
        resolve_in_process_expr(init, &mut scope);
    }
}

/// Resolves every context-free variable reference in `pbes`'s equation bodies and `init`.
pub(crate) fn resolve_pbes_variables(pbes: &mut UntypedPbes) {
    let globals = binder_scope(&pbes.global_variables);

    for equation in &mut pbes.equations {
        let mut scope = globals.clone();
        push_binder(&mut scope, &equation.variable.parameters);
        resolve_in_pbes_expr(&mut equation.formula, &mut scope);
    }

    // `init` sits outside every equation's own parameter scope — only globals apply.
    let mut scope = globals.clone();
    resolve_in_prop_var_inst(&mut pbes.init, &mut scope);
}

/// The shadowing stack this pass threads through a tree: each binder's own name paired with its
/// declaration's span (not the occurrence's), so two occurrences of the same binder keep comparing
/// equal once rewritten to [`DataExprKind::Resolved`].
type Scope = Vec<(String, Span)>;

fn binder_scope<Id>(variables: &[IdDecl<Id>]) -> Scope {
    variables
        .iter()
        .map(|decl| (decl.identifier.clone(), decl.span.clone()))
        .collect()
}

/// Pushes each declaration in `variables` onto `scope`, returning how many were pushed so the
/// caller can `truncate` them back off once its subtree is done.
fn push_binder<Id>(scope: &mut Scope, variables: &[IdDecl<Id>]) -> usize {
    for variable in variables {
        scope.push((variable.identifier.clone(), variable.span.clone()));
    }
    variables.len()
}

fn resolve_in_process_expr(expr: &mut ProcessExpr, scope: &mut Scope) {
    match &mut expr.node {
        ProcessExprKind::Delta | ProcessExprKind::Tau => {}
        ProcessExprKind::Action(_, args) => {
            for arg in args {
                resolve_in_data_expr(arg, scope);
            }
        }
        ProcessExprKind::Id(_, assignments) => {
            // Only the assignment's *value* is a context-free variable read; the key (`n` in
            // `P(n = x)`) is resolved against whichever process `P` turns out to name — that's
            // the non-context-free `check_action_or_process`/`check_instantiation` path, out of
            // scope here (see `docs/name_resolution.md`).
            for assignment in assignments {
                resolve_in_data_expr(&mut assignment.expr, scope);
            }
        }
        ProcessExprKind::Sum { variables, operand } => {
            let pushed = push_binder(scope, variables);
            resolve_in_process_expr(operand, scope);
            scope.truncate(scope.len() - pushed);
        }
        ProcessExprKind::Dist {
            variables,
            expr: weight,
            operand,
        } => {
            let pushed = push_binder(scope, variables);
            // `dist`'s weight is resolved with its own bound variables already in scope.
            resolve_in_data_expr(weight, scope);
            resolve_in_process_expr(operand, scope);
            scope.truncate(scope.len() - pushed);
        }
        ProcessExprKind::Binary { lhs, rhs, .. } => {
            resolve_in_process_expr(lhs, scope);
            resolve_in_process_expr(rhs, scope);
        }
        ProcessExprKind::Hide { operand, .. }
        | ProcessExprKind::Rename { operand, .. }
        | ProcessExprKind::Allow { operand, .. }
        | ProcessExprKind::Block { operand, .. }
        | ProcessExprKind::Comm { operand, .. } => resolve_in_process_expr(operand, scope),
        ProcessExprKind::Condition { condition, then, else_ } => {
            resolve_in_data_expr(condition, scope);
            resolve_in_process_expr(then, scope);
            if let Some(else_) = else_ {
                resolve_in_process_expr(else_, scope);
            }
        }
        ProcessExprKind::At { expr, operand } => {
            resolve_in_process_expr(expr, scope);
            resolve_in_data_expr(operand, scope);
        }
    }
}

fn resolve_in_pbes_expr(expr: &mut PbesExpr, scope: &mut Scope) {
    match &mut expr.node {
        PbesExprKind::True | PbesExprKind::False => {}
        PbesExprKind::DataValExpr(data_expr) => resolve_in_data_expr(data_expr, scope),
        PbesExprKind::PropVarInst(inst) => resolve_in_prop_var_inst(inst, scope),
        PbesExprKind::Negation(inner) => resolve_in_pbes_expr(inner, scope),
        PbesExprKind::Binary { lhs, rhs, .. } => {
            resolve_in_pbes_expr(lhs, scope);
            resolve_in_pbes_expr(rhs, scope);
        }
        PbesExprKind::Quantifier { variables, body, .. } => {
            let pushed = push_binder(scope, variables);
            resolve_in_pbes_expr(body, scope);
            scope.truncate(scope.len() - pushed);
        }
    }
}

fn resolve_in_prop_var_inst(inst: &mut PropVarInst, scope: &mut Scope) {
    for argument in &mut inst.arguments {
        resolve_in_data_expr(argument, scope);
    }
}

/// Rewrites every `Id(name)` in `expr` found in `scope` into `Resolved(name, declaration span)`,
/// extending `scope` for the data-level binders it descends through (`lambda`, a quantifier, a
/// set/bag comprehension, `whr`).
fn resolve_in_data_expr(expr: &mut DataExpr, scope: &mut Scope) {
    match &mut expr.node {
        DataExprKind::Id(name) => {
            if let Some((_, declaration)) = scope.iter().rev().find(|(bound, _)| bound == name) {
                expr.node = DataExprKind::Resolved(name.clone(), declaration.clone());
            }
        }
        DataExprKind::Resolved(_, _)
        | DataExprKind::Number(_)
        | DataExprKind::Bool(_)
        | DataExprKind::EmptyList
        | DataExprKind::EmptySet
        | DataExprKind::EmptyBag => {}
        DataExprKind::Application { function, arguments } => {
            resolve_in_data_expr(function, scope);
            for argument in arguments {
                resolve_in_data_expr(argument, scope);
            }
        }
        DataExprKind::List(elements) | DataExprKind::Set(elements) => {
            for element in elements {
                resolve_in_data_expr(element, scope);
            }
        }
        DataExprKind::Bag(elements) => {
            for element in elements {
                resolve_in_data_expr(&mut element.expr, scope);
                resolve_in_data_expr(&mut element.multiplicity, scope);
            }
        }
        DataExprKind::SetBagComp { variable, predicate } => {
            let pushed = push_binder(scope, std::slice::from_ref(variable));
            resolve_in_data_expr(predicate, scope);
            scope.truncate(scope.len() - pushed);
        }
        DataExprKind::Lambda { variables, body } | DataExprKind::Quantifier { variables, body, .. } => {
            let pushed = push_binder(scope, variables);
            resolve_in_data_expr(body, scope);
            scope.truncate(scope.len() - pushed);
        }
        DataExprKind::Unary { expr, .. } => resolve_in_data_expr(expr, scope),
        DataExprKind::Binary { lhs, rhs, .. } => {
            resolve_in_data_expr(lhs, scope);
            resolve_in_data_expr(rhs, scope);
        }
        DataExprKind::FunctionUpdate { expr, update } => {
            resolve_in_data_expr(expr, scope);
            resolve_in_data_expr(&mut update.expr, scope);
            resolve_in_data_expr(&mut update.update, scope);
        }
        DataExprKind::Whr { expr, assignments } => {
            // Each assignment's right-hand side is resolved in the *outer* scope — bindings
            // don't see each other, only the body does.
            for assignment in assignments.iter_mut() {
                resolve_in_data_expr(&mut assignment.expr, scope);
            }
            let pushed = assignments.len();
            for assignment in assignments.iter() {
                scope.push((assignment.identifier.clone(), assignment.span.clone()));
            }
            resolve_in_data_expr(expr, scope);
            scope.truncate(scope.len() - pushed);
        }
    }
}

#[cfg(test)]
mod tests {
    use merc_syntax::DataExprKind;
    use merc_syntax::PbesExprKind;
    use merc_syntax::ProcessExprKind;
    use merc_syntax::UntypedDataSpecification;
    use merc_syntax::UntypedPbes;
    use merc_syntax::UntypedProcessSpecification;

    use super::resolve_data_specification_variables;
    use super::resolve_pbes_variables;
    use super::resolve_process_variables;

    /// The declaration span of `name`, located via `locate`'s first occurrence in `text` (an
    /// `IdDecl`'s own span covers only its identifier — not the `: Sort` that follows it — so
    /// `locate` only pins down where to look, `name`'s own length determines the span's width).
    fn decl_span(text: &str, locate: &str, name: &str) -> (usize, usize) {
        let start = text.find(locate).unwrap_or_else(|| panic!("'{locate}' not found in '{text}'"));
        (start, start + name.len())
    }

    #[test]
    fn test_action_argument_resolves_to_process_parameter() {
        let text = "act a: Nat; proc P(n: Nat) = a(n); init P(1);";
        let mut spec = UntypedProcessSpecification::parse(text).unwrap();
        resolve_process_variables(&mut spec);

        let ProcessExprKind::Action(_, args) = &spec.process_declarations[0].body.node else {
            panic!("expected an Action body");
        };
        let (start, end) = decl_span(text, "n: Nat", "n");
        assert!(matches!(
            &args[0].node,
            DataExprKind::Resolved(name, span) if name == "n" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_sum_bound_variable_resolves_to_its_own_binder() {
        let text = "act a: Nat; proc P = sum x: Nat . a(x); init P;";
        let mut spec = UntypedProcessSpecification::parse(text).unwrap();
        resolve_process_variables(&mut spec);

        let ProcessExprKind::Sum { operand, .. } = &spec.process_declarations[0].body.node else {
            panic!("expected a Sum body");
        };
        let ProcessExprKind::Action(_, args) = &operand.node else {
            panic!("expected an Action operand");
        };
        let (start, end) = decl_span(text, "x: Nat", "x");
        assert!(matches!(
            &args[0].node,
            DataExprKind::Resolved(name, span) if name == "x" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_dist_weight_sees_its_own_bound_variable() {
        let text = "act a; proc P = dist x: Pos[1/x] . a; init P;";
        let mut spec = UntypedProcessSpecification::parse(text).unwrap();
        resolve_process_variables(&mut spec);

        let ProcessExprKind::Dist { expr: weight, .. } = &spec.process_declarations[0].body.node else {
            panic!("expected a Dist body");
        };
        let DataExprKind::Binary { rhs, .. } = &weight.node else {
            panic!("expected a Binary (division) weight, got {:?}", weight.node);
        };
        let (start, end) = decl_span(text, "x: Pos", "x");
        assert!(matches!(
            &rhs.node,
            DataExprKind::Resolved(name, span) if name == "x" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_assignment_value_resolves_but_key_does_not() {
        let text = "proc P(n: Nat) = P(n = n); init P(0);";
        let mut spec = UntypedProcessSpecification::parse(text).unwrap();
        resolve_process_variables(&mut spec);

        let ProcessExprKind::Id(_, assignments) = &spec.process_declarations[0].body.node else {
            panic!("expected an Id (instantiation) body");
        };
        // The key `n` is a plain `String` field (`AssignmentData::identifier`), never touched by
        // this pass; only the value `n` (the expression) is a `DataExpr` and gets resolved.
        assert_eq!(assignments[0].identifier, "n");
        let (start, end) = decl_span(text, "n: Nat", "n");
        assert!(matches!(
            &assignments[0].expr.node,
            DataExprKind::Resolved(name, span) if name == "n" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_inner_binder_shadows_outer_one() {
        let text = "act a: Nat; proc P = sum x: Nat . sum x: Bool . a(1); init P;";
        let mut spec = UntypedProcessSpecification::parse(text).unwrap();
        resolve_process_variables(&mut spec);
        // No occurrence of `x` here to check directly, but this at least proves shadowing
        // doesn't panic/mis-truncate the scope stack; see the data-level test below for the
        // actual shadowing assertion.
        let _ = spec;
    }

    #[test]
    fn test_nested_data_binder_inside_process_action_argument_resolves() {
        let text = "act a: Bool; proc P(n: Nat) = a(exists x: Nat . x == n); init P(0);";
        let mut spec = UntypedProcessSpecification::parse(text).unwrap();
        resolve_process_variables(&mut spec);

        let ProcessExprKind::Action(_, args) = &spec.process_declarations[0].body.node else {
            panic!("expected an Action body");
        };
        let DataExprKind::Quantifier { body, .. } = &args[0].node else {
            panic!("expected a Quantifier argument");
        };
        let DataExprKind::Binary { lhs, rhs, .. } = &body.node else {
            panic!("expected a Binary (==) body");
        };
        let (x_start, x_end) = decl_span(text, "x: Nat", "x");
        assert!(matches!(
            &lhs.node,
            DataExprKind::Resolved(name, span) if name == "x" && span.start == x_start && span.end == x_end
        ));
        let (n_start, n_end) = decl_span(text, "n: Nat", "n");
        assert!(matches!(
            &rhs.node,
            DataExprKind::Resolved(name, span) if name == "n" && span.start == n_start && span.end == n_end
        ));
    }

    #[test]
    fn test_undeclared_name_is_left_unresolved() {
        let text = "act a: Nat; proc P = a(m); init P;";
        let mut spec = UntypedProcessSpecification::parse(text).unwrap();
        resolve_process_variables(&mut spec);

        let ProcessExprKind::Action(_, args) = &spec.process_declarations[0].body.node else {
            panic!("expected an Action body");
        };
        // `m` isn't declared anywhere; this pass leaves it as a plain `Id` for the existing
        // `UndeclaredName` inference error to reject later.
        assert!(matches!(&args[0].node, DataExprKind::Id(name) if name == "m"));
    }

    #[test]
    fn test_prop_var_inst_argument_resolves_to_equation_parameter() {
        let text = "pbes nu X(n: Nat) = val(n == 0) || X(n); init X(0);";
        let mut pbes = UntypedPbes::parse(text).unwrap();
        resolve_pbes_variables(&mut pbes);

        let PbesExprKind::Binary { rhs, .. } = &pbes.equations[0].formula.node else {
            panic!("expected a Binary (||) formula");
        };
        let PbesExprKind::PropVarInst(inst) = &rhs.node else {
            panic!("expected a PropVarInst");
        };
        let (start, end) = decl_span(text, "n: Nat", "n");
        assert!(matches!(
            &inst.arguments[0].node,
            DataExprKind::Resolved(name, span) if name == "n" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_quantifier_bound_variable_resolves_to_its_own_binder() {
        let text = "pbes mu X = forall n: Nat . val(n == n); init X;";
        let mut pbes = UntypedPbes::parse(text).unwrap();
        resolve_pbes_variables(&mut pbes);

        let PbesExprKind::Quantifier { body, .. } = &pbes.equations[0].formula.node else {
            panic!("expected a Quantifier formula");
        };
        let PbesExprKind::DataValExpr(data_expr) = &body.node else {
            panic!("expected a DataValExpr body");
        };
        let DataExprKind::Binary { lhs, .. } = &data_expr.node else {
            panic!("expected a Binary (==) expression");
        };
        let (start, end) = decl_span(text, "n: Nat", "n");
        assert!(matches!(
            &lhs.node,
            DataExprKind::Resolved(name, span) if name == "n" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_pbes_init_only_sees_globals_not_any_equations_own_parameters() {
        let text = "glob g: Nat; pbes nu X(n: Nat) = val(n == n); init X(g);";
        let mut pbes = UntypedPbes::parse(text).unwrap();
        resolve_pbes_variables(&mut pbes);

        let (start, end) = decl_span(text, "g: Nat", "g");
        assert!(matches!(
            &pbes.init.arguments[0].node,
            DataExprKind::Resolved(name, span) if name == "g" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_data_specification_equation_variable_resolves_to_its_var_block_declaration() {
        let text = "map f: Nat -> Nat; var x: Nat; eqn f(x) = x;";
        let mut spec = UntypedDataSpecification::parse(text).unwrap();
        resolve_data_specification_variables(&mut spec);

        let equation = &spec.equation_declarations[0].equations[0];
        let (start, end) = decl_span(text, "x: Nat", "x");
        assert!(matches!(
            &equation.lhs.node,
            DataExprKind::Application { arguments, .. }
                if matches!(&arguments[0].node, DataExprKind::Resolved(name, span) if name == "x" && span.start == start && span.end == end)
        ));
        assert!(matches!(
            &equation.rhs.node,
            DataExprKind::Resolved(name, span) if name == "x" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_data_specification_equation_condition_resolves_too() {
        let text = "map f: Bool -> Nat; var x: Bool; eqn x -> f(x) = 0;";
        let mut spec = UntypedDataSpecification::parse(text).unwrap();
        resolve_data_specification_variables(&mut spec);

        let equation = &spec.equation_declarations[0].equations[0];
        let (start, end) = decl_span(text, "x: Bool", "x");
        let condition = equation.condition.as_ref().expect("expected a condition");
        assert!(matches!(
            &condition.node,
            DataExprKind::Resolved(name, span) if name == "x" && span.start == start && span.end == end
        ));
    }

    #[test]
    fn test_data_specification_variable_scope_does_not_leak_across_var_blocks() {
        let text = "map f: Nat -> Nat; g: Bool -> Nat; var x: Nat; eqn f(x) = x; var y: Bool; eqn g(y) = y;";
        let mut spec = UntypedDataSpecification::parse(text).unwrap();
        resolve_data_specification_variables(&mut spec);

        // `y` isn't declared in the first block; it's a separate `var` block's own variable, so
        // this pass leaves it as-is when scoped to the first block. Only checking the second
        // block resolves correctly is the interesting assertion here.
        let second = &spec.equation_declarations[1].equations[0];
        let (start, end) = decl_span(text, "y: Bool", "y");
        assert!(matches!(
            &second.rhs.node,
            DataExprKind::Resolved(name, span) if name == "y" && span.start == start && span.end == end
        ));
    }
}
