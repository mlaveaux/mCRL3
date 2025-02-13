use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use mcrl2::aterm::ATerm;
use mcrl2::aterm::Symbol;
use mcrl2::aterm::TermBuilder;
use mcrl2::aterm::TermPool;
use mcrl2::aterm::Yield;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::syntax::ConditionSyntax;
use crate::syntax::RewriteRuleSyntax;
use crate::syntax::RewriteSpecificationSyntax;

#[derive(Parser)]
#[grammar = "rec_grammar.pest"]
pub struct RecParser;

/// Parses a REC specification. REC files can import other REC files.
/// Returns a RewriteSpec containing all the rewrite rules and a list of terms that need to be rewritten.
///
/// Arguments
/// `contents` - A REC specification as string
/// `path` - An optional path to a folder in which other importable REC files can be found.
#[allow(non_snake_case)]
fn parse_REC(
    tp: &mut TermPool,
    contents: &str,
    path: Option<PathBuf>,
) -> Result<(RewriteSpecificationSyntax, Vec<ATerm>), Box<dyn Error>> {
    // Initialise return result
    let mut rewrite_spec = RewriteSpecificationSyntax::default();
    let mut terms = vec![];

    // Use Pest parser (generated automatically from the grammar.pest file)
    let mut pairs = RecParser::parse(Rule::rec_spec, contents)?;

    // Get relevant sections from the REC file
    let pair = pairs.next().unwrap();
    let mut inner = pair.into_inner();
    let header = inner.next().unwrap();
    let _sorts = inner.next().unwrap();
    let cons = inner.next().unwrap();
    let _opns = inner.next().unwrap();
    let vars = inner.next().unwrap();
    let rules = inner.next().unwrap();
    let eval = inner.next().unwrap();
    let (_name, include_files) = parse_header(header);

    rewrite_spec.rewrite_rules = parse_rewrite_rules(tp, rules);
    rewrite_spec.constructors = parse_constructors(cons);
    if eval.as_rule() == Rule::eval {
        terms.extend_from_slice(&parse_eval(tp, eval));
    }

    rewrite_spec.variables = parse_vars(vars);

    // REC files can import other REC files. Import all referenced by the header.
    for file in include_files {
        if let Some(p) = &path {
            let include_path = p.parent().unwrap();
            let file_name = PathBuf::from_str(&(file.to_lowercase() + ".rec")).unwrap();
            let load_file = include_path.join(file_name);
            let contents = fs::read_to_string(load_file).unwrap();
            let (include_spec, include_terms) = parse_REC(tp, &contents, path.clone())?;

            // Add rewrite rules and terms to the result.
            terms.extend_from_slice(&include_terms);
            rewrite_spec
                .rewrite_rules
                .extend_from_slice(&include_spec.rewrite_rules);
            rewrite_spec.constructors.extend_from_slice(&include_spec.constructors);
            for s in include_spec.variables {
                if !rewrite_spec.variables.contains(&s) {
                    rewrite_spec.variables.push(s);
                }
            }
        }
    }

    Ok((rewrite_spec, terms))
}

/// Load a REC specification from a specified file.
#[allow(non_snake_case)]
pub fn load_REC_from_file(
    tp: &mut TermPool,
    file: PathBuf,
) -> Result<(RewriteSpecificationSyntax, Vec<ATerm>), Box<dyn Error>> {
    let contents = fs::read_to_string(file.clone()).unwrap();
    parse_REC(tp, &contents, Some(file))
}

/// Load and join multiple REC specifications
#[allow(non_snake_case)]
pub fn load_REC_from_strings(
    tp: &mut TermPool,
    specs: &[&str],
) -> Result<(RewriteSpecificationSyntax, Vec<ATerm>), Box<dyn Error>> {
    let mut rewrite_spec = RewriteSpecificationSyntax::default();
    let mut terms = vec![];

    for spec in specs {
        let (include_spec, include_terms) = parse_REC(tp, spec, None)?;
        rewrite_spec.merge(&include_spec);
        terms.extend_from_slice(&include_terms);
    }

    Ok((rewrite_spec, terms))
}

