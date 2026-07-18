//! Lowers a checked [StarkSpecification] to an [IrProgram]. See
//! `IR_LOWERING_PLAN.md` for the full design; this implements its Steps 0-3
//! (crate infra, `Value`, the IR arena, and expression/function/global/
//! variable/penalty lowering).
//!
//! Components' controller states, the environment block, perturbations,
//! distances and formulas (`IR_LOWERING_PLAN.md`'s Steps 4-5) have no IR
//! representation yet — [lower] reports each as a
//! [DiagnosticKind::NotYetSupported] diagnostic (with a span) rather than
//! panicking, so a spec using them fails gracefully instead of crashing.
//!
//! One deliberate deviation from the plan's stated Step 3 order ("Globals,
//! Variables, Functions"): a variable's initializer may call a function
//! declared earlier in the source (`resolve.rs` resolves functions *before*
//! variables for exactly this reason), so this pass lowers functions
//! *before* variables — a function's [FunctionId] and return type must exist
//! before anything that calls it can be lowered. Constants and parameters
//! can never call a function (they resolve before functions do), so globals
//! keep their place first.
//!
//! Because `spec` only exists if resolution and type checking both
//! succeeded, every `DefRef::id`/`StateRef::id`/`Binding` is `Some` and every
//! `DefId` is typed — violations are asserted (`.expect`/`debug_assert!`)
//! rather than diagnosed, mirroring `resolve.rs`'s and `typecheck.rs`'s own
//! contracts.

use std::collections::HashMap;

use log::debug;
use log::trace;
use merc_utilities::Span;

use crate::ast;
use crate::ast::Binding;
use crate::ast::DefId;
use crate::ast::Expression;
use crate::ast::ExpressionKind;
use crate::ast::Function;
use crate::ast::FunctionStatement;
use crate::ast::MathFunction;
use crate::ast::Ty;
use crate::ast::Variable;
use crate::diagnostics::DiagnosticKind;
use crate::diagnostics::Diagnostics;
use crate::ir::BinaryOp;
use crate::ir::CommandNode;
use crate::ir::CommandRef;
use crate::ir::ComponentId;
use crate::ir::ComponentIr;
use crate::ir::ExprList;
use crate::ir::ExprNode;
use crate::ir::ExprRef;
use crate::ir::FunctionId;
use crate::ir::FunctionIr;
use crate::ir::GlobalInit;
use crate::ir::IrProgram;
use crate::ir::IrStateId;
use crate::ir::MathBinaryFunction;
use crate::ir::MathUnaryFunction;
use crate::ir::PenaltyIr;
use crate::ir::SlotId;
use crate::ir::SlotInfo;
use crate::ir::SlotKind;
use crate::ir::StateIr;
use crate::ir::StmtNode;
use crate::ir::StmtRef;
use crate::ir::Update;
use crate::ir::VariableInfo;
use crate::resolve::SymbolTable;
use crate::specification::StarkSpecification;
use crate::typecheck::TypeTable;
use crate::types::StarkType;
use crate::value::CustomValue;
use crate::value::Value;

/// Lowers `spec` to an [IrProgram].
///
/// The `Result` exists for exactly one error class: constructs that resolve
/// and type-check but have no IR representation yet (see
/// `MISSING_GRAMMAR_FEATURES.md`; Java has the same hole). Everything else is
/// infallible.
pub fn lower(spec: &StarkSpecification) -> Result<IrProgram, Diagnostics> {
    let mut lowerer = Lowerer::new(spec);

    lowerer.check_not_yet_supported();
    lowerer.allocate_variable_slots();
    lowerer.allocate_global_slots();
    lowerer.lower_globals();
    lowerer.lower_functions();
    lowerer.lower_variables();
    lowerer.lower_components();
    lowerer.lower_environment();
    lowerer.lower_penalties();

    debug!(
        "lowered {} expression(s), {} statement(s), {} command(s), {} slot(s), {} global(s), \
         {} variable(s), {} function(s), {} component(s)/{} state(s), {} penalty/-ies; \
         {} diagnostic(s)",
        lowerer.exprs.len(),
        lowerer.stmts.len(),
        lowerer.commands.len(),
        lowerer.slots.len(),
        lowerer.globals.len(),
        lowerer.variables.len(),
        lowerer.functions.len(),
        lowerer.components.len(),
        lowerer.states.len(),
        lowerer.penalties.len(),
        lowerer.diagnostics.items().len()
    );

    let program = IrProgram {
        exprs: lowerer.exprs,
        expr_spans: lowerer.expr_spans,
        expr_types: lowerer.expr_types,
        expr_lists: lowerer.expr_lists,
        stmts: lowerer.stmts,
        commands: lowerer.commands,
        slots: lowerer.slots,
        variables: lowerer.variables,
        globals: lowerer.globals,
        functions: lowerer.functions,
        penalties: lowerer.penalties,
        states: lowerer.states,
        components: lowerer.components,
        environment: lowerer.environment,
    };

    debug_assert!(
        program.validate().is_ok(),
        "lower produced an internally inconsistent arena: {:?}",
        program.validate().err()
    );

    lowerer.diagnostics.into_result(program)
}

struct Lowerer<'a> {
    spec: &'a StarkSpecification,
    symbols: &'a SymbolTable,
    types: &'a TypeTable,
    /// Every `type` element's `DefId`, pre-mapped to the [CustomValue] it
    /// folds to — built once so `lower_reference` doesn't have to re-walk
    /// `spec.ast().types` for every reference.
    custom_values: HashMap<DefId, CustomValue>,

    exprs: Vec<ExprNode>,
    expr_spans: Vec<Span>,
    expr_types: Vec<StarkType>,
    expr_lists: Vec<ExprRef>,

    stmts: Vec<StmtNode>,
    commands: Vec<CommandNode>,

    slots: Vec<SlotInfo>,
    /// `DefId -> SlotId` for every constant, parameter and variable.
    def_slots: Vec<Option<SlotId>>,
    /// `LocalId -> SlotId` for every function argument and `let` binding.
    local_slots: Vec<Option<SlotId>>,
    /// `DefId -> FunctionId` for every function, filled in as each is
    /// lowered (in declaration order).
    def_functions: Vec<Option<FunctionId>>,
    /// The function currently being lowered, if any — `None` while lowering
    /// a global/variable initializer or a penalty, which aren't inside any
    /// function body.
    current_function: Option<FunctionId>,
    /// `ast::StateId -> IrStateId`, a straight 1:1 mapping since the AST's
    /// own `StateId` is already flat across every component (see
    /// [IrStateId]'s doc comment). Allocated up front per component, before
    /// any state body is lowered, so a `step`/`exec` to a later sibling
    /// state resolves just as well as one to an earlier sibling.
    def_states: Vec<Option<IrStateId>>,

    variables: Vec<VariableInfo>,
    globals: Vec<GlobalInit>,
    functions: Vec<FunctionIr>,
    penalties: Vec<PenaltyIr>,
    states: Vec<StateIr>,
    components: Vec<ComponentIr>,
    environment: Option<CommandRef>,

    diagnostics: Diagnostics,
}

