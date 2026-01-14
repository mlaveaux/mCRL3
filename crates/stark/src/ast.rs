use std::collections::HashMap;

use merc_utilities::MercError;

use crate::parse::StarkParser;


/// Represents the AST of a controller.
pub enum Controller {
    Action(Vec<Update>, Box<Controller>),
    Assignemnt(Vec<Update>, Box<Controller>),
    Effect(),
    Exec(usize),
    Choice(Box<Controller>, Box<Controller>),
    IfThenElse(Expression, Box<Controller>, Box<Controller>),
    Paralellel(Box<Controller>, Box<Controller>),
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
    Normal { mean: Box<Expression>, std_dev: Box<Expression> },
    Uniform { values: Vec<Expression> },
    Range { min: Option<Box<Expression>>, max: Option<Box<Expression>> },

    // Prefix operators
    Not(Box<Expression>),
    UnaryPlus(Box<Expression>),
    UnaryMinus(Box<Expression>),

    // Binary operators
    Binary(BinaryOp, Box<Expression>, Box<Expression>),

    // Postfix operators
    Call { function: Box<Expression>, arguments: Vec<Expression> },
    Aggregate { target: Box<Expression>, op: AggregateOp, argument: Option<Box<Expression>> },
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

struct Update {

}