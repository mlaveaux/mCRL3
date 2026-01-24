use std::ops::Range;

use log::debug;
use log::info;
use log::trace;
use merc_io::TimeProgress;
use merc_utilities::MercError;
use oxidd::BooleanFunction;
use oxidd::BooleanFunctionQuant;
use oxidd::BooleanOperator;
use oxidd::Function;
use oxidd::HasLevel;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::VarNo;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::error::DuplicateVarName;
use oxidd::util::OptBool;
use oxidd::util::OutOfMemory;
use oxidd::util::SatCountCache;
use oxidd_dump::Visualizer;
use rustc_hash::FxBuildHasher;
use rustc_hash::FxHashMap;

use crate::CubeIterAll;
use crate::SymbolicLtsBdd;
use crate::required_bits_64;

/// Computes the signature refinement of the given symbolic LTS using strong bisimulation.
///
/// # Details
///
/// The implementation is based on the following paper:
///  
/// > Tom van Dijk and Jaco van de Pol. Multi-core Symbolic Bisimulation Minimization.
pub fn sigref_symbolic(manager_ref: &BDDManagerRef, lts: &SymbolicLtsBdd, visualize: bool) -> Result<(), MercError> {
    // There can only be one block per state, so we need as many bits as required to
    // represent all states.
    let number_of_states = lts
        .states()
        .sat_count::<u64, FxBuildHasher>(lts.state_variable_indices().len() as u32, &mut SatCountCache::default());
    debug!("Number of states: {}", number_of_states);

    let num_of_block_bits = required_bits_64(number_of_states);
    debug!("Number of block bits: {}", num_of_block_bits);

    let block_variable_names = (0..num_of_block_bits)
        .map(|i| format!("b_{}", i))
        .collect::<Vec<String>>();

    // Create variables in the BDD manager
    let block_variables = manager_ref
        .with_manager_exclusive(|manager| -> Result<Range<VarNo>, DuplicateVarName> {
            manager.add_named_vars(block_variable_names)
        })
        .map_err(|e| format!("Failed to create variables: {e}"))?;

    // Create BDD functions for the block variables
    let block_variables_bdds = block_variables
        .map(|var_no| manager_ref.with_manager_shared(|manager| BDDFunction::var(manager, var_no)))
        .collect::<Result<Vec<BDDFunction>, OutOfMemory>>()?;

    // Keep track of local information.
    let mut num_of_blocks = 1;
    let mut old_num_of_blocks = 0;
    let mut iteration = 0usize;

    let progress = TimeProgress::new(
        |(iterations, num_of_blocks): (usize, usize)| {
            info!("iteration {}: {} blocks", iterations, num_of_blocks);
        },
        1,
    );

    let mut signature_to_block = FxHashMap::default();

    // Stores the partition of the states as BDD.
    let mut partition = lts
        .states()
        .and(&encode_block(manager_ref, &block_variables_bdds, 0)?)?;

    while num_of_blocks != old_num_of_blocks {
        // No fixed point reached yet, so keep refining.
        old_num_of_blocks = num_of_blocks;
        trace!("Iteration {} ({} blocks)", iteration, num_of_blocks);

        iteration += 1;

        if visualize {
            // Visualize the current partition.
            manager_ref.with_manager_shared(|manager| {
                Visualizer::new()
                    .add(&format!("partition_{iteration}"), manager, [&partition])
                    .serve()
            })?;
        }

        // Compute the new signatures w.r.t. the previous partition.
        let mut signature = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));
        for group in lts.transition_groups() {
            let group_signature = signature_strong(&partition, group.relation(), lts.next_state_variables())?;
            signature = signature.or(&group_signature)?;
        }

        if visualize {
            // Visualize the computed signature.
            manager_ref.with_manager_shared(|manager| {
                Visualizer::new()
                    .add(&format!("signature_{iteration}"), manager, [&signature])
                    .serve()
            })?;
        }

        // Build the new partition based on the signatures.
        partition = refine(
            manager_ref,
            &mut signature_to_block,
            &block_variables_bdds,
            &lts.state_variable_indices(),
            &signature,
            &partition,
        )?;

        num_of_blocks = signature_to_block.len();
        progress.print((iteration, num_of_blocks));

        // Clear the block assignment for the next iteration.
        signature_to_block.clear();
    }

    info!(
        "Signature refinement completed in {} iterations with {} blocks",
        iteration, num_of_blocks
    );

    print_partition(&partition, &block_variables_bdds, lts.state_variables(), lts.state_variable_bits(), num_of_block_bits)?;

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