impl<'a> Lowerer<'a> {
    fn new(spec: &'a StarkSpecification) -> Self {
        let symbols = spec.symbols();
        let types = spec.types();
        Lowerer {
            spec,
            symbols,
            types,
            custom_values: build_custom_value_map(spec),
            exprs: Vec::new(),
            expr_spans: Vec::new(),
            expr_types: Vec::new(),
            expr_lists: Vec::new(),
            stmts: Vec::new(),
            commands: Vec::new(),
            slots: Vec::new(),
            def_slots: vec![None; symbols.defs.len()],
            local_slots: vec![None; symbols.locals.len()],
            def_functions: vec![None; symbols.defs.len()],
            current_function: None,
            def_states: vec![None; symbols.states.len()],
            variables: Vec::new(),
            globals: Vec::new(),
            functions: Vec::new(),
            penalties: Vec::new(),
            states: Vec::new(),
            components: Vec::new(),
            environment: None,
            diagnostics: Diagnostics::new(),
        }
    }

    // -- Arena builders ---------------------------------------------------

    fn push_expr(&mut self, node: ExprNode, span: Span, ty: StarkType) -> ExprRef {
        let id = ExprRef::new(self.exprs.len() as u32);
        self.exprs.push(node);
        self.expr_spans.push(span);
        self.expr_types.push(ty);
        id
    }

    fn push_expr_list(&mut self, items: Vec<ExprRef>) -> ExprList {
        let start = self.expr_lists.len() as u32;
        let len = items.len() as u32;
        self.expr_lists.extend(items);
        ExprList { start, len }
    }

    fn push_stmt(&mut self, node: StmtNode) -> StmtRef {
        let id = StmtRef::new(self.stmts.len() as u32);
        self.stmts.push(node);
        id
    }

    fn push_command(&mut self, node: CommandNode) -> CommandRef {
        let id = CommandRef::new(self.commands.len() as u32);
        self.commands.push(node);
        id
    }

    fn alloc_slot(&mut self, name: String, ty: StarkType, kind: SlotKind, span: Span) -> SlotId {
        let id = SlotId::new(self.slots.len() as u32);
        trace!("allocating slot {id:?} for `{name}` : {ty} ({kind:?})");
        self.slots.push(SlotInfo { name, ty, kind, span });
        id
    }

    fn expr_type(&self, id: ExprRef) -> StarkType {
        self.expr_types[id.value() as usize].clone()
    }

    // -- Constructs with no IR representation yet -------------------------

    /// Reports every perturbation, distance and formula in `spec` as not yet
    /// supported (components/controllers and the environment are lowered —
    /// see [Self::lower_components]/[Self::lower_environment]). Collected as
    /// diagnostics (rather than the first one short-circuiting) so a spec
    /// using several of these still reports all of them at once, the way
    /// `resolve.rs`/`typecheck.rs` do for their own diagnostics.
    fn check_not_yet_supported(&mut self) {
        for perturbation in &self.spec.ast().perturbations {
            self.diagnostics.error(
                perturbation.name.span.clone(),
                DiagnosticKind::NotYetSupported {
                    construct: "perturbations",
                },
            );
        }
        for distance in &self.spec.ast().distances {
            self.diagnostics.error(
                distance.name.span.clone(),
                DiagnosticKind::NotYetSupported { construct: "distances" },
            );
        }
        for formula in &self.spec.ast().formulas {
            self.diagnostics.error(
                formula.name.span.clone(),
                DiagnosticKind::NotYetSupported {
                    construct: "ROBTL formulas",
                },
            );
        }
    }

    // -- Slot allocation ----------------------------------------------------

    /// Allocates `[0, n_variables)`: the global `variables { .. }` block,
    /// then every component's local one, matching `StarkGlobalVariableCollector`.
    fn allocate_variable_slots(&mut self) {
        for variable in &self.spec.ast().variables {
            self.allocate_variable_slot(variable);
        }
        for component in &self.spec.ast().components {
            for variable in &component.variables {
                self.allocate_variable_slot(variable);
            }
        }
    }

    fn allocate_variable_slot(&mut self, variable: &Variable) {
        let Some(id) = variable.id else { return };
        let ty = self.types.type_of(id).cloned().unwrap_or(StarkType::Error);
        let slot = self.alloc_slot(
            variable.name.name.clone(),
            ty,
            SlotKind::Variable,
            variable.name.span.clone(),
        );
        self.def_slots[id.value()] = Some(slot);
    }

    /// Allocates `[n_variables, n_globals)`: `const`s then `param`s, each in
    /// declaration order. Both already have a type from `typecheck.rs`, so —
    /// unlike locals — there's no need to defer filling in [SlotInfo::ty].
    fn allocate_global_slots(&mut self) {
        for constant in &self.spec.ast().constants {
            let Some(id) = constant.id else { continue };
            let ty = self.types.type_of(id).cloned().unwrap_or(StarkType::Error);
            let slot = self.alloc_slot(
                constant.name.name.clone(),
                ty,
                SlotKind::Global,
                constant.name.span.clone(),
            );
            self.def_slots[id.value()] = Some(slot);
        }
        for parameter in &self.spec.ast().parameters {
            let Some(id) = parameter.id else { continue };
            let ty = self.types.type_of(id).cloned().unwrap_or(StarkType::Error);
            let slot = self.alloc_slot(
                parameter.name.name.clone(),
                ty,
                SlotKind::Global,
                parameter.name.span.clone(),
            );
            self.def_slots[id.value()] = Some(slot);
        }
    }

    // -- Globals, variables, penalties --------------------------------------

    fn lower_globals(&mut self) {
        for constant in &self.spec.ast().constants {
            self.lower_global(constant.id, &constant.value);
        }
        for parameter in &self.spec.ast().parameters {
            self.lower_global(parameter.id, &parameter.value);
        }
    }

    fn lower_global(&mut self, id: Option<DefId>, value: &Expression) {
        let Some(id) = id else { return };
        let slot = self.def_slots[id.value()].expect("global slot allocated during slot allocation");
        let value = self.lower_expression(value);
        self.globals.push(GlobalInit { slot, value });
    }

