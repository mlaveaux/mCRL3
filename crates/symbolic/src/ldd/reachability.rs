use log::debug;
use log::info;
use log::trace;
use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;

use merc_data::DataExpression;
use merc_io::TimeProgress;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::LddDisplay;
use crate::SymbolicLPS;
use crate::TransitionGroup;

/// A symbolic LTS — extends [SymbolicLPS] with LTS-specific metadata.
pub trait SymbolicLTS: SymbolicLPS {
    /// Returns the LDD representing the set of states.
    fn states(&self) -> &LDDFunction;

    /// Returns the action labels for the LTS.
    fn action_labels(&self) -> &[String];

    /// Returns the possible values for each process parameter.
    fn parameter_values(&self) -> &[Vec<DataExpression>];
}

/// The order in which transition groups are applied during reachability.
///
/// Mirrors the `saturation` × `chaining` matrix of the mCRL2 `lpsreach` tool. Saturation and chaining
/// do not change the resulting reachable set, only how quickly the symbolic representation converges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ExplorationStrategy {
    /// Plain breadth-first: every group computes successors of the original frontier.
    #[default]
    BreadthFirst,
    /// Successors found by earlier groups feed into later groups within the same iteration.
    Chaining,
    /// Each group is applied to a fixpoint before moving on to the next one.
    Saturation,
    /// Like [Self::Saturation], but after each group all earlier groups are re-applied to a fixpoint.
    SaturationChaining,
}

/// Options controlling [reachability_with_options].
#[derive(Clone, Copy, Debug, Default)]
pub struct ReachabilityOptions {
    /// The strategy used to apply the transition groups.
    pub strategy: ExplorationStrategy,

    /// Whether to detect and report deadlock states (states without outgoing transitions).
    pub detect_deadlocks: bool,
}

/// The result of a reachability run.
pub struct ReachabilityResult {
    /// The set of reachable states.
    pub states: LDDFunction,

    /// The deadlock states (reachable states with no outgoing transition), or `None` when
    /// [ReachabilityOptions::detect_deadlocks] was not requested.
    pub deadlocks: Option<LDDFunction>,
}

/// Performs reachability analysis using the given initial state and transitions.
///
/// Uses the default [ReachabilityOptions]; see [reachability_with_options] for strategies and
/// deadlock detection. Returns only the reachable states.
pub fn reachability<L: SymbolicLPS>(
    storage: &LDDManagerRef,
    lts: &mut L,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
    Ok(reachability_with_options(storage, lts, &ReachabilityOptions::default(), timing)?.states)
}

/// Performs reachability analysis using the given initial state, transitions and [ReachabilityOptions].
pub fn reachability_with_options<L: SymbolicLPS>(
    storage: &LDDManagerRef,
    lts: &mut L,
    options: &ReachabilityOptions,
    timing: &Timing,
) -> Result<ReachabilityResult, MercError> {
    // Created once: the learning context (and any value interning it holds) must persist across
    // iterations so successors learned in later iterations stay consistent with earlier ones.
    let mut context = lts.create_context();
    let mut todo = lts.initial_state().clone();
    let mut states = lts.initial_state().clone();
    let mut deadlocks: Option<LDDFunction> = if options.detect_deadlocks {
        Some(storage.with_manager_shared(|m| LDDFunction::empty_set(m))?)
    } else {
        None
    };
    let mut iteration = 0;

    trace!("states = {}", LddDisplay::new(&states));
    let progress = TimeProgress::new(
        |(iteration, num_of_states)| {
            info!("explored {} state(s) after {} iteration(s)", num_of_states, iteration);
        },
        1,
    );

    timing.measure("reachability", || {
        while !todo.is_empty() {
            debug!("Iteration {}: todo size = {}", iteration, todo.len());

            let (todo1, step_deadlocks) = step(
                storage,
                lts,
                &mut context,
                &todo,
                options.strategy,
                options.detect_deadlocks,
                timing,
            )?;

            trace!("todo1 = {}", LddDisplay::new(&todo1));

            todo = todo1.minus(&states)?;
            states = states.union(&todo)?;
            if let Some(accumulated) = &mut deadlocks {
                *accumulated = accumulated.union(&step_deadlocks)?;
            }
            if progress.is_due() {
                progress.print((iteration, states.len()));
            }
            iteration += 1;
        }

        Ok(ReachabilityResult { states, deadlocks })
    })
}

