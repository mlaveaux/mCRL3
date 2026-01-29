use std::fmt;
use std::ops::Range;

use itertools::Itertools;
use log::debug;
use log::info;
use log::trace;
use merc_io::TimeProgress;
use merc_utilities::MercError;
use oxidd::BooleanFunction;
use oxidd::BooleanFunctionQuant;
use oxidd::BooleanOperator;
use oxidd::Function;
use oxidd::FunctionSubst;
use oxidd::HasLevel;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::Subst;
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
use crate::ValuesIter;
use crate::required_bits_64;
use crate::to_value;

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
    let block_variable_indices: Vec<VarNo> = manager_ref
        .with_manager_exclusive(|manager| -> Result<Range<VarNo>, DuplicateVarName> {
            manager.add_named_vars(block_variable_names)
        })
        .map_err(|e| format!("Failed to create variables: {e}"))?
        .collect();

    // Create BDD functions for the block variables
    let block_variables_bdds = block_variable_indices
        .iter()
        .map(|var_no| manager_ref.with_manager_shared(|manager| BDDFunction::var(manager, *var_no)))
        .collect::<Result<Vec<BDDFunction>, OutOfMemory>>()?;

    let mut signature_to_block = FxHashMap::default();

    // Substitution to replace next state variables with current state variables.
    let next_state_substitution = Subst::new(lts.state_variable_indices(), lts.next_state_variables());

    // Determine the variables in the support of a signature function.
    let signature_variables = lts
        .next_state_variable_indices()
        .iter()
        .chain(lts.action_variable_indices())
        .chain(block_variable_indices.iter())
        .cloned()
        .collect::<Vec<VarNo>>();

    // Determine the variables in the support of a partition function.
    let partition_variables = lts
        .next_state_variable_indices()
        .iter()
        .chain(block_variable_indices.iter())
        .cloned()
        .collect::<Vec<VarNo>>();

    // Stores the partition of the states as BDD.
    let mut partition = lts
        .states()
        .and(&manager_ref.with_manager_shared(|manager| encode_block(manager, &block_variables_bdds, 0))?)?;

    // In the sigref algorithm, the partition is defined over the next state. When we compute the signature
    // we then get (s, a, b), since in the signature we need to consider the block of the next state.
    partition = partition.substitute(&next_state_substitution)?;

    // Keep track of local information.
    let mut num_of_blocks = 0;
    let mut old_num_of_blocks = 1;
    let mut iteration = 0usize;

    let progress = TimeProgress::new(
        |(iterations, num_of_blocks): (usize, usize)| {
            info!("iteration {}: {} blocks", iterations, num_of_blocks);
        },
        1,
    );

    if visualize {
        // Visualize the initial partition.
        manager_ref.with_manager_shared(|manager| {
            Visualizer::new()
                .add("initial_partition", manager, [&partition])
                .serve()
        })?;
    }

    trace!(
        "Initial partition: {}",
        PartitionDisplay::new(
            &partition,
            &partition_variables,
            lts.state_variable_num_of_bits(),
            num_of_block_bits
        )
    );

    while num_of_blocks != old_num_of_blocks {
        // No fixed point reached yet, so keep refining.
        old_num_of_blocks = num_of_blocks;
        trace!("Iteration {} ({} blocks)", iteration, num_of_blocks);

        iteration += 1;

        // Compute the new signatures w.r.t. the previous partition.
        let mut signature = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));
        for group in lts.transition_groups() {
            let group_signature = signature_strong(&partition, group.relation(), group.write_variables_bdd())?;
            signature = signature.or(&group_signature)?;
        }

        // Substitute next state variables with current state variables to align
        // with the partition representation, required for `refine`.
        signature = signature.substitute(&next_state_substitution)?;

        trace!(
            "Signature at iteration {}: {}",
            iteration,
            SignatureDisplay::new(
                &signature,
                &signature_variables,
                lts.state_variable_num_of_bits(),
                lts.action_variable_indices().len() as u32,
                num_of_block_bits
            )
        );

        if visualize {
            // Visualize the computed signature.
            manager_ref.with_manager_shared(|manager| {
                Visualizer::new()
                    .add(&format!("signature_{iteration}"), manager, [&signature])
                    .serve()
            })?;
        }

        // Build the new partition based on the signatures.
        partition = manager_ref.with_manager_shared(|manager| {
            refine(
                manager,
                &mut signature_to_block,
                &block_variables_bdds,
                lts.next_state_variable_indices(),
                &signature,
                &partition,
            )
        })?;

        if visualize {
            // Visualize the current partition.
            manager_ref.with_manager_shared(|manager| {
                Visualizer::new()
                    .add(&format!("partition_{iteration}"), manager, [&partition])
                    .serve()
            })?;
        }

        trace!(
            "Partition at iteration {}: {}",
            iteration,
            PartitionDisplay::new(
                &partition,
                &partition_variables,
                lts.state_variable_num_of_bits(),
                num_of_block_bits
            )
        );

        num_of_blocks = signature_to_block.len();
        progress.print((iteration, num_of_blocks));

        // Clear the block assignment for the next iteration.
        signature_to_block.clear();
    }

    info!(
        "Signature refinement completed in {} iterations with {} blocks",
        iteration, num_of_blocks
    );

    Ok(())
}

