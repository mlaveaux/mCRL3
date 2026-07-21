//! The public entry point for **robustness analysis**: checking a lowered
//! specification's `formula` and `distance` declarations against the system
//! it describes.
//!
//! Where [Simulation](super::Simulation) answers "what does one run of this
//! specification look like", [Analysis] answers the question the language
//! actually exists for: *how much does the system's behaviour change when the
//! environment is perturbed, and is that change within tolerance?* Concretely
//! it drives the three layers below it —
//!
//! ```text
//! formula   \D[d, p] >= eta, \G, \F, \U, &&, ||, !     (super::formula)
//!    |       compares a distance against a threshold
//! distance  < rho, \F, \G, \U, min, max, weights       (super::distance)
//!    |       lifts a penalty to a pair of distributions
//! sequence  a sample set per step, perturbed copies    (super::sequence)
//! ```
//!
//! # Why the sequence is a separate argument
//!
//! Every check takes the [EvolutionSequence] it runs against as an explicit
//! `&mut` parameter rather than [Analysis] owning it. That mirrors the
//! original, which passes the sequence into each check, and it is what lets
//! one analysis — one RNG stream, one set of options — be reused
//! across several sequences, and lets a sequence be reused across several
//! formulas without regenerating it. Generation is the expensive part, so
//! keeping it out of the analysis object is deliberate: checking five
//! formulas against one sequence samples the system once, not five times.
//!
//! # Reproducibility
//!
//! Everything stochastic — initial sampling, stepping, perturbation values,
//! and the bootstrap resampling — draws from the single RNG this object
//! owns, so a whole analysis is reproducible from its seed. As with
//! [Simulation](super::Simulation), the stream is **not** bit-compatible with
//! the original's; only the distributions match.

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::ir::IrProgram;
use crate::ir::PerturbationId;
use crate::value::EvalError;

use super::sequence::EvolutionSequence;
use super::store::Store;

/// The statistical knobs of an analysis. The defaults are the original's:
/// 50 bootstrap replicas at a quantile of 1.96 (a 95% normal interval).
#[derive(Clone, Copy, Debug)]
pub struct AnalysisOptions {
    /// Samples per step in the reference evolution sequence — how finely the
    /// state distribution is approximated. Larger is more accurate and
    /// linearly more expensive.
    pub sample_size: usize,
    /// How many perturbed samples are drawn per reference sample. The
    /// perturbed sequence therefore holds `sample_size * scale` samples, and each
    /// reference sample is compared against the `scale` perturbed samples
    /// descended from it.
    pub scale: usize,
    /// Bootstrap replicas (`m`). Below `2` the confidence interval collapses
    /// to the point estimate, which makes the three-valued semantics behave
    /// like the two-valued one.
    pub bootstrap_replicas: usize,
    /// The standard-normal quantile (`z`) the confidence interval spans.
    pub quantile: f64,
}

impl Default for AnalysisOptions {
    fn default() -> AnalysisOptions {
        AnalysisOptions {
            sample_size: 100,
            scale: 1,
            bootstrap_replicas: 50,
            quantile: 1.96,
        }
    }
}

/// A robustness analysis over one lowered program: the RNG stream, the
/// options, and the `const`/`param` store that interval bounds, thresholds
/// and perturbation timings are evaluated against.
///
/// The interesting methods live in the sibling modules — [Analysis::check]
/// and [Analysis::check_boolean] in [super::formula], [Analysis::distance] in
/// [super::distance] — since each is a faithful port of one reference file
/// and reads better next to the doc comment explaining that file.
pub struct Analysis<'a, R: Rng> {
    pub(crate) program: &'a IrProgram,
    /// A store used only for program-level constants. Its `[0, n_variables)`
    /// prefix is never stepped, so reading a *variable* through it would be
    /// meaningless — but no interval bound, threshold or weight can refer to
    /// one, since those are all evaluated outside any state in the original
    /// too.
    pub(crate) globals: Store,
    pub(crate) rng: R,
    pub(crate) options: AnalysisOptions,
}

