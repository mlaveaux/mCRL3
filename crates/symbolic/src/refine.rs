use merc_utilities::MercError;
use oxidd::BooleanFunction;
use oxidd::BooleanFunctionQuant;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::util::OutOfMemory;

use crate::CubeIterAll;
use crate::SummandGroupBdd;
use crate::SymbolicLtsBdd;
use crate::bdd_from_cube;

/// Strong bisimulation refinement algorithms for symbolic LTSs.
///
///
///
pub fn refine_bisimulation(manager_ref: &BDDManagerRef, lts: &SymbolicLtsBdd) -> Result<BDDFunction, MercError> {
    
    // Computes the BDD representing all (next) state variables.
    let state_vars = manager_ref.with_manager_shared(|manager| -> Result<_, OutOfMemory> {
        let mut bdd: BDDFunction = BDDFunction::f(manager);

        for var in lts.state_variables().iter().chain(lts.next_state_variables().iter()) {
            let var = BDDFunction::var(manager, *var)?;
            bdd = bdd.and(&var)?;
        }

        Ok(bdd)
    })?;

    // Computes the vector of action label BDDs.
    let action_vars = manager_ref.with_manager_shared(|manager| -> Result<_, OutOfMemory> {
        lts.action_variables().iter().map(|var| BDDFunction::var(manager, *var)).collect::<Result<Vec<_>, OutOfMemory>>()
    })?;

    // Split the transition group
    let mut split_groups = Vec::new();
    for group in lts.transition_groups() {
        let action_bdd = group.relation().exists(&state_vars)?;

        for cube in CubeIterAll::new(&action_bdd) {
            // Every cube is a single action.
            let cube = cube?;
            let label_bdd = bdd_from_cube(manager_ref, &action_vars, &cube)?;

            split_groups.push(SummandGroupBdd::new(group.relation().clone().and(&label_bdd)?, group.read_variables().clone(), group.write_variables().clone()));            
        }
    }

    // B_0(p, b) = 1 where |b| is 0.
    let mut blocks = lts.states().clone();    

    let mut iteration = 0;
    loop {
        // Check if B_i is stable w.r.t. all the transition relations.
        for group in &split_groups {
            
            // Forall b, p, p', q: B_i(p, b) and B_i(q,b) and Ta(p, p')

            // If the predicate is not satisfied, we split the blocks to obtain B_{i+1}(p, b b1 b2) = B_i(p, b1) and (b <=> exists p': Ra(p, p') and B_i(p', b2))
            let additional_b = manager_ref.with_manager_exclusive(|manager| {
                manager.add_named_vars((0..5).map(|index| format!("b_{iteration}_{index}")))
            }).map_err(|e| e.to_string())?;
        }

        iteration += 1;
    }

    Ok(blocks)
}
