//! Abstract syntax tree for the STARK specification language.
//!
//! This mirrors the structure of the original STARK ANTLR grammar
//! (`StarkSpecificationLanguage.g4`). The tree is produced by `consume.rs`
//! (structural declarations) together with the Pratt parsers in `precedence.rs`
//! (expressions and the perturbation / distance / ROBTL sub-languages).
//!
//! Declarations carry an `id: Option<DefId>` (or `Option<StateId>` for
//! controller states) that is `None` after parsing and filled in by name
//! resolution (`resolve.rs`). Every place a declared name is *referenced*
//! (rather than declared) uses [DefRef], [StateRef] or, inside expressions,
//! [Binding] — all `None`/absent until resolution runs.

pub use merc_utilities::Span;
pub use merc_utilities::Spanned;
use merc_utilities::TagIndex;

/// A unique tag for top-level declarations: constants, parameters, variables,
/// functions, penalties, components, custom types, perturbations, distances
/// and formulas all share this single namespace, mirroring the original
/// STARK `SymbolTable`'s single `symbols` map.
pub struct DefTag;
/// The index type assigned to a top-level declaration during name resolution.
pub type DefId = TagIndex<usize, DefTag>;

/// A unique tag for controller states, which are scoped to their component.
pub struct StateTag;
/// The index type assigned to a controller state during name resolution.
pub type StateId = TagIndex<usize, StateTag>;

/// A unique tag for local bindings: function arguments, `let` bindings, and
/// the `it` iterator parameter.
pub struct LocalTag;
/// The index type assigned to a local binding during name resolution.
pub type LocalId = TagIndex<usize, LocalTag>;

/// An expression node together with the source span it was parsed from.
///
/// Mirrors rustc's `Expr`/`ExprKind` split: [Expression] is the spanned node
/// that appears everywhere in the tree, and [ExpressionKind] is the bare
/// variant data. Sub-expressions recurse through `Box<Expression>` (not
/// `Box<ExpressionKind>`), so every level of nesting carries its own span.
pub type Expression = Spanned<ExpressionKind>;

/// A reference to a top-level declaration (variable, constant, parameter,
/// function, penalty, distance, perturbation, formula or component),
/// resolved to a [DefId] by name resolution. `id` is `None` until then.
#[derive(Clone, Debug)]
pub struct DefRef {
    pub id: Option<DefId>,
    pub name: Identifier,
}

impl DefRef {
    pub fn new(name: Identifier) -> Self {
        DefRef { id: None, name }
    }
}

/// A reference to a controller state (`step`/`exec` target, or a component's
/// `init` expression), resolved to a [StateId] within its enclosing
/// component. `id` is `None` until then.
#[derive(Clone, Debug)]
pub struct StateRef {
    pub id: Option<StateId>,
    pub name: Identifier,
}

impl StateRef {
    pub fn new(name: Identifier) -> Self {
        StateRef { id: None, name }
    }
}

/// What an expression-level name reference resolves to: either a top-level
/// declaration or a local binding (function argument, `let` binding, or the
/// `it` iterator parameter).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Binding {
    Def(DefId),
    Local(LocalId),
}

/// A complete parsed STARK specification: the ordered list of every top-level
/// declaration in the source.
#[derive(Clone, Debug, Default)]
pub struct UntypedStarkSpecification {
    pub constants: Vec<Constant>,
    pub parameters: Vec<Parameter>,
    pub variables: Vec<Variable>,
    pub types: Vec<TypeDeclaration>,
    pub functions: Vec<Function>,
    pub components: Vec<Component>,
    pub environment: Option<Environment>,
    pub penalties: Vec<Penalty>,
    pub perturbations: Vec<Perturbation>,
    pub distances: Vec<Distance>,
    pub formulas: Vec<Formula>,
}

