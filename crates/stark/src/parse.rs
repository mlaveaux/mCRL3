use pest::Parser;
use pest_derive::Parser;

use merc_utilities::MercError;

use crate::ast::UntypedStarkSpecification;
use crate::consume::ParseNode;

#[derive(Parser)]
#[grammar = "stark_grammar.pest"]
pub struct StarkParser;

impl UntypedStarkSpecification {
    /// Parse the given stark specification into an AST.
    pub fn parse(input: &str) -> Result<Self, MercError> {
        let mut result = StarkParser::parse(Rule::UntypedStarkSpecification, input)?;
        let root = result.next().expect("Could not parse STARK specification");
        Ok(StarkParser::UntypedStarkSpecification(ParseNode::new(root))?)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::UntypedStarkSpecification;

    #[test]
    fn test_parse_engine_stark() {
        if let Err(x) = UntypedStarkSpecification::parse(include_str!("../../../examples/stark/engine.stark")) {
            panic!("Failed to parse: {}", x);
        }
    }

    #[test]
    fn test_parse_random_walk_stark() {
        if let Err(x) = UntypedStarkSpecification::parse(include_str!("../../../examples/stark/random_walk.stark")) {
            panic!("Failed to parse: {}", x);
        }
    }

    #[test]
    fn test_parse_single_vehicle_stark() {
        if let Err(x) = UntypedStarkSpecification::parse(include_str!("../../../examples/stark/single_vehicle.stark")) {
            panic!("Failed to parse: {}", x);
        }
    }

    #[test]
    fn test_parse_toll_stark() {
        if let Err(x) = UntypedStarkSpecification::parse(include_str!("../../../examples/stark/toll.stark")) {
            panic!("Failed to parse: {}", x);
        }
    }

    #[test]
    fn test_parse_two_vehicles_stark() {
        if let Err(x) = UntypedStarkSpecification::parse(include_str!("../../../examples/stark/two_vehicles.stark")) {
            panic!("Failed to parse: {}", x);
        }
    }
}
