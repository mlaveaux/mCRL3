//! Name resolution: assigns every declaration a stable [DefId]/[StateId]/
//! [LocalId] and rewrites every reference in place to point at the
//! declaration it names, mirroring `parsing/SymbolTable.java`.
//!
//! STARK has no forward references: a name is only visible to expressions
//! that come *after* its declaration in source order (this is what
//! `SpecificationLanguageValidator`'s single top-down visitor pass implies,
//! and nothing in the example specs relies on forward references either —
//! including function self-recursion, which this resolver also rejects: a
//! function's own name is registered only *after* its body has been
//! resolved). Concretely, this means one linear walk over the declarations
//! is sufficient: by the time a name is used, everything it could legally
//! refer to has already been registered.
//!
//! The one exception is controller states: `step`/`exec` inside a state may
//! target a *later* state in the same component (state machines are
//! naturally mutually recursive), so each component's states are registered
//! in a first pass before any state body is resolved.
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
//! This pass only binds names — it does not compute or check types (see
//! `typecheck.rs`). A reference that fails to resolve is left with its `id`
//! (or `binding`) as `None` and a diagnostic is recorded; `typecheck.rs`
//! treats `None` as already-erred and does not re-report it.

use std::collections::HashMap;

use merc_utilities::Span;

use crate::ast::*;
use crate::diagnostics::Diagnostics;

/// What kind of thing a top-level [DefId] names.
#[derive(Clone, Debug)]
pub enum DefKind {
    Constant,
    Parameter,
    Variable { global: bool },
    Function { argument_count: usize },
    Penalty,
    Component,
    /// An element of a custom `type X = A | B | C;` declaration.
    TypeElement { type_name: String },
    Type,
    Perturbation,
    Distance,
    Formula,
}

