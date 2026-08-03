//! Name resolution: assigns every declaration a stable [DefId]/[StateId]/
//! [LocalId] and rewrites every reference in place to point at the
//! declaration it names.
//!
//! STARK has no forward references: a name is only visible to expressions that
//! come *after* its declaration in source order. Concretely, this means one
//! linear walk over the declarations is sufficient: by the time a name is used,
//! everything it could legally refer to has already been registered.
//!
//! There are two exceptions, both handled by registering names in a first
//! pass before any body is resolved:
//!
//! * Controller states: `step`/`exec` inside a state may target a *later*
//!   state in the same component, so each component's states are registered up front.
//! * State variables: every `variables`/`global variables` block and every
//!   component's variable block is declared before anything else in the
//!   specification, so a function body, environment block or component may
//!   read a state variable regardless of where it is declared. This mirrors
//!   the original, which has a dedicated pass collecting exactly these names
//!   ahead of everything else.
//!
//! Caveat: [UntypedStarkSpecification] buckets declarations by kind (all
//! constants, then all parameters, then all variables, …) rather than
//! preserving one linear source-order list, so this pass resolves in a
//! fixed kind order — constants/parameters, then types, then functions,
//! then variables, then components, then the environment, then
//! penalties/perturbations/distances/formulas — rather than true
//! interleaved source order. Functions are resolved before variables
//! because variable initializers call helper functions (e.g.
//! `eval_rd(INIT_SPEED)`) in several of the example specs, even though the
//! functions themselves are declared first in the source too. This only
//! differs from true source order when declarations of different kinds
//! reference each other out of this grouping, which none of the example
//! specs do (once a couple of their own pre-existing bugs — a stray
//! reference to a global instead of a same-named parameter, an
//! under-scoped `let` — are fixed; see the fixed-up `examples/stark/*.stark`
//! files).
//!
//! Because variables are pre-declared, their names also resolve inside
//! expressions that are evaluated once at load time, before any variable
//! store exists — a `const`/`param` value, or a variable's own range or
//! initializer, including a self-reference like `real X = X;`. Resolution
//! order no longer rules those out, so a post-pass
//! ([Resolver::check_static_expressions]) rejects them explicitly, including
//! reads reached indirectly through a function call. The original accepts
//! all of these and evaluates them to its absorbing error value at runtime
//! with no diagnostic.
//!
//! This pass only binds names — it does not compute or check types (see
//! `typecheck.rs`). A reference that fails to resolve is left with its `id`
//! (or `binding`) as `None` and a diagnostic is recorded; `typecheck.rs`
//! treats `None` as already-erred and does not re-report it.

use std::collections::HashMap;

use log::debug;
use log::trace;
use merc_utilities::Span;

use crate::ast::*;
use crate::diagnostics::DiagnosticKind;
use crate::diagnostics::Diagnostics;

/// What kind of thing a top-level [DefId] names.
#[derive(Clone, Debug)]
pub enum DefKind {
    Constant,
    Parameter,
    Variable {
        global: bool,
    },
    Function {
        argument_count: usize,
    },
    Penalty,
    Component,
    /// An element of a custom `type X = A | B | C;` declaration.
    TypeElement {
        type_name: String,
    },
    Type,
    Perturbation,
    Distance,
    Formula,
}

impl DefKind {
    /// Whether a plain `ExpressionKind::Reference` may resolve to this kind.
    fn is_referenceable_value(&self) -> bool {
        matches!(
            self,
            DefKind::Constant | DefKind::Parameter | DefKind::Variable { .. } | DefKind::TypeElement { .. }
        )
    }

    fn describe(&self) -> &'static str {
        match self {
            DefKind::Constant => "a constant",
            DefKind::Parameter => "a parameter",
            DefKind::Variable { .. } => "a variable",
            DefKind::Function { .. } => "a function",
            DefKind::Penalty => "a penalty",
            DefKind::Component => "a component",
            DefKind::TypeElement { .. } => "a type element",
            DefKind::Type => "a type",
            DefKind::Perturbation => "a perturbation",
            DefKind::Distance => "a distance",
            DefKind::Formula => "a formula",
        }
    }

    fn is_variable_kind(kind: &DefKind) -> bool {
        matches!(kind, DefKind::Variable { .. })
    }
    fn is_function_kind(kind: &DefKind) -> bool {
        matches!(kind, DefKind::Function { .. })
    }
    fn is_penalty_kind(kind: &DefKind) -> bool {
        matches!(kind, DefKind::Penalty)
    }
    fn is_distance_kind(kind: &DefKind) -> bool {
        matches!(kind, DefKind::Distance)
    }
    fn is_perturbation_kind(kind: &DefKind) -> bool {
        matches!(kind, DefKind::Perturbation)
    }
    fn is_formula_kind(kind: &DefKind) -> bool {
        matches!(kind, DefKind::Formula)
    }
}

pub struct DefEntry {
    pub kind: DefKind,
    pub name: String,
    pub span: Span,
}

pub struct StateEntry {
    pub name: String,
    pub span: Span,
    /// The component this state belongs to.
    pub component: DefId,
}

pub struct LocalEntry {
    pub name: String,
    pub span: Span,
}

/// The result of name resolution: every declaration encountered, indexed by
/// the [DefId] / [StateId] / [LocalId] assigned to it.
#[derive(Default)]
pub struct SymbolTable {
    pub defs: Vec<DefEntry>,
    pub states: Vec<StateEntry>,
    pub locals: Vec<LocalEntry>,
    /// Top-level names, for lookups that don't go through an already-resolved
    /// [DefRef].
    pub names: HashMap<String, DefId>,
}

impl SymbolTable {
    pub fn def(&self, id: DefId) -> &DefEntry {
        &self.defs[id.value()]
    }

    pub fn state(&self, id: StateId) -> &StateEntry {
        &self.states[id.value()]
    }

    pub fn local(&self, id: LocalId) -> &LocalEntry {
        &self.locals[id.value()]
    }

    pub fn by_name(&self, name: &str) -> Option<DefId> {
        self.names.get(name).copied()
    }
}

/// Resolves every name in `spec` in place, returning the resulting
/// [SymbolTable] together with every diagnostic found along the way.
pub fn resolve(spec: &mut UntypedStarkSpecification) -> (SymbolTable, Diagnostics) {
    let mut resolver = Resolver {
        table: SymbolTable::default(),
        scopes: Vec::new(),
        diagnostics: Diagnostics::new(),
        functions_reading_variables: HashMap::new(),
    };
    resolver.resolve_specification(spec);

    // Every scope opened during the walk must have been closed again.
    debug_assert!(
        resolver.scopes.is_empty(),
        "{} local scope(s) left open after resolution",
        resolver.scopes.len()
    );
    // `declare` always pushes a `DefEntry` and inserts into `names` together.
    debug_assert_eq!(
        resolver.table.defs.len(),
        resolver.table.names.len(),
        "symbol table's `defs` and `names` disagree on how many names were declared"
    );

    debug!(
        "resolved {} definition(s), {} controller state(s), {} local binding(s); {} diagnostic(s)",
        resolver.table.defs.len(),
        resolver.table.states.len(),
        resolver.table.locals.len(),
        resolver.diagnostics.items().len()
    );

    // The contract every later pass relies on: a specification that resolved
    // cleanly has *no* `None` ids left anywhere. Checking it here means a
    // resolver bug surfaces as a failure in this pass rather than as a
    // confusing `unwrap` far downstream in lowering.
    #[cfg(debug_assertions)]
    if !resolver.diagnostics.has_errors() {
        assert_fully_resolved(spec);
    }

    (resolver.table, resolver.diagnostics)
}

