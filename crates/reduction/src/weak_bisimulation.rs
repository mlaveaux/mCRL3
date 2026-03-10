//! Authors: Maurice Laveaux, Eduardo Costa Martins
//!
//! Implements the weak bisimulation algorithm by Eduardo Costa Martins.
#![forbid(unsafe_code)]

use std::iter;

use bitvec::bitvec;
use bitvec::order::Lsb0;

use bitvec::vec::BitVec;
use log::info;
use log::trace;
use merc_collections::BlockIndex;
use merc_io::TimeProgress;
use merc_lts::IncomingTransitions;
use merc_lts::LabelIndex;
use merc_lts::LabelledTransitionSystem;
use merc_lts::LTS;
use merc_utilities::Timing;

use crate::reduce_lts;
use crate::tau_loop_elimination_and_reorder;
use crate::Equivalence;
use crate::MarkedBlockPartition;

/// Type alias because we use bitvec for marking states
type BitArray = BitVec<u64, Lsb0>;

/// Apply weak bisimulation reduction
///
/// # Details
///
/// The `preprocess` flag indicates whether to preprocess the LTS using
/// branching bisimulation.
pub fn weak_bisimulation<L: LTS>(
    lts: L,
    preprocess: bool,
    timing: &Timing,
) -> (LabelledTransitionSystem<L::Label>, MarkedBlockPartition) {
    // Preprocess the LTS if desired.
    if preprocess {
        let lts = timing.measure("preprocess", || {
            reduce_lts(lts, Equivalence::BranchingBisim, true, timing)
        });
        weak_bisimulation_impl(lts, timing)
    } else {
        weak_bisimulation_impl(lts, timing)
    }
}

/// Apply weak bisimulation reduction using the parallel variant of the algorithm
///
/// # Details
///
/// The `preprocess` flag indicates whether to preprocess the LTS using
/// branching bisimulation.
pub fn weak_bisimulation_parallel<L: LTS>(
    lts: L,
    preprocess: bool,
    timing: &Timing,
) -> (LabelledTransitionSystem<L::Label>, MarkedBlockPartition) {
    // Preprocess the LTS if desired.
    if preprocess {
        let lts = timing.measure("preprocess", || {
            reduce_lts(lts, Equivalence::BranchingBisim, true, timing)
        });
        weak_bisimulation_parallel_impl(lts, timing)
    } else {
        weak_bisimulation_parallel_impl(lts, timing)
    }
}

/// Core weak bisimulation algorithm implementation.
fn weak_bisimulation_impl<L: LTS>(
    lts: L,
    timing: &Timing,
) -> (LabelledTransitionSystem<L::Label>, MarkedBlockPartition) {
    let tau_loop_free_lts = timing.measure("preprocess", || tau_loop_elimination_and_reorder(lts));

    timing.measure("reduction", || {
        let mut blocks = MarkedBlockPartition::new(tau_loop_free_lts.num_of_states());

        let mut act_mark = bitvec![u64, Lsb0; 0; tau_loop_free_lts.num_of_states()];
        let mut tau_mark = bitvec![u64, Lsb0; 0; tau_loop_free_lts.num_of_states()];

        let incoming = IncomingTransitions::new(&tau_loop_free_lts);

        let progress = TimeProgress::new(
            |num_of_blocks: usize| {
                info!("Found {} blocks...", num_of_blocks);
            },
            1,
        );

        loop {
            let mut stable = true;
            for block_index in (0usize..blocks.num_of_blocks()).map(BlockIndex::new) {
                progress.print(blocks.num_of_blocks());
                if *blocks.block(block_index).annotation() {
                    continue;
                }

                trace!("Stabilising block {:?}", block_index);
                stable = false;
                blocks.mark_block_stable(block_index);

                // tau is the first label.
                for label in tau_loop_free_lts
                    .labels()
                    .iter()
                    .enumerate()
                    .map(|(i, _)| LabelIndex::new(i))
                {
                    compute_weak_act(
                        &mut act_mark,
                        &mut tau_mark,
                        &tau_loop_free_lts,
                        &blocks,
                        &incoming,
                        block_index,
                        label,
                    );

                    // Note that we cannot use the block references here, and instead uses indices, because stabilise
                    // also modifies the blocks structure.
                    for block_prime in (0usize..blocks.num_of_blocks()).map(BlockIndex::new) {
                        stabilise(block_prime, &mut act_mark, &mut blocks);
                    }
                }
            }

            if stable {
                // Quit the outer loop.
                trace!("Partition is stable!");
                break;
            }
        }

        (tau_loop_free_lts, blocks)
    })
}