/// Computes the strong signature refinement of the given partition and
/// relation.
///
/// # Details
///
/// For strong bisimulation the signature is defined as follows, where `P` is
/// the previous partition defined over the next state variables, and `relation` is defined
/// over the states, next states and the action label bits.
///
/// > ∃ s'. (relation(s, s', a) ∧ P(s', b))
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
/// assigned to each state, which should be by definition.
/// 
/// TODO: Definition
fn refine<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    signature_to_block: &mut FxHashMap<BDDFunction, u64>,
    block_variables_bdds: &[BDDFunction],
    state_variables: &[VarNo],
    signature: &BDDFunction,
    partition: &BDDFunction,
) -> Result<BDDFunction, MercError> {
    // TODO: Handle the `true` case.
    if !partition.satisfiable() || !signature.satisfiable() {
        // In this case the state is not part of the partition function, or (s,
        // a) not part of the actions. So return empty.
        return Ok(partition.clone());
    }

    // topVar
    let level = {
        let fnode = manager.get_node(partition.as_edge(manager)).unwrap_inner();
        let gnode = manager.get_node(signature.as_edge(manager)).unwrap_inner();
        let flevel = fnode.level();
        let glevel = gnode.level();
        flevel.min(glevel)
    };

    if state_variables.contains(&level) {
        // Match paths on the level s_i, for irrelevant variables we take both paths.
        let (s_high, s_low) = {
            let gnode = manager.get_node(signature.as_edge(manager)).unwrap_inner();
            if gnode.level() == level {
                signature.cofactors().expect("Not a terminal node")
            } else {
                (signature.clone(), signature.clone())
            }
        };
        let (p_high, p_low) = {
            let fnode = manager.get_node(partition.as_edge(manager)).unwrap_inner();
            if fnode.level() == level {
                partition.cofactors().expect("Not a terminal node")
            } else {
                (partition.clone(), partition.clone())
            }
        };

        let low = refine(
            manager,
            signature_to_block,
            block_variables_bdds,
            state_variables,
            &s_low,
            &p_low,
        )?;
        let high = refine(
            manager,
            signature_to_block,
            block_variables_bdds,
            state_variables,
            &s_high,
            &p_high,
        )?;

        // 7. result := BDDnode(topVar, high, low)
        Ok(BDDFunction::var(manager, level)?.ite(&high, &low)?)
    } else {
        // 9. else:
        // \sigma (the signature function) now encodes the state signature (a, B)
        // P (the partition function) encodes the current block assignment

        // 10. B := decode_block(partition)
        let block_index = decode_block(manager, partition);
        if let Some(block) = signature_to_block.get(signature) {
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
                Ok(encode_block(manager, block_variables_bdds, *block)?)
            }
        } else {
            let new_block_index = signature_to_block.len() as u64;
            trace!("Creating new block {new_block_index}");
            signature_to_block.insert(signature.clone(), new_block_index);
            Ok(encode_block(manager, block_variables_bdds, new_block_index)?)
        }
    }
}

/// Encodes the given block number into a BDD using the given variables as bits.
///
/// # Details
///
/// Encodes the bits starting with the least significant bit, which is the
/// inverse of the encoding used in [crate::ldd_to_bdd]. The intuition is
/// (potentially) that the block numbers are often small numbers, so the most
/// significant bits are more likely to be 0 and these will collapse to singular
/// nodes at the bottom layers.
fn encode_block<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    variables: &[BDDFunction],
    block_no: u64,
) -> Result<BDDFunction, MercError> {
    debug_assert!(
        variables.len() >= required_bits_64(block_no) as usize,
        "Not enough variables to encode block number {}",
        block_no
    );

    let mut result = BDDFunction::t(manager);
    for (i, var) in variables.iter().enumerate() {
        if block_no & (1 << i) != 0 {
            // bit is 1
            result = var.ite(&result, &BDDFunction::f(manager))?;
        } else {
            // bit is 0
            result = var.ite(&BDDFunction::f(manager), &result)?;
        }
    }

    Ok(result)
}

/// Decodes the given block number from a BDD using the given variables as bits.
///
/// # Details
///
/// Should be the inverse of [encode_block].
fn decode_block<'id>(_manager: &<BDDFunction as Function>::Manager<'id>, block: &BDDFunction) -> u64 {
    let mut result = 0u64;
    let mut mask = 1u64;
    let mut block = block.clone();

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

/// Display helper that prints all vectors represented by the given signature BDD as numbers, by decoding
/// the BDD layers as `bits`, see [crate::ldd_to_bdd].
pub struct SignatureDisplay<'a> {
    signature: &'a BDDFunction,

    /// The number of bits per state variable, the action bits and block bits.
    num_of_bits: &'a [u32],
    action_bits: u32,
    block_bits: u32,

    /// The variables that contribute to the signature.
    variables: &'a Vec<VarNo>,
}

