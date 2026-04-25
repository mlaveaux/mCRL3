#![forbid(unsafe_code)]

use merc_lts::LTS;
use merc_lts::LabelledTransitionSystem;
use merc_utilities::Timing;

use crate::branching_bisim_sigref;
use crate::branching_bisim_sigref_naive;
use crate::quotient_lts_block;
use crate::quotient_lts_naive;
use crate::strong_bisim_sigref;
use crate::strong_bisim_sigref_naive;
use crate::weak_bisim_sigref_inductive_naive;
use crate::weak_bisim_sigref_naive;
use crate::weak_bisimulation;
use crate::weak_bisimulation_parallel;

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Equivalence {
    /// Partition based refinement algorithms.
    WeakBisim,
    WeakBisimParallel,
    /// Various signature based reduction algorithms.
    WeakBisimSigref,
    WeakBisimSigrefNaive,
    StrongBisim,
    StrongBisimNaive,
    BranchingBisim,
    BranchingBisimNaive,
    BranchingBisimDivergencePreserving,
    BranchingBisimDivergencePreservingNaive,
}

/// Reduces the given LTS modulo the given equivalence using signature refinement
pub fn reduce_lts<L: LTS>(
    lts: L,
    equivalence: Equivalence,
    preprocess: bool,
    timing: &Timing,
) -> LabelledTransitionSystem<L::Label> {
    let state = lts.initial_state_index();
    match equivalence {
        Equivalence::WeakBisim => {
            let (lts, _, partition) = weak_bisimulation(lts, state, preprocess, timing);
            timing.measure("quotient", || quotient_lts_naive(&lts, &partition, true))
        }
        Equivalence::WeakBisimParallel => {
            let (lts, _, partition) = weak_bisimulation_parallel(lts, state, preprocess, timing);
            timing.measure("quotient", || quotient_lts_naive(&lts, &partition, true))
        }
        Equivalence::WeakBisimSigref => {
            let (lts, _, partition) = weak_bisim_sigref_inductive_naive(lts, state, preprocess, timing);
            timing.measure("quotient", || quotient_lts_naive(&lts, &partition, true))
        }
        Equivalence::WeakBisimSigrefNaive => {
            let (lts, _, partition) = weak_bisim_sigref_naive(lts, state, preprocess, timing);
            timing.measure("quotient", || quotient_lts_naive(&lts, &partition, true))
        }
        Equivalence::StrongBisim => {
            let (lts, partition) = strong_bisim_sigref(lts, timing);
            timing.measure("quotient", || quotient_lts_block::<_, false>(&lts, &partition))
        }
        Equivalence::StrongBisimNaive => {
            let (lts, partition) = strong_bisim_sigref_naive(lts, timing);
            timing.measure("quotient", || quotient_lts_naive(&lts, &partition, false))
        }
        Equivalence::BranchingBisim => {
            let (lts, _, partition) = branching_bisim_sigref(lts, state, false, timing);
            timing.measure("quotient", || quotient_lts_block::<_, true>(&lts, &partition))
        }
        Equivalence::BranchingBisimNaive => {
            let (lts, _, partition) = branching_bisim_sigref_naive(lts, state, false, timing);
            timing.measure("quotient", || quotient_lts_naive(&lts, &partition, true))
        }
        Equivalence::BranchingBisimDivergencePreserving => {
            let (lts, _, partition) = branching_bisim_sigref(lts, state, true, timing);
            timing.measure("quotient", || quotient_lts_block::<_, true>(&lts, &partition))
        }
        Equivalence::BranchingBisimDivergencePreservingNaive => {
            let (lts, _, partition) = branching_bisim_sigref_naive(lts, state, true, timing);
            timing.measure("quotient", || quotient_lts_naive(&lts, &partition, true))
        }
    }
}