fn weak_bisimulation_parallel_impl<L: LTS>(
    lts: L,
    timing: &Timing,
) -> (LabelledTransitionSystem<L::Label>, MarkedBlockPartition) {
    let tau_loop_free_lts = timing.measure("preprocess", || tau_loop_elimination_and_reorder(lts));

    let progress = TimeProgress::new(
        |num_of_blocks: usize| {
            info!("Found {} blocks...", num_of_blocks);
        },
        1,
    );

    timing.measure("reduction", || {
        let mut blocks = MarkedBlockPartition::new(tau_loop_free_lts.num_of_states());

        let mut act_mark = bitvec![u64, Lsb0; 0; tau_loop_free_lts.num_of_states()];
        let mut tau_mark = bitvec![u64, Lsb0; 0; tau_loop_free_lts.num_of_states()];

        // Represents the s.marked[a] from the pseudocode.
        let mut marked = Vec::from_iter(iter::repeat_n(bitvec![u64, Lsb0; 0; tau_loop_free_lts.labels().len()], tau_loop_free_lts.num_of_states()));

        let incoming = IncomingTransitions::new(&tau_loop_free_lts);

        loop {
            let mut stable = true;
            for block_index in (0usize..blocks.num_of_blocks()).map(BlockIndex::new) {
                progress.print(blocks.num_of_blocks());
                if *blocks.block(block_index).annotation() {
                    continue;
                }

                trace!("Stabilising block {:?}", block_index);
                stable = false;
                blocks.mark_block_stable(block_index);

                compute_weak_act(
                    &mut act_mark,
                    &mut tau_mark,
                    &tau_loop_free_lts,
                    &blocks,
                    &incoming,
                    block_index,
                    LabelIndex::new(0),
                );

                compute_weak_acts(&mut marked, &tau_mark, &tau_loop_free_lts, &incoming, &blocks, block_index);

                while let Some(label) = find_act(&tau_loop_free_lts, &blocks, &mut marked) {
                    for block_index in (0usize..blocks.num_of_blocks()).map(BlockIndex::new) {
                        stabilise_act(block_index, label, &mut marked, &mut blocks);
                    }
                }
            }

            if stable {
                // Quit the outer loop.
                trace!("Partition is stable!");
                break;
            }
        }

        (tau_loop_free_lts, blocks)
    })
}

/// Sets s.act_mark to true iff exists t: S. s =\not{a}=> t
/// If a = tau, then also updates s.tau_mark
fn compute_weak_act<L: LTS>(
    act_mark: &mut BitArray,
    tau_mark: &mut BitArray,
    lts: &L,
    blocks: &MarkedBlockPartition,
    incoming: &IncomingTransitions,
    block: BlockIndex,
    label: LabelIndex,
) {
    for s in lts.iter_states() {
        // s.act_mark := true iff s in B && a == tau
        act_mark.set(
            *s,
            lts.is_hidden_label(label) && blocks.iter_block(block).any(|state| state == *s),
        );

        for transition in lts.outgoing_transitions(s) {
            if transition.label == label {
                // s.act_mark := true iff a != tau && tau_mark[t]
                if !lts.is_hidden_label(transition.label) && tau_mark[*transition.to] {
                    act_mark.set(*s, true);
                }
            }
        }
    }

    for t in lts.iter_states() {
        // t.tau_mark := t.act_mark if a == tau
        if lts.is_hidden_label(label) {
            tau_mark.set(*t, act_mark[*t]);
        }

        if act_mark[*t] {
            for transition in incoming.incoming_silent_transitions(t) {
                act_mark.set(*transition.to, true);
            }
        }
    }
}

/// Computing weak reachability for all actions at once. The `marked` array contains |Act| entries per state.
///
/// # Details
///
/// Requires s.tau_mark iff s ->> B.
/// For all a in A sets s.marked[a] iff s =[a]> B.
/// 
/// Note that `B` is only used for debugging checks, and is not used in the actual algorithm.
fn compute_weak_acts<L: LTS>(marked: &mut Vec<BitArray>, tau_mark: &BitArray, lts: &L, incoming: &IncomingTransitions<'_>, blocks: &MarkedBlockPartition, block: BlockIndex) {
    if cfg!(debug_assertions) {
        // Check that compute_weak_act results in the same markings as the optimised compute_weak_acts procedure.
        
        // Determine the act_mark for every label.
        let mut tau_mark_copy = tau_mark.clone();

        // Skip the tau action (index 0)
        let act_mark = (1..lts.labels().len()).map(|label| {
            let mut act_mark = bitvec![u64, Lsb0; 0; lts.num_of_states()];

            for s in lts.iter_states() {
                act_mark.set(*s, marked[*s][label]);
            }

            compute_weak_act(&mut act_mark, &mut tau_mark_copy, lts, &blocks, incoming, block, LabelIndex::new(label));
            debug_assert_eq!(tau_mark_copy, *tau_mark, "The tau mark should not be modified by compute_weak_act when a is not tau");
            act_mark
        }).collect::<Vec<_>>();

        // Compute the markings using the optimised procedure.
        compute_weak_acts_inner(marked, tau_mark, lts, incoming);
        
        // Check for correctness.
        for label in 1..lts.labels().len() {
            debug_assert!(act_mark[label].iter().zip(marked.iter()).all(|(a, m)| a == m[label]), "The act mark should be the same as the corresponding column in marked");
        }
    } else {
        // No checking for correctness.
        compute_weak_acts_inner(marked, tau_mark, lts, incoming);
    }
}