impl UntypedStarkSpecification {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Top-level declarations
// ---------------------------------------------------------------------------

/// `const name = value;`
#[derive(Clone, Debug)]
pub struct Constant {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub value: Expression,
}

/// `param name = value;`
#[derive(Clone, Debug)]
pub struct Parameter {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub value: Expression,
}

/// A single variable in a (`global`) `variables { ... }` block, or in a
/// component's local `variables { ... }` block.
#[derive(Clone, Debug)]
pub struct Variable {
    pub id: Option<DefId>,
    pub global: bool,
    pub ty: Ty,
    pub name: Identifier,
    pub range: Option<Range>,
    pub initial_value: Expression,
}

/// `type name = A | B | C;`
#[derive(Clone, Debug)]
pub struct TypeDeclaration {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub elements: Vec<Identifier>,
}

/// `penalty name = expr`
#[derive(Clone, Debug)]
pub struct Penalty {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub value: Expression,
}

/// `function name(args) { body }`
#[derive(Clone, Debug)]
pub struct Function {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub arguments: Vec<FunctionArgument>,
    pub body: FunctionStatement,
}

#[derive(Clone, Debug)]
pub struct FunctionArgument {
    pub id: Option<LocalId>,
    pub ty: Ty,
    pub name: Identifier,
}

#[derive(Clone, Debug)]
pub enum FunctionStatement {
    Return(Expression),
    IfThenElse {
        guard: Expression,
        then_branch: Box<FunctionStatement>,
        else_branch: Option<Box<FunctionStatement>>,
    },
    Let {
        id: Option<LocalId>,
        name: Identifier,
        value: Expression,
        body: Box<FunctionStatement>,
    },
    Block(Box<FunctionStatement>),
}

// ---------------------------------------------------------------------------
// Components and controllers
// ---------------------------------------------------------------------------

/// `component name { variables { .. } controller { .. } init .. }`
#[derive(Clone, Debug)]
pub struct Component {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub variables: Vec<Variable>,
    pub states: Vec<ControllerState>,
    /// The `init` expression: the parallel composition of state references.
    pub init: Vec<StateRef>,
}

/// `state name { .. }`
#[derive(Clone, Debug)]
pub struct ControllerState {
    pub id: Option<StateId>,
    pub name: Identifier,
    pub body: Vec<ControllerCommand>,
}

#[derive(Clone, Debug)]
pub enum ControllerCommand {
    /// `[steps #] step target;`
    Step {
        steps: Option<Expression>,
        target: StateRef,
    },
    /// `exec target;`
    Exec(StateRef),
    /// `let id = value in body`
    Let {
        id: Option<LocalId>,
        name: Identifier,
        value: Expression,
        body: Vec<ControllerCommand>,
    },
    /// `[when guard] target' = value;`
    Assignment(Update),
    /// `if (guard) { .. } else { .. }`
    IfThenElse {
        guard: Expression,
        then_branch: Vec<ControllerCommand>,
        else_branch: Option<Vec<ControllerCommand>>,
    },
    /// A nested `{ .. }` block.
    Block(Vec<ControllerCommand>),
}

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

/// `environment { .. }`
#[derive(Clone, Debug)]
pub struct Environment {
    pub commands: Vec<EnvironmentCommand>,
}

#[derive(Clone, Debug)]
pub enum EnvironmentCommand {
    /// `[when guard] target' = value;`
    Assignment(Update),
    /// `if (guard) cmd [else cmd]`
    IfThenElse {
        guard: Expression,
        then_branch: Box<EnvironmentCommand>,
        else_branch: Option<Box<EnvironmentCommand>>,
    },
    /// `let a = e1 and b = e2 in cmd`
    Let {
        bindings: Vec<LocalVariable>,
        body: Box<EnvironmentCommand>,
    },
    /// A nested `{ .. }` block.
    Block(Vec<EnvironmentCommand>),
}

#[derive(Clone, Debug)]
pub struct LocalVariable {
    pub id: Option<LocalId>,
    pub name: Identifier,
    pub value: Expression,
}

/// A `[when guard] target' = value;` assignment shared by controllers and the
/// environment. `target` is the primed variable name (without the trailing
/// `'`), resolved to the [DefId] of the variable it updates.
#[derive(Clone, Debug)]
pub struct Update {
    pub guard: Option<Expression>,
    pub target: DefRef,
    pub value: Expression,
}

// ---------------------------------------------------------------------------
// Robustness sub-languages (perturbation / distance / ROBTL)
// ---------------------------------------------------------------------------

/// `perturbation name = expr;`
#[derive(Clone, Debug)]
pub struct Perturbation {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub value: PerturbationExpression,
}

#[derive(Clone, Debug)]
pub enum PerturbationExpression {
    Nil,
    Reference(DefRef),
    /// `[ v1 <- e1, v2 <- e2 ] @ time`
    Atomic {
        assignments: Vec<PerturbationAssignment>,
        time: Expression,
    },
    /// `left ; right`
    Sequence(Box<PerturbationExpression>, Box<PerturbationExpression>),
    /// `argument ^ iterations`
    Iteration {
        argument: Box<PerturbationExpression>,
        iterations: Expression,
    },
}

#[derive(Clone, Debug)]
pub struct PerturbationAssignment {
    pub target: DefRef,
    pub value: Expression,
}

/// `distance name = expr;`
#[derive(Clone, Debug)]
pub struct Distance {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub value: DistanceExpression,
}

#[derive(Clone, Debug)]
pub enum DistanceExpression {
    /// A reference to another named `distance` declaration.
    Reference(DefRef),
    /// `< penalty`
    AtomicLeft(DefRef),
    /// `> penalty`
    AtomicRight(DefRef),
    /// `\F[from,to] argument`
    Eventually {
        from: Expression,
        to: Expression,
        argument: Box<DistanceExpression>,
    },
    /// `\G[from,to] argument`
    Globally {
        from: Expression,
        to: Expression,
        argument: Box<DistanceExpression>,
    },
    /// `left \U[from,to] right`
    Until {
        from: Expression,
        to: Expression,
        left: Box<DistanceExpression>,
        right: Box<DistanceExpression>,
    },
    /// `left op threshold`
    Threshold {
        op: ComparisonOp,
        left: Box<DistanceExpression>,
        threshold: Expression,
    },
    Min(Box<DistanceExpression>, Box<DistanceExpression>),
    Max(Box<DistanceExpression>, Box<DistanceExpression>),
    /// `w1 * d1 + w2 * d2 + ...`
    LinearCombination(Vec<(Expression, DistanceExpression)>),
}

/// `formula name = formula;`
#[derive(Clone, Debug)]
pub struct Formula {
    pub id: Option<DefId>,
    pub name: Identifier,
    pub value: RobtlFormula,
}

#[derive(Clone, Debug)]
pub enum RobtlFormula {
    True,
    False,
    /// A reference to another named `formula` declaration.
    Reference(DefRef),
    /// `\D[distance, perturbation] op value`
    Distance {
        distance: DefRef,
        perturbation: DefRef,
        op: ComparisonOp,
        value: Expression,
    },
    Not(Box<RobtlFormula>),
    Globally {
        from: Expression,
        to: Expression,
        argument: Box<RobtlFormula>,
    },
    Eventually {
        from: Expression,
        to: Expression,
        argument: Box<RobtlFormula>,
    },
    And(Box<RobtlFormula>, Box<RobtlFormula>),
    Or(Box<RobtlFormula>, Box<RobtlFormula>),
    Until {
        from: Expression,
        to: Expression,
        left: Box<RobtlFormula>,
        right: Box<RobtlFormula>,
    },
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    // Literals
    False,
    True,
    Integer(i64),
    Real(f64),
    /// A name reference: a constant/parameter/variable, a local binding
    /// (function argument, `let` binding, `it`), or (before resolution)
    /// unresolved. `binding` is filled in by `resolve.rs`.
    Reference {
        name: String,
        binding: Option<Binding>,
    },
    /// The `it` lambda parameter used inside aggregate/perturbation contexts.
    Iterator,