/// Refines the partition w.r.t. the given signature by assigning block numbers
/// to signatures.
/// 
/// # Details
/// 
/// This function assumes that in the partition only a single block number is
/// assigned to each state. The same applies for the signature function.
fn refine(
    manager_ref: &BDDManagerRef,
    signature_to_block: &mut FxHashMap<BDDFunction, u64>,
    block_variables_bdds: &[BDDFunction],
    state_variables: &[VarNo],
    signature: &BDDFunction,
    partition: &BDDFunction,
) -> Result<BDDFunction, MercError> {
    // TODO: Caching
    // TODO: Very much not optimal with all the with_manager_shared calls.

    if !partition.satisfiable() || !signature.satisfiable() {
        // In this case the state is not part of the partition function, or (s,
        // a) not part of the actions. So return empty.
        return Ok(partition.clone());
    }
    
    // topVar
    let level = manager_ref.with_manager_shared(|manager| {
        let fnode = manager.get_node(partition.as_edge(manager)).unwrap_inner();
        let gnode = manager.get_node(signature.as_edge(manager)).unwrap_inner();
        let flevel = fnode.level();
        let glevel = gnode.level();
        flevel.min(glevel)
    });

    if state_variables.contains(&level) {
        // Match paths on the level s_i, for irrelevant variables we take both paths.
        let (s_high, s_low) = manager_ref.with_manager_shared(|manager| {
            let gnode = manager.get_node(signature.as_edge(manager)).unwrap_inner();
            if gnode.level() == level {
                signature.cofactors().unwrap()
            } else {
                (signature.clone(), signature.clone())
            }
        });
        let (p_high, p_low) = manager_ref.with_manager_shared(|manager| {
            let fnode = manager.get_node(partition.as_edge(manager)).unwrap_inner();
            if fnode.level() == level {
                partition.cofactors().unwrap()
            } else {
                (partition.clone(), partition.clone())
            }
        });

        let low = refine(
            manager_ref,
            signature_to_block,
            block_variables_bdds,
            state_variables,
            &s_low,
            &p_low,
        )?;
        let high = refine(
            manager_ref,
            signature_to_block,
            block_variables_bdds,
            state_variables,
            &s_high,
            &p_high,
        )?;

        // 7. result := BDDnode(topVar, high, low)
        Ok(manager_ref
            .with_manager_shared(|manager| BDDFunction::var(manager, level))?
            .ite(&high, &low)?)
    } else {
        // 9. else:
        // \sigma (the signature function) now encodes the state signature (a, B)
        // P (the partition function) encodes the current block assignment

        // 10. B := decode_block(partition)
        let block_index = decode_block(manager_ref, partition);
        if let Some(block) = signature_to_block.get(&signature) {
            // 11. If blocks[B].signature == \bottom then
            // 12.     blocks[B].signature := signature
            // 13. if blocks[B].signature == signature then
            // 14.     return P
            if *block == block_index {
                trace!("Found existing signature for {block_index}");
                Ok(partition.clone()) // The partition just encodes the current block.
            } else {
                // New partition needed
                trace!("Return existing block {block}");
                Ok(encode_block(manager_ref, &block_variables_bdds, *block)?)
            }
        } else {
            let new_block_index = signature_to_block.len() as u64;
            trace!("Creating new block {new_block_index}");
            signature_to_block.insert(signature.clone(), new_block_index);
            Ok(encode_block(manager_ref, &block_variables_bdds, new_block_index)?)
        }
    }
}

/// Encodes the given block number into a BDD using the given variables as bits.
/// 
/// # Details
/// 
/// Encodes the bits starting with the least significant bit, which is the
/// inverse of [ldd_to_bdd]. The intuition potentially is that the block numbers
/// are often small numbers, so the most significant bits are more likely to be
/// 0 and these will collapse to singular nodes.
fn encode_block(
    manager_ref: &BDDManagerRef,
    variables: &[BDDFunction],
    block_no: u64,
) -> Result<BDDFunction, MercError> {
    let mut result = manager_ref.with_manager_shared(|manager| BDDFunction::t(manager));
    for (i, var) in variables.iter().enumerate() {
        if block_no & (1 << i) != 0 {
            // bit is 1
            result = var.ite(
                &result,
                &manager_ref.with_manager_shared(|manager| BDDFunction::f(manager)),
            )?;
        } else {
            // bit is 0
            result = var.ite(
                &manager_ref.with_manager_shared(|manager| BDDFunction::f(manager)),
                &result,
            )?;
        }
    }

    Ok(result)
}

