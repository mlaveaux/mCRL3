use core::fmt;

use ahash::AHashSet;
use mcrl2::aterm::ATerm;
use mcrl2::aterm::TermPool;
use sabre::rewrite_specification::Condition;
use sabre::rewrite_specification::RewriteSpecification;
use sabre::rewrite_specification::Rule;
use sabre::utilities::to_untyped_data_expression;

/// A rewrite specification contains all the bare info we need for rewriting (in particular no type information) as a syntax tree.
/// Parsing a REC file results in a RewriteSpecificationSyntax.
#[derive(Clone, Default, Debug)]
pub struct RewriteSpecificationSyntax {
    pub rewrite_rules: Vec<RewriteRuleSyntax>,
    pub constructors: Vec<(String, usize)>,
    pub variables: Vec<String>,
}

impl RewriteSpecificationSyntax {
    pub fn to_rewrite_spec(&self, tp: &mut TermPool) -> RewriteSpecification {
        // The names for all variables
        let variables = AHashSet::from_iter(self.variables.clone());

        // Store the rewrite rules in the maximally shared term storage
        let mut rewrite_rules = Vec::new();
        for rule in &self.rewrite_rules {
            // Convert the conditions.
            let mut conditions = vec![];
            for c in &rule.conditions {
                let condition = Condition {
                    lhs: to_untyped_data_expression(tp, &c.lhs, &variables),
                    rhs: to_untyped_data_expression(tp, &c.rhs, &variables),
                    equality: c.equality,
                };
                conditions.push(condition);
            }

            rewrite_rules.push(Rule {
                lhs: to_untyped_data_expression(tp, &rule.lhs, &variables),
                rhs: to_untyped_data_expression(tp, &rule.rhs, &variables),
                conditions,
            });
        }

        RewriteSpecification { rewrite_rules }
    }

    pub fn merge(&mut self, include_spec: &RewriteSpecificationSyntax) {
        self.rewrite_rules.extend_from_slice(&include_spec.rewrite_rules);
        self.constructors.extend_from_slice(&include_spec.constructors);

        for s in &include_spec.variables {
            if !self.variables.contains(s) {
                self.variables.push(s.clone());
            }
        }
    }
}

impl fmt::Display for RewriteSpecificationSyntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Variables: ")?;
        for variable in &self.variables {
            writeln!(f, "{}", variable)?;
        }
        writeln!(f, "Rewrite rules: ")?;
        for rule in &self.rewrite_rules {
            writeln!(f, "{}", rule)?;
        }
        writeln!(f)
    }
}

/// Syntax tree for rewrite rules
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RewriteRuleSyntax {
    pub lhs: ATerm,
    pub rhs: ATerm,
    pub conditions: Vec<ConditionSyntax>,
}

impl fmt::Display for RewriteRuleSyntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.lhs, self.rhs)
    }
}

/// Syntax tree for conditional part of a rewrite rule
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConditionSyntax {
    pub lhs: ATerm,
    pub rhs: ATerm,
    pub equality: bool, // The condition either specifies that lhs and rhs are equal or different
}
