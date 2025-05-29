use std::sync::LazyLock;

use pest::iterators::Pairs;
use pest::pratt_parser::Assoc;
use pest::pratt_parser::Assoc::Left;
use pest::pratt_parser::Assoc::Right;
use pest::pratt_parser::Op;
use pest::pratt_parser::PrattParser;
use pest_consume::Node;

use crate::DataExpr;
use crate::DataExprBinaryOp;
use crate::DataExprUnaryOp;
use crate::Mcrl2Parser;
use crate::Rule;
use crate::ast::SortExpression;

static SORT_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        // Sort operators
        .op(Op::infix(Rule::SortExprFunction, Left)) // $right 0
        .op(Op::infix(Rule::SortExprProduct, Right)) // $left 1
});

/// Parses a sequence of `Rule` pairs into a `SortExpression` using a Pratt parser for operator precedence.
pub fn parse_sortexpr(pairs: Pairs<Rule>) -> SortExpression {
    SORT_PRATT_PARSER
        .map_primary(|primary| {
            match primary.as_rule() {
                Rule::Id => SortExpression::Reference(Mcrl2Parser::Id(Node::new(primary)).unwrap()),
                Rule::SortExpr => Mcrl2Parser::SortExpr(Node::new(primary)).unwrap(),

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
            _ => unimplemented!("Unexpected binary operator: {:?}", op.as_rule()),
        })
        .parse(pairs)
}

static DATAEXPR_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::postfix(Rule::DataExprWhr)) // $left 0
        .op(Op::prefix(Rule::DataExprForall) | Op::prefix(Rule::DataExprExists) | Op::prefix(Rule::DataExprLambda)) // $right 1
        .op(Op::infix(Rule::DataExprImpl, Assoc::Right)) // $right 2
        .op(Op::infix(Rule::DataExprDisj, Assoc::Right)) // $right 3
        .op(Op::infix(Rule::DataExprConj, Assoc::Right)) // $right 4
        .op(Op::infix(Rule::DataExprEq, Assoc::Left) | Op::infix(Rule::DataExprNeq, Assoc::Left)) // $left 5
        .op(Op::infix(Rule::DataExprLess, Assoc::Left)
            | Op::infix(Rule::DataExprLeq, Assoc::Left)
            | Op::infix(Rule::DataExprGeq, Assoc::Left)
            | Op::infix(Rule::DataExprGreater, Assoc::Left)
            | Op::infix(Rule::DataExprIn, Assoc::Left)) // $left 6
        .op(Op::infix(Rule::DataExprCons, Assoc::Right)) // $right 7
        .op(Op::infix(Rule::DataExprSnoc, Assoc::Left)) // $left 8
        .op(Op::infix(Rule::DataExprConcat, Assoc::Left)) // $left 9
        .op(Op::infix(Rule::DataExprAdd, Assoc::Left) | Op::infix(Rule::DataExprSubtract, Assoc::Left)) // $left 10
        .op(Op::infix(Rule::DataExprDiv, Assoc::Left)
            | Op::infix(Rule::DataExprIntDiv, Assoc::Left)
            | Op::infix(Rule::DataExprMod, Assoc::Left)) // $left 11
        .op(Op::infix(Rule::DataExprMult, Assoc::Left)
            | Op::infix(Rule::DataExprAt, Assoc::Left) // $left 12
            | Op::prefix(Rule::DataExprMinus)
            | Op::prefix(Rule::DataExprNegation)
            | Op::prefix(Rule::DataExprSize)) // $right 12
        .op(Op::postfix(Rule::DataExprUpdate) | Op::postfix(Rule::DataExprApplication)) // ) // $left 13
});

