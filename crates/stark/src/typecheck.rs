//! Type checking: expression type inference, plus the function-body
//! inference that gives an unannotated function its return type.
//!
//! Runs after `resolve.rs`: every reference/call already carries a resolved
//! [DefId]/[LocalId], so this pass never re-derives "is this name defined" /
//! "is this the right kind of name" — `resolve.rs` already decided that.
//! Because `resolve.rs` assigns [LocalId]s uniquely across the whole spec
//! (never reused between scopes), a flat `Vec<Option<StarkType>>` indexed by
//! `LocalId` stands in for the original's stack of nested type-evaluation
//! scopes — no scope stack is needed here, only "has this local's type been
//! computed yet".
//!
//! A `None` binding/id (left by `resolve.rs` for something that failed to
//! resolve) is treated as already-erred: this pass returns
//! [StarkType::Error] for it without recording a second diagnostic for the
//! same spot.
//!
//! Two spots deliberately diverge from the original: `&&`/`||` never
//! propagate a `random[..]` result there (the disjunction case even computes
//! whether either operand is random and then never uses it — reading as an
//! unfinished path, not a deliberate choice, since the relational case right
//! next to it does propagate), and neither does the *unary* math-call path,
//! while the binary one does. This port propagates randomness in both cases,
//! for consistency with every other
//! boolean/real-producing operator. No case in the ported
//! `ExpressionTypeInferenceTest` exercises either edge case, so this
//! doesn't contradict anything being ported.

use log::debug;
use log::trace;

use crate::ast::*;
use crate::diagnostics::DiagnosticKind;
use crate::diagnostics::Diagnostics;
use crate::resolve::DefKind;
use crate::resolve::SymbolTable;
use crate::types::StarkType;

/// A function's argument types (positional, matching its declared
/// arguments) and its return type, inferred from its `return` statements
/// (STARK functions have no return-type annotation).
#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub arguments: Vec<StarkType>,
    pub return_type: StarkType,
}

/// The result of type checking: the type of every [DefId] that has one
/// (constants, parameters, variables, type elements — `None` for kinds that
/// don't carry a single expression type, like components or functions), and
/// the signature of every function.
#[derive(Clone, Debug)]
pub struct TypeTable {
    def_types: Vec<Option<StarkType>>,
    signatures: Vec<Option<FunctionSignature>>,
}

impl TypeTable {
    pub fn type_of(&self, id: DefId) -> Option<&StarkType> {
        debug_assert!(
            id.value() < self.def_types.len(),
            "{id:?} is out of range for a table of {} definition(s) — mismatched SymbolTable?",
            self.def_types.len()
        );
        self.def_types[id.value()].as_ref()
    }

    pub fn signature_of(&self, id: DefId) -> Option<&FunctionSignature> {
        debug_assert!(
            id.value() < self.signatures.len(),
            "{id:?} is out of range for a table of {} definition(s) — mismatched SymbolTable?",
            self.signatures.len()
        );
        self.signatures[id.value()].as_ref()
    }
}

/// Type-checks `spec` against the `symbols` produced by [crate::resolve::resolve],
/// returning the inferred type of every declaration together with every
/// diagnostic found.
pub fn typecheck(spec: &UntypedStarkSpecification, symbols: &SymbolTable) -> (TypeTable, Diagnostics) {
    let mut checker = Checker {
        symbols,
        def_types: vec![None; symbols.defs.len()],
        signatures: vec![None; symbols.defs.len()],
        locals: vec![None; symbols.locals.len()],
        diagnostics: Diagnostics::new(),
    };
    checker.check_specification(spec);

    debug_assert_eq!(
        checker.def_types.len(),
        symbols.defs.len(),
        "the type table must stay indexable by every DefId the symbol table knows"
    );
    debug_assert_eq!(
        checker.locals.len(),
        symbols.locals.len(),
        "the local type table must stay indexable by every LocalId the symbol table knows"
    );

    let typed = checker.def_types.iter().filter(|t| t.is_some()).count();
    let signatures = checker.signatures.iter().filter(|s| s.is_some()).count();
    debug!(
        "type-checked {typed}/{} definition(s) and {signatures} function signature(s); {} diagnostic(s)",
        symbols.defs.len(),
        checker.diagnostics.items().len()
    );

    (
        TypeTable {
            def_types: checker.def_types,
            signatures: checker.signatures,
        },
        checker.diagnostics,
    )
}

struct Checker<'a> {
    symbols: &'a SymbolTable,
    def_types: Vec<Option<StarkType>>,
    signatures: Vec<Option<FunctionSignature>>,
    locals: Vec<Option<StarkType>>,
    diagnostics: Diagnostics,
}

