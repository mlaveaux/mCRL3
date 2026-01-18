use std::ops::Range;

use merc_io::TimeProgress;
use merc_utilities::MercError;
use oxidd::BooleanFunction;
use oxidd::BooleanFunctionQuant;
use oxidd::BooleanOperator;
use oxidd::LevelNo;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::VarNo;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::error::DuplicateVarName;
use oxidd::util::OutOfMemory;
use oxidd::util::SatCountCache;
use rustc_hash::FxBuildHasher;

use crate::SymbolicLtsBdd;
use crate::required_bits_64;

/// Computes the signature refinement of the given symbolic LTS using strong bisimulation.
///
/// # Details
///
/// The implementation is based on the following paper:
///  
/// > Tom van Dijk and Jaco van de Pol. Multi-core Symbolic Bisimulation Minimization.
pub fn sigref_symbolic(manager_ref: &BDDManagerRef, lts: &SymbolicLtsBdd) -> Result<(), MercError> {
    // There can only be one block per state, so we need as many bits as required to
    // represent all states.
    let number_of_states = lts
        .states()
        .sat_count::<u64, FxBuildHasher>(LevelNo::MAX, &mut SatCountCache::default());

    let block_vars = (0..required_bits_64(number_of_states))
        .map(|i| format!("b_{}", i))
        .collect::<Vec<String>>();

    // Create variables in the BDD manager
    let _variables = manager_ref
        .with_manager_exclusive(|manager| -> Result<Range<VarNo>, DuplicateVarName> {
            manager.add_named_vars(block_vars)
        })
        .map_err(|e| format!("Failed to create variables: {e}"))?;

    // Keep track of local information.
    let num_of_blocks = 0;
    let mut old_num_of_blocks = 0;
    let mut iteration = 0usize;

    let progress = TimeProgress::new(|(iterations, num_of_blocks): (usize, usize)| {
        println!(
            "  iteration {}: {} blocks",
            iterations, num_of_blocks
        );
    }, 1);

    // Stores the partition of the states as BDD.
    let partition = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));

    while num_of_blocks != old_num_of_blocks {
        // No fixed point reached yet, so keep refining.
        old_num_of_blocks = num_of_blocks;
        iteration += 1;

        // Compute the new signatures w.r.t. the previous partition.
        let mut signature = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));
        for group in lts.transition_groups() {
            let group_signature = signature_strong(&partition, group.relation(), lts.next_state_variables())?;
            signature = signature.or(&group_signature)?;
        }

        // Build the new partition based on the signatures.

        progress.print((iteration, num_of_blocks));
    }

    Ok(())
}

/// Computes the strong signature refinement of the given partition and relation.
///
/// # Details
///
///
fn signature_strong(
    partition: &BDDFunction,
    relation: &BDDFunction,
    next_state_vars: &BDDFunction,
) -> Result<BDDFunction, OutOfMemory> {
    partition.apply_exists(BooleanOperator::And, relation, next_state_vars)
}

#[cfg(test)]
mod tests {
    use merc_ldd::Storage;

    use crate::SymbolicLtsBdd;
    use crate::read_symbolic_lts;
    use crate::sigref_symbolic;

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_symbolic_lts_bdd() {
        let input = include_bytes!("../../../examples/lts/abp.sym");

        let mut storage = Storage::new();
        let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);
        let symbolic_lts = read_symbolic_lts(&mut storage, &input[..]).unwrap();

        let symbolic_lts = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &symbolic_lts).unwrap();

        let _reduced = sigref_symbolic(&manager_ref, &symbolic_lts).unwrap();
    }
}
