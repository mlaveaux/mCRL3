use mcrl3_utilities::debug_trace;
use pest::Parser;
use pest_consume::Error;
use pest_consume::match_nodes;

use mcrl3_utilities::MCRL3Error;

use crate::ComplexSort;
use crate::ConstructorDecl;
use crate::DataExpr;
use crate::DataExprUpdate;
use crate::EqnDecl;
use crate::EqnSpec;
use crate::IdDecl;
use crate::Mcrl2Parser;
use crate::Rule;
use crate::Sort;
use crate::SortExpression;
use crate::UntypedDataSpecification;
use crate::UntypedProcessSpecification;
use crate::VarDecl;
use crate::parse_dataexpr;
use crate::parse_sortexpr;

/// Parses the given mCRL2 specification into an AST.
impl UntypedProcessSpecification {
    pub fn parse(spec: &str) -> std::result::Result<UntypedProcessSpecification, MCRL3Error> {
        pest::set_error_detail(true);

        let mut result = Mcrl2Parser::parse(Rule::MCRL2Spec, spec)?;
        let root = result.next().ok_or("Could not parse mCRL2 specification")?;
        debug_trace!("Parse tree {}", DisplayPair(root.clone()));

        Ok(Mcrl2Parser::MCRL2Spec(ParseNode::new(root)).unwrap())
    }
}

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type ParseNode<'i> = pest_consume::Node<'i, Rule, ()>;

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

    fn DataExpr(expr: ParseNode) -> ParseResult<DataExpr> {
        Ok(parse_dataexpr(expr.children().as_pairs().clone()))
    }

    pub(crate) fn DataExprPrimary(expr: ParseNode) -> ParseResult<DataExpr> {
        Ok(parse_dataexpr(expr.children().as_pairs().clone()))
    }

    pub(crate) fn DataExprTrue(_input: ParseNode) -> ParseResult<DataExpr> {
        Ok(DataExpr::Bool(true))
    }

    pub(crate) fn DataExprFalse(_input: ParseNode) -> ParseResult<DataExpr> {
        Ok(DataExpr::Bool(false))
    }

    pub(crate) fn DataExprUpdate(expr: ParseNode) -> ParseResult<DataExprUpdate> {
        match_nodes!(expr.into_children();
            [DataExpr(expr), DataExpr(update)] => {
                Ok(DataExprUpdate { expr: Box::new(expr), update: Box::new(update) })
            },
        )
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

    fn VarsDeclList(vars: ParseNode) -> ParseResult<Vec<VarDecl>> {
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
        Ok(parse_sortexpr(expr.children().as_pairs().clone()))
    }

    pub fn SortExprAtom(expr: ParseNode) -> ParseResult<SortExpression> {
        Ok(parse_sortexpr(expr.children().as_pairs().clone()))
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

    // Simple sorts.
    pub fn SortExprBool(_input: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Simple(Sort::Bool))
    }

    pub fn SortExprInt(_input: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Simple(Sort::Int))
    }

    pub fn SortExprPos(_input: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Simple(Sort::Pos))
    }

    pub fn SortExprNat(_input: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Simple(Sort::Nat))
    }

    pub fn SortExprReal(_input: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Simple(Sort::Real))
    }

    // Complex sorts
    pub fn SortExprList(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::List,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())),
        ))
    }

    pub fn SortExprSet(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::Set,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())),
        ))
    }

    pub fn SortExprBag(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::Bag,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())),
        ))
    }

    pub fn SortExprFSet(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::FSet,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())),
        ))
    }

    pub fn SortExprFBag(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(
            ComplexSort::FBag,
            Box::new(parse_sortexpr(inner.children().as_pairs().clone())),
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

    fn EOI(_input: ParseNode) -> ParseResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mcrl3_utilities::test_logger;

    use super::*;

    #[test]
    fn test_parse_term() {
        let _ = test_logger();

        use indoc::indoc;

        let spec: &str = indoc! {"map
            f: Boolean # Int -> Int -> Bool;
        "};

        println!("{}", UntypedProcessSpecification::parse(spec).unwrap());
    }

    #[test]
    fn test_parse_procexpr() {
        let _ = test_logger();

        use indoc::indoc;

        let spec: &str = indoc! {"init
            true -> (false -> delta) <> delta;
        "};

        println!("{}", UntypedProcessSpecification::parse(spec).unwrap());
    }
}
