//! Pratt parsers for the STARK sub-languages.
//!
//! The `pest` grammar only produces a flat `prefix* primary postfix* (infix ...)*`
//! token stream for each expression language; these parsers turn that stream into
//! the priority/associativity-resolved AST defined in `ast.rs`.

use std::sync::LazyLock;

use pest::error::ErrorVariant;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::pratt_parser::Assoc;
use pest::pratt_parser::Op;
use pest::pratt_parser::PrattParser;

use merc_pest_consume::Error;

use crate::ast::BinaryOp;
use crate::ast::ComparisonOp;
use crate::ast::DistanceExpression;
use crate::ast::Expression;
use crate::ast::Identifier;
use crate::ast::MathFunction;
use crate::ast::PerturbationAssignment;
use crate::ast::PerturbationExpression;
use crate::ast::RobtlFormula;
use crate::consume::ParseResult;
use crate::parse::Rule;

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

fn identifier(pair: &Pair<'_, Rule>) -> Identifier {
    Identifier::new(pair.as_str().to_string(), pair.as_span().into())
}

fn error<T>(pair: &Pair<'_, Rule>, message: impl Into<String>) -> ParseResult<T> {
    Err(Error::new_from_span(
        ErrorVariant::CustomError {
            message: message.into(),
        },
        pair.as_span(),
    ))
}

/// Parse an `Expression` node's children with the expression Pratt parser.
#[allow(clippy::result_large_err)]
fn parse_expression_node(pair: Pair<'_, Rule>) -> ParseResult<Expression> {
    parse_expression(pair.into_inner())
}

/// Collect the `Expression` children of a node and parse each.
#[allow(clippy::result_large_err)]
fn expression_arguments(pair: Pair<'_, Rule>) -> ParseResult<Vec<Expression>> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::Expression)
        .map(parse_expression_node)
        .collect()
}

fn math_function(name: &str) -> MathFunction {
    match name {
        "abs" => MathFunction::Abs,
        "acos" => MathFunction::Acos,
        "asin" => MathFunction::Asin,
        "atan" => MathFunction::Atan,
        "cbrt" => MathFunction::Cbrt,
        "ceil" => MathFunction::Ceil,
        "cos" => MathFunction::Cos,
        "cosh" => MathFunction::Cosh,
        "exp" => MathFunction::Exp,
        "expm1" => MathFunction::Expm1,
        "floor" => MathFunction::Floor,
        "log" => MathFunction::Log,
        "log10" => MathFunction::Log10,
        "log1p" => MathFunction::Log1p,
        "signum" => MathFunction::Signum,
        "sin" => MathFunction::Sin,
        "sinh" => MathFunction::Sinh,
        "sqrt" => MathFunction::Sqrt,
        "tan" => MathFunction::Tan,
        "atan2" => MathFunction::Atan2,
        "hypot" => MathFunction::Hypot,
        "max" => MathFunction::Max,
        "min" => MathFunction::Min,
        "pow" => MathFunction::Pow,
        other => unreachable!("unknown math function: {other}"),
    }
}

