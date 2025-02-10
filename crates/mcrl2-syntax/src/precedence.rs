use std::sync::LazyLock;

use pest::iterators::Pairs;
use pest::pratt_parser::Assoc::Left;
use pest::pratt_parser::Assoc::Right;
use pest::pratt_parser::Op;
use pest::pratt_parser::PrattParser;

use crate::ast::SortExpression;
use crate::ComplexSort;
use crate::Rule;
use crate::Sort;

static SORT_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        // Sort operators
        .op(Op::infix(Rule::SortExprFunction, Left))
        .op(Op::infix(Rule::SortExprProduct, Right))
});

pub fn parse_sortexpr(pairs: Pairs<Rule>) -> SortExpression {
    SORT_PRATT_PARSER
        .map_primary(|primary|
        {
           match primary.as_rule() {
                Rule::SortExprAtom => {
                    let inner = primary.into_inner().next().unwrap();
                    match inner.as_rule() {
                        Rule::Id => SortExpression::Reference(inner.as_str().to_string()),
                        Rule::SortExprBool => SortExpression::Simple(Sort::Bool),
                        Rule::SortExprInt => SortExpression::Simple(Sort::Int),
                        Rule::SortExprPos => SortExpression::Simple(Sort::Pos),
                        Rule::SortExprNat => SortExpression::Simple(Sort::Nat),
                        Rule::SortExprReal => SortExpression::Simple(Sort::Real),
                        Rule::SortExprList => SortExpression::Complex(ComplexSort::List, Box::new(parse_sortexpr(inner.into_inner()))),
                        _ => unreachable!("Unknown SortExprAtom {inner:?}"),
                    }
                },
                _ => unreachable!("{primary:?}"),
           }
        })
        .map_infix(|lhs, op, rhs| 
        {
            match op.as_rule() {
                Rule::SortExprFunction => SortExpression::Function {
                    domain: Box::new(lhs),
                    range: Box::new(rhs),
                },
                Rule::SortExprProduct => SortExpression::Product {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                _ => unreachable!(),
            }
        })
        .parse(pairs)
}


// #[cfg(test)]
// mod tests {
//     use pest::Parser;

//     use crate::Mcrl2Parser;

//     use super::*;

//     #[test]
//     fn test_precedence() {
//         let term = "List(Data)";

//         let result = Mcrl2Parser::parse(Rule::SortExpr, term).unwrap();
//         print!("{}", parse_sortexpr(result));
//     }
// }