use itertools::Itertools;
use log::trace;
use log::warn;
use merc_lts::LTS;
use merc_lts::StateIndex;
use merc_reduction::Equivalence;
use merc_reduction::Partition;
use merc_reduction::quotient_lts_block;
use merc_reduction::reduce_lts;
use merc_reduction::strong_bisim_sigref;
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
    let reduction = match refinement {
        RefinementType::Trace => Equivalence::StrongBisim,
        RefinementType::Weaktrace => Equivalence::BranchingBisim,
        // TODO: Should be divergence preserving branching bisimulation, but this is not implemented yet.
        RefinementType::StableFailures | RefinementType::ImpossibleFutures => Equivalence::BranchingBisim,
    };

    // For the preprocessing/quotienting step it makes sense to merge both LTSs
    // together in case that some states are equivalent. So we do this in all branches.
    let (merged_lts, initial_spec) = if preprocess {
        if counter_example {
            // If a counter example is to be generated, we only reduce the
            // specification LTS such that the resulting counter example remains valid.
            let reduced_spec = reduce_lts(spec_lts, reduction, true, timing);
            impl_lts.merge_disjoint(&reduced_spec)
        } else {
            let (merged_lts, initial_spec) = impl_lts.merge_disjoint(&spec_lts);

            // Reduce all states in the merged LTS.
            match reduction {
                Equivalence::StrongBisim => {
                    let (preprocess_lts, partition) = strong_bisim_sigref(merged_lts, timing);

                    let initial_spec = partition.block_number(initial_spec);
                    let reduced_lts = quotient_lts_block::<_, false>(&preprocess_lts, &partition);

                    // After partitioning the block becomes the state in the reduced_lts.
                    (reduced_lts, StateIndex::new(*initial_spec))
                }
                _ => {
                    warn!("Preprocessing for {reduction:?} is not implemented yet, skipping preprocessing.");
                    (merged_lts, initial_spec)
                }
            }
        }
    } else {
        impl_lts.merge_disjoint(&spec_lts)
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
    use merc_io::DumpFiles;
    use merc_lts::LTS;
    use merc_lts::mutate_lts;
    use merc_lts::random_lts_monolithic;
    use merc_lts::write_aut;
    use merc_utilities::Timing;
    use merc_utilities::random_test;
    use merc_vpg::solve_zielonka;
    use merc_vpg::translate;
    use rand::rngs::StdRng;

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