impl Checker<'_> {
    // -- Small helpers ---------------------------------------------------

    fn set_def_type(&mut self, id: DefId, ty: StarkType) {
        // Each declaration is visited exactly once, so its type is written
        // exactly once. A second write means the same `DefId` was handed to
        // two declarations, which would silently corrupt every later lookup.
        debug_assert!(
            self.def_types[id.value()].is_none(),
            "{id:?} (`{}`) already has type {:?}, cannot also be {ty:?}",
            self.symbols.def(id).name,
            self.def_types[id.value()]
        );
        trace!("{id:?} (`{}`) : {ty}", self.symbols.def(id).name);
        self.def_types[id.value()] = Some(ty);
    }

    fn def_type(&self, id: DefId) -> StarkType {
        self.def_types[id.value()].clone().unwrap_or(StarkType::Error)
    }

    fn set_local_type(&mut self, id: LocalId, ty: StarkType) {
        // `resolve.rs` assigns every binding site its own `LocalId`, never
        // reusing one across scopes — that is exactly what lets this pass get
        // away with a flat vector instead of a scope stack, so it is worth
        // checking rather than assuming.
        debug_assert!(
            self.locals[id.value()].is_none(),
            "{id:?} (`{}`) already has type {:?}, cannot also be {ty:?}",
            self.symbols.local(id).name,
            self.locals[id.value()]
        );
        trace!("{id:?} (`{}`) : {ty}", self.symbols.local(id).name);
        self.locals[id.value()] = Some(ty);
    }

    fn local_type(&self, id: LocalId) -> StarkType {
        self.locals[id.value()].clone().unwrap_or(StarkType::Error)
    }

    fn ty_of_annotation(&mut self, ty: &Ty, span: &Span) -> StarkType {
        match ty {
            Ty::Integer => StarkType::Integer,
            Ty::Real => StarkType::Real,
            Ty::Boolean => StarkType::Boolean,
            Ty::Named(name) => match self.symbols.by_name(name) {
                Some(id) if matches!(self.symbols.def(id).kind, DefKind::Type) => StarkType::Custom(name.clone()),
                _ => {
                    self.diagnostics
                        .error(span.clone(), DiagnosticKind::UnknownType { name: name.clone() });
                    StarkType::Error
                }
            },
        }
    }

    /// `expected.is_compatible_with(actual)`, recording a diagnostic and
    /// returning `Error` on mismatch — the `inferAndCheck`/`checkType`
    /// pattern: every failure collapses to `Error` so it can't cascade into
    /// more than one diagnostic at the point of use.
    fn expect(&mut self, expected: &StarkType, actual: StarkType, span: &Span) -> StarkType {
        if expected.is_compatible_with(&actual) {
            actual
        } else {
            self.diagnostics.error(
                span.clone(),
                DiagnosticKind::TypeMismatch {
                    expected: expected.clone(),
                    found: actual,
                },
            );
            StarkType::Error
        }
    }

    fn expect_numerical(&mut self, actual: StarkType, span: &Span) -> StarkType {
        if actual.is_numerical() {
            actual
        } else {
            self.diagnostics
                .error(span.clone(), DiagnosticKind::NotNumerical { found: actual });
            StarkType::Error
        }
    }

    fn expect_mergeable(&mut self, left: &StarkType, right: &StarkType, span: &Span) {
        if !left.can_be_merged_with(right) {
            self.diagnostics.error(
                span.clone(),
                DiagnosticKind::IncompatibleTypes {
                    left: left.clone(),
                    right: right.clone(),
                },
            );
        }
    }

    // -- Top-level walk ----------------------------------------------------

    fn check_specification(&mut self, spec: &UntypedStarkSpecification) {
        for constant in &spec.constants {
            let ty = self.check_expression(&constant.value, false);
            if let Some(id) = constant.id {
                self.set_def_type(id, ty);
            }
        }
        for parameter in &spec.parameters {
            let ty = self.check_expression(&parameter.value, false);
            if let Some(id) = parameter.id {
                self.set_def_type(id, ty);
            }
        }
        // Custom type elements carry their owning type as their `StarkType`;
        // this can be computed immediately, no expression involved.
        for ty in &spec.types {
            if ty.id.is_none() {
                continue;
            }
            for element in &ty.elements {
                if let Some(id) = self.symbols.by_name(&element.name) {
                    self.set_def_type(id, StarkType::Custom(ty.name.name.clone()));
                }
            }
        }
        // `resolve.rs` makes state variables visible everywhere, so their
        // types have to be known before any body that might read one. The
        // declared type comes straight from the annotation, so this needs no
        // expression checking and can run ahead of everything else.
        for variable in &spec.variables {
            self.set_variable_type(variable);
        }
        for component in &spec.components {
            for variable in &component.variables {
                self.set_variable_type(variable);
            }
        }
        for function in &spec.functions {
            self.check_function(function);
        }
        for variable in &spec.variables {
            self.check_variable(variable);
        }
        for component in &spec.components {
            for variable in &component.variables {
                self.check_variable(variable);
            }
            for state in &component.states {
                self.check_controller_commands(&state.body);
            }
        }
        if let Some(environment) = &spec.environment {
            for command in &environment.commands {
                self.check_environment_command(command);
            }
        }
        for penalty in &spec.penalties {
            let ty = self.check_expression(&penalty.value, false);
            self.expect_numerical(ty, &penalty.value.span);
        }
        for perturbation in &spec.perturbations {
            self.check_perturbation(&perturbation.value);
        }
        for distance in &spec.distances {
            self.check_distance(&distance.value);
        }
        for formula in &spec.formulas {
            self.check_robtl(&formula.value);
        }
    }

    /// Records a variable's declared type from its annotation alone. Split
    /// out from [Self::check_variable] so every variable's type is known
    /// before any body that reads one is checked.
    fn set_variable_type(&mut self, variable: &Variable) {
        let declared = self.ty_of_annotation(&variable.ty, &variable.name.span);
        if let Some(id) = variable.id {
            self.set_def_type(id, declared);
        }
    }

    /// Checks a variable's range and initializer against its declared type.
    /// That type is read back from [Self::set_variable_type] rather than
    /// re-derived from the annotation, so an unknown type name is reported
    /// once rather than once per pass.
    fn check_variable(&mut self, variable: &Variable) {
        let declared = variable.id.map_or(StarkType::Error, |id| self.def_type(id));
        if let Some(range) = &variable.range {
            let min = self.check_expression(&range.min, false);
            self.expect_numerical(min, &range.min.span);
            let max = self.check_expression(&range.max, false);
            self.expect_numerical(max, &range.max.span);
        }
        let initial = self.check_expression(&variable.initial_value, false);
        self.expect(&declared, initial, &variable.initial_value.span);
    }

    fn check_function(&mut self, function: &Function) {
        trace!("checking function `{}`", function.name.name);
        let mut arguments = Vec::with_capacity(function.arguments.len());
        for argument in &function.arguments {
            let ty = self.ty_of_annotation(&argument.ty, &argument.name.span);
            if let Some(id) = argument.id {
                self.set_local_type(id, ty.clone());
            }
            arguments.push(ty);
        }
        // Function bodies may use random expressions (e.g. `single_vehicle.stark`'s
        // `new_s_speed` and `engine.stark`'s `temperatureUpdateInOneStep` both
        // `return R[...]`).
        let return_type = self.check_function_statement(&function.body, true);
        if let Some(id) = function.id {
            debug_assert!(
                self.signatures[id.value()].is_none(),
                "function `{}` ({id:?}) already has a signature",
                function.name.name
            );
            trace!("`{}` : ({arguments:?}) -> {return_type}", function.name.name);
            self.signatures[id.value()] = Some(FunctionSignature { arguments, return_type });
        }
    }

    /// Returns the type of every `return` reachable from `statement`, merged
    /// together: a function has no return-type annotation, so its type is
    /// inferred from its body.
    fn check_function_statement(&mut self, statement: &FunctionStatement, random_allowed: bool) -> StarkType {
        match statement {
            FunctionStatement::Return(value) => self.check_expression(value, random_allowed),
            FunctionStatement::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                let guard_ty = self.check_expression(guard, random_allowed);
                self.expect(&StarkType::Boolean, guard_ty, &guard.span);
                let then_ty = self.check_function_statement(then_branch, random_allowed);
                match else_branch {
                    Some(else_branch) => {
                        let else_ty = self.check_function_statement(else_branch, random_allowed);
                        self.expect_mergeable(&then_ty, &else_ty, &guard.span);
                        then_ty.merge(&else_ty)
                    }
                    None => then_ty,
                }
            }
            FunctionStatement::Let { id, value, body, .. } => {
                let value_ty = self.check_expression(value, random_allowed);
                if let Some(id) = id {
                    self.set_local_type(*id, value_ty);
                }
                self.check_function_statement(body, random_allowed)
            }
            FunctionStatement::Block(inner) => self.check_function_statement(inner, random_allowed),
        }
    }

    fn check_controller_commands(&mut self, commands: &[ControllerCommand]) {
        for command in commands {
            match command {
                // The branch a controller takes is deterministic given the
                // state (the guard and the step count aren't random), but an
                // assignment's *value* may be — `multiscler.stark`'s
                // controller injects a randomly-sized therapeutic dose
                // (`Rr' = Rr + 1000 + R[-10,10]`), which is exactly the
                // counterexample that overturned this pass's original
                // "controllers are fully deterministic" assumption.
                ControllerCommand::Step { steps, .. } => {
                    if let Some(steps) = steps {
                        let ty = self.check_expression(steps, false);
                        self.expect_numerical(ty, &steps.span);
                    }
                }
                ControllerCommand::Exec(_) => {}
                ControllerCommand::Let { id, value, body, .. } => {
                    let value_ty = self.check_expression(value, true);
                    if let Some(id) = id {
                        self.set_local_type(*id, value_ty);
                    }
                    self.check_controller_commands(body);
                }
                ControllerCommand::Assignment(update) => self.check_update(update, true),
                ControllerCommand::IfThenElse {
                    guard,
                    then_branch,
                    else_branch,
                } => {
                    let guard_ty = self.check_expression(guard, false);
                    self.expect(&StarkType::Boolean, guard_ty, &guard.span);
                    self.check_controller_commands(then_branch);
                    if let Some(else_branch) = else_branch {
                        self.check_controller_commands(else_branch);
                    }
                }
                ControllerCommand::Block(inner) => self.check_controller_commands(inner),
            }
        }
    }

    fn check_environment_command(&mut self, command: &EnvironmentCommand) {
        match command {
            EnvironmentCommand::Assignment(update) => self.check_update(update, true),
            EnvironmentCommand::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                let guard_ty = self.check_expression(guard, true);
                self.expect(&StarkType::Boolean, guard_ty, &guard.span);
                self.check_environment_command(then_branch);
                if let Some(else_branch) = else_branch {
                    self.check_environment_command(else_branch);
                }
            }
            EnvironmentCommand::Let { bindings, body } => {
                for binding in bindings {
                    let ty = self.check_expression(&binding.value, true);
                    if let Some(id) = binding.id {
                        self.set_local_type(id, ty);
                    }
                }
                self.check_environment_command(body);
            }
            EnvironmentCommand::Block(inner) => {
                for command in inner {
                    self.check_environment_command(command);
                }
            }
        }
    }

    fn check_update(&mut self, update: &Update, random_allowed: bool) {
        if let Some(guard) = &update.guard {
            let guard_ty = self.check_expression(guard, random_allowed);
            self.expect(&StarkType::Boolean, guard_ty, &guard.span);
        }
        let value_ty = self.check_expression(&update.value, random_allowed);
        let target_ty = match update.target.id {
            Some(id) => self.def_type(id),
            None => return, // already diagnosed by resolve.rs
        };
        self.expect(&target_ty, value_ty, &update.value.span);
    }

    // -- Sub-languages: no randomness anywhere (interval bounds, thresholds,
    // iteration/time controls are all experiment parameters, not model state).

    fn check_perturbation(&mut self, perturbation: &PerturbationExpression) {
        match perturbation {
            PerturbationExpression::Nil | PerturbationExpression::Reference(_) => {}
            PerturbationExpression::Atomic { assignments, time } => {
                for assignment in assignments {
                    // Perturbation assignment values *are* evidenced to use
                    // randomness in the examples (e.g. `offset_speed <- p_speed * ... * R[0,1]`).
                    let value_ty = self.check_expression(&assignment.value, true);
                    let target_ty = match assignment.target.id {
                        Some(id) => self.def_type(id),
                        None => continue,
                    };
                    self.expect(&target_ty, value_ty, &assignment.value.span);
                }
                let time_ty = self.check_expression(time, false);
                self.expect_numerical(time_ty, &time.span);
            }
            PerturbationExpression::Sequence(left, right) => {
                self.check_perturbation(left);
                self.check_perturbation(right);
            }
            PerturbationExpression::Iteration { argument, iterations } => {
                self.check_perturbation(argument);
                let ty = self.check_expression(iterations, false);
                self.expect_numerical(ty, &iterations.span);
            }
        }
    }

    fn check_distance(&mut self, distance: &DistanceExpression) {
        match distance {
            DistanceExpression::Reference(_)
            | DistanceExpression::AtomicLeft(_)
            | DistanceExpression::AtomicRight(_) => {}
            DistanceExpression::Eventually { from, to, argument }
            | DistanceExpression::Globally { from, to, argument } => {
                self.check_interval(from, to);
                self.check_distance(argument);
            }
            DistanceExpression::Until { from, to, left, right } => {
                self.check_interval(from, to);
                self.check_distance(left);
                self.check_distance(right);
            }
            DistanceExpression::Threshold { left, threshold, .. } => {
                self.check_distance(left);
                let ty = self.check_expression(threshold, false);
                self.expect_numerical(ty, &threshold.span);
            }
            DistanceExpression::Min(left, right) | DistanceExpression::Max(left, right) => {
                self.check_distance(left);
                self.check_distance(right);
            }
            DistanceExpression::LinearCombination(terms) => {
                for (weight, distance) in terms {
                    let ty = self.check_expression(weight, false);
                    self.expect_numerical(ty, &weight.span);
                    self.check_distance(distance);
                }
            }
        }
    }

    fn check_robtl(&mut self, formula: &RobtlFormula) {
        match formula {
            RobtlFormula::True | RobtlFormula::False | RobtlFormula::Reference(_) => {}
            RobtlFormula::Distance { value, .. } => {
                let ty = self.check_expression(value, false);
                self.expect_numerical(ty, &value.span);
            }
            RobtlFormula::Not(inner) => self.check_robtl(inner),
            RobtlFormula::Globally { from, to, argument } | RobtlFormula::Eventually { from, to, argument } => {
                self.check_interval(from, to);
                self.check_robtl(argument);
            }
            RobtlFormula::And(left, right) | RobtlFormula::Or(left, right) => {
                self.check_robtl(left);
                self.check_robtl(right);
            }
            RobtlFormula::Until { from, to, left, right } => {
                self.check_interval(from, to);
                self.check_robtl(left);
                self.check_robtl(right);
            }
        }
    }

    fn check_interval(&mut self, from: &Expression, to: &Expression) {
        let from_ty = self.check_expression(from, false);
        self.expect_numerical(from_ty, &from.span);
        let to_ty = self.check_expression(to, false);
        self.expect_numerical(to_ty, &to.span);
    }

    // -- Expressions ------------------------------------------------------

    /// Always widens to `real` (`2 ^ 3` and `atan2(1,2)` are both `real`,
    /// never `int`), propagating randomness from either operand.
    fn combine_to_real(&mut self, left: &Expression, right: &Expression, random_allowed: bool) -> StarkType {
        let left_ty = self.check_expression(left, random_allowed);
        let left_ty = self.expect_numerical(left_ty, &left.span);
        let right_ty = self.check_expression(right, random_allowed);
        let right_ty = self.expect_numerical(right_ty, &right.span);
        if left_ty.is_random() || right_ty.is_random() {
            StarkType::random(StarkType::Real)
        } else {
            StarkType::Real
        }
    }

    /// [Self::combine_to_real]'s single-operand counterpart, for unary `+`/`-`.
    fn combine_to_real_unary(&mut self, inner: &Expression, random_allowed: bool) -> StarkType {
        let ty = self.check_expression(inner, random_allowed);
        let ty = self.expect_numerical(ty, &inner.span);
        if ty.is_random() {
            StarkType::random(StarkType::Real)
        } else {
            StarkType::Real
        }
    }

    fn check_expression(&mut self, expr: &Expression, random_allowed: bool) -> StarkType {
        match &expr.node {
            ExpressionKind::False | ExpressionKind::True => StarkType::Boolean,
            ExpressionKind::Integer(_) => StarkType::Integer,
            ExpressionKind::Real(_) => StarkType::Real,
            // Only used inside aggregate/lambda contexts, none of which are
            // reachable from the current grammar (see `ast.rs`); typed as
            // `Error` rather than given a made-up type.
            ExpressionKind::Iterator => StarkType::Error,
            ExpressionKind::Reference { binding, .. } => match binding {
                Some(Binding::Def(id)) => self.def_type(*id),
                Some(Binding::Local(id)) => self.local_type(*id),
                None => StarkType::Error,
            },
            ExpressionKind::Normal { mean, std_dev } => {
                if !random_allowed {
                    self.diagnostics
                        .error(expr.span.clone(), DiagnosticKind::RandomNotAllowed);
                    return StarkType::Error;
                }
                let mean_ty = self.check_expression(mean, random_allowed);
                let mean_ty = self.expect(&StarkType::Real, mean_ty, &mean.span);
                let std_ty = self.check_expression(std_dev, random_allowed);
                let std_ty = self.expect(&StarkType::Real, std_ty, &std_dev.span);
                if mean_ty.is_error() || std_ty.is_error() {
                    StarkType::Error
                } else {
                    StarkType::random(StarkType::Real)
                }
            }
            ExpressionKind::Uniform { values } => {
                if !random_allowed {
                    self.diagnostics
                        .error(expr.span.clone(), DiagnosticKind::RandomNotAllowed);
                    return StarkType::Error;
                }
                let mut merged: Option<StarkType> = None;
                for value in values {
                    let ty = self.check_expression(value, random_allowed);
                    merged = Some(match merged {
                        None => ty,
                        Some(acc) => {
                            self.expect_mergeable(&acc, &ty, &value.span);
                            acc.merge(&ty)
                        }
                    });
                }
                match merged {
                    Some(ty) if !ty.is_error() => StarkType::random(ty),
                    _ => StarkType::Error,
                }
            }
            ExpressionKind::Range { min, max } => {
                if !random_allowed {
                    self.diagnostics
                        .error(expr.span.clone(), DiagnosticKind::RandomNotAllowed);
                    return StarkType::Error;
                }
                match (min, max) {
                    (Some(min), Some(max)) => {
                        let min_ty = self.check_expression(min, random_allowed);
                        let min_ty = self.expect(&StarkType::Real, min_ty, &min.span);
                        let max_ty = self.check_expression(max, random_allowed);
                        let max_ty = self.expect(&StarkType::Real, max_ty, &max.span);
                        if min_ty.is_error() || max_ty.is_error() {
                            StarkType::Error
                        } else {
                            StarkType::random(StarkType::Real)
                        }
                    }
                    _ => StarkType::random(StarkType::Real),
                }
            }
            ExpressionKind::Not(inner) => {
                let ty = self.check_expression(inner, random_allowed);
                self.expect(&StarkType::Boolean, ty, &inner.span)
            }
            ExpressionKind::UnaryPlus(inner) | ExpressionKind::UnaryMinus(inner) => {
                // Matches the original, which routes `+`/`-` through the
                // *same* always-widening double-valued mechanism as
                // `abs`/`sqrt`/etc., so unary +/- on an `int` widens the
                // result to `real`, and so does everything built on top of
                // it (`-a + 2` is `real`, not `int`, when `a` is an `int`).
                // Surprising for a spec author writing `-a` expecting an int
                // to stay one; matched here for fidelity with the original
                // tool, but worth reconsidering if it surprises users badly
                // enough in practice.
                self.combine_to_real_unary(inner, random_allowed)
            }
            ExpressionKind::Binary(op, left, right) => self.check_binary(*op, left, right, random_allowed),
            ExpressionKind::Ternary {
                guard,
                then_branch,
                else_branch,
            } => {
                let guard_ty = self.check_expression(guard, random_allowed);
                let guard_ty = self.expect(&StarkType::Boolean, guard_ty, &guard.span);
                let then_ty = self.check_expression(then_branch, random_allowed);
                let else_ty = self.check_expression(else_branch, random_allowed);
                self.expect_mergeable(&then_ty, &else_ty, &else_branch.span);
                let merged = then_ty.merge(&else_ty);
                if !merged.is_error() && guard_ty.is_random() {
                    StarkType::random(merged)
                } else {
                    merged
                }
            }
            ExpressionKind::Call { function, arguments } => self.check_call(function, arguments, random_allowed),
            ExpressionKind::MathCall { function, arguments } => {
                self.check_math_call(*function, arguments, random_allowed)
            }
        }
    }

    fn check_binary(&mut self, op: BinaryOp, left: &Expression, right: &Expression, random_allowed: bool) -> StarkType {
        match op {
            BinaryOp::Pow => self.combine_to_real(left, right, random_allowed),
            BinaryOp::Mult | BinaryOp::Div | BinaryOp::IntDiv | BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Mod => {
                let left_ty = self.check_expression(left, random_allowed);
                let left_ty = self.expect_numerical(left_ty, &left.span);
                let right_ty = self.check_expression(right, random_allowed);
                let right_ty = self.expect_numerical(right_ty, &right.span);
                left_ty.merge(&right_ty)
            }
            BinaryOp::Less | BinaryOp::Leq | BinaryOp::Eq | BinaryOp::Geq | BinaryOp::Greater => {
                let left_ty = self.check_expression(left, random_allowed);
                let right_ty = self.check_expression(right, random_allowed);
                self.expect_mergeable(&left_ty, &right_ty, &right.span);
                if left_ty.is_random() || right_ty.is_random() {
                    StarkType::random(StarkType::Boolean)
                } else {
                    StarkType::Boolean
                }
            }
            BinaryOp::BitAnd | BinaryOp::And | BinaryOp::BitOr | BinaryOp::Or => {
                let left_ty = self.check_expression(left, random_allowed);
                let left_ty = self.expect(&StarkType::Boolean, left_ty, &left.span);
                let right_ty = self.check_expression(right, random_allowed);
                let right_ty = self.expect(&StarkType::Boolean, right_ty, &right.span);
                if left_ty.is_random() || right_ty.is_random() {
                    StarkType::random(StarkType::Boolean)
                } else {
                    StarkType::Boolean
                }
            }
        }
    }

    fn check_call(&mut self, function: &DefRef, arguments: &[Expression], random_allowed: bool) -> StarkType {
        let Some(id) = function.id else {
            // Already diagnosed by resolve.rs; still check the arguments so
            // unrelated mistakes in them are still reported.
            for argument in arguments {
                self.check_expression(argument, random_allowed);
            }
            return StarkType::Error;
        };
        let Some(signature) = self.signatures[id.value()].clone() else {
            // The callee's own signature failed to type-check.
            for argument in arguments {
                self.check_expression(argument, random_allowed);
            }
            return StarkType::Error;
        };
        if signature.arguments.len() != arguments.len() {
            self.diagnostics.error(
                function.name.span.clone(),
                DiagnosticKind::ArityMismatch {
                    name: function.name.name.clone(),
                    expected: signature.arguments.len(),
                    found: arguments.len(),
                },
            );
            for argument in arguments {
                self.check_expression(argument, random_allowed);
            }
            return StarkType::Error;
        }
        for (expected, argument) in signature.arguments.iter().zip(arguments) {
            let actual = self.check_expression(argument, random_allowed);
            self.expect(expected, actual, &argument.span);
        }
        signature.return_type
    }

    fn check_math_call(&mut self, function: MathFunction, arguments: &[Expression], random_allowed: bool) -> StarkType {
        match function {
            MathFunction::Atan2 | MathFunction::Hypot | MathFunction::Max | MathFunction::Min | MathFunction::Pow => {
                // Unlike user-defined calls, a math call's arity is fixed by
                // the grammar (`BinaryMathFunction ~ "(" ~ Expression ~ ","
                // ~ Expression ~ ")"`), so a mismatch here is a parser bug
                // rather than a user error — and the indexing below would
                // otherwise panic without saying why.
                debug_assert_eq!(
                    arguments.len(),
                    2,
                    "binary math function {function:?} parsed with {} argument(s)",
                    arguments.len()
                );
                self.combine_to_real(&arguments[0], &arguments[1], random_allowed)
            }
            _ => {
                debug_assert_eq!(
                    arguments.len(),
                    1,
                    "unary math function {function:?} parsed with {} argument(s)",
                    arguments.len()
                );
                let ty = self.check_expression(&arguments[0], random_allowed);
                let ty = self.expect_numerical(ty, &arguments[0].span);
                if ty.is_random() {
                    StarkType::random(StarkType::Real)
                } else {
                    StarkType::Real
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::typecheck;
    use crate::ast::UntypedStarkSpecification;
    use crate::diagnostics::DiagnosticKind;
    use crate::resolve::resolve;
    use crate::types::StarkType;
    use test_log::test;

    /// Resolves `src` (asserting resolution itself succeeds, since these
    /// tests are only interested in typecheck-time diagnostics) and returns
    /// the diagnostics from `typecheck`.
    fn typecheck_source(src: &str) -> crate::diagnostics::Diagnostics {
        let mut spec = UntypedStarkSpecification::parse(src).unwrap_or_else(|e| panic!("failed to parse: {e}"));
        let (symbols, resolve_diagnostics) = resolve(&mut spec);
        assert!(
            !resolve_diagnostics.has_errors(),
            "failed to resolve:\n{}",
            resolve_diagnostics.render(src)
        );
        typecheck(&spec, &symbols).1
    }

    #[test]
    fn call_with_wrong_argument_count_is_an_error() {
        // Constants are resolved before functions (kind-processing order —
        // see `resolve.rs`), so the call has to live somewhere later in that
        // order, e.g. a variable's initial value.
        let diagnostics = typecheck_source("function f(int x) { return x; }\nvariables { int c = f(1, 2); }");
        assert!(
            diagnostics.any(|kind| matches!(
                kind,
                DiagnosticKind::ArityMismatch {
                    expected: 1,
                    found: 2,
                    ..
                }
            )),
            "{diagnostics}"
        );
    }

    #[test]
    fn non_boolean_if_guard_is_an_error() {
        let diagnostics = typecheck_source("function f() { if (1) return 1; else return 2; }");
        assert!(
            diagnostics.any(|kind| matches!(
                kind,
                DiagnosticKind::TypeMismatch {
                    expected: StarkType::Boolean,
                    found: StarkType::Integer,
                }
            )),
            "{diagnostics}"
        );
    }

    #[test]
    fn incompatible_assignment_is_an_error() {
        let diagnostics = typecheck_source("variables { bool flag range[0,1] = true; }\nenvironment { flag' = 1; }");
        assert!(
            diagnostics.any(|kind| matches!(
                kind,
                DiagnosticKind::TypeMismatch {
                    expected: StarkType::Boolean,
                    found: StarkType::Integer,
                }
            )),
            "{diagnostics}"
        );
    }

    #[test]
    fn random_expression_in_a_disallowed_context_is_an_error() {
        // Variable range bounds are one of the contexts `random_allowed` is
        // deliberately `false` for (see the module doc comment / plan).
        let diagnostics = typecheck_source("variables { real x range[0, R] = 0.; }");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::RandomNotAllowed)),
            "{diagnostics}"
        );
    }

    #[test]
    fn ternary_branch_mismatch_is_an_error() {
        let diagnostics = typecheck_source("function f() { return true ? 1 : false; }");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::IncompatibleTypes { .. })),
            "{diagnostics}"
        );
    }

    #[test]
    fn function_return_type_merge_failure_is_an_error() {
        let diagnostics = typecheck_source("function f(bool b) { if (b) return 1; else return true; }");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::IncompatibleTypes { .. })),
            "{diagnostics}"
        );
    }

    #[test]
    fn unknown_type_annotation_is_an_error() {
        let diagnostics = typecheck_source("variables { Missing x = 0; }");
        assert!(
            diagnostics.any(|kind| matches!(kind, DiagnosticKind::UnknownType { name } if name == "Missing")),
            "{diagnostics}"
        );
    }

    #[test]
    fn typechecks_every_example_specification_without_errors() {
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
            let (symbols, resolve_diagnostics) = resolve(&mut spec);
            assert!(
                !resolve_diagnostics.has_errors(),
                "{name} failed to resolve:\n{}",
                resolve_diagnostics.render(source)
            );
            let (_types, diagnostics) = typecheck(&spec, &symbols);
            assert!(
                !diagnostics.has_errors(),
                "{name} failed to typecheck:\n{}",
                diagnostics.render(source)
            );
        }
    }

    /// The original tool's own expression-type-inference test cases, ported.
    ///
    /// The original tests a bare expression directly against its inference
    /// pass, with "is a random expression allowed here" as an explicit
    /// parameter. There's no equivalent "just an expression, no
    /// spec" entry point here, so each case is hosted inside the smallest
    /// construct that gives it the right `random_allowed` context: a
    /// zero-argument function body (`random_allowed = true`, matching the
    /// original's shared `typeTests` table, which is always checked with
    /// randomness allowed) or a `const` value (`random_allowed = false`,
    /// for the handful of individual tests that check the non-random path).
    mod ported_from_expression_type_inference_test {
        use super::typecheck;
        use crate::ast::UntypedStarkSpecification;
        use crate::resolve::resolve;
        use crate::types::StarkType;

        fn infer_in_function_body(expr: &str) -> StarkType {
            let source = format!("function f() {{ return {expr}; }}");
            let mut spec =
                UntypedStarkSpecification::parse(&source).unwrap_or_else(|e| panic!("failed to parse `{expr}`: {e}"));
            let (symbols, resolve_diagnostics) = resolve(&mut spec);
            assert!(
                !resolve_diagnostics.has_errors(),
                "failed to resolve `{expr}`:\n{}",
                resolve_diagnostics.render(&source)
            );
            let (types, diagnostics) = typecheck(&spec, &symbols);
            assert!(
                !diagnostics.has_errors(),
                "failed to typecheck `{expr}`:\n{}",
                diagnostics.render(&source)
            );
            let id = spec.functions[0].id.expect("function should resolve");
            types
                .signature_of(id)
                .expect("function should have a signature")
                .return_type
                .clone()
        }

        fn infer_in_function_body_with_argument(arg_ty: &str, expr: &str) -> StarkType {
            let source = format!("function f({arg_ty} x) {{ return {expr}; }}");
            let mut spec =
                UntypedStarkSpecification::parse(&source).unwrap_or_else(|e| panic!("failed to parse `{expr}`: {e}"));
            let (symbols, resolve_diagnostics) = resolve(&mut spec);
            assert!(
                !resolve_diagnostics.has_errors(),
                "failed to resolve `{expr}`:\n{}",
                resolve_diagnostics.render(&source)
            );
            let (types, diagnostics) = typecheck(&spec, &symbols);
            assert!(
                !diagnostics.has_errors(),
                "failed to typecheck `{expr}`:\n{}",
                diagnostics.render(&source)
            );
            let id = spec.functions[0].id.expect("function should resolve");
            types
                .signature_of(id)
                .expect("function should have a signature")
                .return_type
                .clone()
        }

        fn infer_as_constant(expr: &str) -> StarkType {
            let source = format!("const c = {expr};");
            let mut spec =
                UntypedStarkSpecification::parse(&source).unwrap_or_else(|e| panic!("failed to parse `{expr}`: {e}"));
            let (symbols, resolve_diagnostics) = resolve(&mut spec);
            assert!(
                !resolve_diagnostics.has_errors(),
                "failed to resolve `{expr}`:\n{}",
                resolve_diagnostics.render(&source)
            );
            let (types, diagnostics) = typecheck(&spec, &symbols);
            assert!(
                !diagnostics.has_errors(),
                "failed to typecheck `{expr}`:\n{}",
                diagnostics.render(&source)
            );
            let id = spec.constants[0].id.expect("constant should resolve");
            types.type_of(id).expect("constant should have a type").clone()
        }

        fn random(t: StarkType) -> StarkType {
            StarkType::random(t)
        }

        /// The original's shared `typeTests` map (`testExpressions`), always
        /// checked with randomness allowed.
        #[test]
        fn test_expressions() {
            let cases: Vec<(&str, StarkType)> = vec![
                ("2", StarkType::Integer),
                ("2.", StarkType::Real),
                ("true", StarkType::Boolean),
                ("false", StarkType::Boolean),
                ("2+3", StarkType::Integer),
                ("2.+3", StarkType::Real),
                ("2+3.", StarkType::Real),
                ("2.+3.", StarkType::Real),
                ("true & true", StarkType::Boolean),
                ("true | true", StarkType::Boolean),
                ("2 ^ 3", StarkType::Real),
                ("2 * 3", StarkType::Integer),
                ("2. * 3", StarkType::Real),
                ("2 * 3.", StarkType::Real),
                ("2. * 3.", StarkType::Real),
                ("2 + 3", StarkType::Integer),
                ("2. + 3", StarkType::Real),
                ("2 + 3.", StarkType::Real),
                ("2. + 3.", StarkType::Real),
                ("2. < 3", StarkType::Boolean),
                ("!true", StarkType::Boolean),
                ("(2<3?1.0:2.0)", StarkType::Real),
                ("(2<3?1.0:2)", StarkType::Real),
                ("(2<3?1:2.0)", StarkType::Real),
                ("(2<3?1:2)", StarkType::Integer),
                ("abs(1)", StarkType::Real),
                ("acos(1)", StarkType::Real),
                ("asin(1)", StarkType::Real),
                ("atan(1)", StarkType::Real),
                ("cbrt(1)", StarkType::Real),
                ("ceil(1)", StarkType::Real),
                ("cos(1)", StarkType::Real),
                ("cosh(1)", StarkType::Real),
                ("exp(1)", StarkType::Real),
                ("expm1(1)", StarkType::Real),
                ("floor(1)", StarkType::Real),
                ("log(1)", StarkType::Real),
                ("log10(1)", StarkType::Real),
                ("log1p(1)", StarkType::Real),
                ("signum(1)", StarkType::Real),
                ("sin(1)", StarkType::Real),
                ("sinh(1)", StarkType::Real),
                ("sqrt(1)", StarkType::Real),
                ("tan(1)", StarkType::Real),
                ("atan2(1,2)", StarkType::Real),
                ("hypot(1,2)", StarkType::Real),
                ("max(1,2)", StarkType::Real),
                ("min(1,2)", StarkType::Real),
                ("pow(1,2)", StarkType::Real),
                ("N[0.,1.]", random(StarkType::Real)),
                ("N[0,1]", random(StarkType::Real)),
                ("U[true,false]", random(StarkType::Boolean)),
                ("U[1,2,3]", random(StarkType::Integer)),
                ("U[1.0,2,3]", random(StarkType::Real)),
                ("R", random(StarkType::Real)),
                ("R[1, 10]", random(StarkType::Real)),
                ("(R<R)", random(StarkType::Boolean)),
                ("(R<R?1:2)", random(StarkType::Integer)),
            ];
            for (expr, expected) in cases {
                assert_eq!(expected, infer_in_function_body(expr), "{expr}");
            }
        }

        #[test]
        fn should_infer_integer_type() {
            assert_eq!(StarkType::Integer, infer_as_constant("2"));
        }

        #[test]
        fn should_infer_integer_type_from_variable() {
            assert_eq!(StarkType::Integer, infer_in_function_body_with_argument("int", "x"));
        }

        #[test]
        fn should_infer_real_type_from_variable() {
            assert_eq!(StarkType::Real, infer_in_function_body_with_argument("real", "x"));
        }

        #[test]
        fn should_infer_boolean_type_from_variable() {
            assert_eq!(StarkType::Boolean, infer_in_function_body_with_argument("bool", "x"));
        }

        #[test]
        fn should_infer_real_type() {
            assert_eq!(StarkType::Real, infer_as_constant("2."));
        }

        #[test]
        fn should_infer_boolean_type_from_true() {
            assert_eq!(StarkType::Boolean, infer_as_constant("true"));
        }

        #[test]
        fn should_infer_boolean_type_from_false() {
            assert_eq!(StarkType::Boolean, infer_as_constant("false"));
        }

        #[test]
        fn should_infer_random_boolean_type() {
            assert_eq!(random(StarkType::Boolean), infer_in_function_body("R < R"));
        }

        #[test]
        fn should_infer_random_real_type() {
            assert_eq!(random(StarkType::Real), infer_in_function_body("R"));
        }
    }
}
