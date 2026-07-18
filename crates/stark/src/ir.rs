//! The evaluation IR that `lower.rs` produces: a flat arena of small, `Copy`
//! nodes rather than a closure tree, so evaluation walks an array instead of
//! chasing pointers. See `IR_LOWERING_PLAN.md` for the design rationale.
//!
//! Currently populated by lowering: constants/parameters (as [GlobalInit]),
//! variables (as [VariableInfo]), functions (as [FunctionIr]), penalties (as
//! [PenaltyIr]), components/controller states (as [ComponentIr]/[StateIr])
//! and the environment block, together with the shared expression/statement/
//! command arenas they're built from. Perturbations, distances and formulas
//! are not lowered yet (`lower.rs` reports them as
//! [crate::diagnostics::DiagnosticKind::NotYetSupported]) and so have no IR
//! representation here yet either — see the plan's Step 5.

use std::fmt;

use merc_utilities::Span;
use merc_utilities::TagIndex;

use crate::types::StarkType;
use crate::value::Value;

// ---------------------------------------------------------------------------
// Index types
// ---------------------------------------------------------------------------
//
// All backed by `u32`, not `usize`: nodes that hold these stay small. Each
// has its own tag so, say, an `ExprRef` can never be mixed up with a
// `SlotId` at a call site even though both are "just a `u32`" underneath.

pub struct ExprTag;
/// An index into [IrProgram]'s expression arena.
pub type ExprRef = TagIndex<u32, ExprTag>;

pub struct StmtTag;
/// An index into [IrProgram]'s statement arena (function bodies).
pub type StmtRef = TagIndex<u32, StmtTag>;

pub struct SlotTag;
/// An index into the flat value store the (future) evaluator maintains —
/// see "Slot layout" in `IR_LOWERING_PLAN.md`.
pub type SlotId = TagIndex<u32, SlotTag>;

pub struct FunctionTag;
/// An index into [IrProgram]'s lowered functions, assigned in declaration
/// order (which — since STARK forbids recursion — is always a valid
/// topological order of the call graph).
pub type FunctionId = TagIndex<u32, FunctionTag>;

pub struct PenaltyTag;
/// An index into [IrProgram]'s lowered penalties.
pub type PenaltyId = TagIndex<u32, PenaltyTag>;

pub struct CommandTag;
/// An index into [IrProgram]'s command arena (controller state bodies and
/// the environment block).
pub type CommandRef = TagIndex<u32, CommandTag>;

pub struct IrStateTag;
/// An index into [IrProgram]'s flat, cross-component controller state list.
/// The AST's own `StateId` (see `ast.rs`) is already flat across every
/// component (`SymbolTable` keeps one `Vec<StateEntry>` for the whole
/// specification, not one per component), so this is a straight 1:1 mapping
/// from it — kept as its own tag purely so the IR never has to import an
/// `ast::` index type to name a slice of its own arena.
pub type IrStateId = TagIndex<u32, IrStateTag>;

pub struct ComponentTag;
/// An index into [IrProgram]'s lowered components.
pub type ComponentId = TagIndex<u32, ComponentTag>;

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

/// A binary operator as the IR needs it: `BinaryOp::Pow` from the AST
/// collapses into `MathBinary(MathBinaryFunction::Pow, ..)` during lowering
/// (see [ExprNode]'s doc comment), so this has no `Pow` case of its own.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Mult,
    Div,
    IntDiv,
    Add,
    Subtract,
    Mod,
    Less,
    Leq,
    Eq,
    Geq,
    Greater,
    BitAnd,
    And,
    BitOr,
    Or,
}

/// The unary half of `ast::MathFunction`, split out so the evaluator never
/// has to check arity for a math call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathUnaryFunction {
    Abs,
    Acos,
    Asin,
    Atan,
    Cbrt,
    Ceil,
    Cos,
    Cosh,
    Exp,
    Expm1,
    Floor,
    Log,
    Log10,
    Log1p,
    Signum,
    Sin,
    Sinh,
    Sqrt,
    Tan,
}

