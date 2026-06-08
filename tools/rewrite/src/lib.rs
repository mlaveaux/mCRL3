use std::fmt::Debug;

use clap::ValueEnum;

use merc_aterm::ATerm;
use merc_data::to_untyped_data_expression;
use merc_sabre::InnermostRewriter;
use merc_sabre::NaiveRewriter;
use merc_sabre::RewriteEngine;
use merc_sabre::RewriteSpecification;
use merc_sabre::SabreRewriter;
use merc_sabre_compiling::SabreCompilingRewriter;
use merc_utilities::MercError;
use merc_utilities::Timing;

/// Selects the rewriter to use.
#[derive(ValueEnum, Debug, Clone)]
pub enum Rewriter {
    Naive,
    /// An innermost rewriter that uses set automata for efficient matching.
    Innermost,
    /// A variant of innermost that generates Rust code for the rewrite rules and compiles it to a dynamic library.
    InnermostCompiling,
    /// A set automaton-based rewriter that applies rules outermost.
    Sabre,
}

/// Rewrites the given REC specification.
pub fn rewrite_rec(
    rewriter: Rewriter,
    spec: &RewriteSpecification,
    syntax_terms: &[ATerm],
    output: bool,
    timing: &Timing,
) -> Result<(), MercError> {
    match rewriter {
        Rewriter::Naive => {
            let mut inner = timing.measure("rewriter_construction", || NaiveRewriter::new(spec));

            timing.measure("rewrite_rec", || {
                for term in syntax_terms {
                    let term = to_untyped_data_expression(term.clone(), None);
                    let result = inner.rewrite(&term);
                    if output {
                        println!("{}", result)
                    }
                }
            });
        }
        Rewriter::Innermost => {
            let mut inner = timing.measure("rewriter_construction", || InnermostRewriter::new(spec));

            timing.measure("rewrite_rec", || {
                for term in syntax_terms {
                    let term = to_untyped_data_expression(term.clone(), None);
                    let result = inner.rewrite(&term);
                    if output {
                        println!("{}", result)
                    }
                }
            });
        }
        Rewriter::InnermostCompiling => {
            let mut inner = timing.measure("rewriter_construction", || {
                SabreCompilingRewriter::new(spec, true, false)
            })?;

            timing.measure("rewrite_rec", || {
                for term in syntax_terms {
                    let term = to_untyped_data_expression(term.clone(), None);
                    let result = inner.rewrite(&term);
                    if output {
                        println!("{}", result)
                    }
                }
            });
        }
        Rewriter::Sabre => {
            let mut sa = timing.measure("rewriter_construction", || SabreRewriter::new(spec));

            timing.measure("rewrite_rec", || {
                for term in syntax_terms {
                    let term = to_untyped_data_expression(term.clone(), None);
                    let result = sa.rewrite(&term);
                    if output {
                        println!("{}", result)
                    }
                }
            });
        }
    }

    Ok(())
}
