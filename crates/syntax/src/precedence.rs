use std::sync::LazyLock;

use pest::iterators::Pairs;
use pest::pratt_parser::Assoc::Left;
use pest::pratt_parser::Assoc::Right;
use pest::pratt_parser::Op;
use pest::pratt_parser::PrattParser;
use pest_consume::Node;

use crate::Mcrl2Parser;
use crate::Rule;
use crate::ast::SortExpression;

static SORT_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        // Sort operators
        .op(Op::infix(Rule::SortExprFunction, Left))
        .op(Op::infix(Rule::SortExprProduct, Right))
});

/// Parses a sequence of `Rule` pairs into a `SortExpression` using a Pratt parser for operator precedence.
///
/// # Arguments
///
/// * `pairs` - A sequence of `Rule` pairs to be parsed.
///
/// # Returns
///
/// A `SortExpression` representing the parsed input.
///
/// # Panics
///
/// This function will panic if it encounters an unexpected rule during parsing.
///
/// # Example
///
/// ```
/// use pest::Parser;
/// use crate::Mcrl2Parser;
/// use crate::Rule;
/// use crate::parse_sortexpr;
///
/// let term = "List(Data)";
/// let result = Mcrl2Parser::parse(Rule::SortExpr, term).unwrap();
/// let sort_expr = parse_sortexpr(result);
/// println!("{}", sort_expr);
/// ```
pub fn parse_sortexpr(pairs: Pairs<Rule>) -> SortExpression {
    SORT_PRATT_PARSER
        .map_primary(|primary| {
            match primary.as_rule() {
                Rule::Id => SortExpression::Reference(Mcrl2Parser::Id(Node::new(primary)).unwrap()),
                Rule::SortExpr => Mcrl2Parser::SortExpr(Node::new(primary)).unwrap(),
                Rule::SortExprAtom => Mcrl2Parser::SortExprAtom(Node::new(primary)).unwrap(),

                Rule::SortExprBool => Mcrl2Parser::SortExprBool(Node::new(primary)).unwrap(),
                Rule::SortExprInt => Mcrl2Parser::SortExprInt(Node::new(primary)).unwrap(),
                Rule::SortExprPos => Mcrl2Parser::SortExprPos(Node::new(primary)).unwrap(),
                Rule::SortExprNat => Mcrl2Parser::SortExprNat(Node::new(primary)).unwrap(),
                Rule::SortExprReal => Mcrl2Parser::SortExprReal(Node::new(primary)).unwrap(),

                Rule::SortExprList => Mcrl2Parser::SortExprList(Node::new(primary)).unwrap(),
                Rule::SortExprSet => Mcrl2Parser::SortExprSet(Node::new(primary)).unwrap(),
                Rule::SortExprBag => Mcrl2Parser::SortExprBag(Node::new(primary)).unwrap(),
                Rule::SortExprFSet => Mcrl2Parser::SortExprFSet(Node::new(primary)).unwrap(),
                Rule::SortExprFBag => Mcrl2Parser::SortExprFBag(Node::new(primary)).unwrap(),

                //Rule::SortExprStruct => unimplemented!(),
                _ => unreachable!(),
            }
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::SortExprFunction => SortExpression::Function {
                domain: Box::new(lhs),
                range: Box::new(rhs),
            },
            Rule::SortExprProduct => SortExpression::Product {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            _ => unreachable!(),
        })
        .parse(pairs)
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::Mcrl2Parser;

    use super::*;

    #[test]
    fn test_precedence() {
        let term = "List(Data)";

        let result = Mcrl2Parser::parse(Rule::SortExpr, term).unwrap();
        print!("{}", parse_sortexpr(result));
    }
}
