use pest::Parser;
use pest_consume::match_nodes;
use pest_consume::Error;

use crate::ast::Mcrl2Specification;
use crate::parse_sortexpr;
use crate::DisplayPair;
use crate::IdsDecl;
use crate::Mcrl2Parser;
use crate::Rule;
use crate::SortExpression;


/// Parses the given mCRL2 specification into an AST.
pub fn parse_mcrl2_specification(spec: &str) -> std::result::Result<Mcrl2Specification, Box<dyn std::error::Error>> {
    pest::set_error_detail(true);

    let mut result = Mcrl2Parser::parse(Rule::MCRL2Spec, spec)?;
    let root = result.next().unwrap();
    println!("{}", DisplayPair(root.clone()));

    //Mcrl2Parser::MCRL2Spec(ParseNode::new(root)).map_err(|e| e.into())
    Ok(Mcrl2Specification {
        map: vec![]
    })
}

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type ParseNode<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl Mcrl2Parser {
    fn MCRL2Spec(spec: ParseNode) -> ParseResult<Mcrl2Specification> {
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

        Ok(Mcrl2Specification {
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

    fn SortExpr(expr: ParseNode) -> ParseResult<SortExpression> {
        Ok(parse_sortexpr(expr.children().as_pairs().clone()))
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

        println!("{}", parse_mcrl2_specification(spec).unwrap());
    }
}