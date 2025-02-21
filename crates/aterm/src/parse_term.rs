
use pest_consume::Error;
use pest_consume::match_nodes;
use pest_derive::Parser;

use crate::{ATerm, Symbol};

#[derive(Parser)]
#[grammar = "term_grammar.pest"]
pub struct TermParser;

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type ParseNode<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl TermParser {
    fn Id(input: ParseNode) -> Result<String, Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    pub fn Term(term: ParseNode) -> Result<ATerm, Error<Rule>> {
        match_nodes!(term.into_children();
            [Id(identifier), Args(args)] => {
                let symbol = Symbol::new(identifier, args.len());

                Ok(ATerm::with_iter(&symbol, args))
            }
        )
    }

    pub fn Args(args: ParseNode) -> Result<Vec<ATerm>, Error<Rule>> {
        match_nodes!(args.into_children();
            [Term(term)..] => {
                Ok(term.collect())
            }
        )
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