impl DefKind {
    /// Whether a plain `Expression::Reference` may resolve to this kind:
    /// whether it names a *value*, as opposed to a function, penalty,
    /// component, perturbation, distance, formula or type, each of which is
    /// only referenceable from its own dedicated syntax (a call, a `\D[...]`,
    /// …), never from a bare name in an ordinary expression.
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
    /// [DefRef] — e.g. `typecheck.rs` validating a `Ty::Named` type
    /// annotation against a declared `type` (a check that has nothing to do
    /// with binding an expression reference, so it isn't performed here).
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
/// [SymbolTable] together with every diagnostic found along the way. Always
/// returns a table — even a spec with unresolved names produces one, with
/// those references left as `None` — so `typecheck.rs` can still make
/// progress on everything that *did* resolve.
pub fn resolve(spec: &mut UntypedStarkSpecification) -> (SymbolTable, Diagnostics) {
    let mut resolver = Resolver {
        table: SymbolTable::default(),
        scopes: Vec::new(),
        diagnostics: Diagnostics::new(),
    };
    resolver.resolve_specification(spec);
    (resolver.table, resolver.diagnostics)
}

struct Resolver {
    table: SymbolTable,
    /// Local scopes (function arguments, `let` bindings), innermost last.
    scopes: Vec<HashMap<String, LocalId>>,
    diagnostics: Diagnostics,
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
            let first_span = self.table.def(existing).span.clone();
            self.diagnostics.error(
                name.span.clone(),
                format!(
                    "duplicate definition of `{}` (first defined at {}..{})",
                    name.name, first_span.start, first_span.end
                ),
            );
            return None;
        }
        let id = DefId::new(self.table.defs.len());
        self.table.defs.push(DefEntry {
            kind,
            name: name.name.clone(),
            span: name.span.clone(),
        });
        self.table.names.insert(name.name.clone(), id);
        Some(id)
    }

    fn declare_state(&mut self, name: &Identifier, component: DefId, states: &mut HashMap<String, StateId>) -> Option<StateId> {
        if let Some(&existing) = states.get(&name.name) {
            let first_span = self.table.state(existing).span.clone();
            self.diagnostics.error(
                name.span.clone(),
                format!(
                    "duplicate controller state `{}` (first defined at {}..{})",
                    name.name, first_span.start, first_span.end
                ),
            );
            return None;
        }
        let id = StateId::new(self.table.states.len());
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
                let first_span = self.table.local(existing).span.clone();
                self.diagnostics.error(
                    name.span.clone(),
                    format!(
                        "duplicate binding `{}` (first defined at {}..{})",
                        name.name, first_span.start, first_span.end
                    ),
                );
                ids.push(None);
                continue;
            }
            let id = LocalId::new(self.table.locals.len());
            self.table.locals.push(LocalEntry {
                name: name.name.clone(),
                span: name.span.clone(),
            });
            frame.insert(name.name.clone(), id);
            ids.push(Some(id));
        }
        self.scopes.push(frame);
        ids
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    // -- Looking up names -------------------------------------------------

    fn lookup_local(&self, name: &str) -> Option<LocalId> {
        self.scopes.iter().rev().find_map(|frame| frame.get(name).copied())
    }

    fn unknown_symbol(&mut self, name: &Identifier) {
        self.diagnostics.error(name.span.clone(), format!("unknown symbol `{}`", name.name));
    }

    /// Resolves a [DefRef] against the top-level namespace, requiring the
    /// resolved declaration's kind to satisfy `expected`.
    fn resolve_def_ref(&mut self, reference: &mut DefRef, expected: impl Fn(&DefKind) -> bool, expected_desc: &str) {
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
                format!("`{}` is {}, expected {}", reference.name.name, kind.describe(), expected_desc),
            );
        }
    }

    fn resolve_state_ref(&mut self, reference: &mut StateRef, states: &HashMap<String, StateId>) {
        match states.get(&reference.name.name) {
            Some(&id) => reference.id = Some(id),
            None => {
                self.diagnostics
                    .error(reference.name.span.clone(), format!("unknown controller state `{}`", reference.name.name));
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
            self.diagnostics.error(span.clone(), format!("unknown symbol `{name}`"));
            return None;
        };
        if self.table.def(id).kind.is_referenceable_value() {
            Some(Binding::Def(id))
        } else {
            let kind = self.table.def(id).kind.clone();
            self.diagnostics.error(
                span.clone(),
                format!(
                    "`{name}` is {}, expected a constant, parameter, variable or type element",
                    kind.describe()
                ),
            );
            None
        }
    }

    // -- Top-level walk -----------------------------------------------------

    fn resolve_specification(&mut self, spec: &mut UntypedStarkSpecification) {
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
    }

    fn resolve_variable(&mut self, variable: &mut Variable) {
        if let Some(range) = &mut variable.range {
            self.resolve_expression(&mut range.min);
            self.resolve_expression(&mut range.max);
        }
        self.resolve_expression(&mut variable.initial_value);
        variable.id = self.declare(&variable.name, DefKind::Variable { global: variable.global });
    }

    fn resolve_type_declaration(&mut self, ty: &mut TypeDeclaration) {
        // A type name colliding with one of its own elements isn't caught by
        // the general duplicate check below (neither is registered yet at
        // the point we'd check), so it needs its own check.
        if ty.elements.iter().any(|e| e.name == ty.name.name) {
            self.diagnostics.error(
                ty.name.span.clone(),
                format!("type `{}` cannot declare an element with the same name", ty.name.name),
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
            DistanceExpression::Eventually { from, to, argument } | DistanceExpression::Globally { from, to, argument } => {
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
            RobtlFormula::Reference(reference) => self.resolve_def_ref(reference, DefKind::is_formula_kind, "a formula"),
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

    fn resolve_expression(&mut self, expr: &mut SpannedExpression) {
        match &mut expr.node {
            Expression::False
            | Expression::True
            | Expression::Integer(_)
            | Expression::Real(_)
            | Expression::Iterator => {}
            Expression::Reference { name, binding } => {
                *binding = self.resolve_reference(name, &expr.span);
            }
            Expression::Normal { mean, std_dev } => {
                self.resolve_expression(mean);
                self.resolve_expression(std_dev);
            }
            Expression::Uniform { values } => {
                for value in values {
                    self.resolve_expression(value);
                }
            }
            Expression::Range { min, max } => {
                if let Some(min) = min {
                    self.resolve_expression(min);
                }
                if let Some(max) = max {
                    self.resolve_expression(max);
                }
            }
            Expression::Not(inner) | Expression::UnaryPlus(inner) | Expression::UnaryMinus(inner) => {
                self.resolve_expression(inner);
            }
            Expression::Binary(_, left, right) => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expression::Ternary {
                guard,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(guard);
                self.resolve_expression(then_branch);
                self.resolve_expression(else_branch);
            }
            Expression::Call { function, arguments } => {
                for argument in arguments.iter_mut() {
                    self.resolve_expression(argument);
                }
                self.resolve_def_ref(function, DefKind::is_function_kind, "a function");
            }
            Expression::MathCall { arguments, .. } => {
                for argument in arguments {
                    self.resolve_expression(argument);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;
    use crate::ast::{Binding, Expression, UntypedStarkSpecification};

    fn resolve_source(src: &str) -> (UntypedStarkSpecification, super::SymbolTable, crate::diagnostics::Diagnostics) {
        let mut spec = UntypedStarkSpecification::parse(src).expect("should parse");
        let (table, diagnostics) = resolve(&mut spec);
        (spec, table, diagnostics)
    }

    #[test]
    fn resolves_a_reference_to_an_earlier_constant() {
        let (spec, _table, diagnostics) = resolve_source("const a = 1;\nconst b = a + 1;");
        assert!(!diagnostics.has_errors(), "{diagnostics}");
        match &spec.constants[1].value.node {
            Expression::Binary(_, lhs, _) => {
                assert!(matches!(lhs.node, Expression::Reference { binding: Some(Binding::Def(_)), .. }));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn forward_reference_is_unknown_symbol() {
        let (_spec, _table, diagnostics) = resolve_source("const a = b + 1;\nconst b = 1;");
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn duplicate_top_level_name_is_an_error() {
        let (_spec, _table, diagnostics) = resolve_source("const a = 1;\nconst a = 2;");
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn calling_a_variable_is_illegal_use_of_name() {
        let (_spec, _table, diagnostics) =
            resolve_source("global variables { int x = 0; }\nconst c = x(1);");
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn referencing_a_function_as_a_value_is_illegal_use_of_name() {
        let (_spec, _table, diagnostics) =
            resolve_source("function f(int x) { return x; }\nconst c = f;");
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn function_cannot_call_itself() {
        let (_spec, _table, diagnostics) = resolve_source("function f(int x) { return f(x); }");
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn let_binding_shadows_outer_constant() {
        let (spec, _table, diagnostics) = resolve_source(
            "const x = 1;\nfunction f(int y) { let x = 2 in return x + y; }",
        );
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
            Expression::Binary(_, lhs, rhs) => {
                assert!(matches!(lhs.node, Expression::Reference { binding: Some(Binding::Local(_)), .. }));
                assert!(matches!(rhs.node, Expression::Reference { binding: Some(Binding::Local(_)), .. }));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn duplicate_function_argument_is_an_error() {
        let (_spec, _table, diagnostics) = resolve_source("function f(int x, int x) { return x; }");
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn controller_state_can_forward_reference_a_sibling_state() {
        let (_spec, _table, diagnostics) = resolve_source(
            "component C {\n  variables { }\n  controller {\n    aiState A { step B; }\n    aiState B { step A; }\n  }\n  init A\n}",
        );
        assert!(!diagnostics.has_errors(), "{diagnostics}");
    }

    #[test]
    fn controller_state_cannot_target_another_components_state() {
        let (_spec, _table, diagnostics) = resolve_source(
            "component C1 {\n  variables { }\n  controller {\n    aiState A { step B; }\n  }\n  init A\n}\ncomponent C2 {\n  variables { }\n  controller {\n    aiState B { exec B; }\n  }\n  init B\n}",
        );
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn custom_type_element_is_referenceable() {
        // `penalty` is resolved after `type` declarations in this resolver's
        // fixed kind order (see the module doc comment), so referencing a
        // type element from a penalty value exercises the forward-visibility
        // that types grant to everything processed after them.
        let (spec, _table, diagnostics) =
            resolve_source("type Color = Red | Green | Blue;\npenalty p = Red");
        assert!(!diagnostics.has_errors(), "{diagnostics}");
        assert!(matches!(
            spec.penalties[0].value.node,
            Expression::Reference { binding: Some(Binding::Def(_)), .. }
        ));
    }

    #[test]
    fn type_element_cannot_share_the_types_own_name() {
        let (_spec, _table, diagnostics) = resolve_source("type Color = Color | Blue;\nconst c = 1;");
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn assignment_target_must_be_a_variable() {
        let (_spec, _table, diagnostics) = resolve_source(
            "const k = 1;\nenvironment { k' = 1; }",
        );
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn resolves_every_example_specification_without_errors() {
        for (name, source) in [
            ("engine", include_str!("../../../examples/stark/engine.stark")),
            ("random_walk", include_str!("../../../examples/stark/random_walk.stark")),
            ("single_vehicle", include_str!("../../../examples/stark/single_vehicle.stark")),
            ("toll", include_str!("../../../examples/stark/toll.stark")),
            ("two_vehicles", include_str!("../../../examples/stark/two_vehicles.stark")),
        ] {
            let mut spec = UntypedStarkSpecification::parse(source).unwrap_or_else(|e| panic!("{name} failed to parse: {e}"));
            let (_table, diagnostics) = resolve(&mut spec);
            assert!(!diagnostics.has_errors(), "{name} failed to resolve:\n{}", diagnostics.render(source));
        }
    }
}