struct Resolver {
    table: SymbolTable,
    /// Local scopes (function arguments, `let` bindings), innermost last.
    scopes: Vec<HashMap<String, LocalId>>,
    diagnostics: Diagnostics,
    /// Functions that read a state variable, mapped to the name of one such
    /// variable (for the diagnostic). Populated by
    /// [Resolver::check_static_expressions]; empty before then.
    functions_reading_variables: HashMap<DefId, String>,
}

impl Resolver {
    // -- Declaring names ------------------------------------------------

    /// Registers a new top-level declaration. On a name clash, records a
    /// duplicate-definition diagnostic and leaves the *new* declaration
    /// unregistered (`names` keeps pointing at the first one, matching
    /// "first wins"); the caller should leave that declaration's `id` as
    /// `None`.
    fn declare(&mut self, name: &Identifier, kind: DefKind) -> Option<DefId> {
        if let Some(&existing) = self.table.names.get(&name.name) {
            let first = self.table.def(existing).span.clone();
            self.diagnostics.error(
                name.span.clone(),
                DiagnosticKind::DuplicateDefinition {
                    name: name.name.clone(),
                    first,
                },
            );
            return None;
        }
        let id = DefId::new(self.table.defs.len());
        trace!("declaring {} `{}` as {id:?}", kind.describe(), name.name);
        self.table.defs.push(DefEntry {
            kind,
            name: name.name.clone(),
            span: name.span.clone(),
        });
        self.table.names.insert(name.name.clone(), id);
        debug_assert_eq!(
            self.table.def(id).name,
            name.name,
            "`{}` was filed under the wrong id",
            name.name
        );
        Some(id)
    }

    fn declare_state(
        &mut self,
        name: &Identifier,
        component: DefId,
        states: &mut HashMap<String, StateId>,
    ) -> Option<StateId> {
        if let Some(&existing) = states.get(&name.name) {
            let first = self.table.state(existing).span.clone();
            self.diagnostics.error(
                name.span.clone(),
                DiagnosticKind::DuplicateControllerState {
                    name: name.name.clone(),
                    first,
                },
            );
            return None;
        }
        let id = StateId::new(self.table.states.len());
        trace!("declaring controller state `{}` as {id:?}", name.name);
        self.table.states.push(StateEntry {
            name: name.name.clone(),
            span: name.span.clone(),
            component,
        });
        states.insert(name.name.clone(), id);
        Some(id)
    }

    /// Opens a new local scope, declaring all of `bindings` at once (so
    /// e.g. `let a = 1 and b = 2 in ..` puts both `a` and `b` in the same
    /// frame). Duplicate names *within this same frame* are diagnosed and
    /// get no id; duplicates against an outer scope are just ordinary
    /// shadowing and are allowed. The returned vector has the same length
    /// and order as `bindings`.
    fn push_scope(&mut self, bindings: &[&Identifier]) -> Vec<Option<LocalId>> {
        let mut frame = HashMap::new();
        let mut ids = Vec::with_capacity(bindings.len());
        for name in bindings {
            if let Some(&existing) = frame.get(&name.name) {
                let first = self.table.local(existing).span.clone();
                self.diagnostics.error(
                    name.span.clone(),
                    DiagnosticKind::DuplicateBinding {
                        name: name.name.clone(),
                        first,
                    },
                );
                ids.push(None);
                continue;
            }
            let id = LocalId::new(self.table.locals.len());
            trace!("binding local `{}` as {id:?} at depth {}", name.name, self.scopes.len());
            self.table.locals.push(LocalEntry {
                name: name.name.clone(),
                span: name.span.clone(),
            });
            frame.insert(name.name.clone(), id);
            ids.push(Some(id));
        }
        self.scopes.push(frame);
        debug_assert_eq!(
            ids.len(),
            bindings.len(),
            "push_scope must return one id slot per binding"
        );
        ids
    }

    fn pop_scope(&mut self) {
        debug_assert!(!self.scopes.is_empty(), "pop_scope without a matching push_scope");
        self.scopes.pop();
    }

    // -- Looking up names -------------------------------------------------

    fn lookup_local(&self, name: &str) -> Option<LocalId> {
        self.scopes.iter().rev().find_map(|frame| frame.get(name).copied())
    }

    fn unknown_symbol(&mut self, name: &Identifier) {
        self.diagnostics.error(
            name.span.clone(),
            DiagnosticKind::UnknownSymbol {
                name: name.name.clone(),
            },
        );
    }

    /// Resolves a [DefRef] against the top-level namespace, requiring the
    /// resolved declaration's kind to satisfy `expected`.
    fn resolve_def_ref(
        &mut self,
        reference: &mut DefRef,
        expected: impl Fn(&DefKind) -> bool,
        expected_desc: &'static str,
    ) {
        let Some(id) = self.table.names.get(&reference.name.name).copied() else {
            self.unknown_symbol(&reference.name);
            return;
        };
        if expected(&self.table.def(id).kind) {
            reference.id = Some(id);
        } else {
            let kind = self.table.def(id).kind.clone();
            self.diagnostics.error(
                reference.name.span.clone(),
                DiagnosticKind::IllegalUseOfName {
                    name: reference.name.name.clone(),
                    found: kind.describe(),
                    expected: expected_desc,
                },
            );
        }
    }

    fn resolve_state_ref(&mut self, reference: &mut StateRef, states: &HashMap<String, StateId>) {
        match states.get(&reference.name.name) {
            Some(&id) => reference.id = Some(id),
            None => {
                self.diagnostics.error(
                    reference.name.span.clone(),
                    DiagnosticKind::UnknownControllerState {
                        name: reference.name.name.clone(),
                    },
                );
            }
        }
    }

    /// Resolves a name reference inside an ordinary expression: locals
    /// shadow top-level declarations, and only "value" kinds are legal here.
    fn resolve_reference(&mut self, name: &str, span: &Span) -> Option<Binding> {
        if let Some(id) = self.lookup_local(name) {
            return Some(Binding::Local(id));
        }
        let Some(id) = self.table.names.get(name).copied() else {
            self.diagnostics
                .error(span.clone(), DiagnosticKind::UnknownSymbol { name: name.to_string() });
            return None;
        };
        if self.table.def(id).kind.is_referenceable_value() {
            Some(Binding::Def(id))
        } else {
            let kind = self.table.def(id).kind.clone();
            self.diagnostics.error(
                span.clone(),
                DiagnosticKind::IllegalUseOfName {
                    name: name.to_string(),
                    found: kind.describe(),
                    expected: "a constant, parameter, variable or type element",
                },
            );
            None
        }
    }

