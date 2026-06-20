#![forbid(unsafe_code)]

use std::fmt;

use itertools::Itertools;
use merc_data::DataExpression;

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
