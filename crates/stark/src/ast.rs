use std::collections::HashMap;

/// Represents the AST of a controller.
pub enum Controller {
    Action(Vec<Update>, Box<Controller>),
    Assignment(Vec<Update>, Box<Controller>),
    Effect(),
    Exec(usize),
    Choice(Box<Controller>, Box<Controller>),
    IfThenElse(Expression, Box<Controller>, Box<Controller>),
    Parallel(Box<Controller>, Box<Controller>),
    Interleave(f64, Box<Controller>, Box<Controller>),
    Step(Box<Controller>),
    Nil,
}

/// Represents an expression in the AST.
pub enum Expression {
    // Literals
    False,
    True,
    Integer(i64),
    Real(f64),
    Identifier(String),
    Iterator,

    // Distributions
    Normal {
        mean: Box<Expression>,
        std_dev: Box<Expression>,
    },
    Uniform {
        values: Vec<Expression>,
    },
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

    // Postfix operators
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Aggregate {
        target: Box<Expression>,
        op: AggregateOp,
        argument: Option<Box<Expression>>,
    },
}

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

pub enum AggregateOp {
    Count,
    Min,
    Max,
    Mean,
}

pub struct StarkSpecification {
    pub controllers: HashMap<String, Controller>,
}

pub struct Update {
    identifier: Identifier,
    value: Expression,
}

pub struct Variable {
    global: bool,
    ty: Ty,
    id: Identifier,
    range: Option<Range>,
    initial_value: Expression,
}

pub struct Constant {
    global: bool,
    ty: Option<Ty>,
    id: Identifier,
    range: Option<Range>,
    initial_value: Expression,
}

impl Constant {
    pub fn new(id: Identifier, value: Expression) -> Self {
        Constant {
            global: true,
            ty: None,
            id,
            range: None,
            initial_value: value,
        }
    }
}

impl Variable {
    pub fn new(
        global: bool,
        ty: Ty,
        id: Identifier,
        range: Option<Range>,
        initial_value: Expression,
    ) -> Self {
        Variable {
            global,
            ty,
            id,
            range,
            initial_value,
        }
    }
}

pub struct Range {
    min: Option<Expression>,
    max: Option<Expression>,
}

impl Range {
    pub fn new(min: Option<Expression>, max: Option<Expression>) -> Self {
        Range { min, max }
    }
}

pub enum Ty {
    Real,
    Integer,
    Boolean,
}

pub struct Identifier {
    name: String,
    span: Span,
}

impl Identifier {
    pub fn new(name: String, span: Span) -> Self {
        Identifier { name, span }
    }
}

pub enum Command {
    Let,
    IfThenElse,
    Assignment,
    Block,
}

/// Source location information, spanning from start to end in the source text.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
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