/// The binary half of `ast::MathFunction`. Also where `BinaryOp::Pow` (`^`)
/// lands, since `BinaryOp::Pow` and `MathFunction::Pow` are the same
/// operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathBinaryFunction {
    Atan2,
    Hypot,
    Max,
    Min,
    Pow,
}

/// A `{ start, len }` slice into [IrProgram::expr_lists], keeping argument
/// and element lists contiguous rather than each becoming its own `Vec`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExprList {
    pub start: u32,
    pub len: u32,
}

impl ExprList {
    pub const EMPTY: ExprList = ExprList { start: 0, len: 0 };
}

/// One node of the expression arena.
///
/// Deliberate simplifications made while lowering (see `IR_LOWERING_PLAN.md`
/// Step 2 for the full rationale):
/// - `Expression::UnaryPlus` disappears (it is the identity).
/// - `Ty` / custom type names disappear; only [StarkType] and slot indices
///   survive (in [IrProgram::expr_types] / [IrProgram::slots]).
/// - `Expression::Reference` (to a constant, parameter or variable) and
///   `Expression::Iterator` both become `Load(slot)` — the distinction
///   between a global, a constant and a `let` binding is erased, since it is
///   exactly what the slot index already encodes. A reference to a `type`
///   element instead folds to `Literal(Value::Custom(..))`, since its value
///   is known outright at lowering time, not computed from an expression.
/// - `FunctionStatement::Block` disappears (it only ever wraps one
///   statement).
#[derive(Clone, Copy, Debug)]
pub enum ExprNode {
    Literal(Value),
    /// A read of `store[slot]` — the whole point of this IR: every name
    /// resolution already did gets baked into the node.
    Load(SlotId),
    Not(ExprRef),
    Negate(ExprRef),
    Binary(BinaryOp, ExprRef, ExprRef),
    MathUnary(MathUnaryFunction, ExprRef),
    MathBinary(MathBinaryFunction, ExprRef, ExprRef),
    Select {
        guard: ExprRef,
        then_branch: ExprRef,
        else_branch: ExprRef,
    },
    Call {
        function: FunctionId,
        arguments: ExprList,
    },
    /// `R`
    SampleUnit,
    /// `R[min,max]`
    SampleRange {
        min: ExprRef,
        max: ExprRef,
    },
    /// `N[mean,variance]`
    SampleNormal {
        mean: ExprRef,
        variance: ExprRef,
    },
    /// `U[..]`
    SampleChoice(ExprList),
}

// ---------------------------------------------------------------------------
// Statements (function bodies)
// ---------------------------------------------------------------------------

/// A function body statement. `Let` is *just* `{ slot, value, body }` — no
/// scope chain, since `slot` is already resolved by lowering.
#[derive(Clone, Copy, Debug)]
pub enum StmtNode {
    Return(ExprRef),
    IfThenElse {
        guard: ExprRef,
        then_branch: StmtRef,
        else_branch: Option<StmtRef>,
    },
    Let {
        slot: SlotId,
        value: ExprRef,
        body: StmtRef,
    },
}

// ---------------------------------------------------------------------------
// Slots, globals, variables, functions, penalties
// ---------------------------------------------------------------------------

/// What kind of thing a [SlotId] was allocated for — for debugging /
/// pretty-printing only, the evaluator's flat store doesn't need it at
/// runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotKind {
    /// `[0, n_variables)`: the simulation state, read/write each step.
    Variable,
    /// `[n_variables, n_globals)`: a `const` or `param`, written once at
    /// startup.
    Global,
    /// `[n_globals, n_slots)`: a function argument or `let` binding.
    Local,
}

/// A slot's name, type and kind, kept around purely for debugging and
/// pretty-printing (`Display`, future diagnostics) — the evaluator indexes
/// the store directly and never needs this.
#[derive(Clone, Debug)]
pub struct SlotInfo {
    pub name: String,
    pub ty: StarkType,
    pub kind: SlotKind,
    pub span: Span,
}