impl<'a> Analysis<'a, StdRng> {
    /// Builds an analysis seeded from a `u64`, for reproducibility.
    pub fn new(program: &'a IrProgram, seed: u64, options: AnalysisOptions) -> Result<Analysis<'a, StdRng>, EvalError> {
        Analysis::with_rng(program, StdRng::seed_from_u64(seed), options)
    }
}

impl<'a, R: Rng> Analysis<'a, R> {
    /// Builds an analysis from an already-constructed RNG — the seam a test
    /// uses to inject a deterministic generator.
    pub fn with_rng(program: &'a IrProgram, mut rng: R, options: AnalysisOptions) -> Result<Analysis<'a, R>, EvalError> {
        let globals = Store::new(program, &mut rng)?;
        Ok(Analysis {
            program,
            globals,
            rng,
            options,
        })
    }

    /// Samples a fresh reference evolution sequence of
    /// [AnalysisOptions::sample_size] trajectories, to check formulas
    /// against.
    pub fn sample(&mut self) -> Result<EvolutionSequence, EvalError> {
        EvolutionSequence::generate(self.program, &mut self.rng, self.options.sample_size)
    }

    /// The distance between `sequence` and a copy of it perturbed from
    /// `step` onwards — the value an atomic formula compares against its
    /// threshold, exposed on its own so a caller can report *how far off* a
    /// system is rather than only whether it passed.
    pub fn distance_under(
        &mut self,
        sequence: &mut EvolutionSequence,
        step: usize,
        distance: crate::ir::DistanceId,
        perturbation: PerturbationId,
    ) -> Result<f64, EvalError> {
        let mut perturbed = self.perturb(sequence, perturbation, step)?;
        self.distance(sequence, &mut perturbed, step, distance)
    }

    /// Builds the perturbed counterpart of `sequence`.
    pub(crate) fn perturb(
        &mut self,
        sequence: &mut EvolutionSequence,
        perturbation: PerturbationId,
        step: usize,
    ) -> Result<EvolutionSequence, EvalError> {
        sequence.perturbed(
            self.program,
            &mut self.globals,
            &mut self.rng,
            perturbation,
            step,
            self.options.scale,
        )
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;
    use crate::UntypedStarkSpecification;
    use crate::eval::TruthValue;
    use crate::lower;

    fn build(source: &str) -> IrProgram {
        let spec = UntypedStarkSpecification::parse(source)
            .expect("should parse")
            .check()
            .expect("should check");
        lower(&spec).expect("should lower")
    }

    /// A deterministic system whose single variable holds still, with a
    /// perturbation that shifts it by a known amount. Because nothing
    /// samples, every trajectory is identical and the distance is exactly
    /// the shift — which makes the expected values below arithmetic rather
    /// than statistical.
    const SHIFT: &str = r"
        global variables {
          real x = 1.0;
        }
        environment {
          x' = x;
        }
        penalty rho = x
        distance d = < rho;
        perturbation shift = [x <- x + 0.25]@0;
    ";

    fn analysis(program: &IrProgram) -> Analysis<'_, StdRng> {
        Analysis::new(
            program,
            0,
            AnalysisOptions {
                sample_size: 4,
                ..AnalysisOptions::default()
            },
        )
        .expect("should initialise")
    }

    #[test]
    fn an_atomic_distance_measures_the_perturbations_shift() {
        let program = build(SHIFT);
        let mut analysis = analysis(&program);
        let mut sequence = analysis.sample().expect("should sample");

        let distance = analysis
            .distance_under(
                &mut sequence,
                0,
                program.distance_decls()[0].root,
                program.perturbation_decls()[0].root,
            )
            .expect("should compute");
        assert_eq!(distance, 0.25);
    }

    #[test]
    fn an_unperturbed_sequence_is_at_distance_zero_from_itself() {
        let program = build(&format!("{SHIFT} perturbation nothing = nil;"));
        let mut analysis = analysis(&program);
        let mut sequence = analysis.sample().expect("should sample");

        let distance = analysis
            .distance_under(
                &mut sequence,
                0,
                program.distance_decls()[0].root,
                program.perturbation_decls()[1].root,
            )
            .expect("should compute");
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn a_threshold_formula_is_decided_when_the_system_is_deterministic() {
        // Deterministic system => every bootstrap resample gives the same
        // distance => a zero-width confidence interval => never `Unknown`.
        let program = build(&format!("{SHIFT} formula within = \\D[d,shift] <= 0.5;"));
        let mut analysis = analysis(&program);
        let mut sequence = analysis.sample().expect("should sample");

        let verdict = analysis
            .check(&mut sequence, 0, program.formula_decls()[0].root)
            .expect("should check");
        assert_eq!(verdict, TruthValue::True);
    }

    #[test]
    fn a_threshold_formula_fails_when_the_shift_exceeds_it() {
        let program = build(&format!("{SHIFT} formula within = \\D[d,shift] <= 0.1;"));
        let mut analysis = analysis(&program);
        let mut sequence = analysis.sample().expect("should sample");

        assert_eq!(
            analysis.check(&mut sequence, 0, program.formula_decls()[0].root),
            Ok(TruthValue::False)
        );
        assert_eq!(
            analysis.check_boolean(&mut sequence, 0, program.formula_decls()[0].root),
            Ok(false)
        );
    }

    #[test]
    fn both_semantics_agree_on_a_deterministic_system() {
        let program = build(&format!("{SHIFT} formula within = \\D[d,shift] <= 0.5;"));
        let mut analysis = analysis(&program);
        let mut sequence = analysis.sample().expect("should sample");
        let root = program.formula_decls()[0].root;

        let three_valued = analysis.check(&mut sequence, 0, root).expect("should check");
        let boolean = analysis.check_boolean(&mut sequence, 0, root).expect("should check");
        assert_eq!(three_valued, TruthValue::from(boolean));
    }

    #[test]
    fn the_same_seed_reproduces_the_same_verdict() {
        // A genuinely stochastic system, so the verdict depends on the RNG
        // stream — including the bootstrap resampling — end to end. Checked
        // at step 1, since the samples only diverge after a step has been
        // taken (`typecheck.rs` forbids sampling in an initializer).
        let program = build(
            r"
            global variables {
              real x = 0.0;
            }
            environment {
              x' = R[0,1];
            }
            penalty rho = x
            distance d = < rho;
            perturbation shift = [x <- x + 0.25]@0;
            formula within = \D[d,shift] <= 0.5;
            ",
        );

        let verdicts: Vec<_> = (0..2)
            .map(|_| {
                let mut analysis = analysis(&program);
                let mut sequence = analysis.sample().expect("should sample");
                analysis
                    .check(&mut sequence, 1, program.formula_decls()[0].root)
                    .expect("should check")
            })
            .collect();
        assert_eq!(verdicts[0], verdicts[1]);
    }
}
