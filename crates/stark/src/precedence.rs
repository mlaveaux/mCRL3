use std::sync::LazyLock;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::pratt_parser::Assoc;
use pest::pratt_parser::Op;
use pest::pratt_parser::PrattParser;

use crate::ast::AggregateOp;
use crate::ast::BinaryOp;
use crate::ast::Expression;
use crate::parse::Rule;
use crate::consume::ParseResult;

pub static EXPRESSION_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::infix(Rule::ExpressionOr, Assoc::Left))
        .op(Op::infix(Rule::ExpressionBitOr, Assoc::Left))
        .op(Op::infix(Rule::ExpressionAnd, Assoc::Left))
        .op(Op::infix(Rule::ExpressionBitAnd, Assoc::Left))
        .op(
            Op::infix(Rule::ExpressionLess, Assoc::Left)
                | Op::infix(Rule::ExpressionLeq, Assoc::Left)
                | Op::infix(Rule::ExpressionEq, Assoc::Left)
                | Op::infix(Rule::ExpressionGeq, Assoc::Left)
                | Op::infix(Rule::ExpressionGreater, Assoc::Left),
        )
        .op(Op::infix(Rule::ExpressionAdd, Assoc::Left) | Op::infix(Rule::ExpressionSubtract, Assoc::Left))
        .op(
            Op::infix(Rule::ExpressionMult, Assoc::Left)
                | Op::infix(Rule::ExpressionDiv, Assoc::Left)
                | Op::infix(Rule::ExpressionIntDiv, Assoc::Left)
                | Op::infix(Rule::ExpressionMod, Assoc::Left),
        )
        .op(Op::infix(Rule::ExpressionPow, Assoc::Right))
        .op(
            Op::prefix(Rule::ExpressionNot)
                | Op::prefix(Rule::ExpressionUnaryPlus)
                | Op::prefix(Rule::ExpressionUnaryMinus),
        )
        .op(Op::postfix(Rule::ExpressionCall) | Op::postfix(Rule::ExpressionAggregate))
});

#[allow(clippy::result_large_err)]
fn parse_expression_primary(primary: Pair<'_, Rule>) -> ParseResult<Expression> {
    match primary.as_rule() {
        Rule::Expression => parse_expression(primary.into_inner()),
        Rule::INTEGER => Ok(Expression::Integer(
            primary.as_str().parse::<i64>().expect("INTEGER token should parse as i64"),
        )),
        Rule::REAL => Ok(Expression::Real(
            primary.as_str().parse::<f64>().expect("REAL token should parse as f64"),
        )),
        Rule::ID => Ok(Expression::Identifier(primary.as_str().to_string())),
        _ => {
            let text = primary.as_str();
            if text == "false" {
                return Ok(Expression::False);
            }
            if text == "true" {
                return Ok(Expression::True);
            }
            if text == "it" {
                return Ok(Expression::Iterator);
            }

            let args: Vec<Expression> = primary
                .clone()
                .into_inner()
                .filter(|pair| pair.as_rule() == Rule::Expression)
                .map(|pair| parse_expression(pair.into_inner()))
                .collect::<Result<Vec<_>, _>>()?;

            if text.starts_with("N[") && args.len() == 2 {
                let mut args = args.into_iter();
                return Ok(Expression::Normal {
                    mean: Box::new(args.next().expect("normal distribution requires mean")),
                    std_dev: Box::new(args.next().expect("normal distribution requires std_dev")),
                });
            }

            if text.starts_with("U[") {
                return Ok(Expression::Uniform { values: args });
            }

            if text.starts_with('R') {
                let mut args = args.into_iter();
                return Ok(Expression::Range {
                    min: args.next().map(Box::new),
                    max: args.next().map(Box::new),
                });
            }

            Ok(Expression::Identifier(text.to_string()))
        }
    }
}

