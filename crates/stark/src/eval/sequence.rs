//! Evolution sequences and sample sets — the stochastic counterpart of
//! [super::sim]'s single trajectory, and what every distance and ROBTL
//! formula is actually evaluated over.
//!
//! Ported from `EvolutionSequence.java` + `SampleSet.java`. Because the
//! language is stochastic, "the state at time `t`" is not one state but a
//! *distribution*, approximated by `size` independently sampled
//! [SystemState]s — a `SampleSet`. An [EvolutionSequence] is the sequence of
//! those sample sets, generated lazily:
//! [EvolutionSequence::generate_up_to] extends it on demand, matching
//! `generateUpTo`.
//!
//! Two sequences (a reference one and a perturbed one) are compared by
//! lifting a *penalty function* — a `real`-valued expression over a state —
//! to distributions. The lifting is the Wasserstein distance between the two
//! sampled distributions of penalty values, computed from the sorted arrays
//! by [wasserstein] exactly as `SampleSet.computeDistance` does.

use rand::Rng;

use crate::ir::IrProgram;
use crate::ir::PenaltyId;
use crate::ir::PerturbationId;
use crate::value::EvalError;
use crate::value::Value;

use super::expr::eval;
use super::perturbation::PerturbationState;
use super::perturbation::apply_effect;
use super::store::Store;
use super::system::SystemState;

/// A sequence of sample sets, one per time step, extended on demand.
///
/// A *perturbed* sequence additionally carries the [PerturbationState] it is
/// being rewritten by — `PerturbedEvolutionSequence` in the reference, which
/// is the same class plus a perturbation that advances alongside generation.
/// Modelling it as a field rather than a subclass keeps one generation path
/// (see [EvolutionSequence::generate_next]).
#[derive(Clone, Debug)]
pub struct EvolutionSequence {
    /// `steps[t]` is the sample set at time `t`; always non-empty (`steps[0]`
    /// is the initial distribution).
    steps: Vec<Vec<SystemState>>,
    /// `None` for an unperturbed sequence.
    perturbation: Option<PerturbationState>,
}

impl EvolutionSequence {
    /// Samples `size` independent initial states — `SampleSet.generate`.
    pub(crate) fn generate<R: Rng + ?Sized>(
        program: &IrProgram,
        rng: &mut R,
        size: usize,
    ) -> Result<EvolutionSequence, EvalError> {
        if size == 0 {
            return Err(EvalError::EmptySampleSet);
        }
        let mut initial = Vec::with_capacity(size);
        for _ in 0..size {
            initial.push(SystemState::new(program, rng)?);
        }
        log::debug!("generated an initial sample set of {size} states");
        Ok(EvolutionSequence {
            steps: vec![initial],
            perturbation: None,
        })
    }

    /// The number of samples in the initial sample set — `size` in the
    /// reference. A perturbed sequence's sample sets are `scale` times larger
    /// *from the perturbed step onwards*, but its shared history (including
    /// step 0, which this reads) keeps the original size.
    pub fn size(&self) -> usize {
        self.steps[0].len()
    }

    /// The `[0, n_variables)` state of every sample at step `t`, generating
    /// the sequence that far if necessary — the sampled distribution itself,
    /// for a caller that wants to plot or export it rather than only ask a
    /// formula about it.
    pub fn states<R: Rng + ?Sized>(
        &mut self,
        program: &IrProgram,
        rng: &mut R,
        t: usize,
    ) -> Result<Vec<&[Value]>, EvalError> {
        self.generate_up_to(program, rng, t)?;
        Ok(self.steps[t].iter().map(|state| state.variables(program)).collect())
    }

    /// The last time step generated so far — `getLastGeneratedStep`.
    fn last_generated_step(&self) -> usize {
        self.steps.len() - 1
    }

    /// Extends the sequence so that step `n` exists — `generateUpTo`.
    pub(crate) fn generate_up_to<R: Rng + ?Sized>(
        &mut self,
        program: &IrProgram,
        rng: &mut R,
        n: usize,
    ) -> Result<(), EvalError> {
        while self.last_generated_step() < n {
            let next = self.generate_next(program, rng)?;
            self.steps.push(next);
        }
        Ok(())
    }

    /// One step of every sample — `generateNextStep`, including
    /// `PerturbedEvolutionSequence`'s override, which advances the
    /// perturbation *before* generating and applies the resulting effect
    /// *after*.
    fn generate_next<R: Rng + ?Sized>(
        &mut self,
        program: &IrProgram,
        rng: &mut R,
    ) -> Result<Vec<SystemState>, EvalError> {
        if let Some(perturbation) = self.perturbation.take() {
            self.perturbation = Some(perturbation.step());
        }
        let mut next = self.steps[self.last_generated_step()].clone();
        for state in &mut next {
            state.sample_next(program, rng)?;
        }
        self.apply_perturbation_effect(program, rng, &mut next)?;
        log::trace!("generated sample set for step {}", self.steps.len());
        Ok(next)
    }

