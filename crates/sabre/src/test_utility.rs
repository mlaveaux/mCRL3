use std::error::Error;

use ahash::AHashSet;
use mcrl3_aterm::ATerm;

use crate::utilities::to_untyped_data_expression;
use crate::Rule;

/// Create a rewrite rule lhs -> rhs with the given names being variables.
pub(crate) fn create_rewrite_rule(
    lhs: &str,
    rhs: &str,
    variables: &[&str],
) -> Result<Rule, Box<dyn Error>> {
    let lhs = ATerm::from_string(lhs)?;
    let rhs = ATerm::from_string(rhs)?;
    let mut vars = AHashSet::new();
    for var in variables {
        vars.insert(var.to_string());
    }

    Ok(Rule {
        conditions: vec![],
        lhs: to_untyped_data_expression(&lhs, &vars).into(),
        rhs: to_untyped_data_expression(&rhs, &vars).into(),
    })
}