/// A slot in the `[0, n_variables)` state prefix: its declared range bounds
/// (if any) and its initial value, both already lowered.
#[derive(Clone, Copy, Debug)]
pub struct VariableInfo {
    pub slot: SlotId,
    pub range: Option<(ExprRef, ExprRef)>,
    pub initial_value: ExprRef,
}

/// A `const`/`param` initializer: `store[slot] = eval(value)`, executed once
/// at startup, in declaration order.
#[derive(Clone, Copy, Debug)]
pub struct GlobalInit {
    pub slot: SlotId,
    pub value: ExprRef,
}

/// A lowered `function name(args) { body }`.
#[derive(Clone, Debug)]
pub struct FunctionIr {
    pub name: String,
    /// One slot per declared argument, positional.
    pub arguments: Vec<SlotId>,
    pub return_type: StarkType,
    pub body: StmtRef,
}

/// A lowered `penalty name = expr;`.
#[derive(Clone, Copy, Debug)]
pub struct PenaltyIr {
    pub value: ExprRef,
}

// ---------------------------------------------------------------------------
// Controllers and the environment
// ---------------------------------------------------------------------------

/// A buffered `[when guard] target' = value;`, shared by controller and
/// environment lowering. Mirrors `ast::Update`, but with the target variable
/// resolved to its [SlotId]. **Buffered, not applied immediately**: every
/// read in the same step sees the pre-update value (see [CommandNode]'s doc
/// comment) — the evaluator is responsible for collecting these and
/// applying them only once the whole step has run.
#[derive(Clone, Copy, Debug)]
pub struct Update {
    pub target: SlotId,
    pub guard: Option<ExprRef>,
    pub value: ExprRef,
}

/// One node of the command arena: a controller state's body or the
/// environment block, both lowered to the same node type since the only
/// difference between them is that an environment never contains a `Step`/
/// `Exec` (see `IR_LOWERING_PLAN.md`'s Step 4).
///
/// A `Vec<ast::ControllerCommand>`/`Vec<ast::EnvironmentCommand>` — a
/// `{ .. }` block — lowers to a left-associated chain of `Sequence(prior,
/// next)` nodes, one per list element; an empty block lowers to no node at
/// all (`None` at the call site), since there is nothing to run.
///
/// **Buffered update semantics**: `Assign` does not write through to
/// `store[slot]` when evaluated — it is the evaluator's job to collect every
/// `Assign` reached during a step into a list and apply them all at the end,
/// so `x' = y; y' = x;` reads *both* sides from the pre-step state (the
/// classic swap). Lowering only has to preserve the structure faithfully;
/// see Step 4's `IR_LOWERING_PLAN.md` note and the `buffered_swap_*` tests.
///
/// **Where control-flow termination lives**: this arena does not itself
/// enforce that every path through a controller state reaches a `Step`/
/// `Exec` — it just mirrors the source's structure. A `Sequence(a, b)` whose
/// `a` is (or contains) a `Step`/`Exec` has an unreachable `b`; that is an
/// evaluator concern (stop walking the chain once a transition is hit), not
/// a lowering one.
#[derive(Clone, Copy, Debug)]
pub enum CommandNode {
    Assign(Update),
    IfThenElse {
        guard: ExprRef,
        then_branch: Option<CommandRef>,
        else_branch: Option<CommandRef>,
    },
    /// `let slot = value in body` — no scope chain, `slot` is already
    /// resolved, same as `StmtNode::Let`.
    Let {
        slot: SlotId,
        value: ExprRef,
        body: Option<CommandRef>,
    },
    /// Runs its left node, then its right node.
    Sequence(CommandRef, CommandRef),
    /// `[steps #] step target;` — controller-only. `steps` (if present) is
    /// evaluated once per step, matching Java's `Controller.doTick(k-1, ..)`.
    Step { steps: Option<ExprRef>, target: IrStateId },
    /// `exec target;` — controller-only.
    Exec(IrStateId),
}