pub fn parse_dataexpr(pairs: Pairs<Rule>) -> DataExpr {
    DATAEXPR_PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::DataExprTrue => Mcrl2Parser::DataExprTrue(Node::new(primary)).unwrap(),
            Rule::DataExprFalse => Mcrl2Parser::DataExprFalse(Node::new(primary)).unwrap(),
            Rule::DataExprEmptyList => Mcrl2Parser::DataExprEmptyList(Node::new(primary)).unwrap(),
            Rule::DataExprEmptySet => Mcrl2Parser::DataExprEmptySet(Node::new(primary)).unwrap(),
            Rule::DataExprEmptyBag => Mcrl2Parser::DataExprEmptyBag(Node::new(primary)).unwrap(),
            Rule::DataExprListEnum => Mcrl2Parser::DataExprListEnum(Node::new(primary)).unwrap(),
            Rule::DataExprBagEnum => Mcrl2Parser::DataExprBagEnum(Node::new(primary)).unwrap(),
            Rule::DataExprSetBagComp => Mcrl2Parser::DataExprSetBagComp(Node::new(primary)).unwrap(),
            Rule::DataExprSetEnum => Mcrl2Parser::DataExprSetEnum(Node::new(primary)).unwrap(),
            Rule::Number => Mcrl2Parser::Number(Node::new(primary)).unwrap(),
            Rule::Id => DataExpr::Id(Mcrl2Parser::Id(Node::new(primary)).unwrap()),

            Rule::DataExprBrackets => {
                // Handle parentheses by recursively parsing the inner expression
                let inner = primary.into_inner().next().unwrap();
                parse_dataexpr(inner.into_inner())
            }

            _ => unimplemented!("Unexpected rule: {:?}", primary.as_rule()),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::DataExprConj => DataExprBinaryOp::Conj,
                Rule::DataExprDisj => DataExprBinaryOp::Disj,
                Rule::DataExprEq => DataExprBinaryOp::Equal,
                Rule::DataExprNeq => DataExprBinaryOp::NotEqual,
                Rule::DataExprLess => DataExprBinaryOp::LessThan,
                Rule::DataExprLeq => DataExprBinaryOp::LessEqual,
                Rule::DataExprGreater => DataExprBinaryOp::GreaterThan,
                Rule::DataExprGeq => DataExprBinaryOp::GreaterEqual,
                Rule::DataExprIn => DataExprBinaryOp::In,
                Rule::DataExprCons => DataExprBinaryOp::Cons,
                Rule::DataExprSnoc => DataExprBinaryOp::Snoc,
                Rule::DataExprConcat => DataExprBinaryOp::Concat,
                Rule::DataExprAdd => DataExprBinaryOp::Add,
                Rule::DataExprSubtract => DataExprBinaryOp::Subtract,
                Rule::DataExprDiv => DataExprBinaryOp::Div,
                Rule::DataExprIntDiv => DataExprBinaryOp::IntDiv,
                Rule::DataExprMod => DataExprBinaryOp::Mod,
                Rule::DataExprMult => DataExprBinaryOp::Multiply,
                Rule::DataExprAt => DataExprBinaryOp::At,
                Rule::DataExprImpl => DataExprBinaryOp::Implies,
                _ => unimplemented!("Unexpected binary operator rule: {:?}", op.as_rule()),
            };

            DataExpr::BinaryOperator {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        })
        .map_postfix(|expr, postfix| match postfix.as_rule() {
            Rule::DataExprUpdate => DataExpr::FunctionUpdate {
                expr: Box::new(expr),
                update: Mcrl2Parser::DataExprUpdate(Node::new(postfix)).unwrap(),
            },
            Rule::DataExprApplication => DataExpr::Application {
                function: Box::new(expr),
                arguments: Mcrl2Parser::DataExprApplication(Node::new(postfix)).unwrap(),
            },
            Rule::DataExprWhr => DataExpr::Whr {
                expr: Box::new(expr),
                assignments: Mcrl2Parser::DataExprWhr(Node::new(postfix)).unwrap(),
            },
            _ => unimplemented!("Unexpected postfix operator: {:?}", postfix.as_rule()),
        })
        .map_prefix(|prefix, expr| match prefix.as_rule() {
            Rule::DataExprForall => DataExpr::Forall {
                variables: Mcrl2Parser::DataExprForall(Node::new(prefix)).unwrap(),
                body: Box::new(expr),
            },
            Rule::DataExprExists => DataExpr::Exists {
                variables: Mcrl2Parser::DataExprExists(Node::new(prefix)).unwrap(),
                body: Box::new(expr),
            },
            Rule::DataExprLambda => DataExpr::Lambda {
                variables: Mcrl2Parser::DataExprLambda(Node::new(prefix)).unwrap(),
                body: Box::new(expr),
            },
            Rule::DataExprNegation => DataExpr::UnaryOperator {
                op: DataExprUnaryOp::Negation,
                expr: Box::new(expr),
            },
            Rule::DataExprMinus => DataExpr::UnaryOperator {
                op: DataExprUnaryOp::Minus,
                expr: Box::new(expr),
            },
            Rule::DataExprSize => DataExpr::UnaryOperator {
                op: DataExprUnaryOp::Size,
                expr: Box::new(expr),
            },
            _ => unimplemented!("Unexpected prefix operator: {:?}", prefix.as_rule()),
        })
        .parse(pairs)
}