    // -- Top-level walk -----------------------------------------------------

    fn resolve_specification(&mut self, spec: &mut UntypedStarkSpecification) {
        debug!(
            "resolving specification: {} constant(s), {} parameter(s), {} type(s), {} function(s), \
             {} variable(s), {} component(s), {} penalty/-ies, {} perturbation(s), {} distance(s), {} formula(s)",
            spec.constants.len(),
            spec.parameters.len(),
            spec.types.len(),
            spec.functions.len(),
            spec.variables.len(),
            spec.components.len(),
            spec.penalties.len(),
            spec.perturbations.len(),
            spec.distances.len(),
            spec.formulas.len()
        );
        // State variables are visible everywhere, not just after their own
        // declaration, so register them all before resolving any body.
        for variable in &mut spec.variables {
            self.declare_variable(variable);
        }
        for component in &mut spec.components {
            for variable in &mut component.variables {
                self.declare_variable(variable);
            }
        }
        for constant in &mut spec.constants {
            self.resolve_expression(&mut constant.value);
            constant.id = self.declare(&constant.name, DefKind::Constant);
        }
        for parameter in &mut spec.parameters {
            self.resolve_expression(&mut parameter.value);
            parameter.id = self.declare(&parameter.name, DefKind::Parameter);
        }
        for ty in &mut spec.types {
            self.resolve_type_declaration(ty);
        }
        for function in &mut spec.functions {
            self.resolve_function(function);
        }
        for variable in &mut spec.variables {
            self.resolve_variable(variable);
        }
        for component in &mut spec.components {
            self.resolve_component(component);
        }
        if let Some(environment) = &mut spec.environment {
            self.resolve_environment_commands(&mut environment.commands);
        }
        for penalty in &mut spec.penalties {
            self.resolve_expression(&mut penalty.value);
            penalty.id = self.declare(&penalty.name, DefKind::Penalty);
        }
        for perturbation in &mut spec.perturbations {
            self.resolve_perturbation(&mut perturbation.value);
            perturbation.id = self.declare(&perturbation.name, DefKind::Perturbation);
        }
        for distance in &mut spec.distances {
            self.resolve_distance(&mut distance.value);
            distance.id = self.declare(&distance.name, DefKind::Distance);
        }
        for formula in &mut spec.formulas {
            self.resolve_robtl(&mut formula.value);
            formula.id = self.declare(&formula.name, DefKind::Formula);
        }
        self.check_static_expressions(spec);
    }

    /// Reports state variables read from expressions that are evaluated once
    /// at load time, before any variable store exists. Pre-declaring
    /// variables makes those names resolve everywhere, so this is what keeps
    /// `const a = X;` and `real X = X;` from being silently accepted — the
    /// original has this hole, evaluating such reads to its absorbing error
    /// value at runtime with no diagnostic.
    ///
    /// Runs as a post-pass so every function is resolved and its
    /// [Self::functions_reading_variables] entry is known.
    fn check_static_expressions(&mut self, spec: &UntypedStarkSpecification) {
        // A function body may legitimately read a variable — it is called
        // from controllers and environment blocks, where the store exists.
        // Calling one from a static expression is what makes it a problem,
        // so the offending functions have to be identified first.
        //
        // `spec.functions` is a valid topological order: a function can only
        // call one declared before it (its own `DefId` is registered after
        // its body resolves, so there is no recursion), which means each
        // callee's entry is already final when its caller is visited.
        for function in &spec.functions {
            let reads = self.statement_reads_variable(&function.body);
            if let (Some(id), Some(name)) = (function.id, reads) {
                self.functions_reading_variables.insert(id, name);
            }
        }

        for constant in &spec.constants {
            self.reject_state_variables(&constant.value, "const");
        }
        for parameter in &spec.parameters {
            self.reject_state_variables(&parameter.value, "param");
        }
        let variables = spec
            .variables
            .iter()
            .chain(spec.components.iter().flat_map(|c| c.variables.iter()));
        for variable in variables {
            if let Some(range) = &variable.range {
                self.reject_state_variables(&range.min, "variable range");
                self.reject_state_variables(&range.max, "variable range");
            }
            self.reject_state_variables(&variable.initial_value, "variable initializer");
        }
    }

    /// Records a diagnostic for every state variable `expr` reads, whether
    /// directly or through a function call. `context` names the kind of
    /// static expression, for the message.
    fn reject_state_variables(&mut self, expr: &Expression, context: &'static str) {
        for_each_subexpression(expr, &mut |expr| {
            let offender = match &expr.node {
                ExpressionKind::Reference {
                    binding: Some(Binding::Def(id)),
                    name,
                } if DefKind::is_variable_kind(&self.table.def(*id).kind) => Some((name.clone(), None)),
                ExpressionKind::Call { function, .. } => function
                    .id
                    .and_then(|id| self.functions_reading_variables.get(&id))
                    .map(|variable| (variable.clone(), Some(function.name.name.clone()))),
                _ => None,
            };
            if let Some((name, via)) = offender {
                self.diagnostics.error(
                    expr.span.clone(),
                    DiagnosticKind::StateVariableInStaticExpression { name, context, via },
                );
            }
        });
    }

    /// The name of some state variable `statement` reads, directly or
    /// through a call, if there is one.
    fn statement_reads_variable(&self, statement: &FunctionStatement) -> Option<String> {
        let mut found = None;
        let mut visit_expression = |expr: &Expression| {
            for_each_subexpression(expr, &mut |expr| {
                if found.is_some() {
                    return;
                }
                found = match &expr.node {
                    ExpressionKind::Reference {
                        binding: Some(Binding::Def(id)),
                        name,
                    } if DefKind::is_variable_kind(&self.table.def(*id).kind) => Some(name.clone()),
                    ExpressionKind::Call { function, .. } => function
                        .id
                        .and_then(|id| self.functions_reading_variables.get(&id))
                        .cloned(),
                    _ => None,
                };
            });
        };
        for_each_statement_expression(statement, &mut visit_expression);
        found
    }

    /// Registers a variable's name. Split out from [Self::resolve_variable]
    /// so every variable in the specification can be declared in one pass up
    /// front; the initializer and range are resolved later.
    fn declare_variable(&mut self, variable: &mut Variable) {
        variable.id = self.declare(
            &variable.name,
            DefKind::Variable {
                global: variable.global,
            },
        );
    }

    /// Resolves the parts of a variable that reference other names. The name
    /// itself is already registered by [Self::declare_variable].
    fn resolve_variable(&mut self, variable: &mut Variable) {
        if let Some(range) = &mut variable.range {
            self.resolve_expression(&mut range.min);
            self.resolve_expression(&mut range.max);
        }
        self.resolve_expression(&mut variable.initial_value);
    }