/// Extracts data from parsed header of REC spec. Returns name and include files.
fn parse_header(pair: Pair<Rule>) -> (String, Vec<String>) {
    debug_assert_eq!(pair.as_rule(), Rule::header);

    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut include_files = vec![];

    for f in inner {
        include_files.push(f.as_str().to_string());
    }

    (name, include_files)
}

/// Extracts data from parsed constructor section, derives the arity of symbols. Types are ignored.
fn parse_constructors(pair: Pair<Rule>) -> Vec<(String, usize)> {
    debug_assert_eq!(pair.as_rule(), Rule::cons);

    let mut constructors = Vec::new();
    for decl in pair.into_inner() {
        debug_assert_eq!(decl.as_rule(), Rule::cons_decl);
        let mut inner = decl.into_inner();
        let symbol = inner.next().unwrap().as_str().to_string();
        let arity = inner.count() - 1;
        constructors.push((symbol, arity));
    }
    constructors
}

/// Extracts data from parsed rewrite rules. Returns list of rewrite rules
fn parse_rewrite_rules(tp: &mut TermPool, pair: Pair<Rule>) -> Vec<RewriteRuleSyntax> {
    debug_assert_eq!(pair.as_rule(), Rule::rules);
    let mut rules = vec![];
    let inner = pair.into_inner();
    for p in inner {
        let rule = parse_rewrite_rule(tp, p);
        rules.push(rule);
    }
    rules
}

// Extracts data from the variable VARS block. Types are ignored.
fn parse_vars(pair: Pair<Rule>) -> Vec<String> {
    debug_assert_eq!(pair.as_rule(), Rule::vars);

    let mut variables = vec![];
    let inner = pair.into_inner();
    for v in inner {
        debug_assert_eq!(v.as_rule(), Rule::var_decl);

        variables.extend(parse_var_decl(v));
    }

    variables
}

fn parse_var_decl(pair: Pair<Rule>) -> Vec<String> {
    debug_assert_eq!(pair.as_rule(), Rule::var_decl);

    let mut variables = vec![];
    let inner = pair.into_inner();
    for v in inner {
        debug_assert_eq!(v.as_rule(), Rule::identifier);

        variables.push(String::from(v.as_str()));
    }

    // There might be a better way, but the last identifier is the type.
    variables.pop();

    variables
}

/// Extracts data from parsed EVAL section, returns a list of terms that need to be rewritten.
fn parse_eval(tp: &mut TermPool, pair: Pair<Rule>) -> Vec<ATerm> {
    assert_eq!(pair.as_rule(), Rule::eval);
    let mut terms = vec![];
    let inner = pair.into_inner();
    for p in inner {
        let term = parse_term(tp, p).unwrap();
        terms.push(term);
    }

    terms
}

/// Constructs a ATerm from a string.
pub fn from_string(tp: &mut TermPool, str: &str) -> Result<ATerm, Box<dyn Error>> {
    let mut pairs = RecParser::parse(Rule::term, str)?;
    parse_term(tp, pairs.next().unwrap())
}

/// Extracts data from parsed term.
fn parse_term(tp: &mut TermPool, pair: Pair<Rule>) -> Result<ATerm, Box<dyn Error>> {
    debug_assert_eq!(pair.as_rule(), Rule::term);

    let mut builder = TermBuilder::<Pair<'_, Rule>, Symbol>::new();

    Ok(builder
        .evaluate(
            tp,
            pair,
            |tp, stack, pair| {
                match pair.as_rule() {
                    Rule::term => {
                        let mut inner = pair.into_inner();
                        let head_symbol = inner.next().unwrap().as_str();

                        // Queue applications for all the arguments.
                        let mut arity = 0;
                        if let Some(args) = inner.next() {
                            for arg in args.into_inner() {
                                stack.push(arg);
                                arity += 1;
                            }
                        }

                        Ok(Yield::Construct(tp.create_symbol(head_symbol, arity)))
                    }
                    _ => {
                        panic!("Should be unreachable!")
                    }
                }
            },
            |tp, symbol, args| Ok(tp.create(&symbol, args)),
        )
        .unwrap())
}