    fn lower_variables(&mut self) {
        for variable in &self.spec.ast().variables {
            self.lower_variable(variable);
        }
        for component in &self.spec.ast().components {
            for variable in &component.variables {
                self.lower_variable(variable);
            }
        }
    }

    fn lower_variable(&mut self, variable: &Variable) {
        let Some(id) = variable.id else { return };
        let slot = self.def_slots[id.value()].expect("variable slot allocated during slot allocation");
        let range = variable
            .range
            .as_ref()
            .map(|range| (self.lower_expression(&range.min), self.lower_expression(&range.max)));
        let initial_value = self.lower_expression(&variable.initial_value);
        self.variables.push(VariableInfo {
            slot,
            range,
            initial_value,
        });
    }

    fn lower_penalties(&mut self) {
        for penalty in &self.spec.ast().penalties {
            let value = self.lower_expression(&penalty.value);
            self.penalties.push(PenaltyIr { value });
        }
    }

    // -- Components / controllers ------------------------------------------

    fn lower_components(&mut self) {
        for component in &self.spec.ast().components {
            self.lower_component(component);
        }
    }

    fn lower_component(&mut self, component: &ast::Component) {
        if component.id.is_none() {
            return;
        }
        trace!(
            "lowering component `{}` with {} state(s)",
            component.name.name,
            component.states.len()
        );
        let component_id = ComponentId::new(self.components.len() as u32);

        // Every state's `IrStateId` (and a placeholder `StateIr`) is
        // allocated before any body is lowered, since a `step`/`exec` may
        // target a state declared later in the same component.
        let mut state_ids = Vec::with_capacity(component.states.len());
        for state in &component.states {
            let Some(id) = state.id else { continue };
            let ir_state = IrStateId::new(self.states.len() as u32);
            self.states.push(StateIr {
                name: state.name.name.clone(),
                component: component_id,
                body: None,
            });
            self.def_states[id.value()] = Some(ir_state);
            state_ids.push(ir_state);
        }

        for state in &component.states {
            let Some(id) = state.id else { continue };
            let ir_state = self.def_states[id.value()].expect("state ir id allocated above");
            trace!("lowering state `{}` -> {ir_state:?}", state.name.name);
            let body = self.lower_controller_command_list(&state.body);
            self.states[ir_state.value() as usize].body = body;
        }

        let initial = component
            .init
            .iter()
            .map(|state_ref| self.lower_state_ref(state_ref))
            .collect();

        self.components.push(ComponentIr {
            name: component.name.name.clone(),
            states: state_ids,
            initial,
        });
    }

    fn lower_state_ref(&self, state_ref: &ast::StateRef) -> IrStateId {
        let id = state_ref.id.expect("state reference resolved by a clean resolution");
        self.def_states[id.value()].expect("state ir id allocated during component lowering")
    }

    /// Lowers a `{ .. }` block of commands to a left-associated chain of
    /// `Sequence` nodes, in source order. `None` for an empty block — there
    /// is nothing to run, so no node is pushed for it.
    fn lower_controller_command_list(&mut self, commands: &[ast::ControllerCommand]) -> Option<CommandRef> {
        let mut result: Option<CommandRef> = None;
        for command in commands {
            let Some(node) = self.lower_controller_command(command) else {
                continue;
            };
            result = Some(match result {
                None => node,
                Some(previous) => self.push_command(CommandNode::Sequence(previous, node)),
            });
        }
        result
    }