#[allow(clippy::result_large_err)]
pub fn parse_expression(pairs: Pairs<Rule>) -> ParseResult<Expression> {
    EXPRESSION_PRATT_PARSER
        .map_primary(parse_expression_primary)
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::ExpressionNot => Ok(Expression::Not(Box::new(rhs?))),
            Rule::ExpressionUnaryPlus => Ok(Expression::UnaryPlus(Box::new(rhs?))),
            Rule::ExpressionUnaryMinus => Ok(Expression::UnaryMinus(Box::new(rhs?))),
            _ => unimplemented!("Unexpected expression prefix operator: {:?}", op.as_rule()),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::ExpressionPow => BinaryOp::Pow,
                Rule::ExpressionMult => BinaryOp::Mult,
                Rule::ExpressionDiv => BinaryOp::Div,
                Rule::ExpressionIntDiv => BinaryOp::IntDiv,
                Rule::ExpressionAdd => BinaryOp::Add,
                Rule::ExpressionSubtract => BinaryOp::Subtract,
                Rule::ExpressionMod => BinaryOp::Mod,
                Rule::ExpressionLess => BinaryOp::Less,
                Rule::ExpressionLeq => BinaryOp::Leq,
                Rule::ExpressionEq => BinaryOp::Eq,
                Rule::ExpressionGeq => BinaryOp::Geq,
                Rule::ExpressionGreater => BinaryOp::Greater,
                Rule::ExpressionBitAnd => BinaryOp::BitAnd,
                Rule::ExpressionAnd => BinaryOp::And,
                Rule::ExpressionBitOr => BinaryOp::BitOr,
                Rule::ExpressionOr => BinaryOp::Or,
                _ => unimplemented!("Unexpected expression binary operator: {:?}", op.as_rule()),
            };

            Ok(Expression::Binary(op, Box::new(lhs?), Box::new(rhs?)))
        })
        .map_postfix(|target, postfix| match postfix.as_rule() {
            Rule::ExpressionCall => {
                let arguments = postfix
                    .into_inner()
                    .filter(|pair| pair.as_rule() == Rule::Expression)
                    .map(|pair| parse_expression(pair.into_inner()))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Expression::Call {
                    function: Box::new(target?),
                    arguments,
                })
            }
            Rule::ExpressionAggregate => {
                let mut children = postfix.into_inner();
                let op = match children
                    .next()
                    .expect("ExpressionAggregate should always contain an op")
                    .as_str()
                {
                    "count" => AggregateOp::Count,
                    "min" => AggregateOp::Min,
                    "max" => AggregateOp::Max,
                    "mean" => AggregateOp::Mean,
                    x => unimplemented!("Unknown aggregate op: {x}"),
                };

                let argument = children
                    .find(|pair| pair.as_rule() == Rule::Expression)
                    .map(|pair| parse_expression(pair.into_inner()))
                    .transpose()?
                    .map(Box::new);

                Ok(Expression::Aggregate {
                    target: Box::new(target?),
                    op,
                    argument,
                })
            }
            _ => unimplemented!("Unexpected expression postfix operator: {:?}", postfix.as_rule()),
        })
        .parse(pairs)
}

pub static PERTURBATION_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::postfix(Rule::PerturbationPostfix))
});

#[allow(clippy::result_large_err)]
pub fn parse_perturbation_expression(pairs: Pairs<Rule>) -> ParseResult<()> {
    PERTURBATION_PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::PerturbationExpression => parse_perturbation_expression(primary.into_inner()),
            Rule::PerturbationPrimary => Ok(()),
            _ => Ok(()),
        })
        .map_postfix(|expr, _| {
            expr?;
            Ok(())
        })
        .parse(pairs)
}

pub static DISTANCE_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::postfix(Rule::DistancePostfix))
        .op(Op::infix(Rule::DistanceInfix, Assoc::Left))
        .op(Op::prefix(Rule::DistancePrefix))
});

#[allow(clippy::result_large_err)]
pub fn parse_distance_expression(pairs: Pairs<Rule>) -> ParseResult<()> {
    DISTANCE_PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::DistanceExpression => parse_distance_expression(primary.into_inner()),
            Rule::DistancePrimary => Ok(()),
            _ => Ok(()),
        })
        .map_prefix(|_, expr| {
            expr?;
            Ok(())
        })
        .map_postfix(|expr, _| {
            expr?;
            Ok(())
        })
        .map_infix(|lhs, _, rhs| {
            lhs?;
            rhs?;
            Ok(())
        })
        .parse(pairs)
}

pub static ROBTL_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::RobtlInfix, Assoc::Left))
        .op(Op::prefix(Rule::RobtlPrefix))
});

#[allow(clippy::result_large_err)]
pub fn parse_robtl_formula(pairs: Pairs<Rule>) -> ParseResult<()> {
    ROBTL_PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::RobtlFormula => parse_robtl_formula(primary.into_inner()),
            Rule::RobtlPrimary => Ok(()),
            _ => Ok(()),
        })
        .map_prefix(|_, expr| {
            expr?;
            Ok(())
        })
        .map_infix(|lhs, _, rhs| {
            lhs?;
            rhs?;
            Ok(())
        })
        .parse(pairs)
}
