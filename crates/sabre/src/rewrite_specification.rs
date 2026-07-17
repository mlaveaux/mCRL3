#![forbid(unsafe_code)]

use std::fmt;

use itertools::Itertools;
use merc_data::BasicSort;
use merc_data::DataExpression;
use merc_data::DataFunctionSymbol;
use merc_data::Mcrl2DataSpecification;
use merc_data::SortExpression;

/// A rewrite specification is a set of rewrite rules, given by [Rule].
#[derive(Debug, Default, Clone)]
pub struct RewriteSpecification {
    rewrite_rules: Vec<Rule>,
}

impl RewriteSpecification {
    /// Create a new rewrite specification from the given rewrite rules.
    pub fn new(rewrite_rules: Vec<Rule>) -> RewriteSpecification {
        RewriteSpecification { rewrite_rules }
    }

    /// Builds a rewrite specification from the equations of a fully typed
    /// mCRL2 data specification, e.g. the output of
    /// `merc_typecheck::DataSpecification::lower_data_specification`.
    ///
    /// A conditional equation `condition -> lhs = rhs` becomes a rule with a
    /// single condition that the (rewritten) condition equals the `Bool`
    /// literal `true`, matching how the mCRL2 rewriter treats equation
    /// conditions.
    pub fn from_data_specification(spec: &Mcrl2DataSpecification) -> RewriteSpecification {
        let true_literal: DataExpression =
            DataFunctionSymbol::with_sort("true", SortExpression::from(BasicSort::new("Bool")).copy()).into();

        let rewrite_rules = spec
            .equations()
            .iter()
            .map(|equation| {
                let conditions = match equation.condition() {
                    Some(condition) => vec![Condition::new(condition.protect(), true_literal.clone(), true)],
                    None => Vec::new(),
                };

                Rule::with_condition(conditions, equation.lhs().protect(), equation.rhs().protect())
            })
            .collect();

        RewriteSpecification::new(rewrite_rules)
    }

    /// Returns the rewrite rules of this specification.
    pub fn rewrite_rules(&self) -> &[Rule] {
        &self.rewrite_rules
    }
}

/// A condition of a conditional rewrite rule.
///
/// Either `lhs == rhs` or `lhs != rhs` depending on equality being true.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Condition {
    pub lhs: DataExpression,
    pub rhs: DataExpression,
    pub equality: bool,
}

impl Condition {
    /// Create a new condition with the given left-hand side, right-hand side and equality.
    pub fn new(lhs: DataExpression, rhs: DataExpression, equality: bool) -> Condition {
        Condition { lhs, rhs, equality }
    }
}

/// A rewrite rule.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Rule {
    /// A conjunction of clauses
    pub conditions: Vec<Condition>,
    pub lhs: DataExpression,
    pub rhs: DataExpression,
}

impl Rule {
    /// Create a rewrite rule with the given conditions, left-hand side and right-hand side.
    pub fn with_condition(conditions: Vec<Condition>, lhs: DataExpression, rhs: DataExpression) -> Rule {
        Rule { conditions, lhs, rhs }
    }

    /// Create a rewrite rule without conditions.
    pub fn new(lhs: DataExpression, rhs: DataExpression) -> Rule {
        Rule {
            conditions: Vec::new(),
            lhs,
            rhs,
        }
    }
}

impl fmt::Display for RewriteSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rule in &self.rewrite_rules {
            writeln!(f, "{rule}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.conditions.is_empty() {
            write!(f, "{} = {}", self.lhs, self.rhs)
        } else {
            write!(
                f,
                "{} -> {} = {}",
                self.conditions.iter().format(", "),
                self.lhs,
                self.rhs
            )
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.equality {
            write!(f, "{} == {}", self.lhs, self.rhs)
        } else {
            write!(f, "{} <> {}", self.lhs, self.rhs)
        }
    }
}

#[cfg(test)]
mod tests {
    use merc_syntax::UntypedDataSpecification;
    use merc_typecheck::DataSpecification;

    use super::*;

    /// Parses and type-checks the given mCRL2 data specification text.
    fn lower(source: &str) -> Mcrl2DataSpecification {
        let untyped = UntypedDataSpecification::parse(source).unwrap();
        let mut data_spec = DataSpecification::from_untyped(untyped).unwrap();
        data_spec.lower_data_specification()
    }

    #[test]
    fn test_from_data_specification_unconditional_equation() {
        let mcrl2_spec = lower(
            "map f: Nat -> Nat;
             var x: Nat;
             eqn f(x) = x;",
        );

        let spec = RewriteSpecification::from_data_specification(&mcrl2_spec);

        let rule = spec
            .rewrite_rules()
            .iter()
            .find(|rule| rule.lhs.to_string().starts_with("f("))
            .expect("the f(x) = x rule should be present");
        assert!(rule.conditions.is_empty());
        assert_eq!(rule.lhs.to_string(), "f(x)");
        assert_eq!(rule.rhs.to_string(), "x");
    }

    #[test]
    fn test_from_data_specification_conditional_equation() {
        let mcrl2_spec = lower(
            "map f: Nat -> Nat;
             var x: Nat;
             eqn x == 0 -> f(x) = x;",
        );

        let spec = RewriteSpecification::from_data_specification(&mcrl2_spec);

        let rule = spec
            .rewrite_rules()
            .iter()
            .find(|rule| rule.lhs.to_string().starts_with("f("))
            .expect("the conditional f(x) = x rule should be present");
        assert_eq!(rule.conditions.len(), 1);
        assert!(rule.conditions[0].equality);
        assert_eq!(rule.conditions[0].rhs.to_string(), "true");
    }
}