fn comparison_op(text: &str) -> ComparisonOp {
    match text {
        "<" => ComparisonOp::Less,
        "<=" => ComparisonOp::Leq,
        "==" => ComparisonOp::Eq,
        ">=" => ComparisonOp::Geq,
        ">" => ComparisonOp::Greater,
        other => unreachable!("unknown comparison operator: {other}"),
    }
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

pub static EXPRESSION_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    // Precedence is defined lowest (loosest) to highest (tightest).
    PrattParser::new()
        .op(Op::infix(Rule::ExpressionOr, Assoc::Left))
        .op(Op::infix(Rule::ExpressionBitOr, Assoc::Left))
        .op(Op::infix(Rule::ExpressionAnd, Assoc::Left))
        .op(Op::infix(Rule::ExpressionBitAnd, Assoc::Left))
        .op(Op::infix(Rule::ExpressionLess, Assoc::Left)
            | Op::infix(Rule::ExpressionLeq, Assoc::Left)
            | Op::infix(Rule::ExpressionEq, Assoc::Left)
            | Op::infix(Rule::ExpressionGeq, Assoc::Left)
            | Op::infix(Rule::ExpressionGreater, Assoc::Left))
        .op(Op::infix(Rule::ExpressionAdd, Assoc::Left) | Op::infix(Rule::ExpressionSubtract, Assoc::Left))
        .op(Op::infix(Rule::ExpressionMult, Assoc::Left)
            | Op::infix(Rule::ExpressionDiv, Assoc::Left)
            | Op::infix(Rule::ExpressionIntDiv, Assoc::Left)
            | Op::infix(Rule::ExpressionMod, Assoc::Left))
        .op(Op::infix(Rule::ExpressionPow, Assoc::Right))
        .op(Op::prefix(Rule::ExpressionNot)
            | Op::prefix(Rule::ExpressionUnaryPlus)
            | Op::prefix(Rule::ExpressionUnaryMinus))
        .op(Op::postfix(Rule::ExpressionCall) | Op::postfix(Rule::ExpressionTernary))
});

#[allow(clippy::result_large_err)]
fn parse_expression_primary(primary: Pair<'_, Rule>) -> ParseResult<Expression> {
    match primary.as_rule() {
        // Parenthesized sub-expression.
        Rule::Expression => parse_expression_node(primary),
        Rule::INTEGER => match primary.as_str().parse::<i64>() {
            Ok(value) => Ok(Expression::Integer(value)),
            Err(_) => error(
                &primary,
                format!(
                    "integer literal `{}` does not fit in a 64-bit integer",
                    primary.as_str()
                ),
            ),
        },
        Rule::REAL => match primary.as_str().parse::<f64>() {
            Ok(value) => Ok(Expression::Real(value)),
            Err(_) => error(&primary, format!("invalid real literal `{}`", primary.as_str())),
        },
        Rule::ID => Ok(Expression::Identifier(primary.as_str().to_string())),
        Rule::ExpressionTrue => Ok(Expression::True),
        Rule::ExpressionFalse => Ok(Expression::False),
        Rule::ExpressionIterator => Ok(Expression::Iterator),
        Rule::ExpressionNormal => {
            let mut args = expression_arguments(primary)?.into_iter();
            Ok(Expression::Normal {
                mean: Box::new(args.next().expect("normal distribution requires a mean")),
                std_dev: Box::new(args.next().expect("normal distribution requires a std dev")),
            })
        }
        Rule::ExpressionUniform => Ok(Expression::Uniform {
            values: expression_arguments(primary)?,
        }),
        Rule::ExpressionRandom => {
            let mut args = expression_arguments(primary)?.into_iter();
            Ok(Expression::Range {
                min: args.next().map(Box::new),
                max: args.next().map(Box::new),
            })
        }
        Rule::ExpressionUnaryMathCall | Rule::ExpressionBinaryMathCall => {
            let mut children = primary.into_inner();
            let function = math_function(
                children
                    .next()
                    .expect("math call should contain a function name")
                    .as_str(),
            );
            let arguments = children
                .filter(|p| p.as_rule() == Rule::Expression)
                .map(parse_expression_node)
                .collect::<ParseResult<Vec<_>>>()?;
            Ok(Expression::MathCall { function, arguments })
        }
        rule => unreachable!("unexpected expression primary: {rule:?}"),
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
            rule => unreachable!("unexpected expression prefix operator: {rule:?}"),
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
                rule => unreachable!("unexpected expression binary operator: {rule:?}"),
            };
            Ok(Expression::Binary(op, Box::new(lhs?), Box::new(rhs?)))
        })
        .map_postfix(|target, postfix| match postfix.as_rule() {
            Rule::ExpressionCall => Ok(Expression::Call {
                function: Box::new(target?),
                arguments: expression_arguments(postfix)?,
            }),
            Rule::ExpressionTernary => {
                let mut branches = expression_arguments(postfix)?.into_iter();
                Ok(Expression::Ternary {
                    guard: Box::new(target?),
                    then_branch: Box::new(branches.next().expect("ternary requires a then branch")),
                    else_branch: Box::new(branches.next().expect("ternary requires an else branch")),
                })
            }
            rule => unreachable!("unexpected expression postfix operator: {rule:?}"),
        })
        .parse(pairs)
}