/// A lowered `state name { .. }`.
#[derive(Clone, Debug)]
pub struct StateIr {
    pub name: String,
    pub component: ComponentId,
    /// `None` only for a state with an empty body — legal to parse, though a
    /// state that never reaches a `step`/`exec` cannot make progress.
    pub body: Option<CommandRef>,
}

/// A lowered `component name { .. }`. States are held flat on [IrProgram]
/// (see [IrStateId]'s doc comment); this only lists which of them are this
/// component's.
#[derive(Clone, Debug)]
pub struct ComponentIr {
    pub name: String,
    pub states: Vec<IrStateId>,
    /// The `init` expression: the parallel composition of initial states.
    pub initial: Vec<IrStateId>,
}

// ---------------------------------------------------------------------------
// The program
// ---------------------------------------------------------------------------

/// The result of lowering: one flat arena, plus the tables that index into
/// it. See the module doc comment for what is (and isn't) populated yet.
#[derive(Clone, Debug, Default)]
pub struct IrProgram {
    pub(crate) exprs: Vec<ExprNode>,
    pub(crate) expr_spans: Vec<Span>,
    pub(crate) expr_types: Vec<StarkType>,
    pub(crate) expr_lists: Vec<ExprRef>,

    pub(crate) stmts: Vec<StmtNode>,
    pub(crate) commands: Vec<CommandNode>,

    pub(crate) slots: Vec<SlotInfo>,
    pub(crate) variables: Vec<VariableInfo>,
    pub(crate) globals: Vec<GlobalInit>,
    pub(crate) functions: Vec<FunctionIr>,
    pub(crate) penalties: Vec<PenaltyIr>,

    pub(crate) states: Vec<StateIr>,
    pub(crate) components: Vec<ComponentIr>,
    /// The environment block, if the specification has one. `None` if it is
    /// absent, or present but empty — both mean "nothing runs".
    pub(crate) environment: Option<CommandRef>,
}

impl IrProgram {
    pub fn expr(&self, id: ExprRef) -> &ExprNode {
        &self.exprs[id.value() as usize]
    }

    pub fn expr_span(&self, id: ExprRef) -> &Span {
        &self.expr_spans[id.value() as usize]
    }

    pub fn expr_type(&self, id: ExprRef) -> &StarkType {
        &self.expr_types[id.value() as usize]
    }

    pub fn expr_list(&self, list: ExprList) -> &[ExprRef] {
        let start = list.start as usize;
        &self.expr_lists[start..start + list.len as usize]
    }

    pub fn stmt(&self, id: StmtRef) -> &StmtNode {
        &self.stmts[id.value() as usize]
    }

    pub fn command(&self, id: CommandRef) -> &CommandNode {
        &self.commands[id.value() as usize]
    }

    pub fn state(&self, id: IrStateId) -> &StateIr {
        &self.states[id.value() as usize]
    }

    pub fn components(&self) -> &[ComponentIr] {
        &self.components
    }

    pub fn component(&self, id: ComponentId) -> &ComponentIr {
        &self.components[id.value() as usize]
    }

    pub fn environment(&self) -> Option<CommandRef> {
        self.environment
    }

    pub fn slot(&self, id: SlotId) -> &SlotInfo {
        &self.slots[id.value() as usize]
    }

    pub fn variables(&self) -> &[VariableInfo] {
        &self.variables
    }

    pub fn globals(&self) -> &[GlobalInit] {
        &self.globals
    }

    pub fn functions(&self) -> &[FunctionIr] {
        &self.functions
    }

    pub fn function(&self, id: FunctionId) -> &FunctionIr {
        &self.functions[id.value() as usize]
    }

    pub fn penalties(&self) -> &[PenaltyIr] {
        &self.penalties
    }

    pub fn penalty(&self, id: PenaltyId) -> &PenaltyIr {
        &self.penalties[id.value() as usize]
    }

