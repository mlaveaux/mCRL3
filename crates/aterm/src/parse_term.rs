use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "term_grammar.pest"]
pub struct TermParser;

#[pest_consume::parser]
impl TermParser {

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