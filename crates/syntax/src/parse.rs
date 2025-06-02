use std::iter;

use log::trace;
use pest::Parser;
use pest_consume::Error;
use pest_consume::match_nodes;

use mcrl3_utilities::MCRL3Error;

use crate::parse_actfrm;
use crate::parse_process_expr;
use crate::parse_regfrm;
use crate::parse_statefrm;
use crate::ActFrm;
use crate::Action;
use crate::Assignment;
use crate::Comm;
use crate::ComplexSort;
use crate::ConstructorDecl;
use crate::DataExpr;
use crate::DataExprUpdate;
use crate::DisplayPair;
use crate::EqnDecl;
use crate::EqnSpec;
use crate::IdDecl;
use crate::Mcrl2Parser;
use crate::MultiAction;
use crate::MultiActionLabel;
use crate::ProcessExpr;
use crate::RegFrm;
use crate::Rename;
use crate::Rule;
use crate::SortExpression;
use crate::StateFrm;
use crate::StateFrmSpec;
use crate::UntypedDataSpecification;
use crate::UntypedProcessSpecification;
use crate::VarDecl;
use crate::parse_dataexpr;
use crate::parse_sortexpr;

/// Parses the given mCRL2 specification into an AST.
impl UntypedProcessSpecification {
    pub fn parse(spec: &str) -> Result<UntypedProcessSpecification, MCRL3Error> {
        pest::set_error_detail(true);

        let mut result = Mcrl2Parser::parse(Rule::MCRL2Spec, spec)?;
        let root = result.next().ok_or("Could not parse mCRL2 specification")?;
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Ok(Mcrl2Parser::MCRL2Spec(ParseNode::new(root))?)
    }
}

/// Parses the given mCRL2 specification into an AST.
impl UntypedDataSpecification {
    pub fn parse(spec: &str) -> Result<UntypedDataSpecification, MCRL3Error> {
        pest::set_error_detail(true);

        let mut result = Mcrl2Parser::parse(Rule::DataSpec, spec)?;
        let root = result.next().ok_or("Could not parse mCRL2 data specification")?;
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Ok(Mcrl2Parser::DataSpec(ParseNode::new(root))?)
    }
}

impl StateFrmSpec {
    pub fn parse(spec: &str) -> Result<StateFrmSpec, MCRL3Error> {
        pest::set_error_detail(true);

        let mut result = Mcrl2Parser::parse(Rule::StateFrmSpec, spec)?;
        let root = result.next().ok_or("Could not parse mCRL2 state formula specification")?;
        trace!("Parse tree {}", DisplayPair(root.clone()));

        Ok(Mcrl2Parser::StateFrmSpec(ParseNode::new(root))?)
    }
}