// ---------------------------------------------------------------------------
// Perturbation expressions
// ---------------------------------------------------------------------------

pub static PERTURBATION_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::PerturbationSemicolon, Assoc::Left))
        .op(Op::postfix(Rule::PerturbationPow))
});

#[allow(clippy::result_large_err)]
fn parse_perturbation_primary(primary: Pair<'_, Rule>) -> ParseResult<PerturbationExpression> {
    match primary.as_rule() {
        Rule::PerturbationExpression => parse_perturbation_expression(primary.into_inner()),
        Rule::PerturbationNil => Ok(PerturbationExpression::Nil),
        Rule::ID => Ok(PerturbationExpression::Reference(identifier(&primary))),
        Rule::PerturbationAtomic => {
            let mut assignments = Vec::new();
            let mut time = None;
            for child in primary.into_inner() {
                match child.as_rule() {
                    Rule::PerturbationAssignment => {
                        let mut inner = child.into_inner();
                        let id = identifier(&inner.next().expect("assignment target"));
                        let value = parse_expression_node(inner.next().expect("assignment value"))?;
                        assignments.push(PerturbationAssignment { id, value });
                    }
                    Rule::Expression => time = Some(parse_expression_node(child)?),
                    rule => unreachable!("unexpected perturbation atomic child: {rule:?}"),
                }
            }
            Ok(PerturbationExpression::Atomic {
                assignments,
                time: time.expect("atomic perturbation requires an @time"),
            })
        }
        rule => unreachable!("unexpected perturbation primary: {rule:?}"),
    }
}

#[allow(clippy::result_large_err)]
pub fn parse_perturbation_expression(pairs: Pairs<Rule>) -> ParseResult<PerturbationExpression> {
    PERTURBATION_PRATT_PARSER
        .map_primary(parse_perturbation_primary)
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::PerturbationSemicolon => Ok(PerturbationExpression::Sequence(Box::new(lhs?), Box::new(rhs?))),
            rule => unreachable!("unexpected perturbation infix operator: {rule:?}"),
        })
        .map_postfix(|argument, postfix| match postfix.as_rule() {
            Rule::PerturbationPow => {
                let iterations = parse_expression_node(
                    postfix
                        .into_inner()
                        .find(|p| p.as_rule() == Rule::Expression)
                        .expect("iteration requires an exponent expression"),
                )?;
                Ok(PerturbationExpression::Iteration {
                    argument: Box::new(argument?),
                    iterations,
                })
            }
            rule => unreachable!("unexpected perturbation postfix operator: {rule:?}"),
        })
        .parse(pairs)
}

// ---------------------------------------------------------------------------
// Distance expressions
// ---------------------------------------------------------------------------

pub static DISTANCE_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::DistanceInfixUntil, Assoc::Left))
        .op(Op::postfix(Rule::DistancePostfixThreshold))
        .op(Op::prefix(Rule::DistancePrefixF) | Op::prefix(Rule::DistancePrefixG))
});

/// Parse the two `Expression` children (`from`, `to`) of an interval operator.
#[allow(clippy::result_large_err)]
fn parse_interval(pair: Pair<'_, Rule>) -> ParseResult<(Expression, Expression)> {
    let mut args = expression_arguments(pair)?.into_iter();
    Ok((
        args.next().expect("interval requires a lower bound"),
        args.next().expect("interval requires an upper bound"),
    ))
}

