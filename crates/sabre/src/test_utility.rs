use std::error::Error;

use ahash::AHashSet;
use mcrl2::aterm::TermPool;

use crate::utilities::to_untyped_data_expression;
use crate::Rule;

/// Create a rewrite rule lhs -> rhs with the given names being variables.
pub(crate) fn create_rewrite_rule(
    tp: &mut TermPool,
    lhs: &str,
    rhs: &str,
    variables: &[&str],
) -> Result<Rule, Box<dyn Error>> {
    let lhs = tp.from_string(lhs)?;
    let rhs = tp.from_string(rhs)?;
    let mut vars = AHashSet::new();
    for var in variables {
        vars.insert(var.to_string());
    }

    Ok(Rule {
        conditions: vec![],
        lhs: to_untyped_data_expression(tp, &lhs, &vars).into(),
        rhs: to_untyped_data_expression(tp, &rhs, &vars).into(),
    })
}