    /// `None` only for `ControllerCommand::Block(&[])`, an empty nested
    /// block — every other command always lowers to a node.
    fn lower_controller_command(&mut self, command: &ast::ControllerCommand) -> Option<CommandRef> {
        match command {
            ast::ControllerCommand::Step { steps, target } => {
                let steps = steps.as_ref().map(|steps| self.lower_expression(steps));
                let target = self.lower_state_ref(target);
                Some(self.push_command(CommandNode::Step { steps, target }))
            }
            ast::ControllerCommand::Exec(target) => {
                let target = self.lower_state_ref(target);
                Some(self.push_command(CommandNode::Exec(target)))
            }
            ast::ControllerCommand::Let { id, name, value, body } => {
                let value_ref = self.lower_expression(value);
                let ty = self.expr_type(value_ref);
                let local_id = id.expect("let binding resolved by a clean resolution");
                let slot = self.alloc_slot(name.name.clone(), ty, SlotKind::Local, name.span.clone());
                self.local_slots[local_id.value()] = Some(slot);
                let body = self.lower_controller_command_list(body);
                Some(self.push_command(CommandNode::Let {
                    slot,
                    value: value_ref,
                    body,
                }))
            }
            ast::ControllerCommand::Assignment(update) => {
                let update = self.lower_update(update);
                Some(self.push_command(CommandNode::Assign(update)))
            }
            ast::ControllerCommand::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                let guard = self.lower_expression(guard);
                let then_branch = self.lower_controller_command_list(then_branch);
                let else_branch = else_branch
                    .as_ref()
                    .and_then(|branch| self.lower_controller_command_list(branch));
                Some(self.push_command(CommandNode::IfThenElse {
                    guard,
                    then_branch,
                    else_branch,
                }))
            }
            // Not a distinct runtime construct — a nested `{ .. }` only
            // introduces grouping in the source, so it lowers to the same
            // `Sequence` chain a top-level list would (and, like any list,
            // may legitimately be empty).
            ast::ControllerCommand::Block(inner) => self.lower_controller_command_list(inner),
        }
    }

    fn lower_update(&mut self, update: &ast::Update) -> Update {
        let guard = update.guard.as_ref().map(|guard| self.lower_expression(guard));
        let value = self.lower_expression(&update.value);
        let target_id = update.target.id.expect("update target resolved by a clean resolution");
        let target = self.def_slots[target_id.value()].expect("variable slot allocated during slot allocation");
        Update { target, guard, value }
    }

    // -- Environment --------------------------------------------------------

    fn lower_environment(&mut self) {
        let Some(environment) = &self.spec.ast().environment else {
            return;
        };
        trace!("lowering the environment block with {} command(s)", environment.commands.len());
        self.environment = self.lower_environment_commands(&environment.commands);
    }

    /// Same idea as [Self::lower_controller_command_list], but over
    /// `ast::EnvironmentCommand` — there is no `Step`/`Exec` here, so a
    /// block simply runs to completion once every command in it has.
    fn lower_environment_commands(&mut self, commands: &[ast::EnvironmentCommand]) -> Option<CommandRef> {
        let mut result: Option<CommandRef> = None;
        for command in commands {
            let Some(node) = self.lower_environment_command(command) else {
                continue;
            };
            result = Some(match result {
                None => node,
                Some(previous) => self.push_command(CommandNode::Sequence(previous, node)),
            });
        }
        result
    }

    fn lower_environment_command(&mut self, command: &ast::EnvironmentCommand) -> Option<CommandRef> {
        match command {
            ast::EnvironmentCommand::Assignment(update) => {
                let update = self.lower_update(update);
                Some(self.push_command(CommandNode::Assign(update)))
            }
            ast::EnvironmentCommand::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                let guard = self.lower_expression(guard);
                let then_branch = self.lower_environment_command(then_branch);
                let else_branch = else_branch.as_ref().and_then(|branch| self.lower_environment_command(branch));
                Some(self.push_command(CommandNode::IfThenElse {
                    guard,
                    then_branch,
                    else_branch,
                }))
            }
            // `let a = e1 and b = e2(a) and .. in body`: each binding sees
            // every binding before it (matching `resolve.rs`'s nested-scope
            // treatment of the same construct), so this lowers to nested
            // `Let`s, innermost-bound-last-declared first, around `body`.
            ast::EnvironmentCommand::Let { bindings, body } => self.lower_environment_let(bindings, body),
            ast::EnvironmentCommand::Block(inner) => self.lower_environment_commands(inner),
        }
    }

    fn lower_environment_let(&mut self, bindings: &[ast::LocalVariable], body: &ast::EnvironmentCommand) -> Option<CommandRef> {
        let Some((first, rest)) = bindings.split_first() else {
            return self.lower_environment_command(body);
        };
        let value_ref = self.lower_expression(&first.value);
        let ty = self.expr_type(value_ref);
        let local_id = first.id.expect("let binding resolved by a clean resolution");
        let slot = self.alloc_slot(first.name.name.clone(), ty, SlotKind::Local, first.name.span.clone());
        self.local_slots[local_id.value()] = Some(slot);
        let inner = self.lower_environment_let(rest, body);
        Some(self.push_command(CommandNode::Let {
            slot,
            value: value_ref,
            body: inner,
        }))
    }

    // -- Functions ------------------------------------------------------

    fn lower_functions(&mut self) {
        for function in &self.spec.ast().functions {
            self.lower_function(function);
        }
    }

    fn lower_function(&mut self, function: &Function) {
        let Some(id) = function.id else { return };
        trace!(
            "lowering function `{}` with {} argument(s)",
            function.name.name,
            function.arguments.len()
        );

        let mut arguments = Vec::with_capacity(function.arguments.len());
        for argument in &function.arguments {
            let Some(local_id) = argument.id else { continue };
            let ty = self.lower_ty(&argument.ty);
            let slot = self.alloc_slot(
                argument.name.name.clone(),
                ty,
                SlotKind::Local,
                argument.name.span.clone(),
            );
            self.local_slots[local_id.value()] = Some(slot);
            arguments.push(slot);
        }

        // Assigned before the body is lowered (rather than after) so a call
        // to this very function inside its own body — impossible per the
        // no-recursion invariant, but this keeps the invariant assertable
        // instead of assumed — would still resolve consistently.
        let function_id = FunctionId::new(self.functions.len() as u32);
        self.def_functions[id.value()] = Some(function_id);

        let return_type = self
            .types
            .signature_of(id)
            .map(|signature| signature.return_type.clone())
            .unwrap_or(StarkType::Error);

        let previous_function = self.current_function;
        self.current_function = Some(function_id);
        let body = self.lower_function_statement(&function.body);
        self.current_function = previous_function;

        trace!("lowered function `{}` -> {function_id:?}", function.name.name);
        self.functions.push(FunctionIr {
            name: function.name.name.clone(),
            arguments,
            return_type,
            body,
        });
    }

    /// A `FunctionStatement::Let`'s slot is allocated here, lazily, rather
    /// than in a separate up-front pass over every function body: since no
    /// local is ever read outside the function it belongs to (STARK's
    /// scoping forbids it), the plan's "one slot-allocation pass before any
    /// lowering" requirement is satisfied just as well by allocating each
    /// local's slot the first time lowering reaches its binding site, in
    /// declaration order — the final ranges (`variables`, then `globals`,
    /// then this scratch tail) come out identical either way.
    fn lower_function_statement(&mut self, statement: &FunctionStatement) -> StmtRef {
        match statement {
            FunctionStatement::Return(value) => {
                let value = self.lower_expression(value);
                self.push_stmt(StmtNode::Return(value))
            }
            FunctionStatement::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                let guard = self.lower_expression(guard);
                let then_branch = self.lower_function_statement(then_branch);
                let else_branch = else_branch.as_ref().map(|branch| self.lower_function_statement(branch));
                self.push_stmt(StmtNode::IfThenElse {
                    guard,
                    then_branch,
                    else_branch,
                })
            }
            FunctionStatement::Let { id, name, value, body } => {
                let value = self.lower_expression(value);
                let ty = self.expr_type(value);
                let local_id = id.expect("let binding resolved by a clean resolution");
                let slot = self.alloc_slot(name.name.clone(), ty, SlotKind::Local, name.span.clone());
                self.local_slots[local_id.value()] = Some(slot);
                let body = self.lower_function_statement(body);
                self.push_stmt(StmtNode::Let { slot, value, body })
            }
            FunctionStatement::Block(inner) => self.lower_function_statement(inner),
        }
    }

    fn lower_ty(&self, ty: &Ty) -> StarkType {
        match ty {
            Ty::Integer => StarkType::Integer,
            Ty::Real => StarkType::Real,
            Ty::Boolean => StarkType::Boolean,
            // Already validated by `typecheck.rs`'s `ty_of_annotation`; no
            // need to re-check it names a declared `type` here.
            Ty::Named(name) => StarkType::Custom(name.clone()),
        }
    }

    // -- Expressions ------------------------------------------------------

    fn lower_expression(&mut self, expr: &Expression) -> ExprRef {
        let span = expr.span.clone();
        match &expr.node {
            ExpressionKind::False => self.push_expr(ExprNode::Literal(Value::Boolean(false)), span, StarkType::Boolean),
            ExpressionKind::True => self.push_expr(ExprNode::Literal(Value::Boolean(true)), span, StarkType::Boolean),
            ExpressionKind::Integer(value) => {
                self.push_expr(ExprNode::Literal(Value::Integer(*value)), span, StarkType::Integer)
            }
            ExpressionKind::Real(value) => {
                self.push_expr(ExprNode::Literal(Value::Real(*value)), span, StarkType::Real)
            }
            ExpressionKind::Iterator => {
                // Only reachable from aggregate/lambda contexts, none of
                // which exist in the current grammar (see `ast.rs` /
                // `MISSING_GRAMMAR_FEATURES.md`) — `typecheck.rs` types this
                // `Error` without diagnosing it for the same reason.
                debug_assert!(
                    false,
                    "ExpressionKind::Iterator is unreachable: no aggregate context exists in the current grammar"
                );
                self.push_expr(ExprNode::Literal(Value::Error), span, StarkType::Error)
            }
            ExpressionKind::Reference { binding, .. } => {
                let binding = binding.expect("reference resolved by a clean resolution");
                self.lower_reference(binding, span)
            }
            ExpressionKind::Normal { mean, std_dev } => {
                let mean = self.lower_expression(mean);
                let variance = self.lower_expression(std_dev);
                self.push_expr(
                    ExprNode::SampleNormal { mean, variance },
                    span,
                    StarkType::random(StarkType::Real),
                )
            }
            ExpressionKind::Uniform { values } => {
                let mut merged: Option<StarkType> = None;
                let mut lowered = Vec::with_capacity(values.len());
                for value in values {
                    let value_ref = self.lower_expression(value);
                    let ty = self.expr_type(value_ref);
                    merged = Some(match merged {
                        None => ty,
                        Some(acc) => acc.merge(&ty),
                    });
                    lowered.push(value_ref);
                }
                let list = self.push_expr_list(lowered);
                let ty = StarkType::random(merged.unwrap_or(StarkType::Error));
                self.push_expr(ExprNode::SampleChoice(list), span, ty)
            }
            ExpressionKind::Range { min, max } => match (min, max) {
                (Some(min), Some(max)) => {
                    let min = self.lower_expression(min);
                    let max = self.lower_expression(max);
                    self.push_expr(
                        ExprNode::SampleRange { min, max },
                        span,
                        StarkType::random(StarkType::Real),
                    )
                }
                // The grammar only ever produces `R` (neither bound) or
                // `R[min,max]` (both) — a mixed case can't be parsed, so
                // treating it the same as `R` (matching `typecheck.rs`'s own
                // `_ => ..` here) never actually applies to any real input.
                _ => self.push_expr(ExprNode::SampleUnit, span, StarkType::random(StarkType::Real)),
            },
            ExpressionKind::Not(inner) => {
                let inner = self.lower_expression(inner);
                let ty = self.expr_type(inner);
                self.push_expr(ExprNode::Not(inner), span, ty)
            }
            // The identity: lowers straight through to its operand, pushing
            // no node of its own.
            ExpressionKind::UnaryPlus(inner) => self.lower_expression(inner),
            ExpressionKind::UnaryMinus(inner) => {
                let inner = self.lower_expression(inner);
                let ty = self.expr_type(inner);
                self.push_expr(ExprNode::Negate(inner), span, ty)
            }
            ExpressionKind::Binary(op, left, right) => self.lower_binary(*op, left, right, span),
            ExpressionKind::Ternary {
                guard,
                then_branch,
                else_branch,
            } => {
                let guard_ref = self.lower_expression(guard);
                let then_ref = self.lower_expression(then_branch);
                let else_ref = self.lower_expression(else_branch);
                let guard_ty = self.expr_type(guard_ref);
                let merged = self.expr_type(then_ref).merge(&self.expr_type(else_ref));
                let ty = if !merged.is_error() && guard_ty.is_random() {
                    StarkType::random(merged)
                } else {
                    merged
                };
                self.push_expr(
                    ExprNode::Select {
                        guard: guard_ref,
                        then_branch: then_ref,
                        else_branch: else_ref,
                    },
                    span,
                    ty,
                )
            }
            ExpressionKind::Call { function, arguments } => self.lower_call(function, arguments, span),
            ExpressionKind::MathCall { function, arguments } => self.lower_math_call(*function, arguments, span),
        }
    }

    fn lower_reference(&mut self, binding: Binding, span: Span) -> ExprRef {
        match binding {
            Binding::Local(local_id) => {
                let slot = self.local_slots[local_id.value()].expect("local slot allocated before its first use");
                let ty = self.slots[slot.value() as usize].ty.clone();
                self.push_expr(ExprNode::Load(slot), span, ty)
            }
            Binding::Def(def_id) => {
                if let Some(&custom) = self.custom_values.get(&def_id) {
                    // A `type` element's value is a fixed ordinal, known
                    // outright at lowering time — no slot, no expression to
                    // evaluate, just a literal.
                    let ty = StarkType::Custom(self.symbols.def(custom.type_id).name.clone());
                    self.push_expr(ExprNode::Literal(Value::Custom(custom)), span, ty)
                } else {
                    let slot = self.def_slots[def_id.value()].expect("def slot allocated during slot allocation");
                    let ty = self.slots[slot.value() as usize].ty.clone();
                    self.push_expr(ExprNode::Load(slot), span, ty)
                }
            }
        }
    }

    /// `combineToRealType` in the Java source: always widens to `real`,
    /// propagating randomness from either operand. Mirrors
    /// `typecheck.rs`'s `combine_to_real`, minus the diagnostics — `spec`
    /// already type-checked, so there is nothing left to reject here.
    fn combine_to_real(&self, left: ExprRef, right: ExprRef) -> StarkType {
        if self.expr_type(left).is_random() || self.expr_type(right).is_random() {
            StarkType::random(StarkType::Real)
        } else {
            StarkType::Real
        }
    }

    fn lower_binary(&mut self, op: ast::BinaryOp, left: &Expression, right: &Expression, span: Span) -> ExprRef {
        use ast::BinaryOp as AstOp;
        match op {
            AstOp::Pow => {
                let left = self.lower_expression(left);
                let right = self.lower_expression(right);
                let ty = self.combine_to_real(left, right);
                self.push_expr(ExprNode::MathBinary(MathBinaryFunction::Pow, left, right), span, ty)
            }
            AstOp::Mult | AstOp::Div | AstOp::IntDiv | AstOp::Add | AstOp::Subtract | AstOp::Mod => {
                let left = self.lower_expression(left);
                let right = self.lower_expression(right);
                let ty = self.expr_type(left).merge(&self.expr_type(right));
                self.push_expr(ExprNode::Binary(map_binary_op(op), left, right), span, ty)
            }
            AstOp::Less | AstOp::Leq | AstOp::Eq | AstOp::Geq | AstOp::Greater => {
                let left = self.lower_expression(left);
                let right = self.lower_expression(right);
                let ty = if self.expr_type(left).is_random() || self.expr_type(right).is_random() {
                    StarkType::random(StarkType::Boolean)
                } else {
                    StarkType::Boolean
                };
                self.push_expr(ExprNode::Binary(map_binary_op(op), left, right), span, ty)
            }
            AstOp::BitAnd | AstOp::And | AstOp::BitOr | AstOp::Or => {
                let left = self.lower_expression(left);
                let right = self.lower_expression(right);
                let ty = if self.expr_type(left).is_random() || self.expr_type(right).is_random() {
                    StarkType::random(StarkType::Boolean)
                } else {
                    StarkType::Boolean
                };
                self.push_expr(ExprNode::Binary(map_binary_op(op), left, right), span, ty)
            }
        }
    }

    fn lower_call(&mut self, function: &ast::DefRef, arguments: &[Expression], span: Span) -> ExprRef {
        let callee_def_id = function.id.expect("call target resolved by a clean resolution");
        let callee_function_id = self.def_functions[callee_def_id.value()].unwrap_or_else(|| {
            panic!(
                "call to `{}` lowered before its callee — the no-recursion invariant should make this impossible",
                function.name.name
            )
        });
        // The no-recursion invariant the flat slot layout depends on: a
        // function can only call one declared strictly before it, so it is
        // always already lowered. Only meaningful function-to-function (a
        // variable initializer or penalty calling a function has no
        // "current function" to compare against, and needs no such check —
        // the callee being in `def_functions` at all already proves it was
        // lowered first).
        if let Some(current) = self.current_function {
            debug_assert!(
                callee_function_id.value() < current.value(),
                "call to {callee_function_id:?} from {current:?} violates the no-recursion invariant"
            );
        }

        let mut lowered_arguments = Vec::with_capacity(arguments.len());
        for argument in arguments {
            lowered_arguments.push(self.lower_expression(argument));
        }
        debug_assert_eq!(
            lowered_arguments.len(),
            self.functions[callee_function_id.value() as usize].arguments.len(),
            "argument count mismatch for `{}` survived type checking",
            function.name.name
        );
        let arguments = self.push_expr_list(lowered_arguments);
        let return_type = self.functions[callee_function_id.value() as usize].return_type.clone();
        self.push_expr(
            ExprNode::Call {
                function: callee_function_id,
                arguments,
            },
            span,
            return_type,
        )
    }

    fn lower_math_call(&mut self, function: MathFunction, arguments: &[Expression], span: Span) -> ExprRef {
        match function {
            MathFunction::Atan2 | MathFunction::Hypot | MathFunction::Max | MathFunction::Min | MathFunction::Pow => {
                debug_assert_eq!(
                    arguments.len(),
                    2,
                    "binary math function {function:?} parsed with {} argument(s)",
                    arguments.len()
                );
                let left = self.lower_expression(&arguments[0]);
                let right = self.lower_expression(&arguments[1]);
                let ty = self.combine_to_real(left, right);
                self.push_expr(ExprNode::MathBinary(map_math_binary(function), left, right), span, ty)
            }
            _ => {
                debug_assert_eq!(
                    arguments.len(),
                    1,
                    "unary math function {function:?} parsed with {} argument(s)",
                    arguments.len()
                );
                let inner = self.lower_expression(&arguments[0]);
                let ty = if self.expr_type(inner).is_random() {
                    StarkType::random(StarkType::Real)
                } else {
                    StarkType::Real
                };
                self.push_expr(ExprNode::MathUnary(map_math_unary(function), inner), span, ty)
            }
        }
    }
}

