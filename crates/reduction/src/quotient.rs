#![forbid(unsafe_code)]

use merc_collections::BlockIndex;
use merc_lts::LTS;
use merc_lts::LabelledTransitionSystem;
use merc_lts::LtsBuilderFast;
use merc_lts::StateIndex;
use merc_lts::Transition;

use crate::BlockPartition;
use crate::Partition;
use crate::diverges;

/// Returns a new LTS based on the given partition.
///
/// The naive version will add the transitions of all states in the block to the quotient LTS.
pub fn quotient_lts_naive<L: LTS, P: Partition>(
    lts: &L,
    partition: &P,
    eliminate_tau_loops: bool,
) -> LabelledTransitionSystem<L::Label> {
    // Introduce the transitions based on the block numbers, the number of blocks is a decent approximation for the number of transitions.
    let mut builder = LtsBuilderFast::with_capacity(
        lts.labels().into(),
        Vec::new(),
        partition.num_of_blocks(), // We expect one transition per state.
    );

    for state_index in lts.iter_states() {
        for transition in lts.outgoing_transitions(state_index) {
            let block = partition.block_number(state_index);
            let to_block = partition.block_number(transition.to);

            // If we eliminate tau loops then check if the 'to' and 'from' end up in the same block
            if !(eliminate_tau_loops && lts.is_hidden_label(transition.label) && block == to_block) {
                debug_assert!(
                    partition.block_number(state_index) < partition.num_of_blocks(),
                    "Quotienting assumes that the block numbers do not exceed the number of blocks"
                );

                builder.add_transition(
                    StateIndex::new(block.value()),
                    &lts.labels()[transition.label],
                    StateIndex::new(to_block.value()),
                );
            }
        }
    }

    builder.require_num_of_states(partition.num_of_blocks());
    builder.finish(
        StateIndex::new(partition.block_number(lts.initial_state_index()).value()),
        true,
    )
}

/// Returns a weak bisimulation quotient that additionally removes transitions
/// subsumed by a one-hidden-step alternative.
pub fn quotient_lts_weak<L: LTS, P: Partition>(lts: &L, partition: &P) -> LabelledTransitionSystem<L::Label> {
    let quotient = quotient_lts_naive(lts, partition, true);
    remove_redundant_transitions(&quotient)
}

/// Weak bisimulation quotient that removes redundant transitions.
fn remove_redundant_transitions<L: LTS>(lts: &L) -> LabelledTransitionSystem<L::Label> {
    let mut builder = LtsBuilderFast::with_capacity(lts.labels().into(), Vec::new(), lts.num_of_transitions());
    builder.require_num_of_states(lts.num_of_states());

    for from in lts.iter_states() {
        for transition in lts.outgoing_transitions(from) {
            if !is_redundant_transition(lts, from, &transition) {
                builder.add_transition(from, &lts.labels()[transition.label], transition.to);
            }
        }
    }

    builder.finish(lts.initial_state_index(), true)
}

/// Returns true when `transition` from `from` is redundant.
///
/// A transition `s -a-> t` is redundant when one of these alternatives exists:
/// - `s -tau-> m -a-> t` (for both hidden and visible `a`)
/// - `s -a-> m -tau-> t` (only for visible `a`)
///
/// This matches the mCRL2-style one-intermediate-state elimination rule.
fn is_redundant_transition<L: LTS>(lts: &L, from: StateIndex, transition: &Transition) -> bool {
    let label = transition.label;
    let target = transition.to;

    let redundant_via_hidden_then_label = lts
        .outgoing_transitions(from)
        .filter(|first| lts.is_hidden_label(first.label))
        .map(|first| first.to)
        .any(|middle| {
            lts.outgoing_transitions(middle).any(|second| {
                if lts.is_hidden_label(label) {
                    lts.is_hidden_label(second.label) && second.to == target
                } else {
                    second.label == label && second.to == target
                }
            })
        });

    if redundant_via_hidden_then_label {
        return true;
    }

    if lts.is_hidden_label(label) {
        return false;
    }

    lts.outgoing_transitions(from)
        .filter(|first| first.label == label)
        .map(|first| first.to)
        .any(|middle| {
            lts.outgoing_transitions(middle)
                .any(|second| lts.is_hidden_label(second.label) && second.to == target)
        })
}

