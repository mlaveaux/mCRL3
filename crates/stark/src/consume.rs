#![allow(clippy::result_large_err)]

use merc_pest_consume::Error;
use merc_pest_consume::match_nodes;

use crate::StarkParser;
use crate::ast::Component;
use crate::ast::Constant;
use crate::ast::ControllerCommand;
use crate::ast::ControllerState;
use crate::ast::DefRef;
use crate::ast::Distance;
use crate::ast::Environment;
use crate::ast::EnvironmentCommand;
use crate::ast::Formula;
use crate::ast::Function;
use crate::ast::FunctionArgument;
use crate::ast::FunctionStatement;
use crate::ast::Identifier;
use crate::ast::LocalVariable;
use crate::ast::Parameter;
use crate::ast::Penalty;
use crate::ast::Perturbation;
use crate::ast::Range;
use crate::ast::SpannedExpression;
use crate::ast::StateRef;
use crate::ast::Ty;
use crate::ast::TypeDeclaration;
use crate::ast::UntypedStarkSpecification;
use crate::ast::Update;
use crate::ast::Variable;
use crate::parse::Rule;
use crate::precedence::parse_distance_expression;
use crate::precedence::parse_expression_node;
use crate::precedence::parse_perturbation_expression;
use crate::precedence::parse_robtl_formula;

/// Type alias for Errors resulting from parsing.
pub(crate) type ParseResult<T> = std::result::Result<T, Error<Rule>>;
pub(crate) type ParseNode<'i> = merc_pest_consume::Node<'i, Rule, ()>;

// ---------------------------------------------------------------------------
// Dispatch helpers for silent alternation groups.
//
// The grammar's `FunctionStatement`, `ControllerCommand` and `EnvironmentCommand`
// rules are silent, so their concrete variant nodes appear directly as children.
// These helpers route a variant node to its consumer.
// ---------------------------------------------------------------------------

fn function_statement(node: ParseNode) -> ParseResult<FunctionStatement> {
    match node.as_rule() {
        Rule::FunctionReturn => StarkParser::FunctionReturn(node),
        Rule::FunctionIfThenElse => StarkParser::FunctionIfThenElse(node),
        Rule::FunctionBlock => StarkParser::FunctionBlock(node),
        Rule::FunctionLet => StarkParser::FunctionLet(node),
        rule => unreachable!("unexpected function statement: {rule:?}"),
    }
}

fn controller_command(node: ParseNode) -> ParseResult<ControllerCommand> {
    match node.as_rule() {
        Rule::ControllerStep => StarkParser::ControllerStep(node),
        Rule::ControllerExec => StarkParser::ControllerExec(node),
        Rule::ControllerLet => StarkParser::ControllerLet(node),
        Rule::ControllerAssignment => StarkParser::ControllerAssignment(node),
        Rule::ControllerIfThenElse => StarkParser::ControllerIfThenElse(node),
        Rule::ControllerBlock => Ok(ControllerCommand::Block(StarkParser::ControllerBlock(node)?)),
        rule => unreachable!("unexpected controller command: {rule:?}"),
    }
}

fn environment_command(node: ParseNode) -> ParseResult<EnvironmentCommand> {
    match node.as_rule() {
        Rule::EnvironmentAssignment => StarkParser::EnvironmentAssignment(node),
        Rule::EnvironmentIfThenElse => StarkParser::EnvironmentIfThenElse(node),
        Rule::EnvironmentLet => StarkParser::EnvironmentLet(node),
        Rule::EnvironmentBlock => Ok(EnvironmentCommand::Block(StarkParser::EnvironmentBlock(node)?)),
        rule => unreachable!("unexpected environment command: {rule:?}"),
    }
}

/// Consume a `[when guard] target' = value;` assignment shared by controllers and
/// the environment.
fn assignment_update(node: ParseNode) -> ParseResult<Update> {
    let mut guard = None;
    let mut target = None;
    let mut value = None;

    for child in node.into_children() {
        match child.as_rule() {
            Rule::WhenGuard => guard = Some(StarkParser::WhenGuard(child)?),
            Rule::NEXT_ID => target = Some(DefRef::new(StarkParser::NEXT_ID(child)?)),
            Rule::Expression => value = Some(StarkParser::Expression(child)?),
            rule => unreachable!("unexpected assignment child: {rule:?}"),
        }
    }

    Ok(Update {
        guard,
        target: target.expect("assignment requires a target"),
        value: value.expect("assignment requires a value"),
    })
}