/// Builds the `type` element `DefId -> CustomValue` map once up front. An
/// element's `DefId` isn't stored back onto the AST by `resolve.rs` (`type`
/// declarations keep their elements as plain `Identifier`s), so this looks
/// each one back up by name via [SymbolTable::by_name] instead.
fn build_custom_value_map(spec: &StarkSpecification) -> HashMap<DefId, CustomValue> {
    let mut map = HashMap::new();
    for declaration in &spec.ast().types {
        let Some(type_id) = declaration.id else { continue };
        for (ordinal, element) in declaration.elements.iter().enumerate() {
            if let Some(element_id) = spec.symbols().by_name(&element.name) {
                map.insert(
                    element_id,
                    CustomValue {
                        type_id,
                        element: ordinal as u32,
                    },
                );
            }
        }
    }
    map
}

fn map_binary_op(op: ast::BinaryOp) -> BinaryOp {
    match op {
        ast::BinaryOp::Pow => unreachable!("BinaryOp::Pow is lowered as MathBinary(Pow, ..), not Binary"),
        ast::BinaryOp::Mult => BinaryOp::Mult,
        ast::BinaryOp::Div => BinaryOp::Div,
        ast::BinaryOp::IntDiv => BinaryOp::IntDiv,
        ast::BinaryOp::Add => BinaryOp::Add,
        ast::BinaryOp::Subtract => BinaryOp::Subtract,
        ast::BinaryOp::Mod => BinaryOp::Mod,
        ast::BinaryOp::Less => BinaryOp::Less,
        ast::BinaryOp::Leq => BinaryOp::Leq,
        ast::BinaryOp::Eq => BinaryOp::Eq,
        ast::BinaryOp::Geq => BinaryOp::Geq,
        ast::BinaryOp::Greater => BinaryOp::Greater,
        ast::BinaryOp::BitAnd => BinaryOp::BitAnd,
        ast::BinaryOp::And => BinaryOp::And,
        ast::BinaryOp::BitOr => BinaryOp::BitOr,
        ast::BinaryOp::Or => BinaryOp::Or,
    }
}

