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
/// When `visualize` has been set to true, intermediate BDDs are visualized using `oxidd-vis`.
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
    let next_state_substitution = Subst::new(lts.next_state_variables_indices(), lts.state_variables());

    while todo.satisfiable() {
        // Apply the transition relations to the todo set.
        let mut todo1 = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));
        for transition in lts.transition_groups() {
            // We explicitly do not quantify over state variables that are not
            // written by the transition group. Otherwise, these state variables
            // would become unconstrained and then after substituting next state
            // variables with current state variables, they would lead to
            // spurious states.
            //
            // This can easily be seen in the definition: `exists s, a. (todo(s) ∧ R(a, s'))`.
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

    Ok(
        states.sat_count::<u64, FxBuildHasher>(lts.state_variable_indices().len() as u32, &mut SatCountCache::default())
            as usize,
    )
}

#[cfg(test)]
mod tests {
    use merc_utilities::random_test;

    use crate::SymbolicLtsBdd;
    use crate::random_symbolic_lts;
    use crate::reachability;
    use crate::reachability_bdd;

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_reachability() {
        random_test(100, |rng| {
            let mut storage = merc_ldd::Storage::new();

            // We don't really check anything here, just ensure that reachability runs without errors.
            let lts = random_symbolic_lts(rng, &mut storage, 10, 5).unwrap();
            let num_reachable_states = reachability(&mut storage, &lts).unwrap();

            let manager_ref = oxidd::bdd::new_manager(2028, 2028, 1);
            let lts_bdd = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &lts).unwrap();

            let num_reachable_states_bdd = reachability_bdd(&manager_ref, &lts_bdd, false).unwrap();

            assert_eq!(
                num_reachable_states, num_reachable_states_bdd,
                "Number of reachable states does not match between BDD and LDD-based reachability."
            );
        });
    }
}
