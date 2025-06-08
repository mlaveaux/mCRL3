use std::hash::Hash;

use arbitrary::Arbitrary;

/// An mCRL2 specification containing declarations.
#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct UntypedProcessSpecification {
    /// Sort declarations
    pub data_specification: UntypedDataSpecification,
    /// Global variables
    pub glob_vars: Vec<VarDecl>,
    /// Action declarations
    pub act_decls: Vec<ActDecl>,
    /// Process declarations
    pub proc_decls: Vec<ProcDecl>,
    /// Initial process
    pub init: Option<ProcessExpr>,
}

#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct UntypedDataSpecification {
    /// Sort declarations
    pub sort_decls: Vec<SortDecl>,
    /// Constructor declarations
    pub cons_decls: Vec<IdDecl>,
    /// Map declarations
    pub map_decls: Vec<IdDecl>,
    /// Equation declarations
    pub eqn_decls: Vec<EqnSpec>,
}

/// A declaration of an identifier with its sort.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct IdDecl {
    /// Identifier being declared
    pub identifier: String,
    /// Sort expression for this identifier
    pub sort: SortExpression,
    /// Source location information
    pub span: Span,
}

/// Expression representing a sort (type).
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SortExpression {
    /// Product of two sorts (A # B)
    Product {
        lhs: Box<SortExpression>,
        rhs: Box<SortExpression>,
    },
    /// Function sort (A -> B)
    Function {
        domain: Box<SortExpression>,
        range: Box<SortExpression>,
    },
    Struct {
        inner: Vec<ConstructorDecl>,
    },
    /// Reference to a named sort
    Reference(String),
    /// Built-in simple sort
    Simple(Sort),
    /// Parameterized complex sort
    Complex(ComplexSort, Box<SortExpression>),
}

/// Constructor declaration
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ConstructorDecl {
    pub name: String,
    pub args: Vec<(Option<String>, SortExpression)>,
    pub projection: Option<String>,
}

/// Built-in simple sorts.
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Sort {
    Bool,
    Pos,
    Int,
    Nat,
    Real,
}

/// Complex (parameterized) sorts.
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ComplexSort {
    List,
    Set,
    FSet,
    FBag,
    Bag,
}

/// Sort declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct SortDecl {
    /// Sort identifier
    pub identifier: String,
    /// Sort expression (if structured)
    pub expr: Option<SortExpression>,
    /// Where the sort is defined
    pub span: Span,
}

/// Variable declaration
#[derive(Arbitrary, Debug, Eq, PartialEq, Hash)]
pub struct VarDecl {
    pub identifier: String,
    pub sort: SortExpression,
    pub span: Span,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct EqnSpec {
    pub variables: Vec<VarDecl>,
    pub equations: Vec<EqnDecl>,
}

/// Equation declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct EqnDecl {
    pub condition: Option<DataExpr>,
    pub lhs: DataExpr,
    pub rhs: DataExpr,
    pub span: Span,
}

/// Action declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ActDecl {
    pub identifier: String,
    pub args: Vec<SortExpression>,
    pub span: Span,
}

/// Process declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ProcDecl {
    pub identifier: String,
    pub params: Vec<VarDecl>,
    pub body: ProcessExpr,
    pub span: Span,
}

#[derive(Arbitrary, Debug, Eq, PartialEq, Hash)]
pub enum DataExprUnaryOp {
    Negation,
    Minus,
    Size,
}

#[derive(Arbitrary, Debug, Eq, PartialEq, Hash)]
pub enum DataExprBinaryOp {
    Conj,
    Disj,
    Implies,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Cons,
    Snoc,
    In,
    Concat,
    Add,
    Subtract,
    Div,
    IntDiv,
    Mod,
    Multiply,
    At,
}

/// Data expression
#[derive(Arbitrary, Debug, Eq, PartialEq, Hash)]
pub enum DataExpr {
    Id(String),
    Number(String), // Is string because the number can be any size.
    Bool(bool),
    Application {
        function: Box<DataExpr>,
        arguments: Vec<DataExpr>,
    },
    EmptyList,
    List(Vec<DataExpr>),
    EmptySet,
    Set(Vec<DataExpr>),
    EmptyBag,
    Bag(Vec<BagElement>),
    SetBagComp {
        variable: VarDecl,
        predicate: Box<DataExpr>,
    },
    Lambda {
        variables: Vec<VarDecl>,
        body: Box<DataExpr>,
    },
    Quantifier {
        op: Quantifier,
        variables: Vec<VarDecl>,
        body: Box<DataExpr>,
    },
    Unary {
        op: DataExprUnaryOp,
        expr: Box<DataExpr>,
    },
    Binary {
        op: DataExprBinaryOp,
        lhs: Box<DataExpr>,
        rhs: Box<DataExpr>,
    },
    FunctionUpdate {
        expr: Box<DataExpr>,
        update: Box<DataExprUpdate>,
    },
    Whr {
        expr: Box<DataExpr>,
        assignments: Vec<Assignment>,
    },
}

#[derive(Arbitrary, Debug, Eq, PartialEq, Hash)]
pub struct BagElement {
    pub expr: DataExpr,
    pub multiplicity: DataExpr,
}

#[derive(Arbitrary, Debug, Eq, PartialEq, Hash)]
pub struct DataExprUpdate {
    pub expr: DataExpr,
    pub update: DataExpr,
}

