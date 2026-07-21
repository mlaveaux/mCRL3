//! Distance expressions: how far apart two evolution sequences are, as a
//! single `f64` per time step.
//!
//! Every node reduces, eventually, to an *atomic* distance — a penalty
//! function lifted to the two sampled distributions by [wasserstein] — with
//! the temporal and lattice operators (`\F`, `\G`, `\U`, `min`, `max`,
//! thresholds, convex combinations) combining those pointwise values over an
//! interval.
//!
//! Two things about the original semantics are preserved deliberately and
//! are easy to get wrong:
//!
//! - **`\F` is a minimum and `\G` is a maximum.** A distance measures
//!   *dissimilarity*, so "eventually close" is the best (smallest) distance
//!   over the interval and "always close" is the worst (largest) one. This
//!   inverts the intuition from the formula layer, where `\F` is a
//!   disjunction.
//! - **A distance interval `[from, to]` excludes `to`**, whereas the
//!   *formula* layer (see [super::formula]) includes it. That inconsistency
//!   is the original's, not this port's, and is preserved so results match.
//!
//! An empty interval yields `NaN` rather than being an error, again matching
//! the original.
//!
//! # Confidence intervals
//!
//! Each node has two evaluations: [Analysis::distance], the plain value, and
//! [Analysis::distance_ci], which additionally carries an empirical-bootstrap
//! confidence interval. Only the three-valued formula semantics needs the
//! latter — it is what lets a verdict be `Unknown` when the threshold falls
//! *inside* the interval, i.e. when the sample size cannot decide the
//! question. See [super::formula].

use rand::Rng;
use rand::RngExt;

use crate::ir::ComparisonOp;
use crate::ir::DistanceId;
use crate::ir::DistanceIr;
use crate::ir::ExprRef;
use crate::value::EvalError;

use super::expr::eval;
use super::robust::Analysis;
use super::sequence::EvolutionSequence;
use super::sequence::ground_geq;
use super::sequence::ground_leq;
use super::sequence::wasserstein;

/// A distance value together with the empirical-bootstrap confidence
/// interval around it — the original's bare `{value, lower, upper}` triple,
/// named.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ci {
    pub value: f64,
    pub lower: f64,
    pub upper: f64,
}

impl Ci {
    /// Combines two intervals component-wise, as every `min`/`max` node
    /// does: the same operator is applied to the value and to both bounds
    /// independently, rather than propagating the bounds of whichever
    /// operand won.
    fn zip(self, other: Ci, combine: fn(f64, f64) -> f64) -> Ci {
        Ci {
            value: combine(self.value, other.value),
            lower: combine(self.lower, other.lower),
            upper: combine(self.upper, other.upper),
        }
    }

    /// A degenerate interval, for a value known exactly.
    fn exact(value: f64) -> Ci {
        Ci {
            value,
            lower: value,
            upper: value,
        }
    }
}

impl ComparisonOp {
    /// Applies this operator to two distance values.
    pub(crate) fn compare(self, left: f64, right: f64) -> bool {
        match self {
            ComparisonOp::Less => left < right,
            ComparisonOp::Leq => left <= right,
            ComparisonOp::Eq => left == right,
            ComparisonOp::Geq => left >= right,
            ComparisonOp::Greater => left > right,
        }
    }
}

/// `NaN`-propagating `min`/`max`, unlike [f64::min]/[f64::max], which return
/// the non-`NaN` operand. An empty interval produces `NaN`, and it must stay
/// `NaN` through the enclosing operators rather than being silently absorbed
/// — which is also what the original does.
fn nan_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.min(b) }
}

fn nan_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.max(b) }
}

