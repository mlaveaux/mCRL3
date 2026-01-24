use log::info;
use oxidd::BooleanFunction;
use oxidd::BooleanFunctionQuant;
use oxidd::BooleanOperator;
use oxidd::FunctionSubst;
use oxidd::ManagerRef;
use oxidd::Subst;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::util::SatCountCache;

use merc_io::TimeProgress;
use merc_utilities::MercError;
use oxidd_dump::Visualizer;
use rustc_hash::FxBuildHasher;

use crate::SymbolicLtsBdd;
use crate::minus;

/// Performs reachability analysis on the given symbolic LTS represented using BDDs.
///
/// # Details
///
/// When `visualize` has been set to true, intermediate BDDs are visualized using `oxidd-vis`.d
pub fn reachability_bdd(
    manager_ref: &BDDManagerRef,
    lts: &SymbolicLtsBdd,
    visualize: bool,
) -> Result<usize, MercError> {
    let mut todo = lts.initial_state().clone();
    let mut states = lts.initial_state().clone(); // The state space.
    let mut iteration = 0;

    let progress = TimeProgress::new(
        |iteration: usize| {
            info!("Iteration {}", iteration);
        },
        1,
    );

    // Substitution to replace next state variables with current state variables.
    let next_state_substitution = Subst::new(lts.next_state_variables_bits(), lts.state_variables());

    while todo.satisfiable() {
        // Apply the transition relations to the todo set.
        let mut todo1 = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));
        for transition in lts.transition_groups() {
            // We explicitly do not quantify over state variables that are not
            // written by the transition group. Otherwise, these state variables
            // would become unconstrained and then after substituting next state
            // variables with current state variables, they would lead to
            // spurious states.
            todo1 = todo1.or(&todo.apply_exists(
                BooleanOperator::And,
                transition.relation(),
                &transition.write_variables_bdd().and(lts.action_variables_bdd())?,
            )?)?;
        }

        // Substitute next state variables with current state variables.
        todo1 = todo1.substitute(&next_state_substitution)?;

        if visualize {
            // Visualize the current partition.
            manager_ref.with_manager_shared(|manager| {
                Visualizer::new()
                    .add(&format!("todo1_{iteration}"), manager, [&todo1])
                    .serve()
            })?;
        }

        // Keep todo states that have not been discovered yet, and add them to the set of discovered states.
        todo = minus(&todo1, &states)?;
        states = states.or(&todo)?;
        progress.print(iteration);
        iteration += 1;
    }

    assert!(states == *lts.states(), "The computed state space does not match the LTS states.");

    Ok(
        states.sat_count::<u64, FxBuildHasher>(lts.state_variable_indices().len() as u32, &mut SatCountCache::default())
            as usize,
    )
}