    /// Independently re-checks the arena's internal consistency: every
    /// `ExprRef`/`StmtRef`/`CommandRef`/`SlotId`/`FunctionId`/`IrStateId`
    /// reachable from a top-level entry (globals, variables, functions,
    /// penalties, components, the environment) is in bounds, and every list
    /// slice lies within `expr_lists`. This is a partial version of the full
    /// check `IR_LOWERING_PLAN.md`'s Step 7 describes — it does not yet
    /// cover perturbations/distances/formulas, since those aren't lowered
    /// yet.
    pub fn validate(&self) -> Result<(), String> {
        let check_expr = |id: ExprRef| -> Result<(), String> {
            if (id.value() as usize) < self.exprs.len() {
                Ok(())
            } else {
                Err(format!(
                    "{id:?} out of bounds for an arena of {} expression(s)",
                    self.exprs.len()
                ))
            }
        };
        let check_slot = |id: SlotId| -> Result<(), String> {
            if (id.value() as usize) < self.slots.len() {
                Ok(())
            } else {
                Err(format!("{id:?} out of bounds for {} slot(s)", self.slots.len()))
            }
        };
        let check_stmt = |id: StmtRef| -> Result<(), String> {
            if (id.value() as usize) < self.stmts.len() {
                Ok(())
            } else {
                Err(format!("{id:?} out of bounds for {} statement(s)", self.stmts.len()))
            }
        };
        let check_command = |id: CommandRef| -> Result<(), String> {
            if (id.value() as usize) < self.commands.len() {
                Ok(())
            } else {
                Err(format!("{id:?} out of bounds for {} command(s)", self.commands.len()))
            }
        };
        let check_state = |id: IrStateId| -> Result<(), String> {
            if (id.value() as usize) < self.states.len() {
                Ok(())
            } else {
                Err(format!("{id:?} out of bounds for {} state(s)", self.states.len()))
            }
        };

        if self.exprs.len() != self.expr_spans.len() || self.exprs.len() != self.expr_types.len() {
            return Err(format!(
                "arena length mismatch: {} expr(s), {} span(s), {} type(s)",
                self.exprs.len(),
                self.expr_spans.len(),
                self.expr_types.len()
            ));
        }

        for (index, node) in self.exprs.iter().enumerate() {
            match *node {
                ExprNode::Literal(_) | ExprNode::SampleUnit => {}
                ExprNode::Load(slot) => check_slot(slot)?,
                ExprNode::Not(inner) | ExprNode::Negate(inner) | ExprNode::MathUnary(_, inner) => check_expr(inner)?,
                ExprNode::Binary(_, left, right) | ExprNode::MathBinary(_, left, right) => {
                    check_expr(left)?;
                    check_expr(right)?;
                }
                ExprNode::Select {
                    guard,
                    then_branch,
                    else_branch,
                } => {
                    check_expr(guard)?;
                    check_expr(then_branch)?;
                    check_expr(else_branch)?;
                }
                ExprNode::Call { function, arguments } => {
                    if (function.value() as usize) >= self.functions.len() {
                        return Err(format!(
                            "{function:?} out of bounds for {} function(s)",
                            self.functions.len()
                        ));
                    }
                    for &argument in self.expr_list_bounds_checked(arguments, index)? {
                        check_expr(argument)?;
                    }
                }
                ExprNode::SampleRange { min, max } => {
                    check_expr(min)?;
                    check_expr(max)?;
                }
                ExprNode::SampleNormal { mean, variance } => {
                    check_expr(mean)?;
                    check_expr(variance)?;
                }
                ExprNode::SampleChoice(list) => {
                    for &element in self.expr_list_bounds_checked(list, index)? {
                        check_expr(element)?;
                    }
                }
            }
        }

        for node in &self.stmts {
            match *node {
                StmtNode::Return(value) => check_expr(value)?,
                StmtNode::IfThenElse {
                    guard,
                    then_branch,
                    else_branch,
                } => {
                    check_expr(guard)?;
                    check_stmt(then_branch)?;
                    if let Some(else_branch) = else_branch {
                        check_stmt(else_branch)?;
                    }
                }
                StmtNode::Let { slot, value, body } => {
                    check_slot(slot)?;
                    check_expr(value)?;
                    check_stmt(body)?;
                }
            }
        }

        for variable in &self.variables {
            check_slot(variable.slot)?;
            check_expr(variable.initial_value)?;
            if let Some((min, max)) = variable.range {
                check_expr(min)?;
                check_expr(max)?;
            }
        }
        for global in &self.globals {
            check_slot(global.slot)?;
            check_expr(global.value)?;
        }
        for function in &self.functions {
            for &argument in &function.arguments {
                check_slot(argument)?;
            }
            check_stmt(function.body)?;
        }
        for penalty in &self.penalties {
            check_expr(penalty.value)?;
        }

        for node in &self.commands {
            match *node {
                CommandNode::Assign(update) => {
                    check_slot(update.target)?;
                    if let Some(guard) = update.guard {
                        check_expr(guard)?;
                    }
                    check_expr(update.value)?;
                }
                CommandNode::IfThenElse {
                    guard,
                    then_branch,
                    else_branch,
                } => {
                    check_expr(guard)?;
                    if let Some(then_branch) = then_branch {
                        check_command(then_branch)?;
                    }
                    if let Some(else_branch) = else_branch {
                        check_command(else_branch)?;
                    }
                }
                CommandNode::Let { slot, value, body } => {
                    check_slot(slot)?;
                    check_expr(value)?;
                    if let Some(body) = body {
                        check_command(body)?;
                    }
                }
                CommandNode::Sequence(left, right) => {
                    check_command(left)?;
                    check_command(right)?;
                }
                CommandNode::Step { steps, target } => {
                    if let Some(steps) = steps {
                        check_expr(steps)?;
                    }
                    check_state(target)?;
                }
                CommandNode::Exec(target) => check_state(target)?,
            }
        }

        for state in &self.states {
            if let Some(body) = state.body {
                check_command(body)?;
            }
            if (state.component.value() as usize) >= self.components.len() {
                return Err(format!(
                    "{:?} out of bounds for {} component(s)",
                    state.component,
                    self.components.len()
                ));
            }
        }
        for component in &self.components {
            for &state in &component.states {
                check_state(state)?;
            }
            for &state in &component.initial {
                check_state(state)?;
            }
        }
        if let Some(environment) = self.environment {
            check_command(environment)?;
        }

        Ok(())
    }

