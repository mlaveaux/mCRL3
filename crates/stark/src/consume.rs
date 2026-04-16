#![allow(clippy::result_large_err)]

use std::collections::HashMap;

use merc_pest_consume::Error;
use merc_pest_consume::match_nodes;

use crate::StarkParser;
use crate::ast::{Constant, Expression, Identifier, Range, StarkSpecification, Ty, Variable};
use crate::parse::Rule;
use crate::precedence::{
    parse_distance_expression, parse_expression, parse_perturbation_expression, parse_robtl_formula,
};

/// Type alias for Errors resulting parsing.
pub(crate) type ParseResult<T> = std::result::Result<T, Error<Rule>>;
pub(crate) type ParseNode<'i> = merc_pest_consume::Node<'i, Rule, ()>;

#[merc_pest_consume::parser]
impl StarkParser {
    pub fn StarkSpecification(input: ParseNode) -> ParseResult<StarkSpecification> {
        let controllers = HashMap::new();

        for child in input.into_children() {
            match child.as_rule() {
                Rule::DeclarationComponent => {
                    // TODO: component parsing is not implemented yet.
                }
                Rule::DeclarationParameter => {}
                Rule::DeclarationConstant => {
                    let _ = Self::DeclarationConstant(child)?;
                }
                Rule::DeclarationVariables => {
                    let _ = Self::DeclarationVariables(child)?;
                }
                Rule::DeclarationType => {}
                Rule::DeclarationEnvironment => {
                    Self::DeclarationEnvironment(child)?;
                }
                Rule::DeclarationPenalty => {}
                Rule::DeclarationFunction => {}
                Rule::DeclarationPerturbation => {
                    Self::DeclarationPerturbation(child)?;
                }
                Rule::DeclarationDistance => {
                    Self::DeclarationDistance(child)?;
                }
                Rule::DeclarationFormula => {
                    Self::DeclarationFormula(child)?;
                }
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }

        Ok(StarkSpecification { controllers })
    }

    fn DeclarationConstant(input: ParseNode) -> ParseResult<Constant> {
        match_nodes!(input.into_children();
            [ID(id), Expression(expr)] => {
                Ok(Constant::new(id, expr))
            }
        )
    }

    fn ID(input: ParseNode) -> ParseResult<Identifier> {
        let span = input.as_span();
        Ok(Identifier::new(input.as_str().to_string(), span.into()))
    }

    fn DeclarationVariables(input: ParseNode) -> ParseResult<Vec<Variable>> {
        match_nodes!(input.into_children();
            [VariableDeclaration(vars)..] => {
                Ok(vars.collect())                    
            }
        )
    }

    fn VariableDeclaration(input: ParseNode) -> ParseResult<Variable> {
        match_nodes!(input.into_children();
            [Ty(ty), ID(id), Expression(min), Expression(max), Expression(initial)] => {
                Ok(Variable::new(
                    false,
                    ty,
                    id,
                    Some(Range::new(Some(min), Some(max))),
                    initial,
                ))
            },
            [Ty(ty), ID(id), Expression(initial)] => {
                Ok(Variable::new(false, ty, id, None, initial))
            }
        )
    }

    fn DeclarationEnvironment(input: ParseNode) -> ParseResult<()> {
        match_nodes!(input.into_children();
            [EnvironmentBlock(block)] => {
                let _ = block;
                Ok(())
            }
        )
    }

    fn DeclarationPerturbation(input: ParseNode) -> ParseResult<()> {
        match_nodes!(input.into_children();
            [ID(_), PerturbationExpression(_)] => Ok(())
        )
    }

    fn DeclarationDistance(input: ParseNode) -> ParseResult<()> {
        match_nodes!(input.into_children();
            [ID(_), DistanceExpression(_)] => Ok(())
        )
    }

    fn DeclarationFormula(input: ParseNode) -> ParseResult<()> {
        match_nodes!(input.into_children();
            [ID(_), RobtlFormula(_)] => Ok(())
        )
    }

    fn EnvironmentBlock(input: ParseNode) -> ParseResult<()> {
        for child in input.into_children() {
            Self::EnvironmentCommand(child)?;
        }
        Ok(())
    }

    fn EnvironmentCommand(_input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn Ty(input: ParseNode) -> ParseResult<Ty> {
        Ok(match input.as_str() {
            "int" => Ty::Integer,
            "real" => Ty::Real,
            "bool" => Ty::Boolean,
            // User-defined types are not yet represented in the AST type enum.
            _ => Ty::Integer,
        })
    }

    pub(crate) fn Expression(input: ParseNode) -> ParseResult<Expression> {
        parse_expression(input.children().as_pairs().clone())
    }

    fn PerturbationExpression(input: ParseNode) -> ParseResult<()> {
        parse_perturbation_expression(input.children().as_pairs().clone())
    }

    fn DistanceExpression(input: ParseNode) -> ParseResult<()> {
        parse_distance_expression(input.children().as_pairs().clone())
    }

    fn RobtlFormula(input: ParseNode) -> ParseResult<()> {
        parse_robtl_formula(input.children().as_pairs().clone())
    }
}