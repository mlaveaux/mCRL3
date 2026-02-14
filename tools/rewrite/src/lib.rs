use std::fmt::Debug;

use clap::ValueEnum;

use merc_aterm::ATerm;
use merc_data::to_untyped_data_expression;
use merc_sabre::InnermostRewriter;
use merc_sabre::NaiveRewriter;
use merc_sabre::RewriteEngine;
use merc_sabre::RewriteSpecification;
use merc_sabre::SabreRewriter;
use merc_utilities::MercError;
use merc_utilities::Timing;

/// Selects the rewriter to use.
#[derive(ValueEnum, Debug, Clone)]
pub enum Rewriter {
    Naive,
    Innermost,
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
    timing.measure("rewrite_rec", || {
        match rewriter {
            Rewriter::Naive => {
                let mut inner = NaiveRewriter::new(spec);

                for term in syntax_terms {
                    let term = to_untyped_data_expression(term.clone(), None);
                    let result = inner.rewrite(&term);
                    if output {
                        println!("{}", result)
                    }
                }
            }
            Rewriter::Innermost => {
                let mut inner = InnermostRewriter::new(spec);

                for term in syntax_terms {
                    let term = to_untyped_data_expression(term.clone(), None);
                    let result = inner.rewrite(&term);
                    if output {
                        println!("{}", result)
                    }
                }
            }
            Rewriter::Sabre => {
                let mut sa = SabreRewriter::new(spec);

                for term in syntax_terms {
                    let term = to_untyped_data_expression(term.clone(), None);
                    let result = sa.rewrite(&term);
                    if output {
                        println!("{}", result)
                    }
                }
            }
        }

        Ok(())
    })
}
