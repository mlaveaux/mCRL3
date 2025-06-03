use log::trace;
use mcrl3_utilities::{DisplayPair, MCRL3Error};
use pest_consume::Error;
use pest_derive::Parser;
use pest::Parser;

use crate::{ParseNode, UntypedDataSpecification, UntypedProcessSpecification, UntypedStateFrmSpec};

#[derive(Parser)]
#[grammar = "mcrl2_grammar.pest"]
pub struct Mcrl2Parser;

/// Parses the given mCRL2 specification into an AST.
impl UntypedProcessSpecification {
    pub fn parse(spec: &str) -> Result<UntypedProcessSpecification, MCRL3Error> {
        pest::set_error_detail(true);

        let mut result = Mcrl2Parser::parse(Rule::MCRL2Spec, spec)?;
        let root = result.next().ok_or("Could not parse mCRL2 specification")?;
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Mcrl2Parser::MCRL2Spec(ParseNode::new(root)).map_err(|err| {
            extend_parser_error(err).into()
        })
    }
}

/// Parses the given mCRL2 specification into an AST.
impl UntypedDataSpecification {
    pub fn parse(spec: &str) -> Result<UntypedDataSpecification, MCRL3Error> {
        pest::set_error_detail(true);

        let mut result = Mcrl2Parser::parse(Rule::DataSpec, spec)?;
        let root = result.next().ok_or("Could not parse mCRL2 data specification")?;
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Mcrl2Parser::DataSpec(ParseNode::new(root)).map_err(|err| {
            extend_parser_error(err).into()
        })
    }
}

impl UntypedStateFrmSpec {
    pub fn parse(spec: &str) -> Result<UntypedStateFrmSpec, MCRL3Error> {
        pest::set_error_detail(true);

        let mut result = Mcrl2Parser::parse(Rule::StateFrmSpec, spec)?;
        let root = result
            .next()
            .ok_or("Could not parse mCRL2 state formula specification")?;
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Mcrl2Parser::StateFrmSpec(ParseNode::new(root)).map_err(|err| {
            extend_parser_error(err).into()
        })
    }
}

fn extend_parser_error(error: Error<Rule>) -> Error<Rule> {
    error.renamed_rules(|rule| {
        match rule {
            Rule::DataExprAdd => "+".to_string(),
            _ => format!("{:?}", rule),
        }
    })
}