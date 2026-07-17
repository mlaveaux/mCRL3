//! Abstract syntax tree for the STARK specification language.
//!
//! This mirrors the structure of the original STARK ANTLR grammar
//! (`StarkSpecificationLanguage.g4`). The tree is produced by `consume.rs`
//! (structural declarations) together with the Pratt parsers in `precedence.rs`
//! (expressions and the perturbation / distance / ROBTL sub-languages).

/// A complete parsed STARK specification: the ordered list of every top-level
/// declaration in the source.
#[derive(Clone, Debug, Default)]
pub struct StarkSpecification {
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

impl StarkSpecification {
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
    pub id: Identifier,
    pub value: Expression,
}

/// `param name = value;`
#[derive(Clone, Debug)]
pub struct Parameter {
    pub id: Identifier,
    pub value: Expression,
}

/// A single variable in a (`global`) `variables { ... }` block, or in a
/// component's local `variables { ... }` block.
#[derive(Clone, Debug)]
pub struct Variable {
    pub global: bool,
    pub ty: Ty,
    pub id: Identifier,
    pub range: Option<Range>,
    pub initial_value: Expression,
}

/// `type name = A | B | C;`
#[derive(Clone, Debug)]
pub struct TypeDeclaration {
    pub id: Identifier,
    pub elements: Vec<Identifier>,
}

/// `penalty name = expr`
#[derive(Clone, Debug)]
pub struct Penalty {
    pub id: Identifier,
    pub value: Expression,
}

/// `function name(args) { body }`
#[derive(Clone, Debug)]
pub struct Function {
    pub id: Identifier,
    pub arguments: Vec<FunctionArgument>,
    pub body: FunctionStatement,
}

#[derive(Clone, Debug)]
pub struct FunctionArgument {
    pub ty: Ty,
    pub id: Identifier,
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
        id: Identifier,
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
    pub id: Identifier,
    pub variables: Vec<Variable>,
    pub states: Vec<ControllerState>,
    /// The `init` expression: the parallel composition of state references.
    pub init: Vec<Identifier>,
}

/// `aiState name { .. }`
#[derive(Clone, Debug)]
pub struct ControllerState {
    pub id: Identifier,
    pub body: Vec<ControllerCommand>,
}

#[derive(Clone, Debug)]
pub enum ControllerCommand {
    /// `[steps #] step target;`
    Step {
        steps: Option<Expression>,
        target: Identifier,
    },
    /// `exec target;`
    Exec(Identifier),
    /// `let id = value in body`
    Let {
        id: Identifier,
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
    pub id: Identifier,
    pub value: Expression,
}

/// A `[when guard] target' = value;` assignment shared by controllers and the
/// environment. `target` is the primed variable name (without the trailing `'`).
#[derive(Clone, Debug)]
pub struct Update {
    pub guard: Option<Expression>,
    pub target: Identifier,
    pub value: Expression,
}

// ---------------------------------------------------------------------------
// Robustness sub-languages (perturbation / distance / ROBTL)
// ---------------------------------------------------------------------------

/// `perturbation name = expr;`
#[derive(Clone, Debug)]
pub struct Perturbation {
    pub id: Identifier,
    pub value: PerturbationExpression,
}

#[derive(Clone, Debug)]
pub enum PerturbationExpression {
    Nil,
    Reference(Identifier),
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
    pub id: Identifier,
    pub value: Expression,
}

/// `distance name = expr;`
#[derive(Clone, Debug)]
pub struct Distance {
    pub id: Identifier,
    pub value: DistanceExpression,
}

#[derive(Clone, Debug)]
pub enum DistanceExpression {
    Reference(Identifier),
    /// `< penalty`
    AtomicLeft(Identifier),
    /// `> penalty`
    AtomicRight(Identifier),
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
    pub id: Identifier,
    pub value: RobtlFormula,
}

#[derive(Clone, Debug)]
pub enum RobtlFormula {
    True,
    False,
    Reference(Identifier),
    /// `\D[distance, perturbation] op value`
    Distance {
        distance: Identifier,
        perturbation: Identifier,
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
pub enum Expression {
    // Literals
    False,
    True,
    Integer(i64),
    Real(f64),
    Identifier(String),
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
        function: Box<Expression>,
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

/// Source location information, spanning from start to end in the source text.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<pest::Span<'_>> for Span {
    fn from(span: pest::Span) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}
