use log::info;
use merc_io::TimeProgress;
use merc_utilities::MercError;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::util::OutOfMemory;
use oxidd::BooleanFunction;
use oxidd::BooleanFunctionQuant;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::VarNo;

use crate::bdd_from_cube;
use crate::compute_vars_bdd;
use crate::variable_rename;
use crate::CubeIterAll;
use crate::SummandGroupBdd;
use crate::SymbolicLtsBdd;

/// Strong bisimulation refinement algorithms for symbolic LTSs.
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
        lts.action_variables()
            .iter()
            .map(|var| BDDFunction::var(manager, *var))
            .collect::<Result<Vec<_>, OutOfMemory>>()
    })?;

    // Split the transition group to only have a single action label per group.
    let mut split_groups = Vec::new();
    for group in lts.transition_groups() {
        let action_bdd = group.relation().exists(&state_vars)?;

        for cube in CubeIterAll::new(&action_bdd) {
            // Every cube is a single action.
            let cube = cube?;
            let label_bdd = bdd_from_cube(manager_ref, &action_vars, &cube)?;

            split_groups.push(SummandGroupBdd::new(
                group.relation().clone().and(&label_bdd)?,
                group.read_variables().to_vec(),
                group.write_variables().to_vec(),
            ));
        }
    }

    // Introduce variables for q, q' after the state and next state variables.
    let q_variables = manager_ref
        .with_manager_exclusive(|manager| {
            manager.add_named_vars((0..lts.state_variables().len()).map(|index| format!("q_{index}")))
        })
        .map_err(|e| e.to_string())?
        .collect::<Vec<_>>();

    let q_prime_variables = manager_ref
        .with_manager_exclusive(|manager| {
            manager.add_named_vars((0..lts.state_variables().len()).map(|index| format!("q_prime_{index}")))
        })
        .map_err(|e| e.to_string())?
        .collect::<Vec<_>>();

    // We interleave the p, q, p', and q' variables that all represent states (and next states).
    manager_ref.with_manager_exclusive(|manager| {
        let order: Vec<_> = lts
            .state_variables()
            .iter()
            .zip(q_variables.iter())
            .zip(lts.next_state_variables().iter().zip(q_prime_variables.iter()))
            .flat_map(|((s, q), (s_prime, q_prime))| [*s, *q, *s_prime, *q_prime])
            .collect();

        oxidd_reorder::set_var_order(manager, &order)
    });

    let p_bdd = compute_vars_bdd(manager_ref, lts.state_variables())?.1;
    let p_prime_bdd = compute_vars_bdd(manager_ref, lts.next_state_variables())?.1;
    let q_bdd = compute_vars_bdd(manager_ref, &q_variables)?.1;
    let q_prime_bdd = compute_vars_bdd(manager_ref, &q_prime_variables)?.1;

    // Create renamings from (p -> q), (q -> p'), (p' -> q').
    let p_to_q: Vec<(VarNo, VarNo)> = lts
        .state_variables()
        .iter()
        .cloned()
        .zip(q_variables.iter().cloned())
        .collect();
    let q_to_p_prime: Vec<(VarNo, VarNo)> = q_variables
        .iter()
        .cloned()
        .zip(lts.next_state_variables().iter().cloned())
        .collect();
    let p_prime_to_q_prime: Vec<(VarNo, VarNo)> = lts
        .next_state_variables()
        .iter()
        .cloned()
        .zip(q_prime_variables.iter().cloned())
        .collect();

    // Represents the b and b' variables (and a renaming from b to b') that must be updated in every iteration.
    let mut b_variables: Vec<VarNo> = Vec::new();
    let mut b_vars_bdd = manager_ref.with_manager_shared(|manager| BDDFunction::t(manager));

    let mut b_prime_variables: Vec<VarNo> = Vec::new();
    let mut b_prime_vars_bdd = manager_ref.with_manager_shared(|manager| BDDFunction::t(manager));

    let mut b_to_b_prime: Vec<(VarNo, VarNo)> = Vec::new();

    // B_0(p, b) = 1 where |b| is 0.
    let mut blocks = lts.states().clone();

    let progress = TimeProgress::new(
        |iteration: usize| {
            info!("iteration {}", iteration);
        },
        1,
    );

    let mut iteration = 0;
    loop {
        // Check if B_i is stable w.r.t. all the transition relations.
        let mut is_stable = true;
        for group in &split_groups {
            // Forall b, p, p', q: B_i(p, b) and B_i(q, b) and Ta(p, p') implies exists b', q': Ta(q, q') and B_i(p', b') and B_i(q', b')

            // Rename B_i(p, b') to B_i(q', b')
            let blocks_q = variable_rename(manager_ref, &blocks, &p_to_q)?;
            let blocks_p_prime = variable_rename(manager_ref, &blocks_q, &q_to_p_prime)?;
            let blocks_p_prime_b_prime = variable_rename(manager_ref, &blocks_p_prime, &b_to_b_prime)?;

            let condition = blocks.and(&blocks_q)?.and(group.relation())?;

            let blocks_q_prime = variable_rename(manager_ref, &blocks_p_prime, &p_prime_to_q_prime)?;
            let blocks_q_prime_b_prime = variable_rename(manager_ref, &blocks_q_prime, &b_to_b_prime)?;

            // Rename Ta(p, p') to Ta(q, q')
            let relation_q = variable_rename(manager_ref, &group.relation(), &p_to_q)?;
            let relation_q_q_prime = variable_rename(manager_ref, &relation_q, &p_prime_to_q_prime)?;

            // Computes exists b', q': Ta(q, q') and B_i(p', b') and B_i(q', b')
            let antecant = relation_q_q_prime
                .and(&blocks_p_prime_b_prime)?
                .and(&blocks_q_prime_b_prime)?
                .exists(&q_prime_bdd.and(&b_prime_vars_bdd)?)?;

            is_stable = is_stable
                && condition
                    .imp(&antecant)?
                    .forall(&b_vars_bdd.and(&p_bdd)?.and(&p_prime_bdd)?.and(&q_prime_bdd)?)?
                    .satisfiable();
            if !is_stable {
                // We are not yet stable, so need to perform additional splitting.
                break;
            }
        }

        if is_stable {
            return Ok(blocks);
        }

        // Introduce new b and b' variables.
        let mut b_vars = manager_ref
            .with_manager_exclusive(|manager| {
                manager.add_named_vars([format!("b_{iteration}"), format!("b_prime_{iteration}")])
            })
            .map_err(|e| e.to_string())?;

        let b_var = b_vars.next().expect("Two variables are added");
        let b_prime_var = b_vars.next().expect("Two variables are added");

        // Update various structs related to the b variables.
        b_variables.push(b_var);
        b_prime_variables.push(b_prime_var);

        b_vars_bdd = manager_ref.with_manager_shared(|manager| b_vars_bdd.and(&BDDFunction::var(manager, b_var)?))?;
        b_prime_vars_bdd =
            manager_ref.with_manager_shared(|manager| b_vars_bdd.and(&BDDFunction::var(manager, b_prime_var)?))?;

        b_to_b_prime.push((b_var, b_prime_var));

        // Add the variables for b2
        let b_next_variables = manager_ref.with_manager_exclusive(|manager| {
            let variables = manager.add_named_vars((0..block_variables.len()).map(|index| format!("b2_{index}")))?.collect();
            
            

            oxidd_reorder::set_var_order(manager, &order)
        })?;

        // B_i+1(p,b b1 b2) = B_i(p, b1) /\  (b <=> exists p' Ra(p, p') /\ B_i(p', b2)).
        blocks = manager_ref.with_manager_shared(|manager| -> Result<BDDFunction, OutOfMemory> {
            Ok(blocks.and(&BDDFunction::var(manager, *b_variables.last().expect("At least one b variable is added"))?)?)
        })?;

        iteration += 1;
        progress.print(iteration);
    }
}