/// The inner implementation of [compute_weak_acts].
fn compute_weak_acts_inner<L: LTS>(marked: &mut Vec<BitArray>, tau_mark: &BitArray, lts: &L, incoming: &IncomingTransitions<'_>) {
    
    // For each s in state do s.marked := 0
    for entry in marked.iter_mut() {
        entry.fill(false);
    }

    for t in lts.iter_states() {
        // For each t -[a]-> u do
        for transition in lts.outgoing_transitions(t) {
            if !lts.is_hidden_label(transition.label) {
                // If u.tau_mark then t.marked[a] := true
                if tau_mark[*transition.to] {
                    marked[*t].set(*transition.label, true);
                }
            }
        }

        // For each s -[tau]-> t do
        for transition in incoming.incoming_silent_transitions(t) {
            marked[transition.to] = marked[transition.to].clone() | marked[*t].clone();
        }
    }
}

/// Finding an action that can be used to perform a refinement step.
fn find_act<L: LTS>(lts: &L, blocks: &MarkedBlockPartition, marked: &mut Vec<BitArray>) -> Option<LabelIndex> {
    for block in (0..blocks.num_of_blocks()).map(BlockIndex::new) {
        // Pick a representative state s from the block
        let s = blocks.iter_block(block).next().expect("Block is non-empty");
        for t in blocks.iter_block(block) {
            if marked[s] != marked[t] {
                // Find an action a such that s.marked[a] != t.marked[a]
                for label in 0..lts.labels().len() {
                    if marked[s][label] != marked[t][label] {
                        return Some(LabelIndex::new(label));
                    }
                }
            }
        }
    }

    None
}

/// Splits the given block according to the given marking.
fn stabilise(block: BlockIndex, act_mark: &mut BitArray, blocks: &mut MarkedBlockPartition) {
    blocks.split_block(block, |state| act_mark[*state]);
}

/// Splits the given block according to the given marking.
fn stabilise_act(block: BlockIndex, act: LabelIndex, marked: &mut Vec<BitArray>, blocks: &mut MarkedBlockPartition) {
    blocks.split_block(block, |state| marked[*state][*act]);
}


#[cfg(test)]
mod tests {
    use merc_io::DumpFiles;
    use merc_lts::random_lts;
    use merc_lts::write_aut;
    use merc_lts::LTS;
    use merc_utilities::random_test;
    use merc_utilities::Timing;

    use crate::compare_lts;
    use crate::reduce_lts;
    use crate::Equivalence;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_weak_bisimulation() {
        random_test(100, |rng| {
            let mut files = DumpFiles::new("test_weak_bisimulation");

            let lts = random_lts(rng, 100, 10, 3);
            let mut timing = Timing::new();
            files.dump("input.aut", |f| write_aut(f, &lts)).unwrap();

            let result = reduce_lts(lts.clone(), Equivalence::WeakBisim, false, &mut timing);
            let expected = reduce_lts(lts, Equivalence::WeakBisimSigref, false, &mut timing);

            assert_eq!(result.num_of_states(), expected.num_of_states());
            assert_eq!(result.num_of_transitions(), expected.num_of_transitions());

            files.dump("result.aut", |f| write_aut(f, &result)).unwrap();
            files.dump("expected.aut", |f| write_aut(f, &expected)).unwrap();

            assert!(compare_lts(
                Equivalence::StrongBisim,
                result,
                expected,
                false,
                &mut timing
            ));
        })
    }

    
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_weak_bisimulation_parallel() {
        random_test(100, |rng| {
            let mut files = DumpFiles::new("test_weak_bisimulation_parallel");

            let lts = random_lts(rng, 100, 10, 3);
            let mut timing = Timing::new();
            files.dump("input.aut", |f| write_aut(f, &lts)).unwrap();

            let result = reduce_lts(lts.clone(), Equivalence::WeakBisim, false, &mut timing);
            let expected = reduce_lts(lts, Equivalence::WeakBisimParallel, false, &mut timing);

            assert_eq!(result.num_of_states(), expected.num_of_states());
            assert_eq!(result.num_of_transitions(), expected.num_of_transitions());

            files.dump("result.aut", |f| write_aut(f, &result)).unwrap();
            files.dump("expected.aut", |f| write_aut(f, &expected)).unwrap();

            assert!(compare_lts(
                Equivalence::StrongBisim,
                result,
                expected,
                false,
                &mut timing
            ));
        })
    }
}
