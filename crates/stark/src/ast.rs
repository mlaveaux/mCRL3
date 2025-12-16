use std::collections::HashMap;

use merc_utilities::MercError;

use crate::parse::StarkParser;


/// Represents the AST of a controller.
pub enum Controller {
    Action(Vec<Update>, Box<Controller>),
    Assignemnt(Vec<Update>, Box<Controller>),
    Effect(),
    /// Executes another controller.
    Exec(usize),
    Choice(Box<Controller>, Box<Controller>),
    IfThenElse(Expression, Box<Controller>, Box<Controller>),
    Nil,
    Paralellel(Box<Controller>, Box<Controller>),
    Interleave(f64, Box<Controller>, Box<Controller>),
    Step(Box<Controller>),
}

pub enum Expression {

}

pub struct StarkSpecification {
    pub controllers: HashMap<String, Controller>,
}

struct Update {

}