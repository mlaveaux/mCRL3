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

/// Performs reachability analysis using the given initial state and transitions.
pub fn reachability<L: SymbolicLPS>(
    storage: &LDDManagerRef,
    lts: &mut L,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
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

            let mut todo1 = LDDFunction::empty_set(storage)?;
            for (i, transition) in lts.transition_groups_mut().iter_mut().enumerate() {
                trace!("Learning successors for transition group {}:", i);
                timing.measure(&format!("learn_successors_{}", i), || {
                    transition.learn_successors(storage, &todo)
                })?;

                let result = todo.relational_product(transition.relation(), transition.meta())?;
                todo1 = todo1.union(&result)?;
            }

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