fn map_math_binary(function: MathFunction) -> MathBinaryFunction {
    match function {
        MathFunction::Atan2 => MathBinaryFunction::Atan2,
        MathFunction::Hypot => MathBinaryFunction::Hypot,
        MathFunction::Max => MathBinaryFunction::Max,
        MathFunction::Min => MathBinaryFunction::Min,
        MathFunction::Pow => MathBinaryFunction::Pow,
        other => unreachable!("{other:?} is not a binary math function"),
    }
}

fn map_math_unary(function: MathFunction) -> MathUnaryFunction {
    match function {
        MathFunction::Abs => MathUnaryFunction::Abs,
        MathFunction::Acos => MathUnaryFunction::Acos,
        MathFunction::Asin => MathUnaryFunction::Asin,
        MathFunction::Atan => MathUnaryFunction::Atan,
        MathFunction::Cbrt => MathUnaryFunction::Cbrt,
        MathFunction::Ceil => MathUnaryFunction::Ceil,
        MathFunction::Cos => MathUnaryFunction::Cos,
        MathFunction::Cosh => MathUnaryFunction::Cosh,
        MathFunction::Exp => MathUnaryFunction::Exp,
        MathFunction::Expm1 => MathUnaryFunction::Expm1,
        MathFunction::Floor => MathUnaryFunction::Floor,
        MathFunction::Log => MathUnaryFunction::Log,
        MathFunction::Log10 => MathUnaryFunction::Log10,
        MathFunction::Log1p => MathUnaryFunction::Log1p,
        MathFunction::Signum => MathUnaryFunction::Signum,
        MathFunction::Sin => MathUnaryFunction::Sin,
        MathFunction::Sinh => MathUnaryFunction::Sinh,
        MathFunction::Sqrt => MathUnaryFunction::Sqrt,
        MathFunction::Tan => MathUnaryFunction::Tan,
        other => unreachable!("{other:?} is not a unary math function"),
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;
    use crate::ast::UntypedStarkSpecification;
    use crate::ir::ExprNode;
    use crate::ir::StmtNode;

    fn lower_source(src: &str) -> IrProgram {
        let spec = UntypedStarkSpecification::parse(src)
            .unwrap_or_else(|e| panic!("failed to parse: {e}"))
            .check()
            .unwrap_or_else(|d| panic!("failed to check:\n{}", d.render(src)));
        lower(&spec).unwrap_or_else(|d| panic!("failed to lower:\n{}", d.render(src)))
    }

    #[test]
    fn lowers_a_constant_to_a_global_init() {
        let program = lower_source("const a = 1 + 2;");
        assert_eq!(program.globals().len(), 1);
        let global = &program.globals()[0];
        assert_eq!(program.slot(global.slot).name, "a");
        assert!(matches!(
            program.expr(global.value),
            ExprNode::Binary(BinaryOp::Add, ..)
        ));
        program.validate().unwrap();
    }

    #[test]
    fn lowers_a_variable_with_a_range() {
        let program = lower_source("variables { int x range[0, 10] = 5; }");
        assert_eq!(program.variables().len(), 1);
        let variable = &program.variables()[0];
        assert_eq!(program.slot(variable.slot).name, "x");
        assert!(variable.range.is_some());
        program.validate().unwrap();
    }

    #[test]
    fn variable_slots_occupy_the_lowest_range() {
        // Constants/parameters resolve before variables (`resolve.rs`'s
        // fixed kind order), but slot *numbers* must still put variables
        // first — this is the one place source/resolve order and slot order
        // deliberately diverge (see `IR_LOWERING_PLAN.md`'s "Slot layout").
        let program = lower_source("const c = 1;\nparam p = 2;\nvariables { int x = 0; }");
        let variable_slot = program.variables()[0].slot;
        let global_slots: Vec<_> = program.globals().iter().map(|g| g.slot).collect();
        for global_slot in global_slots {
            assert!(
                variable_slot.value() < global_slot.value(),
                "variable slot {variable_slot:?} should come before global slot {global_slot:?}"
            );
        }
    }

    #[test]
    fn two_functions_with_same_named_arguments_get_distinct_slots() {
        let program = lower_source("function f(int x) { return x; }\nfunction g(int x) { return x; }");
        assert_eq!(program.functions().len(), 2);
        assert_ne!(program.functions()[0].arguments[0], program.functions()[1].arguments[0]);
    }

    #[test]
    fn let_shadowing_an_argument_gets_its_own_slot() {
        let program = lower_source("function f(int x) { let x = x + 1 in return x; }");
        let function = &program.functions()[0];
        let argument_slot = function.arguments[0];
        let StmtNode::Let { slot: let_slot, .. } = program.stmt(function.body) else {
            panic!("expected a let statement");
        };
        assert_ne!(argument_slot, *let_slot);
    }

    #[test]
    fn call_to_an_earlier_function_resolves_to_its_function_id() {
        let program =
            lower_source("function inc(int x) { return x + 1; }\nfunction twice(int x) { return inc(inc(x)); }");
        assert_eq!(program.functions().len(), 2);
        let twice = &program.functions()[1];
        let StmtNode::Return(value) = program.stmt(twice.body) else {
            panic!("expected a return statement");
        };
        let ExprNode::Call { function, .. } = program.expr(*value) else {
            panic!("expected a call");
        };
        assert_eq!(
            function.value(),
            0,
            "should call `inc`, the first (and only other) function"
        );
        program.validate().unwrap();
    }

    #[test]
    fn a_type_element_reference_folds_to_a_literal() {
        // `type` declarations resolve (and are typed) before variables in
        // `resolve.rs`'s fixed kind order, so a variable's initial value can
        // reference an element of one — unlike a `const`, which resolves
        // before `type` declarations are even seen, or a `penalty`, which
        // must be numerical.
        let program = lower_source("type Color = Red | Green | Blue;\nvariables { Color c = Green; }");
        let variable = &program.variables()[0];
        match program.expr(variable.initial_value) {
            ExprNode::Literal(Value::Custom(custom)) => assert_eq!(custom.element, 1),
            other => panic!("expected a custom literal, found {other:?}"),
        }
    }

    #[test]
    fn pow_lowers_to_a_math_binary_node() {
        let program = lower_source("const c = 2 ^ 3;");
        let global = &program.globals()[0];
        assert!(matches!(
            program.expr(global.value),
            ExprNode::MathBinary(MathBinaryFunction::Pow, ..)
        ));
    }

    #[test]
    fn unary_plus_disappears() {
        let program = lower_source("const c = +1;");
        let global = &program.globals()[0];
        assert!(matches!(
            program.expr(global.value),
            ExprNode::Literal(Value::Integer(1))
        ));
    }

    #[test]
    fn buffered_swap_reads_pre_state_slots() {
        // Both sides of a `let`-based swap read the value bound *before* the
        // swap happened — this doesn't exercise controller/environment
        // lowering (not implemented yet), but confirms the same principle
        // holds for an ordinary function-local `let`, which the buffered
        // controller/environment update semantics (`IR_LOWERING_PLAN.md`
        // Step 4) will build on.
        let program = lower_source("function f(int a, int b) { let t = a in return b + t; }");
        let function = &program.functions()[0];
        let (a_slot, b_slot) = (function.arguments[0], function.arguments[1]);
        let StmtNode::Let {
            slot: t_slot,
            value,
            body,
        } = program.stmt(function.body)
        else {
            panic!("expected a let statement");
        };
        assert!(matches!(program.expr(*value), ExprNode::Load(slot) if *slot == a_slot));
        let StmtNode::Return(sum) = program.stmt(*body) else {
            panic!("expected a return statement");
        };
        let ExprNode::Binary(BinaryOp::Add, left, right) = program.expr(*sum) else {
            panic!("expected an addition");
        };
        assert!(matches!(program.expr(*left), ExprNode::Load(slot) if *slot == b_slot));
        assert!(matches!(program.expr(*right), ExprNode::Load(slot) if *slot == *t_slot));
    }

    #[test]
    fn display_renders_source_like_text() {
        let program = lower_source("const a = 1;\nfunction f(int x) { return x + a; }");
        let rendered = program.to_string();
        assert!(rendered.contains("fn f"), "{rendered}");
        assert!(rendered.contains("return"), "{rendered}");
        assert!(rendered.contains("load"), "{rendered}");
    }

    #[test]
    fn lowers_a_component_with_a_self_looping_state() {
        let program = lower_source("component C {\n  variables { }\n  controller {\n    state A { step A; }\n  }\n  init A\n}");
        assert_eq!(program.components().len(), 1);
        let component = &program.components()[0];
        assert_eq!(component.name, "C");
        assert_eq!(component.states.len(), 1);
        assert_eq!(component.initial, component.states);

        let state = program.state(component.states[0]);
        assert_eq!(state.name, "A");
        let CommandNode::Step { target, .. } = program.command(state.body.expect("non-empty body")) else {
            panic!("expected a step");
        };
        assert_eq!(*target, component.states[0], "should step to itself");
        program.validate().unwrap();
    }

    #[test]
    fn step_to_a_later_sibling_state_resolves() {
        // `A` targets `B`, declared afterwards — states are pre-allocated
        // before any body is lowered so this forward reference resolves.
        let program =
            lower_source("component C {\n  variables { }\n  controller {\n    state A { step B; }\n    state B { step B; }\n  }\n  init A\n}");
        let component = &program.components()[0];
        let (a, b) = (component.states[0], component.states[1]);
        let CommandNode::Step { target, .. } = program.command(program.state(a).body.unwrap()) else {
            panic!("expected a step");
        };
        assert_eq!(*target, b);
    }

    #[test]
    fn controller_assignment_is_sequenced_before_its_step() {
        let program = lower_source(
            "global variables { int x = 0; }\ncomponent C {\n  variables { }\n  controller {\n    state A { x' = x + 1; step A; }\n  }\n  init A\n}",
        );
        let component = &program.components()[0];
        let body = program.state(component.states[0]).body.expect("non-empty body");
        let CommandNode::Sequence(first, second) = program.command(body) else {
            panic!("expected a sequence of the assignment and the step");
        };
        assert!(matches!(program.command(*first), CommandNode::Assign(_)));
        assert!(matches!(program.command(*second), CommandNode::Step { .. }));
        program.validate().unwrap();
    }

    #[test]
    fn environment_buffered_swap_reads_pre_state_slots() {
        // The classic swap, this time through real environment lowering
        // (rather than a function-local `let` standing in for it, as
        // `buffered_swap_reads_pre_state_slots` above does): both
        // assignments must read the *pre*-step value, matching Java's
        // "collect updates, apply them all at the end of the step"
        // semantics (`IR_LOWERING_PLAN.md`'s Step 4).
        let program = lower_source("global variables { int x = 1; int y = 2; }\nenvironment { x' = y; y' = x; }");
        let environment = program.environment().expect("environment block lowered");
        let CommandNode::Sequence(first, second) = program.command(environment) else {
            panic!("expected a sequence of the two assignments");
        };
        let CommandNode::Assign(update_x) = program.command(*first) else {
            panic!("expected the first assignment");
        };
        let CommandNode::Assign(update_y) = program.command(*second) else {
            panic!("expected the second assignment");
        };
        assert_eq!(program.slot(update_x.target).name, "x");
        assert_eq!(program.slot(update_y.target).name, "y");
        let x_slot = update_x.target;
        let y_slot = update_y.target;
        assert!(matches!(program.expr(update_x.value), ExprNode::Load(slot) if *slot == y_slot));
        assert!(matches!(program.expr(update_y.value), ExprNode::Load(slot) if *slot == x_slot));
        program.validate().unwrap();
    }

    #[test]
    fn environment_let_bindings_chain_and_see_each_other() {
        let program = lower_source(
            "global variables { int x = 1; }\nenvironment { let a = x and b = a + 1 in { x' = b; } }",
        );
        let environment = program.environment().expect("environment block lowered");
        let CommandNode::Let { slot: a_slot, body, .. } = program.command(environment) else {
            panic!("expected the outer `let a = ..`");
        };
        let CommandNode::Let { value: b_value, body, .. } = program.command(body.expect("non-empty body")) else {
            panic!("expected the nested `let b = ..`");
        };
        // `b`'s value (`a + 1`) reads the slot the outer `let` just bound.
        let ExprNode::Binary(BinaryOp::Add, left, _) = program.expr(*b_value) else {
            panic!("expected `a + 1`");
        };
        assert!(matches!(program.expr(*left), ExprNode::Load(slot) if slot == a_slot));
        // The `b`-let's own body is the innermost `{ x' = b; }` block — a
        // plain assignment, not another `let`.
        assert!(matches!(program.command(body.expect("non-empty body")), CommandNode::Assign(_)));
        program.validate().unwrap();
    }

    #[test]
    fn environment_if_with_no_else_lowers_with_no_else_branch() {
        let program = lower_source("global variables { bool flag = true; int x = 0; }\nenvironment { if (flag) { x' = 1; } }");
        let environment = program.environment().expect("environment block lowered");
        let CommandNode::IfThenElse { else_branch, .. } = program.command(environment) else {
            panic!("expected an if-then-else");
        };
        assert!(else_branch.is_none());
        program.validate().unwrap();
    }

    #[test]
    fn validate_rejects_a_corrupted_arena() {
        let mut program = lower_source("const a = 1;");
        // Corrupt the arena the same way a lowering bug would: an
        // out-of-bounds `ExprRef` in an otherwise-valid global.
        program.globals[0].value = ExprRef::new(999);
        assert!(program.validate().is_err());
    }

    #[test]
    fn validate_rejects_a_corrupted_statement() {
        // Same idea as `validate_rejects_a_corrupted_arena`, but for a ref
        // that only appears *inside* the statement arena (an `IfThenElse`'s
        // `else_branch`) rather than off a top-level global/variable — this
        // is the case `validate()` used to skip entirely.
        let mut program = lower_source("function f(int x) { if (x > 0) return 1; else return 2; }");
        let function = program.functions()[0].clone();
        let StmtNode::IfThenElse { else_branch, .. } = program.stmt(function.body) else {
            panic!("expected an if-then-else statement");
        };
        assert!(else_branch.is_some(), "expected an else branch");
        let index = function.body.value() as usize;
        let StmtNode::IfThenElse { else_branch, .. } = &mut program.stmts[index] else {
            panic!("expected an if-then-else statement");
        };
        *else_branch = Some(StmtRef::new(999));
        assert!(program.validate().is_err());
    }
}
