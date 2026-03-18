use itertools::Itertools;
use log::debug;
use log::trace;
use log::warn;
use merc_lts::LTS;
use merc_lts::StateIndex;
use merc_reduction::Equivalence;
use merc_reduction::Partition;
use merc_reduction::branching_bisim_sigref;
use merc_reduction::quotient_lts_block;
use merc_reduction::quotient_lts_naive;
use merc_reduction::strong_bisim_sigref;
use merc_reduction::tau_scc_decomposition;
use merc_utilities::Timing;

use crate::CounterExample;
use crate::CounterExampleConstructor;
use crate::is_failures_refinement;
use crate::is_impossible_futures_refinement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum RefinementType {
    /// Checks for (strong) trace inclusion, i.e., whether all traces of the implementation are also traces of the specification.
    Trace,
    /// Checks for weak trace inclusion, i.e., whether all weak traces of the implementation are also weak traces of the specification.
    Weaktrace,
    /// Checks for stable failures inclusion, i.e., whether all stable failures of the implementation are also stable failures of the specification.
    StableFailures,
    /// Checks for impossible futures inclusion, i.e., whether all impossible futures of the implementation are also impossible futures of the specification.
    ImpossibleFutures,
}

/// Determines the exploration strategy for the failures refinement algorithm.
///
/// Typically `BFS` is more suited for counter examples, but `DFS` can be more efficient
/// in practice when a counter example is not required.
#[derive(Clone, Copy)]
pub enum ExplorationStrategy {
    BFS,
    DFS,
}