/// Decodes the given block number from a BDD using the given variables as bits.
fn decode_block(_manager_ref: &BDDManagerRef, partition: &BDDFunction) -> u64 {
    let mut result = 0u64;
    let mut mask = 1u64;
    let mut block = partition.clone();

    while block.satisfiable() {
        if let Some((b_high, b_low)) = block.cofactors() {
            // For a cube: low satisfiable => bit 0, else => bit 1
            if b_low.satisfiable() {
                block = b_low;
            } else {
                result |= mask;
                block = b_high;
            }
            mask <<= 1;
        } else {
            break;
        }
    }

    result
}

/// Prints all vectors represented by the given partition BDD using the given block and state variables.
fn print_partition(partition: &BDDFunction,
    block_variables_bdds: &[BDDFunction],
    state_variables: &[BDDFunction],
    bits: &[u32],
    block_bits: u32,
) -> Result<(), MercError> {    

    // Combine state variables and block variables
    let variables = state_variables.iter().chain(block_variables_bdds.iter()).cloned().collect::<Vec<_>>();
    let mut total_bits = bits.iter().cloned().collect::<Vec<u32>>();
    total_bits.push(block_bits);

    for result in CubeIterAll::new(&variables, partition) {
        let (bits, _) = result?;
        println!("{:?}", to_vector(&bits, &total_bits));
    }

    Ok(())
}

/// Reconstruct all values represented by the given bits and number of bits per value.
fn to_vector(bits: &[OptBool], num_of_bits: &[u32]) -> Vec<u64>{
    let mut values = Vec::new();

    // Every number of bits represent a single value encoded in the bits.
    let mut offset = 0;
    for num_of_bit in num_of_bits {
        values.push(to_value(&bits[offset..(offset + *num_of_bit as usize)]));
        offset += *num_of_bit as usize;
    }

    values
}

/// Reconstruct the value represented by the first num_of_bit bits.
fn to_value(bits: &[OptBool]) -> u64 {
    let mut value = 0u64;
    for i in 0..bits.len() {
        if bits[i as usize] == OptBool::True {
            value |= 1 << i;
        }
    }

    value
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use merc_ldd::Storage;
    use merc_utilities::random_test;
    use oxidd::BooleanFunction;
    use oxidd::Manager;
    use oxidd::ManagerRef;
    use oxidd::VarNo;
    use oxidd::bdd::BDDFunction;
    use oxidd::error::DuplicateVarName;
    use rand::Rng;

    use crate::SymbolicLtsBdd;
    use crate::read_symbolic_lts;
    use crate::required_bits_64;
    use crate::sigref::decode_block;
    use crate::sigref::encode_block;
    use crate::sigref_symbolic;

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_symbolic_lts_bdd() {
        let input = include_bytes!("../../../examples/lts/abp.sym");

        let mut storage = Storage::new();
        let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);
        let symbolic_lts = read_symbolic_lts(&mut storage, &input[..]).unwrap();

        let symbolic_lts = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &symbolic_lts).unwrap();

        let _reduced = sigref_symbolic(&manager_ref, &symbolic_lts, false).unwrap();
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_encode_blocks() {
        random_test(100, |rng| {
            let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);

            let block_number: u64 = rng.random();

            let num_of_bits = required_bits_64(block_number);
            let block_variable_names = (0..num_of_bits).map(|i| format!("b_{}", i)).collect::<Vec<String>>();

            // Create variables in the BDD manager
            let block_variables = manager_ref
                .with_manager_exclusive(|manager| -> Result<Range<VarNo>, DuplicateVarName> {
                    manager.add_named_vars(block_variable_names)
                })
                .unwrap();

            let block_variables_bdds = block_variables
                .map(|var_no| manager_ref.with_manager_shared(|manager| BDDFunction::var(manager, var_no)))
                .collect::<Result<Vec<BDDFunction>, oxidd::util::OutOfMemory>>()
                .unwrap();

            let encoded = encode_block(&manager_ref, &block_variables_bdds, block_number).unwrap();
            let decoded = decode_block(&manager_ref, &encoded);

            assert_eq!(block_number, decoded, "Decoding the block number did not yield the original");
        })
    }
}