#[allow(clippy::result_large_err)]
fn parse_distance_primary(primary: Pair<'_, Rule>) -> ParseResult<DistanceExpression> {
    match primary.as_rule() {
        Rule::DistanceExpression => parse_distance_expression(primary.into_inner()),
        Rule::DistanceAtomicLeft => Ok(DistanceExpression::AtomicLeft(identifier(
            &primary.into_inner().next().expect("penalty reference"),
        ))),
        Rule::DistanceAtomicRight => Ok(DistanceExpression::AtomicRight(identifier(
            &primary.into_inner().next().expect("penalty reference"),
        ))),
        Rule::ID => Ok(DistanceExpression::Reference(identifier(&primary))),
        Rule::DistanceMin => {
            let (left, right) = parse_distance_pair(primary)?;
            Ok(DistanceExpression::Min(Box::new(left), Box::new(right)))
        }
        Rule::DistanceMax => {
            let (left, right) = parse_distance_pair(primary)?;
            Ok(DistanceExpression::Max(Box::new(left), Box::new(right)))
        }
        Rule::DistanceLinearCombination => {
            let mut terms = Vec::new();
            let mut children = primary.into_inner().peekable();
            while let Some(weight_pair) = children.next() {
                let weight = parse_expression_node(weight_pair)?;
                let distance_pair = children.next().expect("linear combination term requires a distance");
                let distance = parse_distance_expression(distance_pair.into_inner())?;
                terms.push((weight, distance));
            }
            Ok(DistanceExpression::LinearCombination(terms))
        }
        rule => unreachable!("unexpected distance primary: {rule:?}"),
    }
}

/// Parse the two `DistanceExpression` children of `min(..)` / `max(..)`.
#[allow(clippy::result_large_err)]
fn parse_distance_pair(pair: Pair<'_, Rule>) -> ParseResult<(DistanceExpression, DistanceExpression)> {
    let mut children = pair.into_inner().filter(|p| p.as_rule() == Rule::DistanceExpression);
    let left = parse_distance_expression(children.next().expect("first argument").into_inner())?;
    let right = parse_distance_expression(children.next().expect("second argument").into_inner())?;
    Ok((left, right))
}

#[allow(clippy::result_large_err)]
pub fn parse_distance_expression(pairs: Pairs<Rule>) -> ParseResult<DistanceExpression> {
    DISTANCE_PRATT_PARSER
        .map_primary(parse_distance_primary)
        .map_prefix(|op, rhs| {
            let (from, to) = parse_interval(op.clone())?;
            let argument = Box::new(rhs?);
            match op.as_rule() {
                Rule::DistancePrefixF => Ok(DistanceExpression::Eventually { from, to, argument }),
                Rule::DistancePrefixG => Ok(DistanceExpression::Globally { from, to, argument }),
                rule => unreachable!("unexpected distance prefix operator: {rule:?}"),
            }
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::DistanceInfixUntil => {
                let (from, to) = parse_interval(op)?;
                Ok(DistanceExpression::Until {
                    from,
                    to,
                    left: Box::new(lhs?),
                    right: Box::new(rhs?),
                })
            }
            rule => unreachable!("unexpected distance infix operator: {rule:?}"),
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::DistancePostfixThreshold => {
                let mut children = op.into_inner();
                let comparison = comparison_op(children.next().expect("threshold operator").as_str());
                let threshold = parse_expression_node(children.next().expect("threshold value"))?;
                Ok(DistanceExpression::Threshold {
                    op: comparison,
                    left: Box::new(lhs?),
                    threshold,
                })
            }
            rule => unreachable!("unexpected distance postfix operator: {rule:?}"),
        })
        .parse(pairs)
}

// ---------------------------------------------------------------------------
// ROBTL formulas
// ---------------------------------------------------------------------------

