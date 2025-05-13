use std::fmt::Debug;
use std::time::Instant;

use ahash::AHashSet;
use clap::ValueEnum;

use mcrl3_data::to_untyped_data_expression;
use mcrl3_rec_tests::load_REC_from_file;
use mcrl3_sabre::InnermostRewriter;
use mcrl3_sabre::RewriteEngine;
use mcrl3_sabre::SabreRewriter;
use mcrl3_utilities::MCRL3Error;

#[derive(ValueEnum, Debug, Clone)]
pub enum Rewriter {
    Innermost,
    Sabre,
}

/// Rewrites the given REC specification.
pub fn rewrite_rec(rewriter: Rewriter, filename_specification: &str, output: bool) -> Result<(), MCRL3Error> {
    let (syntax_spec, syntax_terms) = load_REC_from_file(filename_specification.into())?;

    let spec = syntax_spec.to_rewrite_spec();

    match rewriter {
        Rewriter::Innermost => {
            let mut inner = InnermostRewriter::new(&spec);

            let now = Instant::now();
            for term in &syntax_terms {
                let term = to_untyped_data_expression(term, &AHashSet::new());
                let result = inner.rewrite(&term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Innermost rewrite took {} ms", now.elapsed().as_millis());
        }
        Rewriter::Sabre => {
            let mut sa = SabreRewriter::new(&spec);

            let now = Instant::now();
            for term in &syntax_terms {
                let term = to_untyped_data_expression(term, &AHashSet::new());
                let result = sa.rewrite(&term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Sabre rewrite took {} ms", now.elapsed().as_millis());
        }
    }

    Ok(())
}