    fn expr_list_bounds_checked(&self, list: ExprList, expr_index: usize) -> Result<&[ExprRef], String> {
        let start = list.start as usize;
        let end = start + list.len as usize;
        if end > self.expr_lists.len() {
            return Err(format!(
                "expression {expr_index}'s argument list [{start}, {end}) is out of bounds for {} list slot(s)",
                self.expr_lists.len()
            ));
        }
        Ok(&self.expr_lists[start..end])
    }
}

impl fmt::Display for IrProgram {
    /// Walks the arena and prints resolved, indented, source-like text with
    /// slot names substituted in — this is what's used to inspect lowering
    /// output (and what the snapshot tests assert on), since the raw
    /// `#[derive(Debug)]` form (`Binary(Add, ExprRef(3), ExprRef(7))`) is
    /// unreadable.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for global in &self.globals {
            let slot = self.slot(global.slot);
            writeln!(
                f,
                "#{}:{} {} = {};",
                global.slot.value(),
                slot.ty,
                slot.name,
                self.display_expr(global.value)
            )?;
        }
        if !self.globals.is_empty() {
            writeln!(f)?;
        }

        for variable in &self.variables {
            let slot = self.slot(variable.slot);
            write!(f, "variable #{}:{} {}", variable.slot.value(), slot.ty, slot.name)?;
            if let Some((min, max)) = variable.range {
                write!(f, " range [{}, {}]", self.display_expr(min), self.display_expr(max))?;
            }
            writeln!(f, " = {};", self.display_expr(variable.initial_value))?;
        }
        if !self.variables.is_empty() {
            writeln!(f)?;
        }

        for (index, function) in self.functions.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            let arguments = function
                .arguments
                .iter()
                .map(|&slot| {
                    let info = self.slot(slot);
                    format!("#{}:{} {}", slot.value(), info.ty, info.name)
                })
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(f, "fn {}({arguments}) -> {} {{", function.name, function.return_type)?;
            self.display_stmt(f, function.body, 1)?;
            writeln!(f, "}}")?;
        }
        if !self.functions.is_empty() {
            writeln!(f)?;
        }

        for (index, component) in self.components.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            writeln!(f, "component {} {{", component.name)?;
            for &state in &component.states {
                let state = self.state(state);
                writeln!(f, "  state {} {{", state.name)?;
                if let Some(body) = state.body {
                    self.display_command(f, body, 2)?;
                }
                writeln!(f, "  }}")?;
            }
            let initial = component
                .initial
                .iter()
                .map(|&id| self.state(id).name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(f, "  init {initial}")?;
            writeln!(f, "}}")?;
        }
        if !self.components.is_empty() {
            writeln!(f)?;
        }

        if let Some(environment) = self.environment {
            writeln!(f, "environment {{")?;
            self.display_command(f, environment, 1)?;
            writeln!(f, "}}")?;
        }

        Ok(())
    }
}