/// Performs a single exploration step from the frontier `todo`.
///
/// Returns `(todo1, deadlocks)`: the states reachable this step (the caller subtracts the already
/// visited states) and, when `detect_deadlocks` is set, the subset of `todo` with no outgoing
/// transition in any group. The transition relations are learned on the fly.
fn step<L: SymbolicLPS>(
    storage: &LDDManagerRef,
    lts: &mut L,
    context: &mut <L::Group as TransitionGroup>::Context,
    todo: &LDDFunction,
    strategy: ExplorationStrategy,
    detect_deadlocks: bool,
    timing: &Timing,
) -> Result<(LDDFunction, LDDFunction), MercError> {
    let chaining = matches!(
        strategy,
        ExplorationStrategy::Chaining | ExplorationStrategy::SaturationChaining
    );
    let saturation = matches!(
        strategy,
        ExplorationStrategy::Saturation | ExplorationStrategy::SaturationChaining
    );

    let groups = lts.transition_groups_mut();

    // Potential deadlocks start as the whole frontier; a state is removed as soon as a group is found
    // that takes it to a successor. Only tracked when requested.
    let mut deadlocks = if detect_deadlocks {
        todo.clone()
    } else {
        storage.with_manager_shared(|m| LDDFunction::empty_set(m))?
    };

    if !saturation {
        // Regular breadth-first, or chaining where successors found by earlier groups feed later groups.
        let mut todo1 = if chaining {
            todo.clone()
        } else {
            storage.with_manager_shared(|m| LDDFunction::empty_set(m))?
        };

        for (i, transition) in groups.iter_mut().enumerate() {
            trace!("Learning successors for transition group {}:", i);
            let source = if chaining { todo1.clone() } else { todo.clone() };
            timing.measure(&format!("learn_successors_{}", i), || {
                transition.learn_successors(context, storage, &source)
            })?;

            let result = source.relational_product(transition.relation(), transition.meta())?;
            todo1 = todo1.union(&result)?;

            if detect_deadlocks {
                deadlocks = remove_states_with_successor(&todo1, transition, &deadlocks)?;
            }
        }

        Ok((todo1, deadlocks))
    } else {
        // Saturation: apply each group to a fixpoint before the next, optionally re-saturating earlier
        // groups (chaining) after every group.
        let mut todo1 = todo.clone();

        for i in 0..groups.len() {
            trace!("Learning successors for transition group {}:", i);
            timing.measure(&format!("learn_successors_{}", i), || {
                groups[i].learn_successors(context, storage, &todo1)
            })?;

            // Apply group i repeatedly until it no longer adds new states.
            loop {
                let old = todo1.clone();
                let result = todo1.relational_product(groups[i].relation(), groups[i].meta())?;
                todo1 = todo1.union(&result)?;
                if todo1 == old {
                    break;
                }
            }

            if detect_deadlocks {
                deadlocks = remove_states_with_successor(&todo1, &groups[i], &deadlocks)?;
            }

            // Apply all previously learned groups repeatedly until a fixpoint.
            if chaining {
                loop {
                    let old = todo1.clone();
                    for group in groups.iter().take(i + 1) {
                        let result = todo1.relational_product(group.relation(), group.meta())?;
                        todo1 = todo1.union(&result)?;
                    }
                    if todo1 == old {
                        break;
                    }
                }
            }
        }

        Ok((todo1, deadlocks))
    }
}

/// Removes from `deadlocks` every state that has a `group` transition into `todo1`, i.e. every state
/// that is not actually a deadlock with respect to `group`. Uses the relational predecessor (the
/// inverse of the relational product) restricted to the current deadlock candidates.
fn remove_states_with_successor(
    todo1: &LDDFunction,
    group: &impl TransitionGroup,
    deadlocks: &LDDFunction,
) -> Result<LDDFunction, MercError> {
    let with_successor = todo1.relational_predecessor(group.relation(), group.meta(), deadlocks)?;
    Ok(deadlocks.minus(&with_successor)?)
}

