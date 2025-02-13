use std::fmt;

use itertools::Itertools;
use mcrl2::aterm::ATerm;
use mcrl2::data::BoolSort;
use mcrl2::data::DataExpression;
use mcrl2::data::DataSpecification;

/// A rewrite specification contains the bare info we need for rewriting (can be untyped).
#[derive(Debug, Default, Clone)]
pub struct RewriteSpecification {
    pub rewrite_rules: Vec<Rule>,
}

/// Either lhs == rhs or lhs != rhs depending on equality being true.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Condition {
    pub lhs: DataExpression,
    pub rhs: DataExpression,
    pub equality: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Rule {
    /// A conjunction of clauses
    pub conditions: Vec<Condition>,
    pub lhs: DataExpression,
    pub rhs: DataExpression,
}

impl From<DataSpecification> for RewriteSpecification {
    fn from(value: DataSpecification) -> Self {
        let equations = value.equations();

        // Convert the equations.
        let mut rewrite_rules = vec![];
        for equation in equations {
            if *equation.condition == *BoolSort::true_term() {
                // Ignore the condition if it is trivial.
                rewrite_rules.push(Rule {
                    conditions: vec![],
                    lhs: equation.lhs,
                    rhs: equation.rhs,
                })
            } else {
                let t: ATerm = BoolSort::true_term().into();

                rewrite_rules.push(Rule {
                    conditions: vec![Condition {
                        lhs: equation.condition,
                        rhs: t.into(),
                        equality: true,
                    }],
                    lhs: equation.lhs,
                    rhs: equation.rhs,
                })
            }
        }

        RewriteSpecification { rewrite_rules }
    }
}

impl fmt::Display for RewriteSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rule in &self.rewrite_rules {
            writeln!(f, "{}", rule)?;
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