    fn resolve_type_declaration(&mut self, ty: &mut TypeDeclaration) {
        // A type name colliding with one of its own elements isn't caught by
        // the general duplicate check below (neither is registered yet at
        // the point we'd check), so it needs its own check.
        if ty.elements.iter().any(|e| e.name == ty.name.name) {
            self.diagnostics.error(
                ty.name.span.clone(),
                DiagnosticKind::TypeElementSharesTypeName {
                    name: ty.name.name.clone(),
                },
            );
        } else {
            ty.id = self.declare(&ty.name, DefKind::Type);
        }
        for element in &ty.elements {
            self.declare(
                element,
                DefKind::TypeElement {
                    type_name: ty.name.name.clone(),
                },
            );
        }
    }

    fn resolve_function(&mut self, function: &mut Function) {
        trace!(
            "resolving function `{}` with {} argument(s)",
            function.name.name,
            function.arguments.len()
        );
        let bindings: Vec<&Identifier> = function.arguments.iter().map(|arg| &arg.name).collect();
        let ids = self.push_scope(&bindings);
        for (argument, id) in function.arguments.iter_mut().zip(ids) {
            argument.id = id;
        }
        self.resolve_function_statement(&mut function.body);
        self.pop_scope();

        // Registered *after* the body so the function cannot call itself —
        // see the module doc comment.
        function.id = self.declare(
            &function.name,
            DefKind::Function {
                argument_count: function.arguments.len(),
            },
        );
    }

    fn resolve_function_statement(&mut self, statement: &mut FunctionStatement) {
        match statement {
            FunctionStatement::Return(value) => self.resolve_expression(value),
            FunctionStatement::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(guard);
                self.resolve_function_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_function_statement(else_branch);
                }
            }
            FunctionStatement::Let { id, name, value, body } => {
                self.resolve_expression(value);
                let ids = self.push_scope(&[&*name]);
                *id = ids.into_iter().next().flatten();
                self.resolve_function_statement(body);
                self.pop_scope();
            }
            FunctionStatement::Block(inner) => self.resolve_function_statement(inner),
        }
    }

    fn resolve_component(&mut self, component: &mut Component) {
        trace!(
            "resolving component `{}` with {} variable(s) and {} state(s)",
            component.name.name,
            component.variables.len(),
            component.states.len()
        );
        for variable in &mut component.variables {
            self.resolve_variable(variable);
        }
        component.id = self.declare(&component.name, DefKind::Component);
        let Some(component_id) = component.id else {
            return;
        };

        // States can reference each other regardless of declaration order,
        // so register them all before resolving any body.
        let mut states = HashMap::new();
        for state in &mut component.states {
            state.id = self.declare_state(&state.name, component_id, &mut states);
        }
        for state in &mut component.states {
            self.resolve_controller_commands(&mut state.body, &states);
        }
        for target in &mut component.init {
            self.resolve_state_ref(target, &states);
        }
    }

    fn resolve_controller_commands(&mut self, commands: &mut [ControllerCommand], states: &HashMap<String, StateId>) {
        for command in commands {
            match command {
                ControllerCommand::Step { steps, target } => {
                    if let Some(steps) = steps {
                        self.resolve_expression(steps);
                    }
                    self.resolve_state_ref(target, states);
                }
                ControllerCommand::Exec(target) => self.resolve_state_ref(target, states),
                ControllerCommand::Let { id, name, value, body } => {
                    self.resolve_expression(value);
                    let ids = self.push_scope(&[&*name]);
                    *id = ids.into_iter().next().flatten();
                    self.resolve_controller_commands(body, states);
                    self.pop_scope();
                }
                ControllerCommand::Assignment(update) => self.resolve_update(update),
                ControllerCommand::IfThenElse {
                    guard,
                    then_branch,
                    else_branch,
                } => {
                    self.resolve_expression(guard);
                    self.resolve_controller_commands(then_branch, states);
                    if let Some(else_branch) = else_branch {
                        self.resolve_controller_commands(else_branch, states);
                    }
                }
                ControllerCommand::Block(inner) => self.resolve_controller_commands(inner, states),
            }
        }
    }

    fn resolve_environment_commands(&mut self, commands: &mut [EnvironmentCommand]) {
        for command in commands {
            self.resolve_environment_command(command);
        }
    }

    fn resolve_environment_command(&mut self, command: &mut EnvironmentCommand) {
        match command {
            EnvironmentCommand::Assignment(update) => self.resolve_update(update),
            EnvironmentCommand::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(guard);
                self.resolve_environment_command(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_environment_command(else_branch);
                }
            }
            EnvironmentCommand::Let { bindings, body } => {
                // `let a = e1 and b = e2(a) and ... in body`: each binding's
                // value can see every binding *before* it in the same chain
                // (this is what `toll.stark`'s `new_sens_speed = new_s_speed(
                // ..., token)` relies on, referencing the `token` bound
                // immediately before it) — so each binding opens its own
                // nested scope rather than all of them sharing one frame.
                for binding in bindings.iter_mut() {
                    self.resolve_expression(&mut binding.value);
                    let ids = self.push_scope(&[&binding.name]);
                    binding.id = ids.into_iter().next().flatten();
                }
                self.resolve_environment_command(body);
                for _ in bindings.iter() {
                    self.pop_scope();
                }
            }
            EnvironmentCommand::Block(inner) => self.resolve_environment_commands(inner),
        }
    }

    fn resolve_update(&mut self, update: &mut Update) {
        if let Some(guard) = &mut update.guard {
            self.resolve_expression(guard);
        }
        self.resolve_expression(&mut update.value);
        self.resolve_def_ref(&mut update.target, DefKind::is_variable_kind, "a variable");
    }

    // -- Sub-languages --------------------------------------------------

    fn resolve_perturbation(&mut self, perturbation: &mut PerturbationExpression) {
        match perturbation {
            PerturbationExpression::Nil => {}
            PerturbationExpression::Reference(reference) => {
                self.resolve_def_ref(reference, DefKind::is_perturbation_kind, "a perturbation")
            }
            PerturbationExpression::Atomic { assignments, time } => {
                for assignment in assignments {
                    self.resolve_expression(&mut assignment.value);
                    self.resolve_def_ref(&mut assignment.target, DefKind::is_variable_kind, "a variable");
                }
                self.resolve_expression(time);
            }
            PerturbationExpression::Sequence(left, right) => {
                self.resolve_perturbation(left);
                self.resolve_perturbation(right);
            }
            PerturbationExpression::Iteration { argument, iterations } => {
                self.resolve_perturbation(argument);
                self.resolve_expression(iterations);
            }
        }
    }

    fn resolve_distance(&mut self, distance: &mut DistanceExpression) {
        match distance {
            DistanceExpression::Reference(reference) => {
                self.resolve_def_ref(reference, DefKind::is_distance_kind, "a distance")
            }
            DistanceExpression::AtomicLeft(reference) | DistanceExpression::AtomicRight(reference) => {
                self.resolve_def_ref(reference, DefKind::is_penalty_kind, "a penalty")
            }
            DistanceExpression::Eventually { from, to, argument }
            | DistanceExpression::Globally { from, to, argument } => {
                self.resolve_expression(from);
                self.resolve_expression(to);
                self.resolve_distance(argument);
            }
            DistanceExpression::Until { from, to, left, right } => {
                self.resolve_expression(from);
                self.resolve_expression(to);
                self.resolve_distance(left);
                self.resolve_distance(right);
            }
            DistanceExpression::Threshold { left, threshold, .. } => {
                self.resolve_distance(left);
                self.resolve_expression(threshold);
            }
            DistanceExpression::Min(left, right) | DistanceExpression::Max(left, right) => {
                self.resolve_distance(left);
                self.resolve_distance(right);
            }
            DistanceExpression::LinearCombination(terms) => {
                for (weight, distance) in terms {
                    self.resolve_expression(weight);
                    self.resolve_distance(distance);
                }
            }
        }
    }

    fn resolve_robtl(&mut self, formula: &mut RobtlFormula) {
        match formula {
            RobtlFormula::True | RobtlFormula::False => {}
            RobtlFormula::Reference(reference) => {
                self.resolve_def_ref(reference, DefKind::is_formula_kind, "a formula")
            }
            RobtlFormula::Distance {
                distance,
                perturbation,
                value,
                ..
            } => {
                self.resolve_def_ref(distance, DefKind::is_distance_kind, "a distance");
                self.resolve_def_ref(perturbation, DefKind::is_perturbation_kind, "a perturbation");
                self.resolve_expression(value);
            }
            RobtlFormula::Not(inner) => self.resolve_robtl(inner),
            RobtlFormula::Globally { from, to, argument } | RobtlFormula::Eventually { from, to, argument } => {
                self.resolve_expression(from);
                self.resolve_expression(to);
                self.resolve_robtl(argument);
            }
            RobtlFormula::And(left, right) | RobtlFormula::Or(left, right) => {
                self.resolve_robtl(left);
                self.resolve_robtl(right);
            }
            RobtlFormula::Until { from, to, left, right } => {
                self.resolve_expression(from);
                self.resolve_expression(to);
                self.resolve_robtl(left);
                self.resolve_robtl(right);
            }
        }
    }

    fn resolve_expression(&mut self, expr: &mut Expression) {
        match &mut expr.node {
            ExpressionKind::False
            | ExpressionKind::True
            | ExpressionKind::Integer(_)
            | ExpressionKind::Real(_)
            | ExpressionKind::Iterator => {}
            ExpressionKind::Reference { name, binding } => {
                *binding = self.resolve_reference(name, &expr.span);
            }
            ExpressionKind::Normal { mean, std_dev } => {
                self.resolve_expression(mean);
                self.resolve_expression(std_dev);
            }
            ExpressionKind::Uniform { values } => {
                for value in values {
                    self.resolve_expression(value);
                }
            }
            ExpressionKind::Range { min, max } => {
                if let Some(min) = min {
                    self.resolve_expression(min);
                }
                if let Some(max) = max {
                    self.resolve_expression(max);
                }
            }
            ExpressionKind::Not(inner) | ExpressionKind::UnaryPlus(inner) | ExpressionKind::UnaryMinus(inner) => {
                self.resolve_expression(inner);
            }
            ExpressionKind::Binary(_, left, right) => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            ExpressionKind::Ternary {
                guard,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(guard);
                self.resolve_expression(then_branch);
                self.resolve_expression(else_branch);
            }
            ExpressionKind::Call { function, arguments } => {
                for argument in arguments.iter_mut() {
                    self.resolve_expression(argument);
                }
                self.resolve_def_ref(function, DefKind::is_function_kind, "a function");
            }
            ExpressionKind::MathCall { arguments, .. } => {
                for argument in arguments {
                    self.resolve_expression(argument);
                }
            }
        }
    }
}