#[merc_pest_consume::parser]
impl StarkParser {
    pub fn UntypedStarkSpecification(input: ParseNode) -> ParseResult<UntypedStarkSpecification> {
        let mut spec = UntypedStarkSpecification::new();

        for child in input.into_children() {
            match child.as_rule() {
                Rule::DeclarationConstant => spec.constants.push(Self::DeclarationConstant(child)?),
                Rule::DeclarationParameter => spec.parameters.push(Self::DeclarationParameter(child)?),
                Rule::DeclarationVariables => spec.variables.extend(Self::DeclarationVariables(child)?),
                Rule::DeclarationType => spec.types.push(Self::DeclarationType(child)?),
                Rule::DeclarationFunction => spec.functions.push(Self::DeclarationFunction(child)?),
                Rule::DeclarationComponent => spec.components.push(Self::DeclarationComponent(child)?),
                Rule::DeclarationEnvironment => spec.environment = Some(Self::DeclarationEnvironment(child)?),
                Rule::DeclarationPenalty => spec.penalties.push(Self::DeclarationPenalty(child)?),
                Rule::DeclarationPerturbation => spec.perturbations.push(Self::DeclarationPerturbation(child)?),
                Rule::DeclarationDistance => spec.distances.push(Self::DeclarationDistance(child)?),
                Rule::DeclarationFormula => spec.formulas.push(Self::DeclarationFormula(child)?),
                Rule::EOI => {}
                rule => unreachable!("unexpected top-level declaration: {rule:?}"),
            }
        }

        Ok(spec)
    }

    // --- Leaf tokens -------------------------------------------------------

    fn ID(input: ParseNode) -> ParseResult<Identifier> {
        let span = input.as_span();
        Ok(Identifier::new(input.as_str().to_string(), span.into()))
    }

    fn NEXT_ID(input: ParseNode) -> ParseResult<Identifier> {
        let span = input.as_span();
        // Strip the trailing `'` from the primed variable name.
        let name = input.as_str().trim_end_matches('\'').to_string();
        Ok(Identifier::new(name, span.into()))
    }

    fn Ty(input: ParseNode) -> ParseResult<Ty> {
        let child = input.into_children().next().expect("Ty has a single variant child");
        Ok(match child.as_rule() {
            Rule::TyInt => Ty::Integer,
            Rule::TyReal => Ty::Real,
            Rule::TyBool => Ty::Boolean,
            Rule::TyCustom => Ty::Named(child.as_str().to_string()),
            rule => unreachable!("unexpected type: {rule:?}"),
        })
    }

    pub(crate) fn Expression(input: ParseNode) -> ParseResult<SpannedExpression> {
        parse_expression_node(input.into_pair())
    }

    fn WhenGuard(input: ParseNode) -> ParseResult<SpannedExpression> {
        match_nodes!(input.into_children();
            [Expression(guard)] => Ok(guard)
        )
    }

    // --- Simple declarations ----------------------------------------------

    fn DeclarationConstant(input: ParseNode) -> ParseResult<Constant> {
        match_nodes!(input.into_children();
            [ID(name), Expression(value)] => Ok(Constant { id: None, name, value })
        )
    }

    fn DeclarationParameter(input: ParseNode) -> ParseResult<Parameter> {
        match_nodes!(input.into_children();
            [ID(name), Expression(value)] => Ok(Parameter { id: None, name, value })
        )
    }

    fn DeclarationPenalty(input: ParseNode) -> ParseResult<Penalty> {
        match_nodes!(input.into_children();
            [ID(name), Expression(value)] => Ok(Penalty { id: None, name, value })
        )
    }

    fn DeclarationType(input: ParseNode) -> ParseResult<TypeDeclaration> {
        match_nodes!(input.into_children();
            [ID(name), TypeElement(elements)..] => Ok(TypeDeclaration { id: None, name, elements: elements.collect() })
        )
    }

    fn TypeElement(input: ParseNode) -> ParseResult<Identifier> {
        match_nodes!(input.into_children();
            [ID(id)] => Ok(id)
        )
    }

    // --- Variables ---------------------------------------------------------

    fn DeclarationVariables(input: ParseNode) -> ParseResult<Vec<Variable>> {
        let mut global = false;
        let mut variables = Vec::new();

        for child in input.into_children() {
            match child.as_rule() {
                Rule::GlobalMarker => global = true,
                Rule::VariableDeclaration => variables.push(Self::VariableDeclaration(child)?),
                rule => unreachable!("unexpected variables child: {rule:?}"),
            }
        }

        for variable in &mut variables {
            variable.global = global;
        }

        Ok(variables)
    }

    fn VariableDeclaration(input: ParseNode) -> ParseResult<Variable> {
        match_nodes!(input.into_children();
            [Ty(ty), ID(name), VariableRange(range), Expression(initial_value)] => {
                Ok(Variable { id: None, global: false, ty, name, range: Some(range), initial_value })
            },
            [Ty(ty), ID(name), Expression(initial_value)] => {
                Ok(Variable { id: None, global: false, ty, name, range: None, initial_value })
            }
        )
    }