/// Type alias for Errors resulting parsing.
pub(crate) type ParseResult<T> = std::result::Result<T, Error<Rule>>;
pub(crate) type ParseNode<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl Mcrl2Parser {
    pub fn MCRL2Spec(spec: ParseNode) -> ParseResult<UntypedProcessSpecification> {
        let mut map = Vec::new();
        let mut eqn = Vec::new();

        for child in spec.into_children() {
            match child.as_rule() {
                Rule::MapSpec => {
                    map.append(&mut Mcrl2Parser::MapSpec(child)?);
                }
                Rule::EqnSpec => {
                    eqn.append(&mut Mcrl2Parser::EqnSpec(child)?);
                }
                _ => {
                    // Handle other rules if necessary
                }
            }
        }

        let data_specification = UntypedDataSpecification {
            map_decls: map,
            ..Default::default()
        };

        Ok(UntypedProcessSpecification {
            data_specification,
            ..Default::default()
        })
    }

    fn DataSpec(spec: ParseNode) -> ParseResult<UntypedDataSpecification> {
        let mut map = Vec::new();
        let mut eqn = Vec::new();

        for child in spec.into_children() {
            match child.as_rule() {
                Rule::MapSpec => {
                    map.append(&mut Mcrl2Parser::MapSpec(child)?);
                }
                Rule::EqnSpec => {
                    eqn.append(&mut Mcrl2Parser::EqnSpec(child)?);
                }
                _ => {
                    // Handle other rules if necessary
                }
            }
        }

        Ok(UntypedDataSpecification {
            map_decls: map,
            eqn_decls: eqn,
            ..Default::default()
        })
    }

    fn MapSpec(spec: ParseNode) -> ParseResult<Vec<IdDecl>> {
        let mut ids = Vec::new();

        for decl in spec.into_children() {
            ids.append(&mut Mcrl2Parser::IdsDecl(decl)?);
        }

        Ok(ids)
    }

    fn IdsDecl(decl: ParseNode) -> ParseResult<Vec<IdDecl>> {
        let span = decl.as_span();
        match_nodes!(decl.into_children();
            [IdList(identifiers), SortExpr(sort)] => {
                let id_decls = identifiers.into_iter().map(|identifier| {
                    IdDecl { identifier, sort: sort.clone(), span: span.into() }
                }).collect();

                Ok(id_decls)
            },
        )
    }

    fn EqnSpec(spec: ParseNode) -> ParseResult<Vec<EqnSpec>> {
        let mut ids = Vec::new();

        match_nodes!(spec.into_children();
            [VarSpec(variables), EqnDecl(decls)..] => {
                ids.push(EqnSpec { variables, equations: decls.collect() });
            },
            [EqnDecl(decls)..] => {
                ids.push(EqnSpec { variables: Vec::new(), equations: decls.collect() });
            },
        );

        Ok(ids)
    }

    fn EqnDecl(decl: ParseNode) -> ParseResult<EqnDecl> {
        let span = decl.as_span();
        match_nodes!(decl.into_children();
            [DataExpr(condition), DataExpr(lhs), DataExpr(rhs)] => {
                Ok(EqnDecl { condition: Some(condition), lhs, rhs, span: span.into() })
            },
            [DataExpr(lhs), DataExpr(rhs)] => {
                Ok(EqnDecl { condition: None, lhs, rhs, span: span.into() })
            },
        )
    }

    pub(crate) fn DataExpr(expr: ParseNode) -> ParseResult<DataExpr> {
        parse_dataexpr(expr.children().as_pairs().clone())
    }

    pub(crate) fn DataExprUpdate(expr: ParseNode) -> ParseResult<DataExprUpdate> {
        match_nodes!(expr.into_children();
            [DataExpr(expr), DataExpr(update)] => {
                Ok(DataExprUpdate { expr: Box::new(expr), update: Box::new(update) })
            },
        )
    }

    pub(crate) fn DataExprApplication(expr: ParseNode) -> ParseResult<Vec<DataExpr>> {
        match_nodes!(expr.into_children();
            [DataExprList(expressions)] => {
                return Ok(expressions);
            },
        )
    }

    pub(crate) fn DataExprWhr(expr: ParseNode) -> ParseResult<Vec<Assignment>> {
        match_nodes!(expr.into_children();
            [AssignmentList(assignments)] => {
                Ok(assignments)
            },
        )
    }

    pub(crate) fn AssignmentList(assignments: ParseNode) -> ParseResult<Vec<Assignment>> {
        match_nodes!(assignments.into_children();
            [Assignment(assignment)] => {
                return Ok(vec![assignment]);
            },
            [Assignment(assignment)..] => {
                return Ok(assignment.collect());
            },
        );
    }

    pub(crate) fn Assignment(assignment: ParseNode) -> ParseResult<Assignment> {
        match_nodes!(assignment.into_children();
            [Id(identifier), DataExpr(expr)] => {
                Ok(Assignment { identifier, expr: Box::new(expr) })
            },
        )
    }

    pub(crate) fn DataExprSize(expr: ParseNode) -> ParseResult<DataExpr> {
        match_nodes!(expr.into_children();
            [DataExpr(expr)] => {
                return Ok(DataExpr::Size(Box::new(expr)));
            },
        )
    }

    fn DataExprList(expr: ParseNode) -> ParseResult<Vec<DataExpr>> {
        match_nodes!(expr.into_children();
            [DataExpr(expr)] => {
                return Ok(vec![expr]);
            },
            [DataExpr(expr)..] => {
                return Ok(expr.collect());
            },
        );
    }

    fn VarSpec(vars: ParseNode) -> ParseResult<Vec<VarDecl>> {
        match_nodes!(vars.into_children();
            [VarsDeclList(ids)..] => {
                // Flatten the iterator of Vec<VarDecl> into a single Vec<VarDecl>
                let mut result = Vec::new();
                for id_vec in ids {
                    result.extend(id_vec);
                }
                return Ok(result);
            },
        );
    }

    pub(crate) fn VarsDeclList(vars: ParseNode) -> ParseResult<Vec<VarDecl>> {
        match_nodes!(vars.into_children();
            [VarsDecl(decl)..] => {
                // Flatten the iterator of Vec<VarDecl> into a single Vec<VarDecl>
                return Ok(decl.flatten().collect());
            },
        );
    }

    fn VarsDecl(decl: ParseNode) -> ParseResult<Vec<VarDecl>> {
        let mut vars = Vec::new();

        let span = decl.as_span();
        match_nodes!(decl.into_children();
            [IdList(identifier), SortExpr(sort)] => {
                for id in identifier {
                    vars.push(VarDecl { identifier: id, sort: sort.clone(), span: span.into() });
                }
            },
        );

        Ok(vars)
    }

    pub fn SortExpr(expr: ParseNode) -> ParseResult<SortExpression> {
        parse_sortexpr(expr.children().as_pairs().clone())
    }

    pub fn Id(identifier: ParseNode) -> ParseResult<String> {
        Ok(identifier.as_str().to_string())
    }

    pub fn IdList(identifiers: ParseNode) -> ParseResult<Vec<String>> {
        match_nodes!(identifiers.into_children();
            [Id(ids)..] => {
                return Ok(ids.collect());
            },
        );
    }

    // Complex sorts
    pub fn SortExprList(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::List,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())?),
        ))
    }

    pub fn SortExprSet(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::Set,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())?),
        ))
    }

    pub fn SortExprBag(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::Bag,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())?),
        ))
    }

    pub fn SortExprFSet(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::FSet,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())?),
        ))
    }

    pub fn SortExprFBag(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::FBag,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())?),
        ))
    }

    pub fn SortExprStruct(inner: ParseNode) -> ParseResult<SortExpression> {
        match_nodes!(inner.into_children();
            [ConstrDeclList(inner)] => {
                return Ok(SortExpression::Struct { inner });
            },
        );
    }

    pub fn ConstrDeclList(input: ParseNode) -> ParseResult<Vec<ConstructorDecl>> {
        match_nodes!(input.into_children();
            [ConstrDecl(decl)..] => {
                return Ok(decl.collect());
            },
        );
    }

    pub fn ProjDeclList(input: ParseNode) -> ParseResult<Vec<(Option<String>, SortExpression)>> {
        match_nodes!(input.into_children();
            [ProjDecl(decl)..] => {
                return Ok(decl.collect());
            },
        );
    }

    pub fn ConstrDecl(input: ParseNode) -> ParseResult<ConstructorDecl> {
        match_nodes!(input.into_children();
            [Id(name)] => {
                Ok(ConstructorDecl { name, args: Vec::new(), projection: None })
            },
            [Id(name), ProjDeclList(args)] => {
                Ok(ConstructorDecl { name, args, projection: None })
            },
            [Id(name), ProjDeclList(args), SortExpr(projection)] => {
                Ok(ConstructorDecl { name, args, projection: Some(projection) })
            },
        )
    }

    pub fn ProjDecl(input: ParseNode) -> ParseResult<(Option<String>, SortExpression)> {
        match_nodes!(input.into_children();
            [SortExpr(sort)] => {
                Ok((None, sort))
            },
            [Id(name), SortExpr(sort)] => {
                Ok((Some(name), sort))
            },
        )
    }

    pub(crate) fn DataExprListEnum(input: ParseNode) -> ParseResult<DataExpr> {
        match_nodes!(input.into_children();
            [DataExprList(expressions)] => {
                Ok(DataExpr::List(expressions))
            },
        )
    }

    pub(crate) fn DataExprBagEnum(input: ParseNode) -> ParseResult<DataExpr> {
        match_nodes!(input.into_children();
            [BagEnumEltList(elements)] => {
                Ok(DataExpr::Bag(elements))
            },
        )
    }

    fn BagEnumEltList(input: ParseNode) -> ParseResult<Vec<(DataExpr, DataExpr)>> {
        match_nodes!(input.into_children();
            [BagEnumElt(elements)..] => {
                return Ok(elements.collect());
            },
        )
    }

    fn BagEnumElt(input: ParseNode) -> ParseResult<(DataExpr, DataExpr)> {
        match_nodes!(input.into_children();
            [DataExpr(expr), DataExpr(amount)] => {
                return Ok((expr, amount));
            },
        )
    }

    pub(crate) fn DataExprSetEnum(input: ParseNode) -> ParseResult<DataExpr> {
        match_nodes!(input.into_children();
            [DataExprList(expressions)] => {
                Ok(DataExpr::Set(expressions))
            },
        )
    }

    pub(crate) fn DataExprSetBagComp(input: ParseNode) -> ParseResult<DataExpr> {
        match_nodes!(input.into_children();
            [VarDecl(variable), DataExpr(predicate)] => {
                Ok(DataExpr::SetBagComp { variable, predicate: Box::new(predicate) })
            },
        )
    }

    pub(crate) fn Number(input: ParseNode) -> ParseResult<DataExpr> {
        match_nodes!(input.into_children();
            [number] => {
                Ok(DataExpr::Number(number.as_str().into()))
            },
        )
    }

    fn VarDecl(decl: ParseNode) -> ParseResult<VarDecl> {
        let span = decl.as_span();
        match_nodes!(decl.into_children();
            [Id(identifier), SortExpr(sort)] => {
                Ok(VarDecl { identifier, sort, span: span.into() })
            },
        )
    }

    pub(crate) fn DataExprLambda(input: ParseNode) -> ParseResult<Vec<VarDecl>> {
        match_nodes!(input.into_children();
            [VarsDeclList(vars)] => {
                Ok(vars)
            },
        )
    }

    pub(crate) fn DataExprForall(input: ParseNode) -> ParseResult<Vec<VarDecl>> {
        match_nodes!(input.into_children();
            [VarsDeclList(vars)] => {
                Ok(vars)
            },
        )
    }

    pub(crate) fn DataExprExists(input: ParseNode) -> ParseResult<Vec<VarDecl>> {
        match_nodes!(input.into_children();
            [VarsDeclList(vars)] => {
                Ok(vars)
            },
        )
    }

    pub(crate) fn ActFrm(input: ParseNode) -> ParseResult<ActFrm> {
        parse_actfrm(input.children().as_pairs().clone())
    }

    fn ActIdSet(actions: ParseNode) -> ParseResult<Vec<String>> {
        match_nodes!(actions.into_children();
            [IdList(list)] => {
                return Ok(list);
            },
        );
    }

    fn MultActId(actions: ParseNode) -> ParseResult<MultiActionLabel> {
        match_nodes!(actions.into_children();
            [Id(action), Id(actions)..] => {
                return Ok(MultiActionLabel { actions: iter::once(action).chain(actions).collect() });
            },
        );
    }

    fn MultActIdList(actions: ParseNode) -> ParseResult<Vec<MultiActionLabel>> {
        match_nodes!(actions.into_children();
            [MultActId(action), MultActId(actions)..] => {
                return Ok(iter::once(action).chain(actions).collect());
            },
        );
    }

    fn MultActIdSet(actions: ParseNode) -> ParseResult<Vec<MultiActionLabel>> {
        match_nodes!(actions.into_children();
            [MultActIdList(list)] => {
                return Ok(list);
            },
        );
    }

    fn ProcExpr(input: ParseNode) -> ParseResult<ProcessExpr> {
        parse_process_expr(input.children().as_pairs().clone())
    }

    pub(crate) fn ProcExprBlock(input: ParseNode) -> ParseResult<ProcessExpr> {
        match_nodes!(input.into_children();
            [ActIdSet(actions), ProcExpr(expr)] => {
                Ok(ProcessExpr::Block {
                    actions,
                    operand: Box::new(expr),
                })
            },
        )
    }

    pub(crate) fn ProcExprAllow(input: ParseNode) -> ParseResult<ProcessExpr> {
        match_nodes!(input.into_children();
            [MultActIdSet(actions), ProcExpr(expr)] => {
                Ok(ProcessExpr::Allow {
                    actions,
                    operand: Box::new(expr),
                })
            },
        )
    }

    pub(crate) fn ProcExprHide(input: ParseNode) -> ParseResult<ProcessExpr> {
        match_nodes!(input.into_children();
            [ActIdSet(actions), ProcExpr(expr)] => {
                Ok(ProcessExpr::Hide {
                    actions,
                    operand: Box::new(expr),
                })
            },
        )
    }

    fn ActionList(actions: ParseNode) -> ParseResult<Vec<Action>> {
        match_nodes!(actions.into_children();
            [Action(action), Action(actions)..] => {
                return Ok(iter::once(action).chain(actions).collect());
            },
        );
    }

    fn Action(action: ParseNode) -> ParseResult<Action> {
        match_nodes!(action.into_children();
            [Id(id), DataExprList(args)] => {
                return Ok(Action { id, args });
            },
            [Id(id)] => {
                return Ok(Action { id, args: Vec::new() });
            },
        );
    }

    fn MultiActTau(_input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    pub(crate) fn MultAct(input: ParseNode) -> ParseResult<MultiAction> {
        match_nodes!(input.into_children();
            [MultiActTau(_)] => {
                return Ok(MultiAction { actions: Vec::new() });
            },
            [ActionList(actions)] => {
                return Ok(MultiAction { actions });
            },
        );
    }

    fn CommExpr(action: ParseNode) -> ParseResult<Comm> {
        match_nodes!(action.into_children();
            [Id(id), MultActId(multiact), Id(to)] => {
                let mut actions = vec![id];
                actions.extend(multiact.actions);

                return Ok(Comm {
                    from: MultiActionLabel { actions },
                    to
                });
            },
        );
    }

    fn CommExprList(actions: ParseNode) -> ParseResult<Vec<Comm>> {
        match_nodes!(actions.into_children();
            [CommExpr(action), CommExpr(actions)..] => {
                return Ok(iter::once(action).chain(actions).collect());
            },
        );
    }

    fn CommExprSet(actions: ParseNode) -> ParseResult<Vec<Comm>> {
        match_nodes!(actions.into_children();
            [CommExprList(list)] => {
                Ok(list)
            },
        )
    }

    pub(crate) fn ProcExprRename(input: ParseNode) -> ParseResult<ProcessExpr> {
        match_nodes!(input.into_children();
            [RenExprSet(renames), ProcExpr(expr)] => {
                Ok(ProcessExpr::Rename {
                    renames,
                    operand: Box::new(expr),
                })
            },
        )
    }

    pub(crate) fn ProcExprComm(input: ParseNode) -> ParseResult<ProcessExpr> {
        match_nodes!(input.into_children();
            [CommExprSet(comm), ProcExpr(expr)] => {
                Ok(ProcessExpr::Comm {
                    comm,
                    operand: Box::new(expr),
                })
            },
        )
    }

    fn RenExprSet(renames: ParseNode) -> ParseResult<Vec<Rename>> {
        match_nodes!(renames.into_children();
            [RenExprList(renames)] => {
                Ok(renames)
            },
        )
    }

    fn RenExprList(renames: ParseNode) -> ParseResult<Vec<Rename>> {
        match_nodes!(renames.into_children();
            [RenExpr(renames)..] => {
                return Ok(renames.collect());
            },
        );
    }

    fn RenExpr(renames: ParseNode) -> ParseResult<Rename> {
        match_nodes!(renames.into_children();
            [Id(from), Id(to)] => {
                return Ok(Rename { from, to });
            },
        );
    }

    pub(crate) fn ProcExprSum(input: ParseNode) -> ParseResult<Vec<VarDecl>> {
        match_nodes!(input.into_children();
            [VarsDeclList(variables)] => {
                Ok(variables)
            },
        )
    }

    pub(crate) fn ProcExprDist(input: ParseNode) -> ParseResult<(Vec<VarDecl>, DataExpr)> {
        match_nodes!(input.into_children();
            [VarsDeclList(variables), DataExpr(expr)] => {
                Ok((variables, expr))
            },
        )
    }


    pub(crate) fn StateFrmDelay(input: ParseNode) -> ParseResult<StateFrm> {
        match_nodes!(input.into_children();
            [DataExpr(delay)] => {
                return Ok(StateFrm::Delay(delay));
            },
        );
    }

    pub(crate) fn StateFrmYaled(input: ParseNode) -> ParseResult<StateFrm> {
        match_nodes!(input.into_children();
            [DataExpr(delay)] => {
                return Ok(StateFrm::Yaled(delay));
            },
        );
    }

    pub(crate) fn StateFrmNegation(input: ParseNode) -> ParseResult<StateFrm> {
        match_nodes!(input.into_children();
            [StateFrm(state)] => {
                return Ok(StateFrm::Negation(Box::new(state)));
            },
        );
    }

    pub(crate) fn StateFrmDataValExprMult(input: ParseNode) -> ParseResult<DataExpr> {
        match_nodes!(input.into_children();
            [DataExpr(expr)] => {
                return Ok(expr);
            },
        );
    }

    pub(crate) fn StateFrmRightConstantMultiply(input: ParseNode) -> ParseResult<DataExpr> {
        match_nodes!(input.into_children();
            [ DataExpr(expr)] => {
                return Ok(expr);
            },
        );
    }

    pub(crate) fn StateFrmDataValExpr(input: ParseNode) -> ParseResult<StateFrm> {
        match_nodes!(input.into_children();
            [DataExpr(expr)] => {
                return Ok(StateFrm::DataValExpr(expr));
            },
        );
    }

    pub(crate) fn StateFrmDiamond(input: ParseNode) -> ParseResult<RegFrm> {
        match_nodes!(input.into_children();
            [RegFrm(formula)] => {
                return Ok(formula);
            },
        );
    }

    pub(crate) fn StateFrmBox(input: ParseNode) -> ParseResult<RegFrm> {
        match_nodes!(input.into_children();
            [RegFrm(formula)] => {
                return Ok(formula);
            },
        );
    }

    fn StateFrm(input: ParseNode) -> ParseResult<StateFrm> {
        parse_statefrm(input.children().as_pairs().clone())
    }

    fn StateFrmSpec(input: ParseNode) -> ParseResult<StateFrmSpec> {        
        match_nodes!(input.into_children();
            [StateFrm(state)] => {
                return Ok(StateFrmSpec { 
                    data_specification: UntypedDataSpecification::default(),
                    formula: state
                });
            },
        );
    }

    fn RegFrm(input: ParseNode) -> ParseResult<RegFrm> {
        parse_regfrm(input.children().as_pairs().clone())
    }
    
    fn EOI(_input: ParseNode) -> ParseResult<()> {
        Ok(())
    }
}