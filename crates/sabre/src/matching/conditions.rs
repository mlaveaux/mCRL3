use crate::utilities::TermStack;
use crate::Rule;
use crate::utilities::create_var_map;

/// This is a [Rule] condition stored as semi compressed trees such that they can be
/// subsituted efficiently.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct EMACondition {
    /// Conditions lhs and rhs are stored in the term pool as much as possible with a SemiCompressedTermTree
    pub semi_compressed_lhs: TermStack,
    pub semi_compressed_rhs: TermStack,

    /// whether the lhs and rhs should be equal or different
    pub equality: bool,
}

/// Computes the extended condition from a given rewrite rule.
pub fn extend_conditions(rule: &Rule) -> Vec<EMACondition> {
    let var_map = create_var_map(&rule.lhs);
    let mut conditions = vec![];

    for c in &rule.conditions {
        let ema_condition = EMACondition {
            semi_compressed_lhs: TermStack::from_term(&c.lhs.copy(), &var_map),
            semi_compressed_rhs: TermStack::from_term(&c.rhs.copy(), &var_map),
            equality: c.equality,
        };
        conditions.push(ema_condition);
    }

    conditions
}