    fn VariableRange(input: ParseNode) -> ParseResult<Range> {
        match_nodes!(input.into_children();
            [Expression(min), Expression(max)] => Ok(Range { min, max })
        )
    }

    // --- Functions ---------------------------------------------------------

    fn DeclarationFunction(input: ParseNode) -> ParseResult<Function> {
        let mut name = None;
        let mut arguments = Vec::new();
        let mut body = None;

        for child in input.into_children() {
            match child.as_rule() {
                Rule::ID => name = Some(Self::ID(child)?),
                Rule::FunctionArgument => arguments.push(Self::FunctionArgument(child)?),
                Rule::FunctionBlock => body = Some(Self::FunctionBlock(child)?),
                rule => unreachable!("unexpected function child: {rule:?}"),
            }
        }

        Ok(Function {
            id: None,
            name: name.expect("function requires a name"),
            arguments,
            body: body.expect("function requires a body"),
        })
    }

    fn FunctionArgument(input: ParseNode) -> ParseResult<FunctionArgument> {
        match_nodes!(input.into_children();
            [Ty(ty), ID(name)] => Ok(FunctionArgument { id: None, ty, name })
        )
    }

    fn FunctionReturn(input: ParseNode) -> ParseResult<FunctionStatement> {
        match_nodes!(input.into_children();
            [Expression(value)] => Ok(FunctionStatement::Return(value))
        )
    }

    fn FunctionLet(input: ParseNode) -> ParseResult<FunctionStatement> {
        let mut children = input.into_children();
        let name = Self::ID(children.next().expect("let name"))?;
        let value = Self::Expression(children.next().expect("let value"))?;
        let body = function_statement(children.next().expect("let body"))?;
        Ok(FunctionStatement::Let {
            id: None,
            name,
            value,
            body: Box::new(body),
        })
    }

    fn FunctionIfThenElse(input: ParseNode) -> ParseResult<FunctionStatement> {
        let mut children = input.into_children();
        let guard = Self::Expression(children.next().expect("if guard"))?;
        let then_branch = Box::new(function_statement(children.next().expect("then branch"))?);
        let else_branch = children.next().map(function_statement).transpose()?.map(Box::new);
        Ok(FunctionStatement::IfThenElse {
            guard,
            then_branch,
            else_branch,
        })
    }

    fn FunctionBlock(input: ParseNode) -> ParseResult<FunctionStatement> {
        let mut children = input.into_children();
        let inner = function_statement(children.next().expect("block body"))?;
        Ok(FunctionStatement::Block(Box::new(inner)))
    }

    // --- Components and controllers ---------------------------------------

    fn DeclarationComponent(input: ParseNode) -> ParseResult<Component> {
        let mut name = None;
        let mut variables = Vec::new();
        let mut states = Vec::new();
        let mut init = Vec::new();

        for child in input.into_children() {
            match child.as_rule() {
                Rule::ID => name = Some(Self::ID(child)?),
                Rule::VariableDeclaration => variables.push(Self::VariableDeclaration(child)?),
                Rule::ControllerState => states.push(Self::ControllerState(child)?),
                Rule::ControllerExpression => init = Self::ControllerExpression(child)?,
                rule => unreachable!("unexpected component child: {rule:?}"),
            }
        }

        Ok(Component {
            id: None,
            name: name.expect("component requires a name"),
            variables,
            states,
            init,
        })
    }

    fn ControllerExpression(input: ParseNode) -> ParseResult<Vec<StateRef>> {
        match_nodes!(input.into_children();
            [ID(states)..] => Ok(states.map(StateRef::new).collect())
        )
    }

    fn ControllerState(input: ParseNode) -> ParseResult<ControllerState> {
        let mut children = input.into_children();
        let name = Self::ID(children.next().expect("state name"))?;
        let body = Self::ControllerBlock(children.next().expect("state body"))?;
        Ok(ControllerState { id: None, name, body })
    }

    fn ControllerBlock(input: ParseNode) -> ParseResult<Vec<ControllerCommand>> {
        input.into_children().map(controller_command).collect()
    }

    fn ControllerStep(input: ParseNode) -> ParseResult<ControllerCommand> {
        let mut steps = None;
        let mut target = None;
        for child in input.into_children() {
            match child.as_rule() {
                Rule::Expression => steps = Some(Self::Expression(child)?),
                Rule::ID => target = Some(StateRef::new(Self::ID(child)?)),
                rule => unreachable!("unexpected step child: {rule:?}"),
            }
        }
        Ok(ControllerCommand::Step {
            steps,
            target: target.expect("step requires a target"),
        })
    }