/// Optimised implementation for block partitions.
///
/// Chooses a single state in the block as representative. If `BRANCHING` then the
/// chosen state is a bottom state. For `BRANCHING` it assumes that the input LTS
/// is non-divergent.
pub fn quotient_lts_block<L: LTS, const BRANCHING: bool>(
    lts: &L,
    partition: &BlockPartition,
) -> LabelledTransitionSystem<L::Label> {
    let mut builder = LtsBuilderFast::new(lts.labels().into(), Vec::new());

    for block in (0..partition.num_of_blocks()).map(BlockIndex::new) {
        // Pick any state in the block
        let mut candidate = if let Some(state) = partition.iter_block(block).next() {
            state
        } else {
            panic!("Blocks in the partition should not be empty {}", block);
        };

        if BRANCHING {
            let mut visited = vec![false; lts.num_of_states()];

            // traverse any outgoing transition to find a bottom state.
            'outer: loop {
                if visited[candidate] {
                    // No bottom state exists in this block. Stop early to avoid looping forever.
                    debug_assert!(
                        !diverges(lts, candidate),
                        "The states of the given LTS should be non-divergent."
                    );
                    break;
                }
                visited[candidate] = true;

                if let Some(trans) = lts.outgoing_transitions(candidate).find(|trans| {
                    lts.is_hidden_label(trans.label)
                        && candidate != trans.to // Ignore self loops
                        && partition.block_number(trans.to) == block
                }) {
                    candidate = trans.to;
                    continue 'outer;
                }

                // No outgoing tau transition to the same block, so we found a bottom state.
                break;
            }
        }

        // Add all transitions from the representative state (or the bottom state if BRANCHING) to the quotient LTS.
        for transition in lts.outgoing_transitions(candidate) {
            if BRANCHING {
                debug_assert!(
                    !(lts.is_hidden_label(transition.label)
                        && candidate != transition.to
                        && partition.block_number(transition.to) == block),
                    "The representative {} is not bottom state",
                    candidate
                );
            }

            builder.add_transition(
                StateIndex::new(*block),
                &lts.labels()[transition.label],
                StateIndex::new(*partition.block_number(transition.to)),
            );
        }
    }

    builder.require_num_of_states(partition.num_of_blocks());
    builder.finish(
        StateIndex::new(partition.block_number(lts.initial_state_index()).value()),
        true,
    )
}

#[cfg(test)]
mod tests {
    use merc_io::DumpFiles;
    use merc_lts::random_lts;
    use merc_lts::write_aut;
    use merc_utilities::Timing;
    use merc_utilities::random_test;
    use rand::rngs::StdRng;

    use crate::Equivalence;
    use crate::compare_lts;
    use crate::reduce_lts;

    /// Generates a random LTS, reduces it under `equivalence`, and asserts
    /// that the original and reduced LTS are equivalent.
    fn check_quotient_equivalence(rng: &mut StdRng, equivalence: Equivalence, test_name: &str) {
        let mut timing = Timing::new();
        let mut files = DumpFiles::new(test_name);

        let lts = random_lts(rng, 20, 10, 2);

        files.dump("input.aut", |w| write_aut(w, &lts)).unwrap();

        let reduced = reduce_lts(lts.clone(), equivalence, false, &mut timing);
        files.dump("quotient.aut", |w| write_aut(w, &reduced)).unwrap();

        assert!(
            compare_lts(equivalence, lts, reduced, false, &mut timing),
            "Quotient is not equivalent under {equivalence:?}",
        );
    }

    #[test]
    fn test_random_strong_bisim_quotient() {
        random_test(100, |rng| {
            check_quotient_equivalence(rng, Equivalence::StrongBisim, "test_random_strong_bisim_quotient");
        });
    }

    #[test]
    fn test_random_strong_bisim_naive_quotient() {
        random_test(100, |rng| {
            check_quotient_equivalence(
                rng,
                Equivalence::StrongBisimNaive,
                "test_random_strong_bisim_naive_quotient",
            );
        });
    }

    #[test]
    fn test_random_branching_bisim_quotient() {
        random_test(100, |rng| {
            check_quotient_equivalence(rng, Equivalence::BranchingBisim, "test_random_branching_bisim_quotient");
        });
    }

    #[test]
    fn test_random_branching_bisim_naive_quotient() {
        random_test(100, |rng| {
            check_quotient_equivalence(
                rng,
                Equivalence::BranchingBisimNaive,
                "test_random_branching_bisim_naive_quotient",
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Test is too slow under miri
    fn test_random_weak_bisim_quotient() {
        random_test(100, |rng| {
            check_quotient_equivalence(rng, Equivalence::WeakBisim, "test_random_weak_bisim_quotient");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Test is too slow under miri
    fn test_random_weak_bisim_parallel_quotient() {
        random_test(100, |rng| {
            check_quotient_equivalence(
                rng,
                Equivalence::WeakBisimParallel,
                "test_random_weak_bisim_parallel_quotient",
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Test is too slow under miri
    fn test_random_weak_bisim_sigref_quotient() {
        random_test(100, |rng| {
            check_quotient_equivalence(
                rng,
                Equivalence::WeakBisimSigref,
                "test_random_weak_bisim_sigref_quotient",
            );
        });
    }

    #[test]
    fn test_random_weak_bisim_sigref_naive_quotient() {
        random_test(100, |rng| {
            check_quotient_equivalence(
                rng,
                Equivalence::WeakBisimSigrefNaive,
                "test_random_weak_bisim_sigref_naive_quotient",
            );
        });
    }
}