#[cfg(test)]
mod test {
    use merc_utilities::Timing;
    use oxidd::ManagerRef;
    use oxidd::ldd::LDDFunction;
    use oxidd::ldd::RelationProductMeta;
    use oxidd::ldd::Value;

    use crate::ExplorationStrategy;
    use crate::ReachabilityOptions;
    use crate::SylvanTransitionGroup;
    use crate::SymbolicLPS;
    use crate::from_iter;
    use crate::reachability_with_options;
    use crate::read_sylvan;

    /// Explores the `anderson.4` fixture with the given strategy and returns the reachable state count.
    fn explored_count(strategy: ExplorationStrategy) -> usize {
        let ldd_manager = oxidd::ldd::new_manager(2048, 1024, 1);
        let bytes = include_bytes!("../../../../examples/ldd/anderson.4.ldd");
        let mut lts = read_sylvan(&ldd_manager, &mut &bytes[..]).expect("Loading should work correctly");

        let options = ReachabilityOptions {
            strategy,
            detect_deadlocks: false,
        };
        reachability_with_options(&ldd_manager, &mut lts, &options, &Timing::new())
            .expect("Reachability should work correctly")
            .states
            .len()
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_reachability_strategies_agree() {
        // All strategies must compute the same reachable set, only the convergence speed differs.
        let expected = explored_count(ExplorationStrategy::BreadthFirst);
        assert_eq!(expected, explored_count(ExplorationStrategy::Chaining));
        assert_eq!(expected, explored_count(ExplorationStrategy::Saturation));
        assert_eq!(expected, explored_count(ExplorationStrategy::SaturationChaining));
    }

    /// Minimal hand-built LTS over a single parameter with transitions `0 -> 1 -> 2`, so the only
    /// reachable deadlock is state `2`.
    struct LineLts {
        initial: LDDFunction,
        groups: Vec<SylvanTransitionGroup>,
    }

    impl SymbolicLPS for LineLts {
        type Group = SylvanTransitionGroup;

        fn initial_state(&self) -> &LDDFunction {
            &self.initial
        }

        fn transition_groups(&self) -> &[Self::Group] {
            &self.groups
        }

        fn transition_groups_mut(&mut self) -> &mut [Self::Group] {
            &mut self.groups
        }

        fn create_context(&self) {}
    }

    fn line_lts(manager: &oxidd::ldd::LDDManagerRef) -> LineLts {
        // One read+write of parameter 0; relation short vectors place the read/write values at the
        // positions reported by `relation_product_meta`.
        let RelationProductMeta {
            meta,
            read_positions,
            write_positions,
        } = manager
            .with_manager_shared(|m| LDDFunction::relation_product_meta(m, &[0], &[0]))
            .expect("meta");
        let transition = |from: Value, to: Value| {
            let mut vector: Vec<Value> = vec![0; read_positions.len() + write_positions.len()];
            vector[read_positions[0]] = from;
            vector[write_positions[0]] = to;
            vector
        };

        let relation = from_iter(manager, [transition(0, 1), transition(1, 2)].iter());
        let group = SylvanTransitionGroup::new(relation, meta, vec![0], vec![0]);

        LineLts {
            initial: from_iter(manager, std::iter::once(&vec![0])),
            groups: vec![group],
        }
    }

    #[test]
    fn test_reachability_detect_deadlocks() {
        for strategy in [
            ExplorationStrategy::BreadthFirst,
            ExplorationStrategy::Chaining,
            ExplorationStrategy::Saturation,
            ExplorationStrategy::SaturationChaining,
        ] {
            let manager = oxidd::ldd::new_manager(2048, 1024, 1);
            let mut lts = line_lts(&manager);

            let options = ReachabilityOptions {
                strategy,
                detect_deadlocks: true,
            };
            let result = reachability_with_options(&manager, &mut lts, &options, &Timing::new())
                .expect("Reachability should work correctly");

            // States 0, 1, 2 are reachable and only state 2 has no outgoing transition.
            assert_eq!(result.states.len(), 3, "{strategy:?}");
            let deadlocks = result.deadlocks.expect("detect_deadlocks was requested");
            assert_eq!(deadlocks.len(), 1, "{strategy:?}");
        }
    }
}
