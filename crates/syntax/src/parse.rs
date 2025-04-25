use pest::Parser;
use pest_consume::Error;
use pest_consume::match_nodes;

use mcrl3_utilities::MCRL3Error;

use crate::ComplexSort;
use crate::ConstructorDecl;
use crate::IdDecl;
use crate::Mcrl2Parser;
use crate::Rule;
use crate::Sort;
use crate::SortExpression;
use crate::UntypedDataSpecification;
use crate::UntypedProcessSpecification;
use crate::parse_sortexpr;

/// Parses the given mCRL2 specification into an AST.
impl UntypedProcessSpecification {
    pub fn parse(spec: &str) -> std::result::Result<UntypedProcessSpecification, MCRL3Error> {
        pest::set_error_detail(true);

        let mut result = Mcrl2Parser::parse(Rule::MCRL2Spec, spec)?;
        let root = result.next().unwrap();

        Ok(Mcrl2Parser::MCRL2Spec(ParseNode::new(root)).unwrap())
    }
}

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type ParseNode<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl Mcrl2Parser {
    pub fn MCRL2Spec(spec: ParseNode) -> ParseResult<UntypedProcessSpecification> {
        let mut map = Vec::new();

        for child in spec.into_children() {
            if child.as_rule() == Rule::MapSpec {
                map.append(&mut Mcrl2Parser::MapSpec(child)?);
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
    use super::*;

    #[test]
    fn test_parse_term() {
        use indoc::indoc;

        let spec: &str = indoc! {"map
            f: Boolean # Int -> Int -> Bool;
        "};

        println!("{}", UntypedProcessSpecification::parse(spec).unwrap());
    }
}
