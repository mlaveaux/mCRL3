#![forbid(unsafe_code)]

use merc_lts::LTS;
use merc_utilities::Timing;

use crate::Equivalence;
use crate::Partition;
use crate::branching_bisim_sigref;
use crate::branching_bisim_sigref_naive;
use crate::strong_bisim_sigref;
use crate::strong_bisim_sigref_naive;
use crate::weak_bisim_sigref_inductive_naive;
use crate::weak_bisim_sigref_naive;
use crate::weak_bisimulation;
use crate::weak_bisimulation_parallel;

// Compare two LTSs for equivalence using the given algorithm.
pub fn compare_lts<L: LTS>(equivalence: Equivalence, left: L, right: L, preprocess: bool, timing: &mut Timing) -> bool {
    let (merged, rhs_initial) = timing.measure("merge lts", || left.merge_disjoint(&right));
    drop(right); // No longer needed.

    // Reduce the merged LTS modulo the given equivalence and return the partition
    match equivalence {
        Equivalence::WeakBisim => {
            let (lts, rhs_initial, partition) = weak_bisimulation(merged, rhs_initial, preprocess, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::WeakBisimParallel => {
            let (lts, rhs_initial, partition) = weak_bisimulation_parallel(merged, rhs_initial, preprocess, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::WeakBisimSigref => {
            let (lts, rhs_initial, partition) = weak_bisim_sigref_inductive_naive(merged, rhs_initial, preprocess, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::WeakBisimSigrefNaive => {
            let (lts, rhs_initial, partition) = weak_bisim_sigref_naive(merged, rhs_initial, preprocess, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::StrongBisim => {
            let (lts, partition) = strong_bisim_sigref(merged, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::StrongBisimNaive => {
            let (lts, partition) = strong_bisim_sigref_naive(merged, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::BranchingBisim => {
            let (lts, rhs_initial, partition) = branching_bisim_sigref(merged, rhs_initial, false, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::BranchingBisimNaive => {
            let (lts, rhs_initial, partition) = branching_bisim_sigref_naive(merged, rhs_initial, false, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::BranchingBisimDivergencePreserving => {
            let (lts, rhs_initial, partition) = branching_bisim_sigref(merged, rhs_initial, true, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
        Equivalence::BranchingBisimDivergencePreservingNaive => {
            let (lts, rhs_initial, partition) = branching_bisim_sigref_naive(merged, rhs_initial, true, timing);
            partition.block_number(lts.initial_state_index()) == partition.block_number(rhs_initial)
        }
    }
}

#[cfg(test)]
mod tests {
    use merc_io::DumpFiles;
    use merc_lts::LTS;
    use merc_lts::LabelledTransitionSystem;
    use merc_lts::StateIndex;
    use merc_lts::random_lts;
    use merc_lts::write_aut;
    use merc_utilities::Timing;
    use merc_utilities::random_test;
    use rand::seq::IndexedRandom;

    use crate::compare;
    use crate::compare_lts;

    #[test]
    #[cfg_attr(miri, ignore)] // Test is too slow under miri
    fn test_random_lts_permutation() {
        random_test(100, |rng| {
            let mut timing = Timing::new();
            let mut files = DumpFiles::new("test_random_lts_permutation");

            let lts = random_lts(rng, 10, 3, 3);
            files.dump("input.aut", |w| write_aut(w, &lts)).unwrap();

            // Generate a random permutation of the state indices.
            let permutation = (0..lts.num_of_states())
                .collect::<Vec<_>>()
                .sample(rng, lts.num_of_states())
                .map(|state| StateIndex::new(*state))
                .collect::<Vec<_>>();

            println!("Permutation: {:?}", permutation);

            let permuted_lts = LabelledTransitionSystem::new_from_permutation(lts.clone(), |i| permutation[i]);
            files.dump("permuted.aut", |w| write_aut(w, &permuted_lts)).unwrap();

            // Check that the original and permuted LTS are bisimilar.
            assert!(compare_lts(
                compare::Equivalence::StrongBisim,
                lts,
                permuted_lts,
                false,
                &mut timing
            ));
        })
    }
}