#[derive(Arbitrary, Debug, Eq, PartialEq, Hash)]
pub struct Assignment {
    pub identifier: String,
    pub expr: DataExpr,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ProcExprBinaryOp {
    Sequence,
    Choice,
    Parallel,
    LeftMerge,
    CommMerge,
}

/// Process expression
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ProcessExpr {
    Id(String, Vec<Assignment>),
    Action(String, Vec<DataExpr>),
    Delta,
    Tau,
    Sum {
        variables: Vec<VarDecl>,
        operand: Box<ProcessExpr>,
    },
    Dist {
        variables: Vec<VarDecl>,
        expr: DataExpr,
        operand: Box<ProcessExpr>,
    },
    Binary {
        op: ProcExprBinaryOp,
        lhs: Box<ProcessExpr>,
        rhs: Box<ProcessExpr>,
    },
    Hide {
        actions: Vec<String>,
        operand: Box<ProcessExpr>,
    },
    Rename {
        renames: Vec<Rename>,
        operand: Box<ProcessExpr>,
    },
    Allow {
        actions: Vec<MultiActionLabel>,
        operand: Box<ProcessExpr>,
    },
    Block {
        actions: Vec<String>,
        operand: Box<ProcessExpr>,
    },
    Comm {
        comm: Vec<Comm>,
        operand: Box<ProcessExpr>,
    },
    Condition {
        condition: DataExpr,
        then: Box<ProcessExpr>,
        else_: Option<Box<ProcessExpr>>,
    },
    At {
        expr: Box<ProcessExpr>,
        operand: DataExpr,
    },
}

/// Communication action
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct CommAction {
    pub inputs: Vec<String>,
    pub output: String,
    pub span: Span,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct UntypedStateFrmSpec {
    pub data_specification: UntypedDataSpecification,
    pub formula: StateFrm,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum StateFrmUnaryOp {
    Minus,
    Negation,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum StateFrmOp {
    Addition,
    Implies,
    Disjunction,
    Conjunction,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum FixedPointOperator {
    Least,
    Greatest,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct StateVarDecl {
    pub identifier: String,
    pub arguments: Vec<StateVarAssignment>,
    pub span: Span,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct StateVarAssignment {
    pub identifier: String,
    pub sort: SortExpression,
    pub expr: DataExpr,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ModalityOperator {
    Diamond,
    Box,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum StateFrm {
    True,
    False,
    Delay(DataExpr),
    Yaled(DataExpr),
    Id(String, Vec<DataExpr>),
    DataValExprMult(DataExpr, Box<StateFrm>),
    DataValExprRightMult(Box<StateFrm>, DataExpr),
    DataValExpr(DataExpr),
    Modality {
        operator: ModalityOperator,
        formula: RegFrm,
        expr: Box<StateFrm>,
    },
    Unary {
        op: StateFrmUnaryOp,
        expr: Box<StateFrm>,
    },
    Binary {
        op: StateFrmOp,
        lhs: Box<StateFrm>,
        rhs: Box<StateFrm>,
    },
    Quantifier {
        quantifier: Quantifier,
        variables: Vec<VarDecl>,
        body: Box<StateFrm>,
    },
    FixedPoint {
        operator: FixedPointOperator,
        variable: StateVarDecl,
        body: Box<StateFrm>,
    },
}

/// Represents a multi action label `a | b | c ...`.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct MultiActionLabel {
    pub actions: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Action {
    pub id: String,
    pub args: Vec<DataExpr>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct MultiAction {
    pub actions: Vec<Action>,
}

#[derive(Arbitrary, Debug, Eq, PartialEq, Hash)]
pub enum Quantifier {
    Exists,
    Forall,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ActFrmOp {
    Implies,
    Union,
    Intersect,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ActFrm {
    True,
    False,
    MultAct(MultiAction),
    DataExprVal(DataExpr),
    Negation(Box<ActFrm>),
    Quantifier {
        quantifier: Quantifier,
        variables: Vec<VarDecl>,
        body: Box<ActFrm>,
    },
    Binary {
        op: ActFrmOp,
        lhs: Box<ActFrm>,
        rhs: Box<ActFrm>,
    },
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum RegFrm {
    Action(ActFrm),
    Iteration(Box<RegFrm>),
    Plus(Box<RegFrm>),
    Sequence { lhs: Box<RegFrm>, rhs: Box<RegFrm> },
    Choice { lhs: Box<RegFrm>, rhs: Box<RegFrm> },
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Rename {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Comm {
    pub from: MultiActionLabel,
    pub to: String,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct UntypedActionRenameSpec {
    pub data_spec: UntypedDataSpecification,
    pub act_decls: Vec<ActDecl>,
    pub rename_decls: Vec<ActionRenameDecl>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ActionRenameDecl {
    pub vars_spec: Vec<VarDecl>,
    pub rename_rule: ActionRenameRule,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ActionRenameRule {
    pub condition: Option<DataExpr>,
    pub action: Action,
    pub rhs: ActionRHS,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ActionRHS {
    Tau,
    Delta,
    Action(Action),
}

/// Source location information.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Span {
    /// Start position in source
    pub start: usize,
    /// End position in source
    pub end: usize,
}

impl Arbitrary<'_> for Span {
    fn arbitrary(_u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(Span { start: 0, end: 0 })
    }
}

impl From<pest::Span<'_>> for Span {
    fn from(span: pest::Span) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}
