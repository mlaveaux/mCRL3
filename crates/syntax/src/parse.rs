use log::trace;
use mcrl3_utilities::DisplayPair;
use mcrl3_utilities::MCRL3Error;
use pest::Parser;
use pest_consume::Error;
use pest_derive::Parser;

use crate::DataExprBinaryOp;
use crate::ParseNode;
use crate::UntypedDataSpecification;
use crate::UntypedProcessSpecification;
use crate::UntypedStateFrmSpec;

#[derive(Parser)]
#[grammar = "mcrl2_grammar.pest"]
pub struct Mcrl2Parser;

/// Parses the given mCRL2 specification into an AST.
impl UntypedProcessSpecification {
    pub fn parse(spec: &str) -> Result<UntypedProcessSpecification, MCRL3Error> {
        let mut result = Mcrl2Parser::parse(Rule::MCRL2Spec, spec).map_err(extend_parser_error)?;
        let root = result.next().expect("Could not parse mCRL2 specification");
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Ok(Mcrl2Parser::MCRL2Spec(ParseNode::new(root))?)
    }
}

/// Parses the given mCRL2 specification into an AST.
impl UntypedDataSpecification {
    pub fn parse(spec: &str) -> Result<UntypedDataSpecification, MCRL3Error> {
        let mut result = Mcrl2Parser::parse(Rule::DataSpec, spec).map_err(extend_parser_error)?;
        let root = result.next().expect("Could not parse mCRL2 data specification");
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Ok(Mcrl2Parser::DataSpec(ParseNode::new(root))?)
    }
}

impl UntypedStateFrmSpec {
    pub fn parse(spec: &str) -> Result<UntypedStateFrmSpec, MCRL3Error> {
        let mut result = Mcrl2Parser::parse(Rule::StateFrmSpec, spec).map_err(extend_parser_error)?;
        let root = result
            .next()
            .expect("Could not parse mCRL2 state formula specification");
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Ok(Mcrl2Parser::StateFrmSpec(ParseNode::new(root))?)
    }
}

fn extend_parser_error(error: Error<Rule>) -> Error<Rule> {
    error.renamed_rules(|rule| match rule {
        Rule::DataExprWhr => "DataExpr whr AssignmentList end".to_string(),
        Rule::DataExprUpdate => "DataExpr[(DataExpr -> DataExpr)*]".to_string(),
        Rule::DataExprApplication => "DataExpr(DataExpr*)".to_string(),

        // DataExpr Binary Operators
        Rule::DataExprConj => format!("{}", DataExprBinaryOp::Conj),
        Rule::DataExprDisj => format!("{}", DataExprBinaryOp::Disj),
        Rule::DataExprImpl => format!("{}", DataExprBinaryOp::Implies),
        Rule::DataExprEq => format!("{}", DataExprBinaryOp::Equal),
        Rule::DataExprNeq => format!("{}", DataExprBinaryOp::NotEqual),
        Rule::DataExprLess => format!("{}", DataExprBinaryOp::LessThan),
        Rule::DataExprLeq => format!("{}", DataExprBinaryOp::LessEqual),
        Rule::DataExprGreater => format!("{}", DataExprBinaryOp::GreaterThan),
        Rule::DataExprGeq => format!("{}", DataExprBinaryOp::GreaterEqual),
        Rule::DataExprIn => format!("{}", DataExprBinaryOp::In),
        Rule::DataExprDiv => format!("{}", DataExprBinaryOp::Div),
        Rule::DataExprIntDiv => format!("{}", DataExprBinaryOp::IntDiv),
        Rule::DataExprMod => format!("{}", DataExprBinaryOp::Mod),
        Rule::DataExprMult => format!("{}", DataExprBinaryOp::Multiply),
        Rule::DataExprAdd => format!("{}", DataExprBinaryOp::Add),
        Rule::DataExprSubtract => format!("{}", DataExprBinaryOp::Subtract),
        Rule::DataExprAt => format!("{}", DataExprBinaryOp::At),
        Rule::DataExprCons => format!("{}", DataExprBinaryOp::Cons),
        Rule::DataExprSnoc => format!("{}", DataExprBinaryOp::Snoc),
        Rule::DataExprConcat => format!("{}", DataExprBinaryOp::Concat),

        // Regular Formulas
        Rule::RegFrmAlternative => "RegFrm + RegFrm".to_string(),
        Rule::RegFrmComposition => "RegFrm . RegFrm".to_string(),
        Rule::RegFrmIteration => "RegFrm*".to_string(),
        Rule::RegFrmPlus => "RegFrm+".to_string(),
        _ => format!("{:?}", rule),
    })
}