    // Distributions / random values
    Normal {
        mean: Box<Expression>,
        std_dev: Box<Expression>,
    },
    Uniform {
        values: Vec<Expression>,
    },
    /// `R` or `R[min,max]`.
    Range {
        min: Option<Box<Expression>>,
        max: Option<Box<Expression>>,
    },

    // Prefix operators
    Not(Box<Expression>),
    UnaryPlus(Box<Expression>),
    UnaryMinus(Box<Expression>),

    // Binary operators
    Binary(BinaryOp, Box<Expression>, Box<Expression>),

    // `guard ? then : else`
    Ternary {
        guard: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },

    /// A user-defined function application `name(args)`.
    Call {
        function: DefRef,
        arguments: Vec<Expression>,
    },

    /// A built-in math function application, e.g. `abs(x)`, `max(a, b)`.
    MathCall {
        function: MathFunction,
        arguments: Vec<Expression>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Pow,
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

/// Comparison operators used as thresholds in distance expressions and ROBTL
/// formulas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonOp {
    Less,
    Leq,
    Eq,
    Geq,
    Greater,
}

/// Built-in mathematical functions (both unary and binary arities).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathFunction {
    // Unary
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
    // Binary
    Atan2,
    Hypot,
    Max,
    Min,
    Pow,
}

// ---------------------------------------------------------------------------
// Shared leaf types
// ---------------------------------------------------------------------------

/// A `range [min, max]` bound on a variable declaration.
#[derive(Clone, Debug)]
pub struct Range {
    pub min: Expression,
    pub max: Expression,
}

#[derive(Clone, Debug)]
pub enum Ty {
    Real,
    Integer,
    Boolean,
    /// A user-defined type referenced by name.
    Named(String),
}

/// An identifier together with its source location.
#[derive(Clone, Debug)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

impl Identifier {
    pub fn new(name: String, span: Span) -> Self {
        Identifier { name, span }
    }
}