/// Applies `visit` to `expr` and every subexpression of it, outermost first.
fn for_each_subexpression(expr: &Expression, visit: &mut impl FnMut(&Expression)) {
    visit(expr);
    match &expr.node {
        ExpressionKind::False
        | ExpressionKind::True
        | ExpressionKind::Integer(_)
        | ExpressionKind::Real(_)
        | ExpressionKind::Iterator
        | ExpressionKind::Reference { .. } => {}
        ExpressionKind::Normal { mean, std_dev } => {
            for_each_subexpression(mean, visit);
            for_each_subexpression(std_dev, visit);
        }
        ExpressionKind::Uniform { values } => {
            for value in values {
                for_each_subexpression(value, visit);
            }
        }
        ExpressionKind::Range { min, max } => {
            for bound in [min, max].into_iter().flatten() {
                for_each_subexpression(bound, visit);
            }
        }
        ExpressionKind::Not(inner) | ExpressionKind::UnaryPlus(inner) | ExpressionKind::UnaryMinus(inner) => {
            for_each_subexpression(inner, visit);
        }
        ExpressionKind::Binary(_, left, right) => {
            for_each_subexpression(left, visit);
            for_each_subexpression(right, visit);
        }
        ExpressionKind::Ternary {
            guard,
            then_branch,
            else_branch,
        } => {
            for_each_subexpression(guard, visit);
            for_each_subexpression(then_branch, visit);
            for_each_subexpression(else_branch, visit);
        }
        ExpressionKind::Call { arguments, .. } | ExpressionKind::MathCall { arguments, .. } => {
            for argument in arguments {
                for_each_subexpression(argument, visit);
            }
        }
    }
}

/// Applies `visit` to every expression appearing anywhere in `statement`.
fn for_each_statement_expression(statement: &FunctionStatement, visit: &mut impl FnMut(&Expression)) {
    match statement {
        FunctionStatement::Return(value) => visit(value),
        FunctionStatement::IfThenElse {
            guard,
            then_branch,
            else_branch,
        } => {
            visit(guard);
            for_each_statement_expression(then_branch, visit);
            if let Some(else_branch) = else_branch {
                for_each_statement_expression(else_branch, visit);
            }
        }
        FunctionStatement::Let { value, body, .. } => {
            visit(value);
            for_each_statement_expression(body, visit);
        }
        FunctionStatement::Block(inner) => for_each_statement_expression(inner, visit),
    }
}

