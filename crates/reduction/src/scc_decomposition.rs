#![forbid(unsafe_code)]

use merc_collections::IndexedPartition;

use merc_collections::scc_decomposition;
use merc_lts::AsGraph;
use merc_lts::LTS;

use crate::sort_topological;

/// Computes the strongly connected tau component partitioning of the given LTS.
pub fn tau_scc_decomposition<L: LTS>(lts: &L) -> IndexedPartition {
    scc_decomposition(&AsGraph(lts), |_, label_index, _| lts.is_hidden_label(label_index))
}

/// Returns true iff the labelled transition system has tau-loops.
pub fn has_tau_loop<L: LTS>(lts: &L) -> bool {
    sort_topological(lts, |label_index, _| lts.is_hidden_label(label_index), false).is_err()
}

#[cfg(test)]
mod tests {
    use merc_io::DumpFiles;
    use merc_lts::LabelIndex;
    use merc_lts::LabelledTransitionSystem;
    use merc_lts::StateIndex;
    use merc_lts::TransitionLabel;
    use merc_lts::random_lts;
    use merc_lts::write_aut;
    use merc_utilities::random_test;
    use test_log::test;

    use crate::Partition;
    use crate::quotient_lts_naive;

    use super::LTS;
    use super::has_tau_loop;
    use super::tau_scc_decomposition;

    /// Returns the reachable states from the given state index.
    fn reachable_states<F, L>(lts: &L, state_index: StateIndex, filter: &F) -> Vec<usize>
    where
        F: Fn(StateIndex, LabelIndex, StateIndex) -> bool,
        L: LTS,
    {
        let mut stack = vec![state_index];
        let mut visited = vec![false; lts.num_of_states()];

        // Depth first search to find all reachable states.
        while let Some(inner_state_index) = stack.pop() {
            for transition in lts.outgoing_transitions(inner_state_index) {
                if filter(inner_state_index, LabelIndex::new(0), transition.to) && !visited[transition.to.value()] {
                    visited[transition.to.value()] = true;
                    stack.push(transition.to);
                }
            }
        }

        // All the states that were visited are reachable.
        visited
            .into_iter()
            .enumerate()
            .filter_map(|(index, visited)| if visited { Some(index) } else { None })
            .collect()
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_random_tau_scc_decomposition() {
        random_test(100, |rng| {
            let mut files = DumpFiles::new("test_random_tau_scc_decomposition");

            let lts = random_lts(rng, 10, 3, 3);
            files.dump("input.aut", |f| write_aut(f, &lts)).unwrap();

            let partitioning = tau_scc_decomposition(&lts);
            let reduction = quotient_lts_naive(&lts, &partitioning, true, false);
            assert!(!has_tau_loop(&reduction), "The SCC decomposition contains tau-loops");

            files
                .dump("tau_scc_decomposition.aut", |f| write_aut(f, &reduction))
                .unwrap();

            // Check that states in a strongly connected component are reachable from each other.
            for state_index in lts.iter_states() {
                let reachable = reachable_states(&lts, state_index, &|_, label, _| lts.is_hidden_label(label));

                // All other states in the same block should be reachable.
                let block = partitioning.block_number(state_index);

                for other_state_index in lts
                    .iter_states()
                    .filter(|index| state_index != *index && partitioning.block_number(*index) == block)
                {
                    assert!(
                        reachable.contains(&other_state_index),
                        "State {state_index} and {other_state_index} should be connected"
                    );
                }
            }

            assert!(
                reduction.num_of_states() == tau_scc_decomposition(&reduction).num_of_blocks(),
                "Applying SCC decomposition again should yield the same number of SCC after second application"
            );
        });
    }

    #[test]
    fn test_cycles() {
        let transitions = [(0, 0, 2), (0, 0, 4), (1, 0, 0), (2, 0, 1), (2, 0, 0)]
            .map(|(from, label, to)| (StateIndex::new(from), LabelIndex::new(label), StateIndex::new(to)));

        let lts = LabelledTransitionSystem::new(
            StateIndex::new(0),
            None,
            || transitions.iter().cloned(),
            vec![String::tau_label(), "a".to_string()],
        );

        let _ = tau_scc_decomposition(&lts);
    }
}
