//! The evaluation IR that `lower.rs` produces: a flat arena of small, mostly
//! `Copy` nodes rather than a closure tree, so evaluation walks an array
//! instead of chasing pointers.
//!
//! Populated by lowering: constants/parameters (as [GlobalInit]), variables
//! (as [VariableInfo]), functions (as [FunctionIr]), penalties (as
//! [PenaltyIr]), components/controller states (as [ComponentIr]/[StateIr])
//! and the environment block, together with the shared expression/statement/
//! command arenas they're built from, plus perturbations, distances and
//! ROBTL formulas ([PerturbationIr]/
//! [DistanceIr]/[FormulaIr], each its own small `Box`-free arena analogous to
//! the expression one, with [PerturbationDecl]/[DistanceDecl]/[FormulaDecl]
//! marking which arena entries are named top-level declarations rather than
//! sub-nodes only reachable through one).
//!
//! # Index types
//!
//! Every arena is addressed by its own index type, all backed by `u32` rather
//! than `usize` so the nodes holding them stay small. Each carries its own tag
//! so that, say, an [ExprRef] can never be mixed up with a [SlotId] at a call
//! site even though both are "just a `u32`" underneath.
//!
//! # Robustness sub-languages
//!
//! [PerturbationIr], [DistanceIr] and [FormulaIr] mirror the shape of
//! `ast.rs`'s perturbation, distance and formula expressions, each collapsed
//! the same way the expression arena is: a `Reference(DefRef)` (a reference to
//! another named declaration of the same kind) resolves at lowering time to
//! the referent's `*Id`, so no name lookups survive into the IR. A top-level
//! `perturbation`/`distance`/`formula name = ..;` declaration lowers to one
//! *root* node, pushed last (post-order, same as expressions); its
//! `Sequence`/`Iteration`/`Eventually`/etc. operands are themselves `*Id`s
//! into the very same arena, so a declaration and everything it is built from
//! share one flat, `Box`-free index space. [PerturbationDecl]/[DistanceDecl]/
//! [FormulaDecl] separately record which arena entries are those named roots
//! (as opposed to intermediate sub-nodes only reachable *through* a root) —
//! the same distinction [IrProgram::variables] draws from [IrProgram::exprs].

use std::fmt;

use merc_utilities::Span;
use merc_utilities::TagIndex;

use crate::types::StarkType;
use crate::value::Value;

// ---------------------------------------------------------------------------
// Index types (see the module documentation)
// ---------------------------------------------------------------------------

pub struct ExprTag;
/// An index into [IrProgram]'s expression arena.
pub type ExprRef = TagIndex<u32, ExprTag>;

pub struct StmtTag;
/// An index into [IrProgram]'s statement arena (function bodies).
pub type StmtRef = TagIndex<u32, StmtTag>;

pub struct SlotTag;
/// An index into the flat value store the evaluator maintains — see
/// [IrProgram::n_variables] for the layout of that store.
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

pub struct PerturbationTag;
/// An index into [IrProgram]'s perturbation-expression arena
/// ([IrProgram::perturbations]). Both the root node of a top-level
/// `perturbation name = ..;` declaration (see [PerturbationDecl]) and every
/// sub-node reached from it (a `Sequence`'s operands, an `Iteration`'s
/// argument) share this one index space, mirroring [ExprRef].
pub type PerturbationId = TagIndex<u32, PerturbationTag>;

pub struct DistanceTag;
/// An index into [IrProgram]'s distance-expression arena
/// ([IrProgram::distances]) — same shape as [PerturbationId].
pub type DistanceId = TagIndex<u32, DistanceTag>;

