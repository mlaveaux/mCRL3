use std::error::Error;

use pest_consume::match_nodes;
use pest_derive::Parser;

use crate::{ATerm, Symbol};

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type ParseNode<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "term_grammar.pest"]
pub struct TermParser;

#[pest_consume::parser]
impl TermParser {
    fn Id(input: ParseNode) -> String {
        input.as_str().to_string()
    }

    fn Term(term: ParseNode) -> ATerm {
        match_nodes!(term.into_children();
            [Id(identifier), Term(t0), .., Term(tn)] => {
                let symbol = Symbol::new(identifier, 0);

                Term::new(symbol, vec![t0, tn]);
            }
        );

        unimplemented!();
    }



    
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;

    #[test]
    fn test_parse_term() {
        let term = "f(a, b)";

        let result = TermParser::parse(Rule::TermSpec, term).unwrap();
        print!("{}", result);
    }
}