use std::sync::LazyLock;

use pest::iterators::Pairs;
use pest::pratt_parser::Assoc;
use pest::pratt_parser::Assoc::Left;
use pest::pratt_parser::Assoc::Right;
use pest::pratt_parser::Op;
use pest::pratt_parser::PrattParser;
use pest_consume::Node;

use crate::DataExpr;
use crate::DataExprOp;
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

                Rule::SortExprParens => {
                    // Handle parentheses by recursively parsing the inner expression
                    let inner = primary.into_inner().next().unwrap();
                    parse_sortexpr(inner.into_inner())
                }

                Rule::SortExprStruct => Mcrl2Parser::SortExprStruct(Node::new(primary)).unwrap(),

                _ => unimplemented!("Unexpected rule: {:?}", primary.as_rule()),
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

static DATAEXPR_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::infix(Rule::DataExprConj, Assoc::Left))
        .op(Op::infix(Rule::DataExprDisj, Assoc::Left))
        .op(Op::infix(Rule::DataExprEq, Assoc::Left))

    //.op(Op::prefix(Rule::DataExprNegation) | Op::prefix(Rule::DataExprNo | Op::prefix(Rule::DataExprNot)))
});

pub fn parse_dataexpr(pairs: Pairs<Rule>) -> DataExpr {
    DATAEXPR_PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::DataExprTrue => Mcrl2Parser::DataExprTrue(Node::new(primary)).unwrap(),
            Rule::DataExprFalse => Mcrl2Parser::DataExprFalse(Node::new(primary)).unwrap(),
            Rule::DataExprPrimary => Mcrl2Parser::DataExprPrimary(Node::new(primary)).unwrap(),
            Rule::Id => DataExpr::Id(Mcrl2Parser::Id(Node::new(primary)).unwrap()),

            Rule::DataExprBrackets => {
                // Handle parentheses by recursively parsing the inner expression
                let inner = primary.into_inner().next().unwrap();
                parse_dataexpr(inner.into_inner())
            }

            _ => unimplemented!("Unexpected rule: {:?}", primary.as_rule()),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::DataExprConj => DataExpr::BinaryOperator {
                op: DataExprOp::Conj,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            Rule::DataExprDisj => DataExpr::BinaryOperator {
                op: DataExprOp::Disj,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            _ => unreachable!(),
        })
        .map_postfix(|expr, postfix| match postfix.as_rule() {
            Rule::DataExprUpdate => DataExpr::FunctionUpdate {
                expr: Box::new(expr),
                update: Mcrl2Parser::DataExprUpdate(Node::new(postfix)).unwrap(),
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