impl<R: Rng> Analysis<'_, R> {
    /// Evaluates a distance expression between `reference` and `perturbed` at
    /// time `step`.
    pub(crate) fn distance(
        &mut self,
        reference: &mut EvolutionSequence,
        perturbed: &mut EvolutionSequence,
        step: usize,
        id: DistanceId,
    ) -> Result<f64, EvalError> {
        match self.program.distance(id).clone() {
            DistanceIr::Reference(target) => self.distance(reference, perturbed, step, target),
            DistanceIr::AtomicLeft(penalty) => {
                let (left, right) = self.penalties(reference, perturbed, step, penalty)?;
                wasserstein(ground_leq, &left, &right)
            }
            DistanceIr::AtomicRight(penalty) => {
                let (left, right) = self.penalties(reference, perturbed, step, penalty)?;
                wasserstein(ground_geq, &left, &right)
            }
            // `\F` is the *minimum* over the interval; see the module doc.
            DistanceIr::Eventually { from, to, argument } => {
                self.fold_interval(reference, perturbed, step, from, to, argument, nan_min)
            }
            DistanceIr::Globally { from, to, argument } => {
                self.fold_interval(reference, perturbed, step, from, to, argument, nan_max)
            }
            DistanceIr::Until { from, to, left, right } => {
                let (from, to) = self.interval(from, to, step)?;
                // For each `i`, the worse of "the right expression at `i`"
                // and "the worst the left expression has been strictly
                // before `i`"; then the best such `i`. `running_left`
                // accumulates across iterations, which is equivalent to
                // recomputing the running maximum each time.
                let mut result = 1.0;
                let mut running_left = 0.0;
                for i in from..to {
                    let right_value = self.distance(reference, perturbed, i, right)?;
                    for j in from..i {
                        running_left = nan_max(running_left, self.distance(reference, perturbed, j, left)?);
                    }
                    result = nan_min(result, nan_max(right_value, running_left));
                }
                Ok(result)
            }
            DistanceIr::Threshold { op, left, threshold } => {
                let value = self.distance(reference, perturbed, step, left)?;
                let threshold = self.constant(threshold)?;
                // Note the polarity: *satisfying* the threshold is distance
                // `0.0` (no dissimilarity), violating it is `1.0`.
                Ok(if op.compare(value, threshold) { 0.0 } else { 1.0 })
            }
            DistanceIr::Min(left, right) => {
                let left = self.distance(reference, perturbed, step, left)?;
                let right = self.distance(reference, perturbed, step, right)?;
                Ok(nan_min(left, right))
            }
            DistanceIr::Max(left, right) => {
                let left = self.distance(reference, perturbed, step, left)?;
                let right = self.distance(reference, perturbed, step, right)?;
                Ok(nan_max(left, right))
            }
            DistanceIr::LinearCombination(terms) => {
                let mut total = 0.0;
                for (weight, term) in terms {
                    total += self.constant(weight)? * self.distance(reference, perturbed, step, term)?;
                }
                Ok(total)
            }
        }
    }

    /// [Analysis::distance], plus a bootstrap confidence interval around it.
    pub(crate) fn distance_ci(
        &mut self,
        reference: &mut EvolutionSequence,
        perturbed: &mut EvolutionSequence,
        step: usize,
        id: DistanceId,
    ) -> Result<Ci, EvalError> {
        match self.program.distance(id).clone() {
            DistanceIr::Reference(target) => self.distance_ci(reference, perturbed, step, target),
            DistanceIr::AtomicLeft(penalty) => self.atomic_ci(reference, perturbed, step, penalty, ground_leq),
            DistanceIr::AtomicRight(penalty) => self.atomic_ci(reference, perturbed, step, penalty, ground_geq),
            DistanceIr::Eventually { from, to, argument } => {
                self.fold_interval_ci(reference, perturbed, step, from, to, argument, nan_min)
            }
            DistanceIr::Globally { from, to, argument } => {
                self.fold_interval_ci(reference, perturbed, step, from, to, argument, nan_max)
            }
            DistanceIr::Until { from, to, left, right } => {
                let (from, to) = self.interval(from, to, step)?;
                let mut result = Ci::exact(1.0);
                for i in from..to {
                    let right_value = self.distance_ci(reference, perturbed, i, right)?;
                    // Unlike the plain evaluation above, the original
                    // re-seeds the running left maximum from the left
                    // expression *at `i`* on every iteration before folding
                    // in `[from, i)`. Preserved as written; see `plan.md`.
                    let mut running_left = self.distance_ci(reference, perturbed, i, left)?;
                    for j in from..i {
                        running_left = running_left.zip(self.distance_ci(reference, perturbed, j, left)?, nan_max);
                    }
                    result = result.zip(right_value.zip(running_left, nan_max), nan_min);
                }
                Ok(result)
            }
            DistanceIr::Threshold { op, left, threshold } => {
                let value = self.distance_ci(reference, perturbed, step, left)?;
                let threshold = self.constant(threshold)?;
                let decided = if op.compare(value.value, threshold) { 0.0 } else { 1.0 };
                // If the threshold falls strictly inside the confidence
                // interval, the sample cannot tell which side of it the true
                // distance is on, so the *thresholded* interval spans both
                // outcomes — which is exactly what makes the enclosing
                // formula `Unknown`.
                Ok(if value.lower < threshold && threshold < value.upper {
                    Ci {
                        value: decided,
                        lower: 0.0,
                        upper: 1.0,
                    }
                } else {
                    Ci::exact(decided)
                })
            }
            DistanceIr::Min(left, right) => {
                let left = self.distance_ci(reference, perturbed, step, left)?;
                let right = self.distance_ci(reference, perturbed, step, right)?;
                Ok(left.zip(right, nan_min))
            }
            DistanceIr::Max(left, right) => {
                let left = self.distance_ci(reference, perturbed, step, left)?;
                let right = self.distance_ci(reference, perturbed, step, right)?;
                Ok(left.zip(right, nan_max))
            }
            DistanceIr::LinearCombination(terms) => {
                let mut total = Ci::exact(0.0);
                for (weight, term) in terms {
                    let weight = self.constant(weight)?;
                    let term = self.distance_ci(reference, perturbed, step, term)?;
                    total = Ci {
                        value: total.value + weight * term.value,
                        lower: total.lower + weight * term.lower,
                        upper: total.upper + weight * term.upper,
                    };
                }
                Ok(total)
            }
        }
    }

    /// The two sorted penalty-value distributions an atomic distance compares.
    fn penalties(
        &mut self,
        reference: &mut EvolutionSequence,
        perturbed: &mut EvolutionSequence,
        step: usize,
        penalty: crate::ir::PenaltyId,
    ) -> Result<(Vec<f64>, Vec<f64>), EvalError> {
        let left = reference.eval_penalty(self.program, &mut self.rng, penalty, step)?;
        let right = perturbed.eval_penalty(self.program, &mut self.rng, penalty, step)?;
        Ok((left, right))
    }

    /// An atomic distance with its bootstrap interval.
    fn atomic_ci(
        &mut self,
        reference: &mut EvolutionSequence,
        perturbed: &mut EvolutionSequence,
        step: usize,
        penalty: crate::ir::PenaltyId,
        ground: fn(f64, f64) -> f64,
    ) -> Result<Ci, EvalError> {
        let (left, right) = self.penalties(reference, perturbed, step, penalty)?;
        let value = wasserstein(ground, &left, &right)?;
        let (lower, upper) = self.bootstrap(&left, &right, ground)?;
        Ok(Ci { value, lower, upper })
    }

    /// The empirical bootstrap: resample both distributions with replacement
    /// `m` times, and take a `z`-quantile normal interval around the mean of
    /// the resulting distances.
    ///
    /// The interval is clamped to `[0, 1]` exactly as the original clamps
    /// it, which assumes penalty values are normalised to that range.
    fn bootstrap(&mut self, left: &[f64], right: &[f64], ground: fn(f64, f64) -> f64) -> Result<(f64, f64), EvalError> {
        let m = self.options.bootstrap_replicas;
        if m < 2 {
            // The standard error divides by `m - 1`; with fewer than two
            // replicas there is no spread to estimate, so report the point
            // value as exact rather than dividing by zero.
            let value = wasserstein(ground, left, right)?;
            return Ok((value, value));
        }
        let mut distances = Vec::with_capacity(m);
        let mut total = 0.0;
        for _ in 0..m {
            let left_sample = self.resample(left);
            let right_sample = self.resample(right);
            let distance = wasserstein(ground, &left_sample, &right_sample)?;
            distances.push(distance);
            total += distance;
        }
        let mean = total / m as f64;
        let variance = distances.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / (m - 1) as f64;
        let error = self.options.quantile * variance.sqrt();
        Ok(((mean - error).max(0.0), (mean + error).min(1.0)))
    }

    /// One bootstrap resample: `len` draws with replacement, sorted — the
    /// sort is required because [wasserstein] pairs by rank.
    fn resample(&mut self, data: &[f64]) -> Vec<f64> {
        let mut sample: Vec<f64> = (0..data.len())
            .map(|_| data[self.rng.random_range(0..data.len())])
            .collect();
        sample.sort_by(f64::total_cmp);
        sample
    }

    /// The `\F`/`\G` fold: `argument` over `[from + step, to + step)`, `NaN`
    /// if empty.
    #[expect(clippy::too_many_arguments, reason = "one argument per IR field, plus the fold")]
    fn fold_interval(
        &mut self,
        reference: &mut EvolutionSequence,
        perturbed: &mut EvolutionSequence,
        step: usize,
        from: ExprRef,
        to: ExprRef,
        argument: DistanceId,
        combine: fn(f64, f64) -> f64,
    ) -> Result<f64, EvalError> {
        let (from, to) = self.interval(from, to, step)?;
        let mut folded: Option<f64> = None;
        for i in from..to {
            let value = self.distance(reference, perturbed, i, argument)?;
            folded = Some(match folded {
                Some(previous) => combine(previous, value),
                None => value,
            });
        }
        Ok(folded.unwrap_or(f64::NAN))
    }

    /// [Analysis::fold_interval] for confidence intervals: the value and
    /// both bounds are folded independently.
    #[expect(clippy::too_many_arguments, reason = "one argument per IR field, plus the fold")]
    fn fold_interval_ci(
        &mut self,
        reference: &mut EvolutionSequence,
        perturbed: &mut EvolutionSequence,
        step: usize,
        from: ExprRef,
        to: ExprRef,
        argument: DistanceId,
        combine: fn(f64, f64) -> f64,
    ) -> Result<Ci, EvalError> {
        let (from, to) = self.interval(from, to, step)?;
        let mut folded: Option<Ci> = None;
        for i in from..to {
            let value = self.distance_ci(reference, perturbed, i, argument)?;
            folded = Some(match folded {
                Some(previous) => previous.zip(value, combine),
                None => value,
            });
        }
        Ok(folded.unwrap_or(Ci::exact(f64::NAN)))
    }

    /// Evaluates an interval's bounds and shifts them by `step`, as a Rust
    /// range. Both bounds are [ExprRef]s in the IR rather than folded
    /// constants, so they are evaluated here with the ordinary expression
    /// evaluator, against the program's `const`/`param` slots.
    ///
    /// A negative bound, or `to <= from`, gives an empty range rather than
    /// an error. The original rejects such bounds at construction time,
    /// which this port has no equivalent of since bounds are evaluated on
    /// demand, and the never-panic contract rules out failing here.
    pub(crate) fn interval(&mut self, from: ExprRef, to: ExprRef, step: usize) -> Result<(usize, usize), EvalError> {
        let from = self.constant_integer(from, "the lower bound of an interval")?;
        let to = self.constant_integer(to, "the upper bound of an interval")?;
        let from = (from.max(0) as usize).saturating_add(step);
        let to = (to.max(0) as usize).saturating_add(step);
        Ok((from, to.max(from)))
    }

    /// Evaluates a program-level constant expression — an interval bound, a
    /// threshold, a combination weight. These may only refer to `const`/
    /// `param` slots, which is what [Analysis::globals] holds.
    pub(crate) fn constant(&mut self, id: ExprRef) -> Result<f64, EvalError> {
        eval(self.program, &mut self.globals, &mut self.rng, id)?
            .as_f64("a distance or formula constant")
            .map_err(|kind| EvalError::from(kind).or_span(self.program.expr_span(id)))
    }

    fn constant_integer(&mut self, id: ExprRef, context: &'static str) -> Result<i64, EvalError> {
        eval(self.program, &mut self.globals, &mut self.rng, id)?
            .as_integer(context)
            .map_err(|kind| EvalError::from(kind).or_span(self.program.expr_span(id)))
    }
}