// /Extracts data from parsed rewrite rule
fn parse_rewrite_rule(tp: &mut TermPool, pair: Pair<Rule>) -> RewriteRuleSyntax {
    debug_assert!(pair.as_rule() == Rule::single_rewrite_rule || pair.as_rule() == Rule::rewrite_rule);

    let mut inner = match pair.as_rule() {
        Rule::single_rewrite_rule => pair.into_inner().next().unwrap().into_inner(),
        Rule::rewrite_rule => pair.into_inner(),
        _ => {
            panic!("Unreachable");
        }
    };
    let lhs = parse_term(tp, inner.next().unwrap()).unwrap();
    let rhs = parse_term(tp, inner.next().unwrap()).unwrap();

    // Extract conditions
    let mut conditions = vec![];
    for c in inner {
        assert_eq!(c.as_rule(), Rule::condition);
        let mut c_inner = c.into_inner();
        let lhs_cond = parse_term(tp, c_inner.next().unwrap()).unwrap();
        let equality = match c_inner.next().unwrap().as_str() {
            "=" => true,
            "<>" => false,
            _ => {
                panic!("Unknown comparison operator");
            }
        };
        let rhs_cond = parse_term(tp, c_inner.next().unwrap()).unwrap();

        let condition = ConditionSyntax {
            lhs: lhs_cond,
            rhs: rhs_cond,
            equality,
        };
        conditions.push(condition);
    }

    RewriteRuleSyntax { lhs, rhs, conditions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_parsing() {
        assert!(RecParser::parse(Rule::single_term, "f(a").is_err());
        assert!(RecParser::parse(Rule::single_term, "f()").is_err());
        assert!(RecParser::parse(Rule::single_term, "f(a,)").is_err());
        assert!(RecParser::parse(Rule::single_term, "f").is_ok());
        assert!(RecParser::parse(Rule::single_term, "f(a)").is_ok());
        assert!(RecParser::parse(Rule::single_term, "f(a,b)").is_ok());
        assert!(RecParser::parse(Rule::single_rewrite_rule, "f(a,b) = g(x)").is_ok());
        assert!(RecParser::parse(Rule::single_rewrite_rule, "f(a,b) = g(x) if x = a").is_ok());
        assert!(RecParser::parse(Rule::single_rewrite_rule, "f(a,b) = g(x) if x<> a").is_ok());
        assert!(RecParser::parse(Rule::single_rewrite_rule, "f(a,b) = g(x) if x <= a").is_err());
        assert!(RecParser::parse(Rule::single_rewrite_rule, "f(a,b) = ").is_err());
    }

    #[test]
    fn test_parsing_rewrite_rule() {
        let mut tp = TermPool::new();

        let expected = RewriteRuleSyntax {
            lhs: from_string(&mut tp, "f(x,b)").unwrap(),
            rhs: from_string(&mut tp, "g(x)").unwrap(),
            conditions: vec![
                ConditionSyntax {
                    lhs: from_string(&mut tp, "x").unwrap(),
                    rhs: from_string(&mut tp, "a").unwrap(),
                    equality: true,
                },
                ConditionSyntax {
                    lhs: from_string(&mut tp, "b").unwrap(),
                    rhs: from_string(&mut tp, "b").unwrap(),
                    equality: true,
                },
            ],
        };

        let actual = parse_rewrite_rule(
            &mut tp,
            RecParser::parse(Rule::single_rewrite_rule, "f(x,b) = g(x) if x = a and-if b = b")
                .unwrap()
                .next()
                .unwrap(),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_variable_parsing() {
        let mut pairs = RecParser::parse(Rule::var_decl, "X Y Val Max : Nat").unwrap();
        assert_eq!(parse_var_decl(pairs.next().unwrap()), vec!["X", "Y", "Val", "Max"]);
    }

    #[test]
    fn test_parsing_rec() {
        assert!(RecParser::parse(
            Rule::rec_spec,
            include_str!("../../../examples/REC/rec/missionaries.rec")
        )
        .is_ok());
    }

    #[test]
    fn loading_rec() {
        let mut tp = TermPool::new();
        let _ = parse_REC(
            &mut tp,
            include_str!("../../../examples/REC/rec/missionaries.rec"),
            None,
        );
    }
}