    /// `PerturbedEvolutionSequence.doApply`: rewrites every sample with the
    /// perturbation's current effect, if it has one this tick.
    fn apply_perturbation_effect<R: Rng + ?Sized>(
        &self,
        program: &IrProgram,
        rng: &mut R,
        sample: &mut [SystemState],
    ) -> Result<(), EvalError> {
        let Some(node) = self.perturbation.as_ref().and_then(PerturbationState::effect) else {
            return Ok(());
        };
        for state in sample {
            apply_effect(program, &mut state.store, rng, node)?;
        }
        Ok(())
    }

    /// The sequence obtained by perturbing this one from step `step` onwards
    /// — `EvolutionSequence.apply(perturbation, perturbedStep, scale)`.
    ///
    /// The result **shares this sequence's history** up to `step - 1` (a copy
    /// here, where Java shares immutable `SampleSet` objects) and re-samples
    /// from there: at `step` itself it holds this sequence's sample set
    /// replicated `scale` times, already perturbed. Replication is what makes
    /// the perturbed distribution `scale` times finer-grained than the
    /// reference one while still being paired with it sample-for-sample —
    /// which is the pairing [wasserstein] relies on.
    pub(crate) fn perturbed<R: Rng + ?Sized>(
        &mut self,
        program: &IrProgram,
        globals: &mut Store,
        rng: &mut R,
        id: PerturbationId,
        step: usize,
        scale: usize,
    ) -> Result<EvolutionSequence, EvalError> {
        self.generate_up_to(program, rng, step)?;
        let perturbation = PerturbationState::build(program, globals, rng, id)?;

        // `select(perturbedStep - 1)` — the history strictly before the
        // perturbed step, empty when `step == 0`.
        let mut steps: Vec<Vec<SystemState>> = self.steps[0..step].to_vec();

        let mut perturbed = EvolutionSequence {
            // Placeholder: `apply_perturbation_effect` only reads
            // `self.perturbation`, and `steps` is filled in just below.
            steps: Vec::new(),
            perturbation: Some(perturbation),
        };
        let mut sample: Vec<SystemState> = self.steps[step]
            .iter()
            .flat_map(|state| std::iter::repeat_n(state, scale))
            .cloned()
            .collect();
        perturbed.apply_perturbation_effect(program, rng, &mut sample)?;

        steps.push(sample);
        perturbed.steps = steps;
        Ok(perturbed)
    }

    /// Evaluates a penalty function on every sample at step `t`, returning
    /// the values **sorted ascending** — `SampleSet.evalPenaltyFunction`.
    /// The sort is what makes the two arrays comparable index-by-index in
    /// [wasserstein]: pairing the `i`-th smallest with the `i`-th smallest is
    /// the optimal transport plan on the real line.
    pub(crate) fn eval_penalty<R: Rng + ?Sized>(
        &mut self,
        program: &IrProgram,
        rng: &mut R,
        penalty: PenaltyId,
        t: usize,
    ) -> Result<Vec<f64>, EvalError> {
        self.generate_up_to(program, rng, t)?;
        let expression = program.penalty(penalty).value;
        let mut values = Vec::with_capacity(self.steps[t].len());
        for state in &mut self.steps[t] {
            // `eval` takes the store mutably because a call or a `let` writes
            // its scratch slots; those are outside the `[0, n_variables)`
            // state prefix, so evaluating a penalty cannot disturb the sample.
            values.push(eval(program, &mut state.store, rng, expression)?.as_number("a penalty function")?);
        }
        values.sort_by(f64::total_cmp);
        Ok(values)
    }
}

/// The Wasserstein lifting of a ground distance on reals to the two sampled
/// distributions `reference` and `perturbed` — `SampleSet.computeDistance`.
///
/// Both arrays must be sorted, and `perturbed.len()` must be a multiple `k`
/// of `reference.len()` (it is `k = scale` replicas, by construction in
/// [EvolutionSequence::perturbed]): the `i`-th reference sample is paired
/// with the `k` perturbed samples that descend from it, and the ground
/// distance is averaged over all `perturbed.len()` pairs.
pub(crate) fn wasserstein(
    ground: fn(f64, f64) -> f64,
    reference: &[f64],
    perturbed: &[f64],
) -> Result<f64, EvalError> {
    if reference.is_empty() || perturbed.len() % reference.len() != 0 {
        return Err(EvalError::IncompatibleSampleSizes {
            reference: reference.len(),
            perturbed: perturbed.len(),
        });
    }
    let k = perturbed.len() / reference.len();
    let mut total = 0.0;
    for (i, &left) in reference.iter().enumerate() {
        for &right in &perturbed[i * k..(i + 1) * k] {
            total += ground(left, right);
        }
    }
    Ok(total / perturbed.len() as f64)
}

/// The ground distance behind `distanceLeq` — asymmetric, penalising only
/// the perturbed value being *larger*. This is what `< penalty` (an
/// [crate::ir::DistanceIr::AtomicLeft]) asks for: "how much does perturbing
/// push the penalty up".
pub(crate) fn ground_leq(reference: f64, perturbed: f64) -> f64 {
    (perturbed - reference).max(0.0)
}

