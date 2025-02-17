use std::cell::RefCell;
use std::fmt::Debug;
use std::fs::File;
use std::fs::{self};
use std::io::BufRead;
use std::io::BufReader;
use std::rc::Rc;
use std::time::Instant;

use ahash::AHashSet;
use anyhow::bail;
use clap::ValueEnum;
use mcrl3_rec_tests::load_REC_from_file;
use mcrl3_sabre::utilities::to_untyped_data_expression;
use mcrl3_sabre::InnermostRewriter;
use mcrl3_sabre::RewriteEngine;
use mcrl3_sabre::RewriteSpecification;
use mcrl3_sabre::SabreRewriter;

#[derive(ValueEnum, Debug, Clone)]
pub enum Rewriter {
    Innermost,
    Sabre,
}

/// Rewrites the given REC specification.
pub fn rewrite_rec(rewriter: Rewriter, filename_specification: &str, output: bool) -> anyhow::Result<()> {
    let (syntax_spec, syntax_terms) = load_REC_from_file(filename_specification.into()).unwrap();

    let spec = syntax_spec.to_rewrite_spec();

    match rewriter {
        Rewriter::Innermost => {
            let mut inner = InnermostRewriter::new(&spec);

            let now = Instant::now();
            for term in &syntax_terms {
                let term = to_untyped_data_expression(term, &AHashSet::new());
                let result = inner.rewrite(term);
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
                let result = sa.rewrite(term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Sabre rewrite took {} ms", now.elapsed().as_millis());
        }
    }

    Ok(())
}