pub struct FormulaTag;
/// An index into [IrProgram]'s ROBTL-formula arena ([IrProgram::formulas]) —
/// same shape as [PerturbationId].
pub type FormulaId = TagIndex<u32, FormulaTag>;

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
/// Deliberate simplifications made while lowering:
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
    /// An expression that cannot be evaluated, carrying a `&'static str`
    /// naming why. Lowering emits this only for AST shapes that no grammar
    /// production can currently produce (`ExpressionKind::Iterator`, which
    /// needs an aggregate/lambda context — see `plan.md`),
    /// so reaching one at run time means lowering has a bug; `eval` reports it
    /// as [crate::value::EvalError::Unreachable] rather than inventing a
    /// value. Before errors became a `Result`, this was a `Literal` holding
    /// the old absorbing `Value::Error`.
    Unreachable(&'static str),
    /// A read of `store[slot]` — the whole point of this IR: every name
    /// resolution already did gets baked into the node.
    Load(SlotId),
    Not(ExprRef),
    /// Arithmetic negation (`-x`). **Always widens to `Real`, even for an
    /// integer operand** — matching the original, which routes unary `-`/`+`
    /// through the *same* always-widening double-valued mechanism as the
    /// math functions rather than through a dedicated integer-preserving
    /// path. So `-a + 2` is `real`, not `int`, when `a` is an `int` —
    /// surprising for a spec author writing `-a` expecting an int to stay
    /// one; matched here for fidelity with the original tool, but worth
    /// reconsidering if it surprises users badly enough in practice.
    Negate(ExprRef),
    /// `+x`. Unlike most unary-plus operators this is *not* the identity at
    /// the type level: it widens exactly like `Negate`, through the same
    /// mechanism (see [ExprNode::Negate]'s doc comment), so `+a` for an
    /// integer `a` is `real`, not `a` unchanged.
    /// The *value* is unchanged; only the representation widens.
    Widen(ExprRef),
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

/// A lowered `penalty name = expr;`. Carries its `name` (unlike the rest of
/// this section, which didn't need one before [DistanceIr::AtomicLeft]/
/// [DistanceIr::AtomicRight] started referencing a penalty by [PenaltyId] —
/// printing `< #3` in [IrProgram]'s `Display` impl would otherwise be
/// unreadable).
#[derive(Clone, Debug)]
pub struct PenaltyIr {
    pub name: String,
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
/// `Exec`.
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
/// see the `buffered_swap_*` tests in `lower.rs`.
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
    /// evaluated once per step.
    Step {
        steps: Option<ExprRef>,
        target: IrStateId,
    },
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
// Robustness sub-languages: perturbation / distance / ROBTL formula
// (see the module documentation)
// ---------------------------------------------------------------------------

/// A comparison operator, used by [DistanceIr::Threshold] and
/// [FormulaIr::Distance]. Kept as its own type (mirroring `ast::ComparisonOp`)
/// so `ir.rs` doesn't need to depend on `ast`, the same reason [BinaryOp]
/// doesn't reuse `ast::BinaryOp` directly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonOp {
    Less,
    Leq,
    Eq,
    Geq,
    Greater,
}

/// An unguarded `target <- value` inside a perturbation's atomic block —
/// like [Update] but with no `guard` field, matching
/// `ast::PerturbationAssignment` (a perturbation assignment can never be
/// guarded; see `plan.md`).
#[derive(Clone, Copy, Debug)]
pub struct PerturbationAssignment {
    pub target: SlotId,
    pub value: ExprRef,
}

/// One node of [IrProgram]'s perturbation arena.
#[derive(Clone, Debug)]
pub enum PerturbationIr {
    /// The empty perturbation — leaves every trajectory unperturbed.
    Nil,
    /// A reference to another named `perturbation` declaration.
    Reference(PerturbationId),
    /// `[ v1 <- e1, v2 <- e2, .. ] @ time`.
    Atomic {
        assignments: Vec<PerturbationAssignment>,
        time: ExprRef,
    },
    /// `left ; right`.
    Sequence(PerturbationId, PerturbationId),
    /// `argument ^ iterations`.
    Iteration {
        argument: PerturbationId,
        iterations: ExprRef,
    },
}

/// A top-level `perturbation name = ..;` declaration: `name` plus the
/// [PerturbationId] of its root node in [IrProgram::perturbations].
#[derive(Clone, Debug)]
pub struct PerturbationDecl {
    pub name: String,
    pub root: PerturbationId,
}

/// One node of [IrProgram]'s distance arena — same shape as [PerturbationIr].
#[derive(Clone, Debug)]
pub enum DistanceIr {
    /// A reference to another named `distance` declaration.
    Reference(DistanceId),
    /// `< penalty`.
    AtomicLeft(PenaltyId),
    /// `> penalty`.
    AtomicRight(PenaltyId),
    /// `\F[from,to] argument`.
    Eventually {
        from: ExprRef,
        to: ExprRef,
        argument: DistanceId,
    },
    /// `\G[from,to] argument`.
    Globally {
        from: ExprRef,
        to: ExprRef,
        argument: DistanceId,
    },
    /// `left \U[from,to] right`.
    Until {
        from: ExprRef,
        to: ExprRef,
        left: DistanceId,
        right: DistanceId,
    },
    /// `left op threshold`.
    Threshold {
        op: ComparisonOp,
        left: DistanceId,
        threshold: ExprRef,
    },
    Min(DistanceId, DistanceId),
    Max(DistanceId, DistanceId),
    /// `w1 * d1 + w2 * d2 + ...`.
    LinearCombination(Vec<(ExprRef, DistanceId)>),
}

/// A top-level `distance name = ..;` declaration: `name` plus the
/// [DistanceId] of its root node in [IrProgram::distances].
#[derive(Clone, Debug)]
pub struct DistanceDecl {
    pub name: String,
    pub root: DistanceId,
}

/// One node of [IrProgram]'s ROBTL-formula arena — same shape as
/// [PerturbationIr].
#[derive(Clone, Debug)]
pub enum FormulaIr {
    True,
    False,
    /// A reference to another named `formula` declaration.
    Reference(FormulaId),
    /// `\D[distance, perturbation] op value`.
    Distance {
        distance: DistanceId,
        perturbation: PerturbationId,
        op: ComparisonOp,
        value: ExprRef,
    },
    Not(FormulaId),
    /// `\G[from,to] argument`.
    Globally {
        from: ExprRef,
        to: ExprRef,
        argument: FormulaId,
    },
    /// `\F[from,to] argument`.
    Eventually {
        from: ExprRef,
        to: ExprRef,
        argument: FormulaId,
    },
    And(FormulaId, FormulaId),
    Or(FormulaId, FormulaId),
    /// `left \U[from,to] right`.
    Until {
        from: ExprRef,
        to: ExprRef,
        left: FormulaId,
        right: FormulaId,
    },
}

/// A top-level `formula name = ..;` declaration: `name` plus the [FormulaId]
/// of its root node in [IrProgram::formulas].
#[derive(Clone, Debug)]
pub struct FormulaDecl {
    pub name: String,
    pub root: FormulaId,
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

    pub(crate) perturbations: Vec<PerturbationIr>,
    pub(crate) perturbation_decls: Vec<PerturbationDecl>,
    pub(crate) distances: Vec<DistanceIr>,
    pub(crate) distance_decls: Vec<DistanceDecl>,
    pub(crate) formulas: Vec<FormulaIr>,
    pub(crate) formula_decls: Vec<FormulaDecl>,
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

    /// The number of `[0, n_variables)` slots — the simulation state prefix.
    /// Equal to `self.variables.len()`, since every variable gets exactly one
    /// slot and slot allocation lays this range out first: variables occupy
    /// `[0, n_variables)`, `const`/`param` occupy
    /// `[n_variables, n_globals)`, and function arguments and `let` bindings
    /// occupy `[n_globals, n_slots)`.
    pub fn n_variables(&self) -> u32 {
        self.variables.len() as u32
    }

    /// The number of `[0, n_globals)` slots — variables plus `const`/`param`
    /// globals. Equal to `n_variables() + self.globals.len()`.
    pub fn n_globals(&self) -> u32 {
        self.n_variables() + self.globals.len() as u32
    }

    /// The total number of slots the evaluator's store must hold.
    pub fn n_slots(&self) -> u32 {
        self.slots.len() as u32
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

    /// The raw perturbation-expression arena: both the named roots (see
    /// [Self::perturbation_decls]) and every sub-node reachable from them.
    pub fn perturbations(&self) -> &[PerturbationIr] {
        &self.perturbations
    }

    pub fn perturbation(&self, id: PerturbationId) -> &PerturbationIr {
        &self.perturbations[id.value() as usize]
    }

    /// The top-level `perturbation name = ..;` declarations, in source order.
    pub fn perturbation_decls(&self) -> &[PerturbationDecl] {
        &self.perturbation_decls
    }

    /// The raw distance-expression arena — see [Self::perturbations].
    pub fn distances(&self) -> &[DistanceIr] {
        &self.distances
    }

    pub fn distance(&self, id: DistanceId) -> &DistanceIr {
        &self.distances[id.value() as usize]
    }

    /// The top-level `distance name = ..;` declarations, in source order.
    pub fn distance_decls(&self) -> &[DistanceDecl] {
        &self.distance_decls
    }

    /// The raw ROBTL-formula arena — see [Self::perturbations].
    pub fn formulas(&self) -> &[FormulaIr] {
        &self.formulas
    }

    pub fn formula(&self, id: FormulaId) -> &FormulaIr {
        &self.formulas[id.value() as usize]
    }

    /// The top-level `formula name = ..;` declarations, in source order.
    pub fn formula_decls(&self) -> &[FormulaDecl] {
        &self.formula_decls
    }

    /// Independently re-checks the arena's internal consistency: every
    /// `ExprRef`/`StmtRef`/`CommandRef`/`SlotId`/`FunctionId`/`IrStateId`/
    /// `PenaltyId`/`PerturbationId`/`DistanceId`/`FormulaId` reachable from a
    /// top-level entry (globals, variables, functions, penalties, components,
    /// the environment, perturbation/distance/formula declarations) is in
    /// bounds, and every list slice lies within `expr_lists`.
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
        let check_penalty = |id: PenaltyId| -> Result<(), String> {
            if (id.value() as usize) < self.penalties.len() {
                Ok(())
            } else {
                Err(format!(
                    "{id:?} out of bounds for {} penalty/-ies",
                    self.penalties.len()
                ))
            }
        };
        let check_perturbation = |id: PerturbationId| -> Result<(), String> {
            if (id.value() as usize) < self.perturbations.len() {
                Ok(())
            } else {
                Err(format!(
                    "{id:?} out of bounds for {} perturbation node(s)",
                    self.perturbations.len()
                ))
            }
        };
        let check_distance = |id: DistanceId| -> Result<(), String> {
            if (id.value() as usize) < self.distances.len() {
                Ok(())
            } else {
                Err(format!(
                    "{id:?} out of bounds for {} distance node(s)",
                    self.distances.len()
                ))
            }
        };
        let check_formula = |id: FormulaId| -> Result<(), String> {
            if (id.value() as usize) < self.formulas.len() {
                Ok(())
            } else {
                Err(format!(
                    "{id:?} out of bounds for {} formula node(s)",
                    self.formulas.len()
                ))
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
                ExprNode::Literal(_) | ExprNode::Unreachable(_) | ExprNode::SampleUnit => {}
                ExprNode::Load(slot) => check_slot(slot)?,
                ExprNode::Not(inner)
                | ExprNode::Negate(inner)
                | ExprNode::Widen(inner)
                | ExprNode::MathUnary(_, inner) => check_expr(inner)?,
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

        // The slot partition itself: `[0, n_variables)` variables, then
        // `[n_variables, n_globals)` globals, then locals. `n_variables()` /
        // `n_globals()` derive these boundaries from `variables.len()` /
        // `globals.len()` rather than by scanning `slots`, and the evaluator
        // takes the state prefix as a contiguous slice on that basis — so the
        // layout has to actually hold, not merely be intended.
        let n_variables = self.n_variables() as usize;
        let n_globals = self.n_globals() as usize;
        if n_globals > self.slots.len() {
            return Err(format!(
                "slot partition overflows: {n_variables} variable(s) + {} global(s) exceeds {} slot(s)",
                self.globals.len(),
                self.slots.len()
            ));
        }
        for (index, slot) in self.slots.iter().enumerate() {
            let expected = if index < n_variables {
                SlotKind::Variable
            } else if index < n_globals {
                SlotKind::Global
            } else {
                SlotKind::Local
            };
            if slot.kind != expected {
                return Err(format!(
                    "slot #{index} (`{}`) is {:?}, but the partition \
                     ([0,{n_variables}) variables, [{n_variables},{n_globals}) globals) requires {expected:?}",
                    slot.name, slot.kind
                ));
            }
        }

        for variable in &self.variables {
            check_slot(variable.slot)?;
            if (variable.slot.value() as usize) >= n_variables {
                return Err(format!(
                    "{:?} (`{}`) is a variable but lies outside the [0,{n_variables}) state prefix",
                    variable.slot,
                    self.slot(variable.slot).name
                ));
            }
            check_expr(variable.initial_value)?;
            if let Some((min, max)) = variable.range {
                check_expr(min)?;
                check_expr(max)?;
            }
        }
        for global in &self.globals {
            check_slot(global.slot)?;
            let slot = global.slot.value() as usize;
            if slot < n_variables || slot >= n_globals {
                return Err(format!(
                    "{:?} (`{}`) is a global but lies outside [{n_variables},{n_globals})",
                    global.slot,
                    self.slot(global.slot).name
                ));
            }
            check_expr(global.value)?;
        }
        for function in &self.functions {
            for &argument in &function.arguments {
                check_slot(argument)?;
                if (argument.value() as usize) < n_globals {
                    return Err(format!(
                        "{argument:?} (`{}`) is a function argument but lies inside the \
                         [0,{n_globals}) variable/global range",
                        self.slot(argument).name
                    ));
                }
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

        for node in &self.perturbations {
            match node {
                PerturbationIr::Nil => {}
                PerturbationIr::Reference(target) => check_perturbation(*target)?,
                PerturbationIr::Atomic { assignments, time } => {
                    for assignment in assignments {
                        check_slot(assignment.target)?;
                        check_expr(assignment.value)?;
                    }
                    check_expr(*time)?;
                }
                PerturbationIr::Sequence(left, right) => {
                    check_perturbation(*left)?;
                    check_perturbation(*right)?;
                }
                PerturbationIr::Iteration { argument, iterations } => {
                    check_perturbation(*argument)?;
                    check_expr(*iterations)?;
                }
            }
        }
        for decl in &self.perturbation_decls {
            check_perturbation(decl.root)?;
        }

        for node in &self.distances {
            match node {
                DistanceIr::Reference(target) => check_distance(*target)?,
                DistanceIr::AtomicLeft(penalty) | DistanceIr::AtomicRight(penalty) => check_penalty(*penalty)?,
                DistanceIr::Eventually { from, to, argument } | DistanceIr::Globally { from, to, argument } => {
                    check_expr(*from)?;
                    check_expr(*to)?;
                    check_distance(*argument)?;
                }
                DistanceIr::Until { from, to, left, right } => {
                    check_expr(*from)?;
                    check_expr(*to)?;
                    check_distance(*left)?;
                    check_distance(*right)?;
                }
                DistanceIr::Threshold { left, threshold, .. } => {
                    check_distance(*left)?;
                    check_expr(*threshold)?;
                }
                DistanceIr::Min(left, right) | DistanceIr::Max(left, right) => {
                    check_distance(*left)?;
                    check_distance(*right)?;
                }
                DistanceIr::LinearCombination(terms) => {
                    for &(weight, distance) in terms {
                        check_expr(weight)?;
                        check_distance(distance)?;
                    }
                }
            }
        }
        for decl in &self.distance_decls {
            check_distance(decl.root)?;
        }

        for node in &self.formulas {
            match node {
                FormulaIr::True | FormulaIr::False => {}
                FormulaIr::Reference(target) => check_formula(*target)?,
                FormulaIr::Distance {
                    distance,
                    perturbation,
                    value,
                    ..
                } => {
                    check_distance(*distance)?;
                    check_perturbation(*perturbation)?;
                    check_expr(*value)?;
                }
                FormulaIr::Not(inner) => check_formula(*inner)?,
                FormulaIr::Globally { from, to, argument } | FormulaIr::Eventually { from, to, argument } => {
                    check_expr(*from)?;
                    check_expr(*to)?;
                    check_formula(*argument)?;
                }
                FormulaIr::And(left, right) | FormulaIr::Or(left, right) => {
                    check_formula(*left)?;
                    check_formula(*right)?;
                }
                FormulaIr::Until { from, to, left, right } => {
                    check_expr(*from)?;
                    check_expr(*to)?;
                    check_formula(*left)?;
                    check_formula(*right)?;
                }
            }
        }
        for decl in &self.formula_decls {
            check_formula(decl.root)?;
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
            writeln!(f)?;
        }

        for penalty in &self.penalties {
            writeln!(f, "penalty {} = {};", penalty.name, self.display_expr(penalty.value))?;
        }
        if !self.penalties.is_empty() {
            writeln!(f)?;
        }

        for decl in &self.perturbation_decls {
            writeln!(
                f,
                "perturbation {} = {};",
                decl.name,
                self.display_perturbation(decl.root)
            )?;
        }
        if !self.perturbation_decls.is_empty() {
            writeln!(f)?;
        }

        for decl in &self.distance_decls {
            writeln!(f, "distance {} = {};", decl.name, self.display_distance(decl.root))?;
        }
        if !self.distance_decls.is_empty() {
            writeln!(f)?;
        }

        for decl in &self.formula_decls {
            writeln!(f, "formula {} = {};", decl.name, self.display_formula(decl.root))?;
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
            ExprNode::Unreachable(what) => format!("<unreachable: {what}>"),
            ExprNode::Load(slot) => format!("load #{}:{}", slot.value(), self.slot(slot).name),
            ExprNode::Not(inner) => format!("!{}", self.display_expr(inner)),
            ExprNode::Negate(inner) => format!("-{}", self.display_expr(inner)),
            ExprNode::Widen(inner) => format!("+{}", self.display_expr(inner)),
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

    /// The name of the top-level `perturbation name = ..;` declaration whose
    /// root is `id` — a linear search over [Self::perturbation_decls], fine
    /// for `Display` (debug/test use only, never a hot path). Every
    /// `PerturbationIr::Reference` is built from a resolved `DefRef` at
    /// lowering time (see `lower.rs`), so it always names a real root; the
    /// fallback only matters if the arena were hand-corrupted, as
    /// `validate_rejects_a_corrupted_arena`-style tests do.
    fn perturbation_decl_name(&self, id: PerturbationId) -> &str {
        self.perturbation_decls
            .iter()
            .find(|decl| decl.root == id)
            .map(|decl| decl.name.as_str())
            .unwrap_or("<perturbation>")
    }

    fn distance_decl_name(&self, id: DistanceId) -> &str {
        self.distance_decls
            .iter()
            .find(|decl| decl.root == id)
            .map(|decl| decl.name.as_str())
            .unwrap_or("<distance>")
    }

    fn formula_decl_name(&self, id: FormulaId) -> &str {
        self.formula_decls
            .iter()
            .find(|decl| decl.root == id)
            .map(|decl| decl.name.as_str())
            .unwrap_or("<formula>")
    }

    fn display_perturbation(&self, id: PerturbationId) -> String {
        match self.perturbation(id) {
            PerturbationIr::Nil => "nil".to_string(),
            PerturbationIr::Reference(target) => self.perturbation_decl_name(*target).to_string(),
            PerturbationIr::Atomic { assignments, time } => {
                let assignments = assignments
                    .iter()
                    .map(|assignment| {
                        format!(
                            "{} <- {}",
                            self.slot(assignment.target).name,
                            self.display_expr(assignment.value)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{assignments}] @ {}", self.display_expr(*time))
            }
            PerturbationIr::Sequence(left, right) => {
                format!(
                    "{} ; {}",
                    self.display_perturbation(*left),
                    self.display_perturbation(*right)
                )
            }
            PerturbationIr::Iteration { argument, iterations } => {
                format!(
                    "({})^{}",
                    self.display_perturbation(*argument),
                    self.display_expr(*iterations)
                )
            }
        }
    }

    fn display_distance(&self, id: DistanceId) -> String {
        match self.distance(id) {
            DistanceIr::Reference(target) => self.distance_decl_name(*target).to_string(),
            DistanceIr::AtomicLeft(penalty) => format!("< {}", self.penalty(*penalty).name),
            DistanceIr::AtomicRight(penalty) => format!("> {}", self.penalty(*penalty).name),
            DistanceIr::Eventually { from, to, argument } => format!(
                "\\F[{}, {}] {}",
                self.display_expr(*from),
                self.display_expr(*to),
                self.display_distance(*argument)
            ),
            DistanceIr::Globally { from, to, argument } => format!(
                "\\G[{}, {}] {}",
                self.display_expr(*from),
                self.display_expr(*to),
                self.display_distance(*argument)
            ),
            DistanceIr::Until { from, to, left, right } => format!(
                "{} \\U[{}, {}] {}",
                self.display_distance(*left),
                self.display_expr(*from),
                self.display_expr(*to),
                self.display_distance(*right)
            ),
            DistanceIr::Threshold { op, left, threshold } => format!(
                "{} {} {}",
                self.display_distance(*left),
                display_comparison_op(*op),
                self.display_expr(*threshold)
            ),
            DistanceIr::Min(left, right) => format!(
                "min({}, {})",
                self.display_distance(*left),
                self.display_distance(*right)
            ),
            DistanceIr::Max(left, right) => format!(
                "max({}, {})",
                self.display_distance(*left),
                self.display_distance(*right)
            ),
            DistanceIr::LinearCombination(terms) => terms
                .iter()
                .map(|&(weight, distance)| {
                    format!("{} * {}", self.display_expr(weight), self.display_distance(distance))
                })
                .collect::<Vec<_>>()
                .join(" + "),
        }
    }

    fn display_formula(&self, id: FormulaId) -> String {
        match self.formula(id) {
            FormulaIr::True => "true".to_string(),
            FormulaIr::False => "false".to_string(),
            FormulaIr::Reference(target) => self.formula_decl_name(*target).to_string(),
            FormulaIr::Distance {
                distance,
                perturbation,
                op,
                value,
            } => format!(
                "\\D[{}, {}] {} {}",
                self.distance_decl_name(*distance),
                self.perturbation_decl_name(*perturbation),
                display_comparison_op(*op),
                self.display_expr(*value)
            ),
            FormulaIr::Not(inner) => format!("!{}", self.display_formula(*inner)),
            FormulaIr::Globally { from, to, argument } => format!(
                "\\G[{}, {}] {}",
                self.display_expr(*from),
                self.display_expr(*to),
                self.display_formula(*argument)
            ),
            FormulaIr::Eventually { from, to, argument } => format!(
                "\\F[{}, {}] {}",
                self.display_expr(*from),
                self.display_expr(*to),
                self.display_formula(*argument)
            ),
            FormulaIr::And(left, right) => {
                format!("({} && {})", self.display_formula(*left), self.display_formula(*right))
            }
            FormulaIr::Or(left, right) => {
                format!("({} || {})", self.display_formula(*left), self.display_formula(*right))
            }
            FormulaIr::Until { from, to, left, right } => format!(
                "{} \\U[{}, {}] {}",
                self.display_formula(*left),
                self.display_expr(*from),
                self.display_expr(*to),
                self.display_formula(*right)
            ),
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

fn display_comparison_op(op: ComparisonOp) -> &'static str {
    match op {
        ComparisonOp::Less => "<",
        ComparisonOp::Leq => "<=",
        ComparisonOp::Eq => "==",
        ComparisonOp::Geq => ">=",
        ComparisonOp::Greater => ">",
    }
}