static _PROCEXPR_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::infix(Rule::ProcExprChoice, Assoc::Left)) // $left 1
        .op(Op::prefix(Rule::ProcExprSum) | Op::prefix(Rule::ProcExprDist)) // $right 2
        .op(Op::infix(Rule::ProcExprParallel, Assoc::Right)) // $right 3
        .op(Op::infix(Rule::ProcExprLeftMerge, Assoc::Right)) // $right 4
        .op(Op::infix(Rule::ProcExprIf, Assoc::Right)) // $right 5
        .op(Op::infix(Rule::ProcExprIfThen, Assoc::Right)) // $right 5
        .op(Op::infix(Rule::ProcExprUntil, Assoc::Left)) // $left 6
        .op(Op::infix(Rule::ProcExprSeq, Assoc::Right)) // $right 7
        .op(Op::infix(Rule::ProcExprAt, Assoc::Left)) // $left 8
        .op(Op::infix(Rule::ProcExprComm, Assoc::Left)) // $left 9
});

static _REGFRM_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::infix(Rule::RegFrmAlternative, Assoc::Left)) // $left 1
        .op(Op::infix(Rule::RegFrmComposition, Assoc::Right)) // $right 2
        .op(Op::postfix(Rule::RegFrmIteration) | Op::postfix(Rule::RegFrmPlus)) // $left 3

});

static _PBESEXPR_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::prefix(Rule::PbesExprForall) | Op::prefix(Rule::PbesExprExists)) // $right 0
        .op(Op::infix(Rule::PbesExprImplies, Assoc::Right)) // $right 2
        .op(Op::infix(Rule::PbesExprDisj, Assoc::Right)) // $right 3
        .op(Op::infix(Rule::PbesExprConj, Assoc::Right)) // $right 4
        .op(Op::prefix(Rule::PbesExprNegation)) // $right 5
});

static _PRESEXPR_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::prefix(Rule::PresExprInf) 
            | Op::prefix(Rule::PresExprSup)
            | Op::prefix(Rule::PresExprSum)) // $right 0
        .op(Op::infix(Rule::PresExprAdd, Assoc::Right)) // $right 2
        .op(Op::infix(Rule::PbesExprImplies, Assoc::Right)) // $right 3
        .op(Op::infix(Rule::PbesExprDisj, Assoc::Right)) // $right 4
        .op(Op::infix(Rule::PbesExprConj, Assoc::Right)) // $right 5
        .op(Op::prefix(Rule::PresExprLeftConstantMultiply) | Op::postfix(Rule::PresExprRightConstMultiply)) // $right 6
        .op(Op::prefix(Rule::PbesExprNegation)) // $right 7
});

static _STATEFRM_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::prefix(Rule::StateFrmMu) | Op::prefix(Rule::StateFrmNu)) // $right 1
        .op(Op::prefix(Rule::StateFrmForall) 
            | Op::prefix(Rule::StateFrmExists)
            | Op::prefix(Rule::StateFrmInf)
            | Op::prefix(Rule::StateFrmSup)
            | Op::prefix(Rule::StateFrmSum)) // $right 2
        .op(Op::infix(Rule::StateFrmAddition, Assoc::Left)) // $left 3
        .op(Op::infix(Rule::StateFrmImplication, Assoc::Right)) // $right 4
        .op(Op::infix(Rule::StateFrmDisjunction, Assoc::Right)) // $right 5
        .op(Op::infix(Rule::StateFrmConjunction, Assoc::Right)) // $right 6
        .op(Op::prefix(Rule::StateFrmLeftConstantMultiply) | Op::postfix(Rule::StateFrmRightConstantMultiply)) // $right 7
        .op(Op::prefix(Rule::StateFrmBox) | Op::postfix(Rule::StateFrmDiamond)) // $right 8
        .op(Op::prefix(Rule::StateFrmNegation) | Op::prefix(Rule::StateFrmUnaryMinus)) // $right 9
});



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
