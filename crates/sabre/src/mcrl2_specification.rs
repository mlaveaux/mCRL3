use std::ops::ControlFlow;

use merc_aterm::ATerm;
use merc_aterm::Symbol;
use merc_data::DataApplication;
use merc_data::DataExpression;
use merc_syntax::ComplexSort;
use merc_syntax::DataExpr;
use merc_syntax::EqnDecl;
use merc_syntax::Sort;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::visit_sort_expr;
use merc_utilities::MercError;

use crate::Condition;
use crate::RewriteSpecification;
use crate::Rule;

/// Converts an `UntypedDataSpecification` to a `RewriteSpecification` by converting each equation declaration to a rewrite rule.
pub fn to_rewrite_spec(spec: &UntypedDataSpecification) -> Result<RewriteSpecification, MercError> {
    let spec = complete_data_specification(spec)?;

    let mut rewrite_rules = Vec::new();

    let true_term = DataApplication::with_args(&ATerm::constant(&Symbol::new("true", 0)), &[] as &[ATerm]).into();

    for eqn_spec in &spec.equation_declarations {
        for equation in &eqn_spec.equations {
            rewrite_rules.push(to_rewrite_rule(equation, &true_term));
        }
    }

    Ok(RewriteSpecification::new(rewrite_rules))
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
enum PredefinedSort {
    Basic(Sort),
    Complex(ComplexSort),
}

/// Adds the default data specifications to the provided specification.
fn complete_data_specification(spec: &UntypedDataSpecification) -> Result<UntypedDataSpecification, MercError> {
    let mut result = spec.clone();

    // Sorts that occur in the specification.
    let mut present = std::collections::HashSet::new();
    present.insert(PredefinedSort::Basic(Sort::Bool));

    for sort in &spec.sort_declarations {
        if let Some(expr) = &sort.expr {
            visit_sort_expr::<(), _>(expr, |sort| {
                if let SortExpression::Simple(name) = sort {
                    present.insert(PredefinedSort::Basic(*name));
                }

                if let SortExpression::Complex(complex, _args) = sort {
                    // Args are visited anyway.
                    present.insert(PredefinedSort::Complex(*complex));
                }

                Ok(ControlFlow::Continue(()))
            })?;
        }
    }

    // Append the relevant specifications for the sorts that are present in the specification.
    for sort in present {
        match sort {
            PredefinedSort::Basic(sort) => match sort {
                Sort::Bool => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/bool.mcrl2"
                ))?),
                Sort::Pos => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/pos.mcrl2"
                ))?),
                Sort::Int => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/int.mcrl2"
                ))?),
                Sort::Nat => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/nat.mcrl2"
                ))?),
                Sort::Real => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/real.mcrl2"
                ))?),
            },
            PredefinedSort::Complex(complex) => match complex {
                ComplexSort::List => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/list.mcrl2"
                ))?),
                ComplexSort::Set => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/set.mcrl2"
                ))?),
                ComplexSort::FSet => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/fset.mcrl2"
                ))?),
                ComplexSort::FBag => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/fbag.mcrl2"
                ))?),
                ComplexSort::Bag => result.merge(&UntypedDataSpecification::parse(include_str!(
                    "../../syntax/spec/bag.mcrl2"
                ))?),
            },
        }
    }

    Ok(result)
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