impl<'a> SignatureDisplay<'a> {
    /// Creates a new partition display helper.
    fn new(
        signature: &'a BDDFunction,
        variables: &'a Vec<VarNo>,
        num_of_bits: &'a [u32],
        action_bits: u32,
        block_bits: u32,
    ) -> Self {
        Self {
            signature,
            num_of_bits,
            action_bits,
            block_bits,
            variables,
        }
    }
}

/// Display helper that prints all vectors represented by the given signature BDD as numbers, by decoding
/// the BDD layers as `bits`, see [crate::ldd_to_bdd].
pub struct PartitionDisplay<'a> {
    signature: &'a BDDFunction,

    /// The number of bits per state variable and block bits.
    num_of_bits: &'a [u32],
    block_bits: u32,

    /// The variables that contribute to the signature.
    variables: &'a Vec<VarNo>,
}

impl<'a> PartitionDisplay<'a> {
    /// Creates a new partition display helper.
    fn new(signature: &'a BDDFunction, variables: &'a Vec<VarNo>, num_of_bits: &'a [u32], block_bits: u32) -> Self {
        Self {
            signature,
            num_of_bits,
            block_bits,
            variables,
        }
    }
}

impl fmt::Display for PartitionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Total number of bits for the state variables.
        let total_num_of_bits: u32 = self.num_of_bits.iter().sum();

        // We ignore the output cube, so just pass no variables.
        let mut first = true;
        for cube in CubeIterAll::with_variables(self.signature, self.variables) {
            let cube = cube.map_err(|_| fmt::Error)?;

            debug_assert_eq!(
                cube.len(),
                (total_num_of_bits + self.block_bits) as usize,
                "Unexpected number of bits found"
            );

            if !first {
                writeln!(f)?;
            }
            first = false;

            let (state_bits, block_bits) = cube.split_at(total_num_of_bits as usize);
            debug_assert_eq!(
                block_bits.len(),
                self.block_bits as usize,
                "Unexpected number of block bits found"
            );

            write!(
                f,
                "[{}] -> {}",
                ValuesIter::new(state_bits, self.num_of_bits).format(", "),
                to_block_index(block_bits)
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for SignatureDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Total number of bits for the state variables.
        let total_num_of_bits: u32 = self.num_of_bits.iter().sum();

        // We ignore the output cube, so just pass no variables.
        let mut first = true;
        for cube in CubeIterAll::with_variables(self.signature, self.variables) {
            let cube = cube.map_err(|_| fmt::Error)?;

            debug_assert_eq!(
                cube.len(),
                (total_num_of_bits + self.action_bits + self.block_bits) as usize,
                "Unexpected number of bits found"
            );

            if !first {
                writeln!(f)?;
            }
            first = false;

            let (state_bits, rest) = cube.split_at(total_num_of_bits as usize);
            let (action_bits, block_bits) = rest.split_at(self.action_bits as usize);
            debug_assert_eq!(
                block_bits.len(),
                self.block_bits as usize,
                "Unexpected number of block bits found"
            );

            write!(
                f,
                "[{}] -> ({}, {})",
                ValuesIter::new(state_bits, self.num_of_bits).format(", "),
                to_value(action_bits),
                to_block_index(block_bits)
            )?;
        }

        Ok(())
    }
}

/// Reconstruct the block index represented by the bits, this uses the same encoding
/// as [encode_block]. This is least significant bit first.
fn to_block_index(bits: &[OptBool]) -> u64 {
    let mut value = 0u64;
    for (i, bit) in bits.iter().enumerate() {
        if *bit == OptBool::True {
            value |= 1 << i;
        }
    }

    value
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use oxidd::BooleanFunction;
    use oxidd::Manager;
    use oxidd::ManagerRef;
    use oxidd::VarNo;
    use oxidd::bdd::BDDFunction;
    use oxidd::error::DuplicateVarName;
    use rand::Rng;

    use merc_ldd::Storage;
    use merc_utilities::random_test;

    use crate::SymbolicLtsBdd;
    use crate::random_symbolic_lts;
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

            manager_ref.with_manager_shared(|manager| {
                let block_variables_bdds = block_variables
                    .map(|var_no| BDDFunction::var(manager, var_no))
                    .collect::<Result<Vec<BDDFunction>, oxidd::util::OutOfMemory>>()
                    .unwrap();

                let encoded = encode_block(manager, &block_variables_bdds, block_number).unwrap();
                let decoded = decode_block(manager, &encoded);

                assert_eq!(
                    block_number, decoded,
                    "Decoding the block number did not yield the original"
                );
            });
        })
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_symbolic_sigref() {
        random_test(100, |rng| {
            let mut storage = merc_ldd::Storage::new();

            // We don't really check anything here, just ensure that reachability runs without errors.
            let lts = random_symbolic_lts(rng, &mut storage, 10, 5).unwrap();
            let manager_ref = oxidd::bdd::new_manager(2028, 2028, 1);
            let lts_bdd = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &lts).unwrap();

            sigref_symbolic(&manager_ref, &lts_bdd, false).unwrap();
        });
    }
}