    fn ControllerExec(input: ParseNode) -> ParseResult<ControllerCommand> {
        match_nodes!(input.into_children();
            [ID(target)] => Ok(ControllerCommand::Exec(StateRef::new(target)))
        )
    }

    fn ControllerLet(input: ParseNode) -> ParseResult<ControllerCommand> {
        let mut children = input.into_children();
        let name = Self::ID(children.next().expect("let name"))?;
        let value = Self::Expression(children.next().expect("let value"))?;
        let body = Self::ControllerBlock(children.next().expect("let body"))?;
        Ok(ControllerCommand::Let {
            id: None,
            name,
            value,
            body,
        })
    }

    fn ControllerAssignment(input: ParseNode) -> ParseResult<ControllerCommand> {
        Ok(ControllerCommand::Assignment(assignment_update(input)?))
    }

    fn ControllerIfThenElse(input: ParseNode) -> ParseResult<ControllerCommand> {
        let mut children = input.into_children();
        let guard = Self::Expression(children.next().expect("if guard"))?;
        let then_branch = Self::ControllerBlock(children.next().expect("then branch"))?;
        let else_branch = children.next().map(Self::ControllerBlock).transpose()?;
        Ok(ControllerCommand::IfThenElse {
            guard,
            then_branch,
            else_branch,
        })
    }

    // --- Environment -------------------------------------------------------

    fn DeclarationEnvironment(input: ParseNode) -> ParseResult<Environment> {
        let mut children = input.into_children();
        let commands = Self::EnvironmentBlock(children.next().expect("environment block"))?;
        Ok(Environment { commands })
    }

    fn EnvironmentBlock(input: ParseNode) -> ParseResult<Vec<EnvironmentCommand>> {
        input.into_children().map(environment_command).collect()
    }

    fn EnvironmentAssignment(input: ParseNode) -> ParseResult<EnvironmentCommand> {
        Ok(EnvironmentCommand::Assignment(assignment_update(input)?))
    }

    fn EnvironmentIfThenElse(input: ParseNode) -> ParseResult<EnvironmentCommand> {
        let mut children = input.into_children();
        let guard = Self::Expression(children.next().expect("if guard"))?;
        let then_branch = Box::new(environment_command(children.next().expect("then branch"))?);
        let else_branch = children.next().map(environment_command).transpose()?.map(Box::new);
        Ok(EnvironmentCommand::IfThenElse {
            guard,
            then_branch,
            else_branch,
        })
    }

    fn EnvironmentLet(input: ParseNode) -> ParseResult<EnvironmentCommand> {
        let mut bindings = Vec::new();
        let mut body = None;
        for child in input.into_children() {
            match child.as_rule() {
                Rule::LocalVariable => bindings.push(Self::LocalVariable(child)?),
                _ => body = Some(environment_command(child)?),
            }
        }
        Ok(EnvironmentCommand::Let {
            bindings,
            body: Box::new(body.expect("let requires a body")),
        })
    }

    fn LocalVariable(input: ParseNode) -> ParseResult<LocalVariable> {
        match_nodes!(input.into_children();
            [ID(name), Expression(value)] => Ok(LocalVariable { id: None, name, value })
        )
    }

    // --- Robustness sub-languages -----------------------------------------

    fn DeclarationPerturbation(input: ParseNode) -> ParseResult<Perturbation> {
        let mut children = input.into_children();
        let name = Self::ID(children.next().expect("perturbation name"))?;
        let value = Self::PerturbationExpression(children.next().expect("perturbation value"))?;
        Ok(Perturbation { id: None, name, value })
    }

    fn PerturbationExpression(input: ParseNode) -> ParseResult<crate::ast::PerturbationExpression> {
        parse_perturbation_expression(input.children().as_pairs().clone())
    }

    fn DeclarationDistance(input: ParseNode) -> ParseResult<Distance> {
        let mut children = input.into_children();
        let name = Self::ID(children.next().expect("distance name"))?;
        let value = Self::DistanceExpression(children.next().expect("distance value"))?;
        Ok(Distance { id: None, name, value })
    }

    fn DistanceExpression(input: ParseNode) -> ParseResult<crate::ast::DistanceExpression> {
        parse_distance_expression(input.children().as_pairs().clone())
    }

    fn DeclarationFormula(input: ParseNode) -> ParseResult<Formula> {
        let mut children = input.into_children();
        let name = Self::ID(children.next().expect("formula name"))?;
        let value = Self::RobtlFormula(children.next().expect("formula value"))?;
        Ok(Formula { id: None, name, value })
    }

    fn RobtlFormula(input: ParseNode) -> ParseResult<crate::ast::RobtlFormula> {
        parse_robtl_formula(input.children().as_pairs().clone())
    }
}
