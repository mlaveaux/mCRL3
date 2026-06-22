use log::debug;
use log::info;
use log::trace;
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

/// Performs reachability analysis using the given initial state and transitions.
///
/// Uses the default [ReachabilityOptions]; see [reachability_with_options] to select a strategy.
pub fn reachability<L: SymbolicLPS>(
    storage: &LDDManagerRef,
    lts: &mut L,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
    reachability_with_options(storage, lts, &ReachabilityOptions::default(), timing)
}

/// Performs reachability analysis using the given initial state, transitions and [ReachabilityOptions].
pub fn reachability_with_options<L: SymbolicLPS>(
    storage: &LDDManagerRef,
    lts: &mut L,
    options: &ReachabilityOptions,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
    if options.detect_deadlocks {
        // TODO: deadlock detection needs a relational-predecessor (relprev) operation to remove the
        // source states that have a successor; the oxidd LDD layer does not expose one yet.
        return Err(
            "deadlock detection is not yet supported: it requires a relational-predecessor (relprev) operation in the oxidd LDD layer".into(),
        );
    }

    let mut todo = lts.initial_state().clone();
    let mut states = lts.initial_state().clone();
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

            let todo1 = step(storage, lts, &todo, options.strategy, timing)?;

            trace!("todo1 = {}", LddDisplay::new(&todo1));

            todo = todo1.minus(&states)?;
            states = states.union(&todo)?;
            if progress.is_due() {
                progress.print((iteration, states.len()));
            }
            iteration += 1;
        }

        Ok(states)
    })
}

/// Performs a single exploration step from the frontier `todo`, returning the states reachable this step
/// (the caller subtracts the already visited states). The transition relations are learned on the fly.
fn step<L: SymbolicLPS>(
    storage: &LDDManagerRef,
    lts: &mut L,
    todo: &LDDFunction,
    strategy: ExplorationStrategy,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
    let chaining = matches!(
        strategy,
        ExplorationStrategy::Chaining | ExplorationStrategy::SaturationChaining
    );
    let saturation = matches!(
        strategy,
        ExplorationStrategy::Saturation | ExplorationStrategy::SaturationChaining
    );

    let groups = lts.transition_groups_mut();

    if !saturation {
        // Regular breadth-first, or chaining where successors found by earlier groups feed later groups.
        let mut todo1 = if chaining {
            todo.clone()
        } else {
            LDDFunction::empty_set(storage)?
        };

        for (i, transition) in groups.iter_mut().enumerate() {
            trace!("Learning successors for transition group {}:", i);
            let source = if chaining { todo1.clone() } else { todo.clone() };
            timing.measure(&format!("learn_successors_{}", i), || {
                transition.learn_successors(storage, &source)
            })?;

            let result = source.relational_product(transition.relation(), transition.meta())?;
            todo1 = todo1.union(&result)?;
        }

        Ok(todo1)
    } else {
        // Saturation: apply each group to a fixpoint before the next, optionally re-saturating earlier
        // groups (chaining) after every group.
        let mut todo1 = todo.clone();

        for i in 0..groups.len() {
            trace!("Learning successors for transition group {}:", i);
            timing.measure(&format!("learn_successors_{}", i), || {
                groups[i].learn_successors(storage, &todo1)
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

        Ok(todo1)
    }
}

#[cfg(test)]
mod test {
    use merc_utilities::Timing;

    use crate::ExplorationStrategy;
    use crate::ReachabilityOptions;
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

    #[test]
    fn test_reachability_detect_deadlocks_unsupported() {
        let ldd_manager = oxidd::ldd::new_manager(2048, 1024, 1);
        let bytes = include_bytes!("../../../../examples/ldd/anderson.4.ldd");
        let mut lts = read_sylvan(&ldd_manager, &mut &bytes[..]).expect("Loading should work correctly");

        let options = ReachabilityOptions {
            strategy: ExplorationStrategy::BreadthFirst,
            detect_deadlocks: true,
        };
        assert!(reachability_with_options(&ldd_manager, &mut lts, &options, &Timing::new()).is_err());
    }
}