pub static ROBTL_PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::RobtlOr, Assoc::Left))
        .op(Op::infix(Rule::RobtlAnd, Assoc::Left))
        .op(Op::infix(Rule::RobtlUntil, Assoc::Left))
        .op(Op::prefix(Rule::RobtlNot) | Op::prefix(Rule::RobtlGlobally) | Op::prefix(Rule::RobtlEventually))
});

#[allow(clippy::result_large_err)]
fn parse_robtl_primary(primary: Pair<'_, Rule>) -> ParseResult<RobtlFormula> {
    match primary.as_rule() {
        Rule::RobtlFormula => parse_robtl_formula(primary.into_inner()),
        Rule::RobtlTrue => Ok(RobtlFormula::True),
        Rule::RobtlFalse => Ok(RobtlFormula::False),
        Rule::ID => Ok(RobtlFormula::Reference(identifier(&primary))),
        Rule::RobtlDistance => {
            let mut children = primary.into_inner();
            let distance = identifier(&children.next().expect("distance reference"));
            let perturbation = identifier(&children.next().expect("perturbation reference"));
            let op = comparison_op(children.next().expect("comparison operator").as_str());
            let value = parse_expression_node(children.next().expect("threshold value"))?;
            Ok(RobtlFormula::Distance {
                distance,
                perturbation,
                op,
                value,
            })
        }
        rule => unreachable!("unexpected ROBTL primary: {rule:?}"),
    }
}

#[allow(clippy::result_large_err)]
pub fn parse_robtl_formula(pairs: Pairs<Rule>) -> ParseResult<RobtlFormula> {
    ROBTL_PRATT_PARSER
        .map_primary(parse_robtl_primary)
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::RobtlNot => Ok(RobtlFormula::Not(Box::new(rhs?))),
            Rule::RobtlGlobally => {
                let (from, to) = parse_interval(op)?;
                Ok(RobtlFormula::Globally {
                    from,
                    to,
                    argument: Box::new(rhs?),
                })
            }
            Rule::RobtlEventually => {
                let (from, to) = parse_interval(op)?;
                Ok(RobtlFormula::Eventually {
                    from,
                    to,
                    argument: Box::new(rhs?),
                })
            }
            rule => unreachable!("unexpected ROBTL prefix operator: {rule:?}"),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::RobtlAnd => Ok(RobtlFormula::And(Box::new(lhs?), Box::new(rhs?))),
            Rule::RobtlOr => Ok(RobtlFormula::Or(Box::new(lhs?), Box::new(rhs?))),
            Rule::RobtlUntil => {
                let (from, to) = parse_interval(op)?;
                Ok(RobtlFormula::Until {
                    from,
                    to,
                    left: Box::new(lhs?),
                    right: Box::new(rhs?),
                })
            }
            rule => unreachable!("unexpected ROBTL infix operator: {rule:?}"),
        })
        .parse(pairs)
}

#[cfg(test)]
mod tests {
    use crate::ast::BinaryOp;
    use crate::ast::DistanceExpression;
    use crate::ast::Expression;
    use crate::ast::MathFunction;
    use crate::ast::PerturbationExpression;
    use crate::ast::RobtlFormula;
    use crate::ast::StarkSpecification;
    use crate::ast::Ty;

    /// Parse `const c = <src>;` and return the parsed expression.
    fn expr(src: &str) -> Expression {
        let spec = StarkSpecification::parse(&format!("const c = {src};")).expect("should parse");
        spec.constants.into_iter().next().expect("one constant").value
    }