/// Checks whether `impl_lts` refines `spec_lts` according to the given
/// `refinement` relation.
///
/// # Details
///
/// The `refinement_type` determines (weak) trace inclusions, failures inclusion
/// and divergence failures inclusion etc.
///
/// The `strategy` parameter determines whether a breadth-first search
/// or depth-first search is used to explore the state space. Breadth-first search
/// is often better suited for finding short counter examples, while depth-first
/// search often uses less memory.
///
/// The `preprocess` flag indicates whether preprocessing should be applied to
/// the LTSs. The refinement checks often involve product constructions, and
/// reducing the state space beforehand can lead to significant performance
/// improvements. However, for quick failing checks the preprocessing could cause
/// unnecessary overhead.
pub fn refines<L: LTS>(
    impl_lts: L,
    spec_lts: L,
    refinement: RefinementType,
    strategy: ExplorationStrategy,
    preprocess: bool,
    counter_example: bool,
    timing: &mut Timing,
) -> (bool, Option<CounterExample<L::Label>>) {
    let (reduction, divergence_preserving) = match refinement {
        RefinementType::Trace => (Equivalence::StrongBisim, false),
        RefinementType::Weaktrace | RefinementType::ImpossibleFutures => (Equivalence::BranchingBisim, false),
        RefinementType::StableFailures => (Equivalence::BranchingBisim, true),
    };

    // For the preprocessing/quotienting step it makes sense to merge both LTSs
    // together in case that some states are equivalent. So we do this in all branches.
    let (merged_lts, initial_spec) = if preprocess {
        // Reduce all states in the merged LTS.
        match reduction {
            Equivalence::StrongBisim => {
                let (merged_lts, initial_spec) = impl_lts.merge_disjoint(&spec_lts);
                let (preprocess_lts, partition) = strong_bisim_sigref(merged_lts, timing);

                let impl_block = partition.block_number(preprocess_lts.initial_state_index());
                let spec_block = partition.block_number(initial_spec);

                if impl_block == spec_block {
                    // The initial states are already in the same block, so we can skip the refinement check.
                    debug!(
                        "Initial states are in the same block after strong bisimulation reduction, skipping refinement check."
                    );
                    return (true, None);
                }

                // After partitioning the block becomes the state in the reduced_lts.
                let reduced_lts = quotient_lts_block::<_, false>(&preprocess_lts, &partition);
                (reduced_lts, StateIndex::new(*spec_block))
            }
            Equivalence::BranchingBisim => {
                if divergence_preserving {
                    warn!(
                        "Preprocessing for divergence preserving branching bisimulation has not been implemented yet."
                    );
                    impl_lts.merge_disjoint(&spec_lts)
                } else {
                    let (merged_lts, initial_spec) = impl_lts.merge_disjoint(&spec_lts);
                    let (preprocess_lts, partition) = branching_bisim_sigref(merged_lts, timing);

                    let impl_block = partition.block_number(preprocess_lts.initial_state_index());
                    let spec_block = partition.block_number(initial_spec);

                    if impl_block == spec_block {
                        // The initial states are already in the same block, so we can skip the refinement check.
                        debug!(
                            "Initial states are in the same block after branching bisimulation reduction, skipping refinement check."
                        );
                        return (true, None);
                    }

                    let reduced_lts = quotient_lts_block::<_, true>(&preprocess_lts, &partition);
                    (reduced_lts, StateIndex::new(*spec_block))
                }
            }
            _ => unimplemented!("Preprocessing for refinement type {refinement:?} has not been implemented yet."),
        }
    } else {
        if refinement == RefinementType::ImpossibleFutures {
            // For impossible futures we need to remove tau loops from the implementation.
            let scc_partition = tau_scc_decomposition(&impl_lts);
            let tau_loop_free_lts = quotient_lts_naive(&impl_lts, &scc_partition, true);

            tau_loop_free_lts.merge_disjoint(&spec_lts)
        } else {
            impl_lts.merge_disjoint(&spec_lts)
        }
    };

    // Print the labels of the merged LTS for debugging purposes.
    trace!(
        "Merged LTS labels: {:?}",
        merged_lts.labels().iter().enumerate().format("\n")
    );

    timing.measure("refinement", || {
        if counter_example {
            // Construct a counter example tree, and return a trace.
            let mut ce_constructor = CounterExampleConstructor::new();
            let result = match refinement {
                RefinementType::Trace | RefinementType::Weaktrace | RefinementType::StableFailures => {
                    let (result, ce_state, ce_inner) =
                        is_failures_refinement(&merged_lts, initial_spec, refinement, strategy, &mut ce_constructor);

                    if let Some(state) = ce_state {
                        // Reconstruct a trace from the counter example tree, relabelling the indices to their actual labels.
                        let trace = ce_constructor
                            .reconstruct_trace(state)
                            .iter()
                            .map(|l| merged_lts.labels()[*l].clone())
                            .collect();
                        (
                            result,
                            Some(match refinement {
                                RefinementType::Trace => CounterExample::Trace(trace),
                                RefinementType::Weaktrace => CounterExample::WeakTrace(trace),
                                RefinementType::StableFailures => {
                                    if let Some(inner) = ce_inner {
                                        CounterExample::StableFailures(
                                            trace,
                                            inner.iter().map(|l| merged_lts.labels()[*l].clone()).collect(),
                                        )
                                    } else {
                                        // The stable failures failed because of a weak trace difference.
                                        CounterExample::WeakTrace(trace)
                                    }
                                }
                                _ => unreachable!("Refinement {refinement:?} is not valid in this path"),
                            }),
                        )
                    } else {
                        (result, None)
                    }
                }
                RefinementType::ImpossibleFutures => {
                    let (result, ce_state, ce_inner) =
                        is_impossible_futures_refinement(&merged_lts, initial_spec, strategy, &mut ce_constructor);

                    if let Some(state) = ce_state {
                        // Reconstruct a trace from the counter example tree, relabelling the indices to their actual labels.
                        let trace = ce_constructor
                            .reconstruct_trace(state)
                            .iter()
                            .map(|l| merged_lts.labels()[*l].clone())
                            .collect();
                        (
                            result,
                            Some(if let Some(inner) = ce_inner {
                                CounterExample::ImpossibleFutures(trace, inner)
                            } else {
                                // The impossible futures failed because of a weak trace.
                                CounterExample::WeakTrace(trace)
                            }),
                        )
                    } else {
                        (result, None)
                    }
                }
            };

            trace!("Counter example tree: {:?}", ce_constructor);
            result
        } else {
            // Run without constructing a counter example.
            match refinement {
                RefinementType::Trace | RefinementType::Weaktrace | RefinementType::StableFailures => {
                    let (result, _, _) =
                        is_failures_refinement(&merged_lts, initial_spec, refinement, strategy, &mut ());
                    (result, None)
                }
                RefinementType::ImpossibleFutures => {
                    let (result, _, _) = is_impossible_futures_refinement(&merged_lts, initial_spec, strategy, &mut ());
                    (result, None)
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use rand::rngs::StdRng;

    use merc_io::DumpFiles;
    use merc_lts::LTS;
    use merc_lts::mutate_lts;
    use merc_lts::random_lts_monolithic;
    use merc_lts::write_aut;
    use merc_utilities::Timing;
    use merc_utilities::random_test;
    use merc_vpg::solve_zielonka;
    use merc_vpg::translate;

    use crate::ExplorationStrategy;
    use crate::RefinementType;
    use crate::generate_formula;
    use crate::refines;

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_trace_refinement() {
        random_test(100, |rng| {
            is_refinement_test(
                "test_random_trace_refinement",
                rng,
                RefinementType::Trace,
                ExplorationStrategy::BFS,
                false,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_weak_trace_refinement() {
        random_test(100, |rng| {
            is_refinement_test(
                "test_random_weak_trace_refinement",
                rng,
                RefinementType::Weaktrace,
                ExplorationStrategy::BFS,
                false,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_stable_failures_refinement() {
        random_test(100, |rng| {
            is_refinement_test(
                "test_random_stable_failures_refinement",
                rng,
                RefinementType::StableFailures,
                ExplorationStrategy::BFS,
                false,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_impossible_futures_refinement() {
        random_test(100, |rng| {
            is_refinement_test(
                "test_random_impossible_futures_refinement",
                rng,
                RefinementType::ImpossibleFutures,
                ExplorationStrategy::BFS,
                false,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_trace_refinement_preprocessed() {
        random_test(100, |rng| {
            is_refinement_test(
                "test_random_trace_refinement",
                rng,
                RefinementType::Trace,
                ExplorationStrategy::BFS,
                true,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_weak_trace_refinement_preprocessed() {
        random_test(100, |rng| {
            is_refinement_test(
                "test_random_weak_trace_refinement",
                rng,
                RefinementType::Weaktrace,
                ExplorationStrategy::BFS,
                true,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_stable_failures_refinement_preprocessed() {
        random_test(100, |rng| {
            is_refinement_test(
                "test_random_stable_failures_refinement",
                rng,
                RefinementType::StableFailures,
                ExplorationStrategy::BFS,
                true,
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_random_impossible_futures_refinement_preprocessed() {
        random_test(100, |rng| {
            is_refinement_test(
                "test_random_impossible_futures_refinement",
                rng,
                RefinementType::ImpossibleFutures,
                ExplorationStrategy::BFS,
                true,
            );
        });
    }

    /// Helper function to define a refinement test that can be instantiated for
    /// the various types.
    ///
    /// # Details
    ///
    /// Internally requests a counter example to be generated, and checks that
    /// the counter example is indeed a valid witness for the failure of the
    /// refinement check.
    fn is_refinement_test(
        dump_name: &str,
        rng: &mut StdRng,
        refinement: RefinementType,
        strategy: ExplorationStrategy,
        preprocess: bool,
    ) {
        let mut files = DumpFiles::new(dump_name);

        let spec_lts = random_lts_monolithic(rng, 1000, 5, 3);
        let impl_lts = mutate_lts(&spec_lts, rng, 100).unwrap();

        files.dump("spec.aut", |w| write_aut(w, &spec_lts)).unwrap();
        files.dump("impl.aut", |w| write_aut(w, &impl_lts)).unwrap();

        let mut timing = Timing::default();
        let (result, counter_example) = refines(
            impl_lts.clone(),
            spec_lts.clone(),
            refinement,
            strategy,
            preprocess,
            true,
            &mut timing,
        );

        if !result {
            if let Some(ce) = counter_example {
                let formula = generate_formula(&ce);
                println!("Counter example formula: {}", formula);

                files
                    .dump("counter_example.mcf", |f| Ok(writeln!(f, "{}", formula)?))
                    .unwrap();

                let impl_pg = translate(&impl_lts, &formula).unwrap();
                let spec_pg = translate(&spec_lts, &formula).unwrap();

                let (impl_solution, _) = solve_zielonka(&impl_pg);
                let (spec_solution, _) = solve_zielonka(&spec_pg);

                assert!(
                    impl_solution[impl_lts.initial_state_index()] != spec_solution[spec_lts.initial_state_index()],
                    "Refinement returned false, but the counter example is not distinguishing."
                );
            } else {
                panic!("Expected a counter example.");
            }
        }
    }
}
