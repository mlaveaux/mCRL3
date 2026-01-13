use std::fmt::Debug;
use std::path::Path;

use clap::ValueEnum;

use merc_data::to_untyped_data_expression;
use merc_rec_tests::load_rec_from_file;
use merc_sabre::InnermostRewriter;
use merc_sabre::NaiveRewriter;
use merc_sabre::RewriteEngine;
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
pub fn rewrite_rec(rewriter: Rewriter, filename_specification: &Path, output: bool, timing: &Timing) -> Result<(), MercError> {
    let (syntax_spec, syntax_terms) = load_rec_from_file(filename_specification)?;

    let spec = syntax_spec.to_rewrite_spec();

    let mut rewrite_time = timing.start("rewrite_rec");
    match rewriter {
        Rewriter::Naive => {
            let mut inner = NaiveRewriter::new(&spec);

            for term in &syntax_terms {
                let term = to_untyped_data_expression(term.clone(), None);
                let result = inner.rewrite(&term);
                if output {
                    println!("{}", result)
                }
            }
        }
        Rewriter::Innermost => {
            let mut inner = InnermostRewriter::new(&spec);

            for term in &syntax_terms {
                let term = to_untyped_data_expression(term.clone(), None);
                let result = inner.rewrite(&term);
                if output {
                    println!("{}", result)
                }
            }
        }
        Rewriter::Sabre => {
            let mut sa = SabreRewriter::new(&spec);

            for term in &syntax_terms {
                let term = to_untyped_data_expression(term.clone(), None);
                let result = sa.rewrite(&term);
                if output {
                    println!("{}", result)
                }
            }
        }
    }

    rewrite_time.finish();

    Ok(())
}