    #[test]
    fn arithmetic_precedence() {
        // `1 + 2 * 3` must group as `1 + (2 * 3)`.
        match expr("1 + 2 * 3") {
            Expression::Binary(BinaryOp::Add, lhs, rhs) => {
                assert!(matches!(*lhs, Expression::Integer(1)));
                assert!(matches!(*rhs, Expression::Binary(BinaryOp::Mult, _, _)));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn power_is_right_associative() {
        // `2 ^ 3 ^ 2` must group as `2 ^ (3 ^ 2)`.
        match expr("2 ^ 3 ^ 2") {
            Expression::Binary(BinaryOp::Pow, lhs, rhs) => {
                assert!(matches!(*lhs, Expression::Integer(2)));
                assert!(matches!(*rhs, Expression::Binary(BinaryOp::Pow, _, _)));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn comparison_binds_looser_than_bitand() {
        // `a > b & c` must group as `(a > b) & c` (relations tighter than `&`).
        match expr("a > b & c") {
            Expression::Binary(BinaryOp::BitAnd, lhs, _) => {
                assert!(matches!(*lhs, Expression::Binary(BinaryOp::Greater, _, _)));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn unary_minus_and_not() {
        assert!(matches!(expr("-x"), Expression::UnaryMinus(_)));
        assert!(matches!(expr("!x"), Expression::Not(_)));
    }

    #[test]
    fn math_calls_and_user_calls() {
        match expr("max(1, 2)") {
            Expression::MathCall {
                function: MathFunction::Max,
                arguments,
            } => assert_eq!(arguments.len(), 2),
            other => panic!("unexpected: {other:?}"),
        }
        match expr("abs(x)") {
            Expression::MathCall {
                function: MathFunction::Abs,
                arguments,
            } => assert_eq!(arguments.len(), 1),
            other => panic!("unexpected: {other:?}"),
        }
        // A non-builtin name is a user call, not a math call.
        assert!(matches!(expr("eval_bd(x)"), Expression::Call { .. }));
    }

    #[test]
    fn identifiers_starting_with_keyword_prefixes() {
        // `italic`/`Rate` must be identifiers, not `it` / `R` followed by junk.
        assert!(matches!(expr("italic"), Expression::Identifier(name) if name == "italic"));
        assert!(matches!(expr("Rate"), Expression::Identifier(name) if name == "Rate"));
    }

    #[test]
    fn ternary() {
        assert!(matches!(expr("a ? b : c"), Expression::Ternary { .. }));
    }

    #[test]
    fn distributions() {
        assert!(matches!(expr("N[0, 1]"), Expression::Normal { .. }));
        assert!(matches!(expr("U[1, 2, 3]"), Expression::Uniform { values } if values.len() == 3));
        assert!(matches!(
            expr("R[0, 1]"),
            Expression::Range {
                min: Some(_),
                max: Some(_)
            }
        ));
        assert!(matches!(expr("R"), Expression::Range { min: None, max: None }));
    }

    #[test]
    fn integer_overflow_is_an_error() {
        assert!(StarkSpecification::parse("const c = 99999999999999999999999;").is_err());
    }

    #[test]
    fn variable_with_range_and_type() {
        let spec =
            StarkSpecification::parse("global variables { int counter range [0, 10] = 0; }").expect("should parse");
        let var = &spec.variables[0];
        assert!(var.global);
        assert!(matches!(var.ty, Ty::Integer));
        assert!(var.range.is_some());
        assert_eq!(var.id.name, "counter");
    }

    #[test]
    fn perturbation_sequence_and_iteration() {
        let spec = StarkSpecification::parse("perturbation p = ([x <- 1]@0); ([y <- 2]@0)^3;").expect("should parse");
        // `a ; b^3` groups as `a ; (b^3)`.
        match &spec.perturbations[0].value {
            PerturbationExpression::Sequence(_, right) => {
                assert!(matches!(**right, PerturbationExpression::Iteration { .. }));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn distance_and_formula() {
        let spec = StarkSpecification::parse("distance d = \\G[0, 10] < rho;\nformula f = \\D[d, p] <= 5;")
            .expect("should parse");
        assert!(matches!(spec.distances[0].value, DistanceExpression::Globally { .. }));
        assert!(matches!(spec.formulas[0].value, RobtlFormula::Distance { .. }));
    }
}