/// Asserts the post-condition of a clean resolution: no `id`/`binding` slot
/// anywhere in `spec` is still `None`.
///
/// This is the invariant every later pass is entitled to assume — `typecheck.rs`
/// treats `None` as "already diagnosed", and the planned lowering pass indexes
/// through these ids unconditionally. If resolution reports no diagnostics but
/// leaves a slot empty, that is a resolver bug, and it is much cheaper to catch
/// it here than as an `unwrap` three passes later. Debug builds only.
#[cfg(debug_assertions)]
fn assert_fully_resolved(spec: &UntypedStarkSpecification) {
    fn check_expression(expr: &Expression) {
        match &expr.node {
            ExpressionKind::False
            | ExpressionKind::True
            | ExpressionKind::Integer(_)
            | ExpressionKind::Real(_)
            | ExpressionKind::Iterator => {}
            ExpressionKind::Reference { name, binding } => {
                assert!(
                    binding.is_some(),
                    "reference `{name}` left unbound by a clean resolution"
                );
            }
            ExpressionKind::Normal { mean, std_dev } => {
                check_expression(mean);
                check_expression(std_dev);
            }
            ExpressionKind::Uniform { values } => values.iter().for_each(check_expression),
            ExpressionKind::Range { min, max } => {
                min.iter().for_each(|e| check_expression(e));
                max.iter().for_each(|e| check_expression(e));
            }
            ExpressionKind::Not(inner) | ExpressionKind::UnaryPlus(inner) | ExpressionKind::UnaryMinus(inner) => {
                check_expression(inner)
            }
            ExpressionKind::Binary(_, left, right) => {
                check_expression(left);
                check_expression(right);
            }
            ExpressionKind::Ternary {
                guard,
                then_branch,
                else_branch,
            } => {
                check_expression(guard);
                check_expression(then_branch);
                check_expression(else_branch);
            }
            ExpressionKind::Call { function, arguments } => {
                assert!(
                    function.id.is_some(),
                    "call to `{}` left unresolved",
                    function.name.name
                );
                arguments.iter().for_each(check_expression);
            }
            ExpressionKind::MathCall { arguments, .. } => arguments.iter().for_each(check_expression),
        }
    }

    fn check_variable(variable: &Variable) {
        assert!(
            variable.id.is_some(),
            "variable `{}` left undeclared",
            variable.name.name
        );
        if let Some(range) = &variable.range {
            check_expression(&range.min);
            check_expression(&range.max);
        }
        check_expression(&variable.initial_value);
    }

    fn check_update(update: &Update) {
        update.guard.iter().for_each(check_expression);
        check_expression(&update.value);
        assert!(
            update.target.id.is_some(),
            "assignment target `{}` left unresolved",
            update.target.name.name
        );
    }

    fn check_function_statement(statement: &FunctionStatement) {
        match statement {
            FunctionStatement::Return(value) => check_expression(value),
            FunctionStatement::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                check_expression(guard);
                check_function_statement(then_branch);
                else_branch.iter().for_each(|s| check_function_statement(s));
            }
            FunctionStatement::Let { id, name, value, body } => {
                check_expression(value);
                assert!(id.is_some(), "let binding `{}` left unbound", name.name);
                check_function_statement(body);
            }
            FunctionStatement::Block(inner) => check_function_statement(inner),
        }
    }

    fn check_controller_commands(commands: &[ControllerCommand]) {
        for command in commands {
            match command {
                ControllerCommand::Step { steps, target } => {
                    steps.iter().for_each(check_expression);
                    assert!(
                        target.id.is_some(),
                        "step target `{}` left unresolved",
                        target.name.name
                    );
                }
                ControllerCommand::Exec(target) => {
                    assert!(
                        target.id.is_some(),
                        "exec target `{}` left unresolved",
                        target.name.name
                    );
                }
                ControllerCommand::Let { id, name, value, body } => {
                    check_expression(value);
                    assert!(id.is_some(), "let binding `{}` left unbound", name.name);
                    check_controller_commands(body);
                }
                ControllerCommand::Assignment(update) => check_update(update),
                ControllerCommand::IfThenElse {
                    guard,
                    then_branch,
                    else_branch,
                } => {
                    check_expression(guard);
                    check_controller_commands(then_branch);
                    else_branch.iter().for_each(|b| check_controller_commands(b));
                }
                ControllerCommand::Block(inner) => check_controller_commands(inner),
            }
        }
    }

    fn check_environment_command(command: &EnvironmentCommand) {
        match command {
            EnvironmentCommand::Assignment(update) => check_update(update),
            EnvironmentCommand::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                check_expression(guard);
                check_environment_command(then_branch);
                else_branch.iter().for_each(|c| check_environment_command(c));
            }
            EnvironmentCommand::Let { bindings, body } => {
                for binding in bindings {
                    check_expression(&binding.value);
                    assert!(binding.id.is_some(), "let binding `{}` left unbound", binding.name.name);
                }
                check_environment_command(body);
            }
            EnvironmentCommand::Block(inner) => inner.iter().for_each(check_environment_command),
        }
    }

    fn check_perturbation(perturbation: &PerturbationExpression) {
        match perturbation {
            PerturbationExpression::Nil => {}
            PerturbationExpression::Reference(reference) => {
                assert!(
                    reference.id.is_some(),
                    "perturbation `{}` left unresolved",
                    reference.name.name
                );
            }
            PerturbationExpression::Atomic { assignments, time } => {
                for assignment in assignments {
                    check_expression(&assignment.value);
                    assert!(
                        assignment.target.id.is_some(),
                        "perturbation target `{}` left unresolved",
                        assignment.target.name.name
                    );
                }
                check_expression(time);
            }
            PerturbationExpression::Sequence(left, right) => {
                check_perturbation(left);
                check_perturbation(right);
            }
            PerturbationExpression::Iteration { argument, iterations } => {
                check_perturbation(argument);
                check_expression(iterations);
            }
        }
    }

    fn check_distance(distance: &DistanceExpression) {
        match distance {
            DistanceExpression::Reference(reference)
            | DistanceExpression::AtomicLeft(reference)
            | DistanceExpression::AtomicRight(reference) => {
                assert!(reference.id.is_some(), "`{}` left unresolved", reference.name.name);
            }
            DistanceExpression::Eventually { from, to, argument }
            | DistanceExpression::Globally { from, to, argument } => {
                check_expression(from);
                check_expression(to);
                check_distance(argument);
            }
            DistanceExpression::Until { from, to, left, right } => {
                check_expression(from);
                check_expression(to);
                check_distance(left);
                check_distance(right);
            }
            DistanceExpression::Threshold { left, threshold, .. } => {
                check_distance(left);
                check_expression(threshold);
            }
            DistanceExpression::Min(left, right) | DistanceExpression::Max(left, right) => {
                check_distance(left);
                check_distance(right);
            }
            DistanceExpression::LinearCombination(terms) => {
                for (weight, distance) in terms {
                    check_expression(weight);
                    check_distance(distance);
                }
            }
        }
    }

    fn check_robtl(formula: &RobtlFormula) {
        match formula {
            RobtlFormula::True | RobtlFormula::False => {}
            RobtlFormula::Reference(reference) => {
                assert!(
                    reference.id.is_some(),
                    "formula `{}` left unresolved",
                    reference.name.name
                );
            }
            RobtlFormula::Distance {
                distance,
                perturbation,
                value,
                ..
            } => {
                assert!(
                    distance.id.is_some(),
                    "distance `{}` left unresolved",
                    distance.name.name
                );
                assert!(
                    perturbation.id.is_some(),
                    "perturbation `{}` left unresolved",
                    perturbation.name.name
                );
                check_expression(value);
            }
            RobtlFormula::Not(inner) => check_robtl(inner),
            RobtlFormula::Globally { from, to, argument } | RobtlFormula::Eventually { from, to, argument } => {
                check_expression(from);
                check_expression(to);
                check_robtl(argument);
            }
            RobtlFormula::And(left, right) | RobtlFormula::Or(left, right) => {
                check_robtl(left);
                check_robtl(right);
            }
            RobtlFormula::Until { from, to, left, right } => {
                check_expression(from);
                check_expression(to);
                check_robtl(left);
                check_robtl(right);
            }
        }
    }

    for constant in &spec.constants {
        assert!(
            constant.id.is_some(),
            "constant `{}` left undeclared",
            constant.name.name
        );
        check_expression(&constant.value);
    }
    for parameter in &spec.parameters {
        assert!(
            parameter.id.is_some(),
            "parameter `{}` left undeclared",
            parameter.name.name
        );
        check_expression(&parameter.value);
    }
    for ty in &spec.types {
        assert!(ty.id.is_some(), "type `{}` left undeclared", ty.name.name);
    }
    for function in &spec.functions {
        assert!(
            function.id.is_some(),
            "function `{}` left undeclared",
            function.name.name
        );
        for argument in &function.arguments {
            assert!(
                argument.id.is_some(),
                "argument `{}` of `{}` left unbound",
                argument.name.name,
                function.name.name
            );
        }
        check_function_statement(&function.body);
    }
    for variable in &spec.variables {
        check_variable(variable);
    }
    for component in &spec.components {
        assert!(
            component.id.is_some(),
            "component `{}` left undeclared",
            component.name.name
        );
        component.variables.iter().for_each(check_variable);
        for state in &component.states {
            assert!(
                state.id.is_some(),
                "controller state `{}` left undeclared",
                state.name.name
            );
            check_controller_commands(&state.body);
        }
        for target in &component.init {
            assert!(
                target.id.is_some(),
                "init target `{}` left unresolved",
                target.name.name
            );
        }
    }
    if let Some(environment) = &spec.environment {
        environment.commands.iter().for_each(check_environment_command);
    }
    for penalty in &spec.penalties {
        assert!(penalty.id.is_some(), "penalty `{}` left undeclared", penalty.name.name);
        check_expression(&penalty.value);
    }
    for perturbation in &spec.perturbations {
        assert!(
            perturbation.id.is_some(),
            "perturbation `{}` left undeclared",
            perturbation.name.name
        );
        check_perturbation(&perturbation.value);
    }
    for distance in &spec.distances {
        assert!(
            distance.id.is_some(),
            "distance `{}` left undeclared",
            distance.name.name
        );
        check_distance(&distance.value);
    }
    for formula in &spec.formulas {
        assert!(formula.id.is_some(), "formula `{}` left undeclared", formula.name.name);
        check_robtl(&formula.value);
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;
    use crate::ast::Binding;
    use crate::ast::ExpressionKind;
    use crate::ast::UntypedStarkSpecification;
    use crate::diagnostics::DiagnosticKind;
    // Overrides the built-in `#[test]` so `RUST_LOG=merc_stark=trace cargo test`
    // shows this pass's `debug!`/`trace!` output.
    use test_log::test;

    fn resolve_source(
        src: &str,
    ) -> (
        UntypedStarkSpecification,
        super::SymbolTable,
        crate::diagnostics::Diagnostics,
    ) {
        let mut spec = UntypedStarkSpecification::parse(src).expect("should parse");
        let (table, diagnostics) = resolve(&mut spec);
        (spec, table, diagnostics)
    }

    #[test]
    fn resolves_a_reference_to_an_earlier_constant() {
        let (spec, _table, diagnostics) = resolve_source("const a = 1;\nconst b = a + 1;");
        assert!(!diagnostics.has_errors(), "{diagnostics}");
        match &spec.constants[1].value.node {
            ExpressionKind::Binary(_, lhs, _) => {
                assert!(matches!(
                    lhs.node,
                    ExpressionKind::Reference {
                        binding: Some(Binding::Def(_)),
                        ..
                    }
                ));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn forward_reference_is_unknown_symbol() {
        let (_spec, _table, diagnostics) = resolve_source("const a = b + 1;\nconst b = 1;");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::UnknownSymbol { name } if name == "b")),
            "{diagnostics}"
        );
    }

    #[test]
    fn a_function_may_read_a_state_variable_declared_later() {
        // Variables are pre-declared, so this resolves even though the
        // `variables` block comes after the function that reads it — and
        // even though variables are otherwise resolved after functions.
        let (_spec, _table, diagnostics) =
            resolve_source("function f() {\n  return X * 2.0;\n}\nglobal variables {\n  real X = 1.0;\n}");
        assert!(!diagnostics.has_errors(), "{diagnostics}");
    }

    #[test]
    fn a_component_variable_is_visible_before_its_component() {
        let source = "function f() {\n  return v * 2.0;\n}\n\
             component C {\n  variables {\n    real v = 1.0;\n  }\n  \
             controller {\n    state Idle {\n      step Idle;\n    }\n  }\n  init Idle\n}";
        let (_spec, _table, diagnostics) = resolve_source(source);
        assert!(!diagnostics.has_errors(), "{diagnostics}");
    }

    /// The name of the state variable reported by the first
    /// `StateVariableInStaticExpression` diagnostic, if any.
    fn static_violation(diagnostics: &crate::diagnostics::Diagnostics) -> Option<String> {
        diagnostics.items().iter().find_map(|item| match &item.kind {
            DiagnosticKind::StateVariableInStaticExpression { name, .. } => Some(name.clone()),
            _ => None,
        })
    }

    #[test]
    fn a_constant_cannot_read_a_state_variable() {
        // Pre-declaring variables makes `X` resolve here, so without the
        // static check this would be silently accepted.
        let (_spec, _table, diagnostics) = resolve_source("global variables {\n  real X = 1.0;\n}\nconst a = X;");
        assert_eq!(static_violation(&diagnostics).as_deref(), Some("X"), "{diagnostics}");
    }

    #[test]
    fn a_variable_initializer_cannot_read_itself() {
        let (_spec, _table, diagnostics) = resolve_source("global variables {\n  real X = X;\n}");
        assert_eq!(static_violation(&diagnostics).as_deref(), Some("X"), "{diagnostics}");
    }

    #[test]
    fn a_variable_initializer_cannot_read_a_variable_through_a_function() {
        let (_spec, _table, diagnostics) = resolve_source(
            "function f() {\n  return X * 2.0;\n}\nglobal variables {\n  real X = 1.0;\n  real Y = f();\n}",
        );
        assert_eq!(static_violation(&diagnostics).as_deref(), Some("X"), "{diagnostics}");
    }

    #[test]
    fn a_function_reading_a_variable_is_fine_when_no_static_expression_calls_it() {
        let (_spec, _table, diagnostics) =
            resolve_source("function f() {\n  return X * 2.0;\n}\nglobal variables {\n  real X = 1.0;\n}");
        assert!(!diagnostics.has_errors(), "{diagnostics}");
    }

    #[test]
    fn a_variable_range_may_still_use_a_constant() {
        let (_spec, _table, diagnostics) =
            resolve_source("const M = 5.0;\nglobal variables {\n  real X range [0,M] = 1.0;\n}");
        assert!(!diagnostics.has_errors(), "{diagnostics}");
    }

    #[test]
    fn duplicate_variable_names_are_still_caught_by_the_pre_pass() {
        let (_spec, _table, diagnostics) = resolve_source("global variables {\n  real X = 1.0;\n  real X = 2.0;\n}");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::DuplicateDefinition { name, .. } if name == "X")),
            "{diagnostics}"
        );
    }

    #[test]
    fn duplicate_top_level_name_is_an_error() {
        let (_spec, _table, diagnostics) = resolve_source("const a = 1;\nconst a = 2;");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::DuplicateDefinition { name, .. } if name == "a")),
            "{diagnostics}"
        );
    }

    #[test]
    fn calling_a_variable_is_illegal_use_of_name() {
        let (_spec, _table, diagnostics) =
            // In a `const` this would only report "unknown symbol": constants
            // resolve before variables in the fixed kind order (see the module
            // doc comment), so `x` isn't declared yet there. A `penalty`
            // resolves after both, so the name *is* found — and rejected for
            // being the wrong kind, which is what this test is about.
            resolve_source("global variables { int x = 0; }\npenalty p = x(1)");
        assert!(
            diagnostics.any(|kind| matches!(
                kind,
                DiagnosticKind::IllegalUseOfName { name, found, expected }
                    if name == "x" && *found == "a variable" && *expected == "a function"
            )),
            "{diagnostics}"
        );
    }

    #[test]
    fn referencing_a_function_as_a_value_is_illegal_use_of_name() {
        let (_spec, _table, diagnostics) =
            // Likewise: a `penalty` resolves after functions, so `f` is found
            // and then rejected as a non-value, rather than being reported as
            // an unknown symbol.
            resolve_source("function f(int x) { return x; }\npenalty p = f");
        assert!(
            diagnostics.any(|kind| matches!(
                kind,
                DiagnosticKind::IllegalUseOfName { name, found, .. } if name == "f" && *found == "a function"
            )),
            "{diagnostics}"
        );
    }

    #[test]
    fn function_cannot_call_itself() {
        let (_spec, _table, diagnostics) = resolve_source("function f(int x) { return f(x); }");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::UnknownSymbol { name } if name == "f")),
            "{diagnostics}"
        );
    }

    #[test]
    fn let_binding_shadows_outer_constant() {
        let (spec, _table, diagnostics) =
            resolve_source("const x = 1;\nfunction f(int y) { let x = 2 in return x + y; }");
        assert!(!diagnostics.has_errors(), "{diagnostics}");
        // Both `x` (the let) and `y` (the argument) resolve locally.
        let crate::ast::FunctionStatement::Block(inner) = &spec.functions[0].body else {
            panic!("expected a block");
        };
        let crate::ast::FunctionStatement::Let { body, .. } = inner.as_ref() else {
            panic!("expected a let statement");
        };
        let crate::ast::FunctionStatement::Return(value) = body.as_ref() else {
            panic!("expected a return statement");
        };
        match &value.node {
            ExpressionKind::Binary(_, lhs, rhs) => {
                assert!(matches!(
                    lhs.node,
                    ExpressionKind::Reference {
                        binding: Some(Binding::Local(_)),
                        ..
                    }
                ));
                assert!(matches!(
                    rhs.node,
                    ExpressionKind::Reference {
                        binding: Some(Binding::Local(_)),
                        ..
                    }
                ));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn duplicate_function_argument_is_an_error() {
        let (_spec, _table, diagnostics) = resolve_source("function f(int x, int x) { return x; }");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::DuplicateBinding { name, .. } if name == "x")),
            "{diagnostics}"
        );
    }

    #[test]
    fn controller_state_can_forward_reference_a_sibling_state() {
        let (_spec, _table, diagnostics) = resolve_source(
            "component C {\n  variables { }\n  controller {\n    state A { step B; }\n    state B { step A; }\n  }\n  init A\n}",
        );
        assert!(!diagnostics.has_errors(), "{diagnostics}");
    }

    #[test]
    fn controller_state_cannot_target_another_components_state() {
        let (_spec, _table, diagnostics) = resolve_source(
            "component C1 {\n  variables { }\n  controller {\n    state A { step B; }\n  }\n  init A\n}\ncomponent C2 {\n  variables { }\n  controller {\n    state B { exec B; }\n  }\n  init B\n}",
        );
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::UnknownControllerState { name } if name == "B")),
            "{diagnostics}"
        );
    }

    #[test]
    fn custom_type_element_is_referenceable() {
        // `penalty` is resolved after `type` declarations in this resolver's
        // fixed kind order (see the module doc comment), so referencing a
        // type element from a penalty value exercises the forward-visibility
        // that types grant to everything processed after them.
        let (spec, _table, diagnostics) = resolve_source("type Color = Red | Green | Blue;\npenalty p = Red");
        assert!(!diagnostics.has_errors(), "{diagnostics}");
        assert!(matches!(
            spec.penalties[0].value.node,
            ExpressionKind::Reference {
                binding: Some(Binding::Def(_)),
                ..
            }
        ));
    }

    #[test]
    fn type_element_cannot_share_the_types_own_name() {
        let (_spec, _table, diagnostics) = resolve_source("type Color = Color | Blue;\nconst c = 1;");
        assert!(
            diagnostics
                .any(|kind| matches!(kind, DiagnosticKind::TypeElementSharesTypeName { name } if name == "Color")),
            "{diagnostics}"
        );
    }

    #[test]
    fn assignment_target_must_be_a_variable() {
        let (_spec, _table, diagnostics) = resolve_source("const k = 1;\nenvironment { k' = 1; }");
        assert!(
            diagnostics.any(|kind| matches!(
                kind,
                DiagnosticKind::IllegalUseOfName { name, found, expected }
                    if name == "k" && *found == "a constant" && *expected == "a variable"
            )),
            "{diagnostics}"
        );
    }

    #[test]
    fn resolves_every_example_specification_without_errors() {
        for (name, source) in [
            ("engine", include_str!("../../../examples/stark/engine.stark")),
            ("random_walk", include_str!("../../../examples/stark/random_walk.stark")),
            (
                "single_vehicle",
                include_str!("../../../examples/stark/single_vehicle.stark"),
            ),
            ("toll", include_str!("../../../examples/stark/toll.stark")),
            (
                "two_vehicles",
                include_str!("../../../examples/stark/two_vehicles.stark"),
            ),
            ("monitoring", include_str!("../../../examples/stark/monitoring.stark")),
            (
                "agriculturalDT",
                include_str!("../../../examples/stark/agriculturalDT.stark"),
            ),
            ("tollbooth", include_str!("../../../examples/stark/tollbooth.stark")),
        ] {
            let mut spec =
                UntypedStarkSpecification::parse(source).unwrap_or_else(|e| panic!("{name} failed to parse: {e}"));
            let (_table, diagnostics) = resolve(&mut spec);
            assert!(
                !diagnostics.has_errors(),
                "{name} failed to resolve:\n{}",
                diagnostics.render(source)
            );
        }
    }
}