impl IrProgram {
    fn display_command(&self, f: &mut fmt::Formatter<'_>, id: CommandRef, indent: usize) -> fmt::Result {
        let pad = "  ".repeat(indent);
        match *self.command(id) {
            CommandNode::Assign(update) => {
                let target = self.slot(update.target);
                if let Some(guard) = update.guard {
                    write!(f, "{pad}when {} ", self.display_expr(guard))?;
                } else {
                    write!(f, "{pad}")?;
                }
                writeln!(f, "{}' = {};", target.name, self.display_expr(update.value))
            }
            CommandNode::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                writeln!(f, "{pad}if {} {{", self.display_expr(guard))?;
                if let Some(then_branch) = then_branch {
                    self.display_command(f, then_branch, indent + 1)?;
                }
                if let Some(else_branch) = else_branch {
                    writeln!(f, "{pad}}} else {{")?;
                    self.display_command(f, else_branch, indent + 1)?;
                }
                writeln!(f, "{pad}}}")
            }
            CommandNode::Let { slot, value, body } => {
                let info = self.slot(slot);
                writeln!(
                    f,
                    "{pad}let {} #{} = {};",
                    info.name,
                    slot.value(),
                    self.display_expr(value)
                )?;
                if let Some(body) = body {
                    self.display_command(f, body, indent)?;
                }
                Ok(())
            }
            CommandNode::Sequence(left, right) => {
                self.display_command(f, left, indent)?;
                self.display_command(f, right, indent)
            }
            CommandNode::Step { steps, target } => {
                write!(f, "{pad}step {}", self.state(target).name)?;
                if let Some(steps) = steps {
                    write!(f, " x {}", self.display_expr(steps))?;
                }
                writeln!(f, ";")
            }
            CommandNode::Exec(target) => writeln!(f, "{pad}exec {};", self.state(target).name),
        }
    }

    fn display_stmt(&self, f: &mut fmt::Formatter<'_>, id: StmtRef, indent: usize) -> fmt::Result {
        let pad = "  ".repeat(indent);
        match *self.stmt(id) {
            StmtNode::Return(value) => writeln!(f, "{pad}return {}", self.display_expr(value)),
            StmtNode::IfThenElse {
                guard,
                then_branch,
                else_branch,
            } => {
                writeln!(f, "{pad}if {} {{", self.display_expr(guard))?;
                self.display_stmt(f, then_branch, indent + 1)?;
                if let Some(else_branch) = else_branch {
                    writeln!(f, "{pad}}} else {{")?;
                    self.display_stmt(f, else_branch, indent + 1)?;
                }
                writeln!(f, "{pad}}}")
            }
            StmtNode::Let { slot, value, body } => {
                let info = self.slot(slot);
                writeln!(
                    f,
                    "{pad}let {} #{} = {};",
                    info.name,
                    slot.value(),
                    self.display_expr(value)
                )?;
                self.display_stmt(f, body, indent)
            }
        }
    }

    /// Renders an expression as source-like text, substituting slot names.
    fn display_expr(&self, id: ExprRef) -> String {
        match *self.expr(id) {
            ExprNode::Literal(value) => value.to_string(),
            ExprNode::Load(slot) => format!("load #{}:{}", slot.value(), self.slot(slot).name),
            ExprNode::Not(inner) => format!("!{}", self.display_expr(inner)),
            ExprNode::Negate(inner) => format!("-{}", self.display_expr(inner)),
            ExprNode::Binary(op, left, right) => {
                format!(
                    "({} {} {})",
                    self.display_expr(left),
                    display_binary_op(op),
                    self.display_expr(right)
                )
            }
            ExprNode::MathUnary(function, inner) => {
                format!("{}({})", display_math_unary(function), self.display_expr(inner))
            }
            ExprNode::MathBinary(function, left, right) => format!(
                "{}({}, {})",
                display_math_binary(function),
                self.display_expr(left),
                self.display_expr(right)
            ),
            ExprNode::Select {
                guard,
                then_branch,
                else_branch,
            } => format!(
                "select({}, {}, {})",
                self.display_expr(guard),
                self.display_expr(then_branch),
                self.display_expr(else_branch)
            ),
            ExprNode::Call { function, arguments } => {
                let function = self.function(function);
                let arguments = self
                    .expr_list(arguments)
                    .iter()
                    .map(|&argument| self.display_expr(argument))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({arguments})", function.name)
            }
            ExprNode::SampleUnit => "R".to_string(),
            ExprNode::SampleRange { min, max } => {
                format!("R[{}, {}]", self.display_expr(min), self.display_expr(max))
            }
            ExprNode::SampleNormal { mean, variance } => {
                format!("N[{}, {}]", self.display_expr(mean), self.display_expr(variance))
            }
            ExprNode::SampleChoice(list) => {
                let elements = self
                    .expr_list(list)
                    .iter()
                    .map(|&element| self.display_expr(element))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("U[{elements}]")
            }
        }
    }
}

