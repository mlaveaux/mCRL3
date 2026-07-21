//! ROBTL formulas: the top of the verification stack. A formula is checked
//! against *one* evolution sequence — the reference behaviour — and each
//! atomic proposition compares that sequence against a perturbed copy of
//! itself.
//!
//! Two semantics, both from the original:
//!
//! - [Analysis::check] — the **three-valued** semantics, the one the tool
//!   uses by default. A verdict may be [TruthValue::Unknown] when the sample
//!   size is too small to place the true distance on one side of the
//!   threshold; this is statistical honesty, not a modelling gap, and it is
//!   the reason the distance layer computes confidence intervals at all.
//! - [Analysis::check_boolean] — the **two-valued** semantics, which compares
//!   point estimates only. It is cheaper (no bootstrap) and is what you want
//!   when you have already decided the sample is large enough.
//!
//! Note the interval convention differs from [super::distance]'s: a formula's
//! `[from, to]` **includes** `to`, whereas a distance's excludes it.
//! Preserved as-is; see the distance module doc.

use rand::Rng;

use crate::ir::FormulaId;
use crate::ir::FormulaIr;
use crate::value::EvalError;

use super::robust::Analysis;
use super::sequence::EvolutionSequence;

/// A three-valued verdict. [TruthValue::Unknown] means the
/// samples were not conclusive, not that the formula is undefined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruthValue {
    True,
    False,
    Unknown,
}

impl TruthValue {
    /// Kleene conjunction: `false` is absorbing, so an `Unknown` operand only
    /// matters when the other one is not already decisive.
    pub fn and(self, other: TruthValue) -> TruthValue {
        match (self, other) {
            (TruthValue::False, _) | (_, TruthValue::False) => TruthValue::False,
            (TruthValue::True, TruthValue::True) => TruthValue::True,
            _ => TruthValue::Unknown,
        }
    }

    /// Kleene disjunction — the dual of [TruthValue::and].
    pub fn or(self, other: TruthValue) -> TruthValue {
        match (self, other) {
            (TruthValue::True, _) | (_, TruthValue::True) => TruthValue::True,
            (TruthValue::False, TruthValue::False) => TruthValue::False,
            _ => TruthValue::Unknown,
        }
    }

    /// Kleene negation: `Unknown` is its own negation. Named to complete the
    /// `and`/`or`/`not` trio rather than to mirror `std::ops::Not`, which
    /// would force a `Not` impl for one call site.
    #[expect(clippy::should_implement_trait, reason = "reads as the Kleene-logic trio and/or/not")]
    pub fn not(self) -> TruthValue {
        match self {
            TruthValue::True => TruthValue::False,
            TruthValue::False => TruthValue::True,
            TruthValue::Unknown => TruthValue::Unknown,
        }
    }

    /// `1.0`/`0.0`/`-1.0`, for callers that want a numeric verdict.
    pub fn as_f64(self) -> f64 {
        match self {
            TruthValue::True => 1.0,
            TruthValue::Unknown => 0.0,
            TruthValue::False => -1.0,
        }
    }
}

impl From<bool> for TruthValue {
    fn from(value: bool) -> TruthValue {
        if value { TruthValue::True } else { TruthValue::False }
    }
}

