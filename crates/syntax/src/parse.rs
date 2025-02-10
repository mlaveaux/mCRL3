use pest::Parser;
use pest_consume::match_nodes;
use pest_consume::Error;

use crate::parse_sortexpr;
use crate::ComplexSort;
use crate::IdsDecl;
use crate::Mcrl2Parser;
use crate::Rule;
use crate::Sort;
use crate::SortExpression;

/// An mCRL2 specification containing declarations.
#[derive(Debug)]
pub struct UntypedProcessSpecification {
    /// Vector of identifier declarations
    pub map: Vec<IdsDecl>,
}


/// Parses the given mCRL2 specification into an AST.
impl UntypedProcessSpecification {
    pub fn parse(spec: &str) -> std::result::Result<UntypedProcessSpecification, Box<dyn std::error::Error>> {
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
            match child.as_rule() {
                Rule::MapSpec => {
                    map.append(&mut Mcrl2Parser::MapSpec(child)?);
                },
                _ => {

                }
            }
        }

        Ok(UntypedProcessSpecification {
            map
        })
    }

    fn MapSpec(spec: ParseNode) -> ParseResult<Vec<IdsDecl>> {
        let mut ids = Vec::new();
        
        for decl in spec.into_children() {
            ids.push(Mcrl2Parser::IdsDecl(decl)?);
        }

        Ok(ids)
    }

    fn IdsDecl(decl: ParseNode) -> ParseResult<IdsDecl> {
        let span = decl.as_span();
        match_nodes!(decl.into_children();
            [IdList(identifiers), SortExpr(sort)] => {
                return Ok(IdsDecl { identifiers, sort, span: span.into() });
            },
        );
    }

    fn IdList(list: ParseNode) -> ParseResult<Vec<String>> {
        Ok(list.into_children().map(|i| {
            i.as_str().to_string()
        }).collect())
    }

    pub fn SortExpr(expr: ParseNode) -> ParseResult<SortExpression> {
        Ok(parse_sortexpr(expr.children().as_pairs().clone()))
    }

    pub fn SortExprAtom(expr: ParseNode) -> ParseResult<SortExpression> {
        Ok(parse_sortexpr(expr.children().as_pairs().clone()))
    }

    pub fn Id(identifier: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Reference(identifier.as_str().to_string()))
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
        Ok(SortExpression::Complex(ComplexSort::List, Box::new(parse_sortexpr(inner.children().as_pairs().clone()))))
    }

    pub fn SortExprSet(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(ComplexSort::Set, Box::new(parse_sortexpr(inner.children().as_pairs().clone()))))
    }

    pub fn SortExprBag(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(ComplexSort::Bag, Box::new(parse_sortexpr(inner.children().as_pairs().clone()))))
    }

    pub fn SortExprFSet(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(ComplexSort::FSet, Box::new(parse_sortexpr(inner.children().as_pairs().clone()))))
    }

    pub fn SortExprFBag(inner: ParseNode) -> ParseResult<SortExpression> {
        Ok(SortExpression::Complex(ComplexSort::FBag, Box::new(parse_sortexpr(inner.children().as_pairs().clone()))))
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