fn display_binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Mult => "*",
        BinaryOp::Div => "/",
        BinaryOp::IntDiv => "div",
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Mod => "%",
        BinaryOp::Less => "<",
        BinaryOp::Leq => "<=",
        BinaryOp::Eq => "==",
        BinaryOp::Geq => ">=",
        BinaryOp::Greater => ">",
        BinaryOp::BitAnd => "&",
        BinaryOp::And => "&&",
        BinaryOp::BitOr => "|",
        BinaryOp::Or => "||",
    }
}

fn display_math_unary(function: MathUnaryFunction) -> &'static str {
    match function {
        MathUnaryFunction::Abs => "abs",
        MathUnaryFunction::Acos => "acos",
        MathUnaryFunction::Asin => "asin",
        MathUnaryFunction::Atan => "atan",
        MathUnaryFunction::Cbrt => "cbrt",
        MathUnaryFunction::Ceil => "ceil",
        MathUnaryFunction::Cos => "cos",
        MathUnaryFunction::Cosh => "cosh",
        MathUnaryFunction::Exp => "exp",
        MathUnaryFunction::Expm1 => "expm1",
        MathUnaryFunction::Floor => "floor",
        MathUnaryFunction::Log => "log",
        MathUnaryFunction::Log10 => "log10",
        MathUnaryFunction::Log1p => "log1p",
        MathUnaryFunction::Signum => "signum",
        MathUnaryFunction::Sin => "sin",
        MathUnaryFunction::Sinh => "sinh",
        MathUnaryFunction::Sqrt => "sqrt",
        MathUnaryFunction::Tan => "tan",
    }
}

fn display_math_binary(function: MathBinaryFunction) -> &'static str {
    match function {
        MathBinaryFunction::Atan2 => "atan2",
        MathBinaryFunction::Hypot => "hypot",
        MathBinaryFunction::Max => "max",
        MathBinaryFunction::Min => "min",
        MathBinaryFunction::Pow => "pow",
    }
}