/// The mirror of [ground_leq], behind `distanceGeq` / `> penalty`.
pub(crate) fn ground_geq(reference: f64, perturbed: f64) -> f64 {
    (reference - perturbed).max(0.0)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use test_log::test;

    use super::*;
    use crate::UntypedStarkSpecification;
    use crate::lower;

    fn build(source: &str) -> IrProgram {
        let spec = UntypedStarkSpecification::parse(source)
            .expect("should parse")
            .check()
            .expect("should check");
        lower(&spec).expect("should lower")
    }

    const COUNTER: &str = r"
        global variables {
          int x = 0;
        }
        environment {
          x' = x + 1;
        }
    ";

    #[test]
    fn a_sequence_generates_lazily_and_advances_every_sample() {
        let program = build(COUNTER);
        let mut rng = StdRng::seed_from_u64(0);
        let mut sequence = EvolutionSequence::generate(&program, &mut rng, 4).expect("should generate");
        assert_eq!(sequence.last_generated_step(), 0);

        let sample = sequence.states(&program, &mut rng, 3).expect("should generate up to 3");
        assert_eq!(sample.len(), 4);
        for state in sample {
            assert_eq!(state, &[Value::Integer(3)]);
        }
        assert_eq!(sequence.last_generated_step(), 3);
    }

    #[test]
    fn a_perturbed_sequence_shares_history_and_diverges_from_the_perturbed_step() {
        let program = build(&format!("{COUNTER} perturbation bump = [x <- x + 100]@0;"));
        let mut rng = StdRng::seed_from_u64(0);
        let mut globals = Store::new(&program, &mut rng).expect("should initialise");
        let mut reference = EvolutionSequence::generate(&program, &mut rng, 2).expect("should generate");
        let root = program.perturbation_decls()[0].root;

        let mut perturbed = reference
            .perturbed(&program, &mut globals, &mut rng, root, 2, 3)
            .expect("should perturb");

        // Shared history: step 1 is identical, and the perturbation only
        // takes effect from step 2.
        assert_eq!(
            perturbed.states(&program, &mut rng, 1).expect("step 1")[0],
            &[Value::Integer(1)]
        );
        // Replicated `scale = 3` times, and perturbed at the step itself.
        let at_step = perturbed.states(&program, &mut rng, 2).expect("step 2");
        assert_eq!(at_step.len(), 6);
        assert_eq!(at_step[0], &[Value::Integer(102)]);
        // Step 0 is shared history at the original, unreplicated size.
        assert_eq!(perturbed.size(), 2);
        // The atomic perturbation fires once, so the offset persists but is
        // not re-applied.
        assert_eq!(
            perturbed.states(&program, &mut rng, 3).expect("step 3")[0],
            &[Value::Integer(103)]
        );
        assert_eq!(
            reference.states(&program, &mut rng, 3).expect("step 3")[0],
            &[Value::Integer(3)]
        );
    }

    #[test]
    fn wasserstein_averages_the_ground_distance_over_every_pair() {
        // 1 reference sample against 2 perturbed replicas: (|3-1| + |5-1|)/2.
        let distance = wasserstein(|a, b| (b - a).abs(), &[1.0], &[3.0, 5.0]).expect("should compute");
        assert_eq!(distance, 3.0);
    }

    #[test]
    fn wasserstein_rejects_incommensurable_sample_sizes() {
        assert_eq!(
            wasserstein(|a, b| (b - a).abs(), &[1.0, 2.0], &[3.0, 4.0, 5.0]),
            Err(EvalError::IncompatibleSampleSizes {
                reference: 2,
                perturbed: 3
            })
        );
    }

    #[test]
    fn the_asymmetric_ground_distances_only_penalise_one_direction() {
        assert_eq!(ground_leq(1.0, 3.0), 2.0);
        assert_eq!(ground_leq(3.0, 1.0), 0.0);
        assert_eq!(ground_geq(1.0, 3.0), 0.0);
        assert_eq!(ground_geq(3.0, 1.0), 2.0);
    }

    #[test]
    fn a_penalty_is_evaluated_on_every_sample_and_returned_sorted() {
        // `typecheck.rs` forbids sampling in a variable's initializer, so the
        // samples only diverge once a step has been taken — hence step `1`
        // rather than `0`.
        let program = build(
            r"
            global variables {
              real x = 0.0;
            }
            environment {
              x' = R[0,10];
            }
            penalty rho = x
            ",
        );
        let mut rng = StdRng::seed_from_u64(7);
        let mut sequence = EvolutionSequence::generate(&program, &mut rng, 5).expect("should generate");
        let values = sequence
            .eval_penalty(&program, &mut rng, PenaltyId::new(0), 1)
            .expect("should evaluate");

        assert_eq!(values.len(), 5);
        assert!(values.is_sorted());
        // The samples are genuinely independent draws, not one value copied.
        assert!(values[0] < values[4]);
    }
}