impl<R: Rng> Analysis<'_, R> {
    /// Checks a formula against `sequence` at time `step`, under the
    /// three-valued semantics.
    pub fn check(
        &mut self,
        sequence: &mut EvolutionSequence,
        step: usize,
        id: FormulaId,
    ) -> Result<TruthValue, EvalError> {
        match self.program.formula(id).clone() {
            FormulaIr::True => Ok(TruthValue::True),
            FormulaIr::False => Ok(TruthValue::False),
            FormulaIr::Reference(target) => self.check(sequence, step, target),
            FormulaIr::Distance {
                distance,
                perturbation,
                op,
                value,
            } => {
                let threshold = self.constant(value)?;
                let mut perturbed = self.perturb(sequence, perturbation, step)?;
                let result = self.distance_ci(sequence, &mut perturbed, step, distance)?;
                log::debug!(
                    "\\D at step {step}: distance {} in [{}, {}] against threshold {threshold}",
                    result.value,
                    result.lower,
                    result.upper
                );
                // Undecidable exactly when the threshold falls strictly
                // inside the confidence interval: the samples are consistent
                // with the true distance being on either side of it.
                if result.lower < threshold && threshold < result.upper {
                    Ok(TruthValue::Unknown)
                } else {
                    Ok(TruthValue::from(op.compare(result.value, threshold)))
                }
            }
            FormulaIr::Not(inner) => Ok(self.check(sequence, step, inner)?.not()),
            FormulaIr::Globally { from, to, argument } => {
                let (from, to) = self.interval(from, to, step)?;
                let mut value = TruthValue::True;
                // `to` is inclusive here, unlike a distance interval.
                for i in from..=to {
                    value = value.and(self.check(sequence, i, argument)?);
                    // `false` is absorbing, so nothing later can change the
                    // verdict — the original short-circuits here too.
                    if value == TruthValue::False {
                        break;
                    }
                }
                Ok(value)
            }
            FormulaIr::Eventually { from, to, argument } => {
                let (from, to) = self.interval(from, to, step)?;
                let mut value = TruthValue::False;
                for i in from..=to {
                    value = value.or(self.check(sequence, i, argument)?);
                    if value == TruthValue::True {
                        break;
                    }
                }
                Ok(value)
            }
            FormulaIr::And(left, right) => {
                let left = self.check(sequence, step, left)?;
                let right = self.check(sequence, step, right)?;
                Ok(left.and(right))
            }
            FormulaIr::Or(left, right) => {
                let left = self.check(sequence, step, left)?;
                let right = self.check(sequence, step, right)?;
                Ok(left.or(right))
            }
            FormulaIr::Until { from, to, left, right } => {
                let (from, to) = self.interval(from, to, step)?;
                // Walk forward while the left side still holds, looking for
                // a point where the right side does.
                // `left_value` accumulates the conjunction of the left side
                // over everything seen so far.
                let mut value = TruthValue::False;
                let mut left_value = TruthValue::True;
                for i in from..=to {
                    if value == TruthValue::True || left_value == TruthValue::False {
                        break;
                    }
                    value = left_value.and(self.check(sequence, i, right)?);
                    if value != TruthValue::True {
                        left_value = left_value.and(self.check(sequence, i, left)?);
                    }
                }
                Ok(value)
            }
        }
    }

    /// Checks a formula under the two-valued semantics. Compares point
    /// estimates, so it never needs the bootstrap and never answers
    /// "unknown".
    pub fn check_boolean(
        &mut self,
        sequence: &mut EvolutionSequence,
        step: usize,
        id: FormulaId,
    ) -> Result<bool, EvalError> {
        match self.program.formula(id).clone() {
            FormulaIr::True => Ok(true),
            FormulaIr::False => Ok(false),
            FormulaIr::Reference(target) => self.check_boolean(sequence, step, target),
            FormulaIr::Distance {
                distance,
                perturbation,
                op,
                value,
            } => {
                let threshold = self.constant(value)?;
                let mut perturbed = self.perturb(sequence, perturbation, step)?;
                let result = self.distance(sequence, &mut perturbed, step, distance)?;
                Ok(op.compare(result, threshold))
            }
            FormulaIr::Not(inner) => Ok(!self.check_boolean(sequence, step, inner)?),
            FormulaIr::Globally { from, to, argument } => {
                let (from, to) = self.interval(from, to, step)?;
                for i in from..=to {
                    if !self.check_boolean(sequence, i, argument)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            FormulaIr::Eventually { from, to, argument } => {
                let (from, to) = self.interval(from, to, step)?;
                for i in from..=to {
                    if self.check_boolean(sequence, i, argument)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            FormulaIr::And(left, right) => {
                // Short-circuiting, as in the original.
                Ok(self.check_boolean(sequence, step, left)? && self.check_boolean(sequence, step, right)?)
            }
            FormulaIr::Or(left, right) => {
                Ok(self.check_boolean(sequence, step, left)? || self.check_boolean(sequence, step, right)?)
            }
            FormulaIr::Until { from, to, left, right } => {
                let (from, to) = self.interval(from, to, step)?;
                for i in from..=to {
                    if self.check_boolean(sequence, i, right)? {
                        let mut holds = true;
                        for j in from..i {
                            if !self.check_boolean(sequence, j, left)? {
                                holds = false;
                                break;
                            }
                        }
                        if holds {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
        }
    }
}
