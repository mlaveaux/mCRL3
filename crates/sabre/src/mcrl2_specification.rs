use merc_aterm::ATerm;
use merc_aterm::Symbol;
use merc_data::DataApplication;
use merc_data::DataExpression;
use merc_syntax::DataExpr;
use merc_syntax::EqnDecl;
use merc_syntax::UntypedDataSpecification;
use merc_utilities::MercError;

use crate::Condition;
use crate::RewriteSpecification;
use crate::Rule;

/// Converts an `DataSpecification` to a `RewriteSpecification` by converting each equation declaration to a rewrite rule.
pub fn to_rewrite_spec(spec: &UntypedDataSpecification) -> Result<RewriteSpecification, MercError> {
    let mut rewrite_rules = Vec::new();

    let true_term = DataApplication::with_args(&ATerm::constant(&Symbol::new("true", 0)), &[] as &[ATerm]).into();

    for eqn_spec in &spec.equation_declarations {
        for equation in &eqn_spec.equations {
            rewrite_rules.push(to_rewrite_rule(equation, &true_term));
        }
    }

    Ok(RewriteSpecification::new(rewrite_rules))
}

/// Convert a `crate::DataExpr` to a term representation.
pub fn data_expr_to_term(_expr: &DataExpr) -> DataExpression {
    unimplemented!("Conversion of data expressions to terms is not yet implemented");
}

/// Convert an `EqnDecl` to a rewrite rule in the `Rule` form.
fn to_rewrite_rule(equation: &EqnDecl, true_term: &DataExpression) -> Rule {
    // Convert the left-hand side and right-hand side terms
    let lhs = data_expr_to_term(&equation.lhs);
    let rhs = data_expr_to_term(&equation.rhs);

    if let Some(condition) = &equation.condition {
        let condition = Condition::new(data_expr_to_term(condition), true_term.clone(), true);

        Rule::with_condition(vec![condition], lhs, rhs)
    } else {
        Rule::new(lhs, rhs)
    }
}
