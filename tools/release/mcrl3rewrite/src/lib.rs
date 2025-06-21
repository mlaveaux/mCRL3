use std::fmt::Debug;
use std::time::Instant;

use clap::ValueEnum;

use mcrl3_data::to_untyped_data_expression;
use mcrl3_rec_tests::load_rec_from_file;
use mcrl3_sabre::InnermostRewriter;
use mcrl3_sabre::NaiveRewriter;
use mcrl3_sabre::RewriteEngine;
use mcrl3_sabre::SabreRewriter;
use mcrl3_sabre_compiling::SabreCompilingRewriter;
use mcrl3_utilities::MCRL3Error;

/// Selects the rewriter to use.
#[derive(ValueEnum, Debug, Clone)]
pub enum Rewriter {
    Naive,
    Innermost,
    Sabre,
    SabreCompiling,
}

/// Rewrites the given REC specification.
pub fn rewrite_rec(rewriter: Rewriter, filename_specification: &str, output: bool) -> Result<(), MCRL3Error> {
    let (syntax_spec, syntax_terms) = load_rec_from_file(filename_specification.into())?;

    let spec = syntax_spec.to_rewrite_spec();

    match rewriter {
        Rewriter::Naive => {
            let mut inner = NaiveRewriter::new(&spec);

            let now = Instant::now();
            for term in &syntax_terms {
                let term = to_untyped_data_expression(term, None);
                let result = inner.rewrite(&term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Naive rewrite took {} ms", now.elapsed().as_millis());
        }
        Rewriter::Innermost => {
            let mut inner = InnermostRewriter::new(&spec);

            let now = Instant::now();
            for term in &syntax_terms {
                let term = to_untyped_data_expression(term, None);
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
                let term = to_untyped_data_expression(term, None);
                let result = sa.rewrite(&term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Sabre rewrite took {} ms", now.elapsed().as_millis());
        },
        Rewriter::SabreCompiling => {            
            let mut sa = SabreCompilingRewriter::new(&spec, true, true)?;

            let now = Instant::now();
            for term in &syntax_terms {
                let term = to_untyped_data_expression(term, None);
                let result = sa.rewrite(&term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Compiling sabre rewrite took {} ms", now.elapsed().as_millis());
        }
    }

    Ok(())
}
