use std::fmt;
use std::ops::Range;

use itertools::Itertools;
use log::debug;
use log::info;
use log::trace;
use merc_io::TimeProgress;
use merc_utilities::MercError;
use merc_utilities::Timing;
use oxidd::BooleanFunction;
use oxidd::BooleanFunctionQuant;
use oxidd::BooleanOperator;
use oxidd::Edge;
use oxidd::Function;
use oxidd::HasLevel;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::Node;
use oxidd::VarNo;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::error::DuplicateVarName;
use oxidd::util::Borrowed;
use oxidd::util::OptBool;
use oxidd::util::OutOfMemory;
use oxidd::util::SatCountCache;
use oxidd_core::function::EdgeOfFunc;
use oxidd_core::util::EdgeDropGuard;
use oxidd_dump::Visualizer;
use oxidd_rules_bdd::simple::BDDTerminal;
use rustc_hash::FxBuildHasher;
use rustc_hash::FxHashMap;

use crate::CubeIterAll;
use crate::SymbolicLtsBdd;
use crate::ValuesIter;
use crate::collect_children;
use crate::compute_vars_bdd;
use crate::reduce;
use crate::required_bits_64;
use crate::to_value;
use crate::variable_rename;

/// Computes the reduction of the given symbolic LTS using symbolic signature
/// refinement.
///
/// # Details
///
/// The implementation is based on the following paper:
///  
/// > Tom van Dijk and Jaco van de Pol. Multi-core Symbolic Bisimulation
/// > Minimization.
pub fn sigref_symbolic(
    manager_ref: &BDDManagerRef,
    lts: &SymbolicLtsBdd,
    timing: &Timing,
    split_signature: bool,
    visualize: bool,
) -> Result<BDDFunction, MercError> {
    // There can only be one block per state, so we need as many bits as required to
    // represent all states.
    let number_of_states = lts
        .states()
        .sat_count::<u64, FxBuildHasher>(lts.state_variables().len() as u32, &mut SatCountCache::default());
    debug!("Number of states: {}", number_of_states);

    let split_partition_groups = if split_signature {
        combine_transition_groups(manager_ref, lts)?
    } else {
        // We do not use the grouping
        Vec::new()
    };

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
    let block_variables_bdds = compute_vars_bdd(manager_ref, &block_variable_indices)?.0;

    let mut signature_to_block = FxHashMap::default();

    // Substitution to replace next state variables with current state variables.
    let state_substitution: Vec<(VarNo, VarNo)> = lts
        .state_variables()
        .iter()
        .cloned()
        .zip(lts.next_state_variables().iter().cloned())
        .collect();

    // Determine the variables in the support of a signature function.
    let signature_variables = lts
        .next_state_variables()
        .iter()
        .chain(lts.action_variables())
        .chain(block_variable_indices.iter())
        .cloned()
        .collect::<Vec<VarNo>>();

    // Determine the variables in the support of a partition function.
    let partition_variables = lts
        .next_state_variables()
        .iter()
        .chain(block_variable_indices.iter())
        .cloned()
        .collect::<Vec<VarNo>>();

    // Stores the partition of the states as BDD.
    let mut partition = lts.states().and(&manager_ref.with_manager_shared(
        |manager| -> Result<BDDFunction, OutOfMemory> {
            Ok(BDDFunction::from_edge(
                manager,
                encode_block(manager, &block_variables_bdds, 0)?,
            ))
        },
    )?)?;

    // In the sigref algorithm, the partition is defined over the next state. When we compute the signature
    // we then get (s, a, b), since in the signature we need to consider the block of the next state.
    partition = variable_rename(manager_ref, &partition, &state_substitution)?;

    // Keep track of local information.
    let mut num_of_blocks = 0;
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

    // Determine the write variables BDDs for all transition groups.
    let write_variable_bdd = lts
        .transition_groups()
        .iter()
        .map(|group| -> Result<_, MercError> {
            let variables = group.write_variables().to_vec();

            Ok(compute_vars_bdd(manager_ref, &variables)?.1)
        })
        .collect::<Result<Vec<BDDFunction>, MercError>>()?;

    let mut signature_index = 0;
    loop {
        // No fixed point reached yet, so keep refining.
        let old_num_of_blocks = num_of_blocks;
        trace!("Iteration {} ({} blocks)", iteration, num_of_blocks);

        // Compute the new signatures w.r.t. the previous partition.
        let signature = timing.measure("signature", || -> Result<BDDFunction, OutOfMemory> {
            let mut signature = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));

            if split_signature && !split_partition_groups.is_empty() {
                // Only compute the signature w.r.t. all transition groups that share actions.
                for index in &split_partition_groups[signature_index] {
                    let group_signature = signature_strong(
                        &partition,
                        lts.transition_groups()[*index].relation(),
                        &write_variable_bdd[*index],
                    )?;

                    let group_signature = variable_rename(manager_ref, &group_signature, &state_substitution)?;
                    signature = timing.measure("signature_or", || signature.or(&group_signature))?;
                }
            } else {
                // Compute the full signature by combining all transition groups.
                for (index, (group, write_vars)) in lts
                    .transition_groups()
                    .iter()
                    .zip(write_variable_bdd.iter())
                    .enumerate()
                {
                    // Compute the signature for this transition group.
                    //
                    // Observe that we explicitly do not quantify over state
                    // variables that are not written by the transition group.
                    // Otherwise, these state variables would become unconstrained
                    // and then after substituting next state variables with current
                    // state variables, they would lead to spurious states.
                    let group_signature = timing.measure(&format!("group_signature_{}", index), || {
                        signature_strong(&partition, group.relation(), write_vars)
                    })?;

                    // Substitute next state variables with current state variables to align
                    // with the partition representation, required for `refine`.
                    let group_signature = variable_rename(manager_ref, &group_signature, &state_substitution)?;
                    signature = timing.measure("signature_or", || signature.or(&group_signature))?;
                }
            }

            Ok(signature)
        })?;

        trace!(
            "Signature at iteration {}: {}",
            iteration,
            SignatureDisplay::new(
                &signature,
                &signature_variables,
                lts.state_variable_num_of_bits(),
                lts.action_variables().len() as u32,
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
        partition = timing.measure("refine", || {
            refine(
                manager_ref,
                &mut signature_to_block,
                &block_variables_bdds,
                lts.next_state_variables(),
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
        iteration += 1;

        if split_signature {
            if signature_index == 0 && num_of_blocks == old_num_of_blocks {
                // We only reached a fixed point if after a full cycle no changes occurred.
                break;
            }

            signature_index += 1;
            if signature_index >= split_partition_groups.len() {
                // Start from the first transition group again.
                signature_index = 0;
            }
        } else if num_of_blocks == old_num_of_blocks {
            break;
        }

        // Clear the block assignment for the next iteration.
        signature_to_block.clear();
    }

    info!(
        "Signature refinement completed in {} iterations with {} blocks",
        iteration, num_of_blocks
    );

    Ok(partition)
}

/// Computes a partitioning of the transition groups for the given LTS based on the split signature option.
///
/// # Details
///
/// Two transition groups are combined if they share action labels, this ensures that in split signature mode
/// the action labels of all transition groups are disjoint.
fn combine_transition_groups(manager_ref: &BDDManagerRef, lts: &SymbolicLtsBdd) -> Result<Vec<Vec<usize>>, MercError> {
    // In split signature mode we must ensure that the action labels of all transition groups are disjoint. We do this by merging
    // transition groups that share action labels.

    // Computes the BDD representing all action labels.
    let all_state_vars = manager_ref.with_manager_shared(|manager| -> Result<_, OutOfMemory> {
        let mut bdd: BDDFunction = BDDFunction::f(manager);

        for var in lts.state_variables().iter().chain(lts.action_variables().iter()) {
            let var = BDDFunction::var(manager, *var)?;
            bdd = bdd.and(&var)?;
        }

        Ok(bdd)
    })?;

    // Compute the action labels for all transition groups.
    let representatives = lts
        .transition_groups()
        .iter()
        .map(|group| group.relation().exists(&all_state_vars))
        .collect::<Result<Vec<BDDFunction>, OutOfMemory>>()?;

    // For the split signature we must ensure that all action labels are disjoint.
    let mut result: Vec<Vec<usize>> = Vec::new();

    for (i, _transition_group) in lts.transition_groups().iter().enumerate() {
        // See if the element can be added to an existing group, by taking the first element of
        // each group as representative.
        if let Some(group) = result.iter_mut().find(|g: &&mut Vec<usize>| {
            if let Some(first) = g.first() {
                representatives[*first]
                    .and(&representatives[i])
                    .expect("Out of memory oxidd") // TODO: faillable try_find.
                    .satisfiable()
            } else {
                false
            }
        }) {
            // Add to existing group
            group.push(i);
        } else {
            // Create new group, and add the representative
            result.push(vec![i]);
        }
    }

    debug!("Combined transition groups: {:?}", result);
    Ok(result)
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
/// This function computes the new partition `P` such that:
///
/// > For all states s, t it holds that P(s) == P(t) iff signature(s) == signature(t)
fn refine(
    manager_ref: &BDDManagerRef,
    signature_to_block: &mut FxHashMap<BDDFunction, u64>,
    block_variables_bdds: &[BDDFunction],
    state_variables: &[VarNo],
    signature: &BDDFunction,
    partition: &BDDFunction,
) -> Result<BDDFunction, MercError> {
    manager_ref.with_manager_shared(|manager| {
        let mut cache = FxHashMap::default();

        Ok(BDDFunction::from_edge(
            manager,
            refine_edge(
                manager,
                &mut cache,
                signature_to_block,
                block_variables_bdds,
                state_variables,
                signature.as_edge(manager).borrowed(),
                partition.as_edge(manager).borrowed(),
            )?,
        ))
    })
}

/// Recursive implementation of the [refine] function.
fn refine_edge<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    cache: &mut FxHashMap<(BDDFunction, BDDFunction), BDDFunction>,
    signature_to_block: &mut FxHashMap<BDDFunction, u64>,
    block_variables_bdds: &[BDDFunction],
    state_variables: &[VarNo],
    signature: Borrowed<EdgeOfFunc<'id, BDDFunction>>,
    partition: Borrowed<EdgeOfFunc<'id, BDDFunction>>,
) -> Result<EdgeOfFunc<'id, BDDFunction>, OutOfMemory> {
    let plevel = match manager.get_node(&partition) {
        Node::Terminal(terminal) => {
            if terminal == BDDTerminal::False {
                // In this case the state is not part of the partition function, so return empty.
                return Ok(manager.clone_edge(&partition));
            }

            unreachable!();
        }
        Node::Inner(node) => node.level(),
    };

    if let Some(cached) = cache.get(&(
        BDDFunction::from_edge(manager, manager.clone_edge(&signature)),
        BDDFunction::from_edge(manager, manager.clone_edge(&partition)),
    )) {
        return Ok(manager.clone_edge(cached.as_edge(manager)));
    }

    // topVar
    let lowest_level = {
        let slevel = match manager.get_node(&signature) {
            Node::Terminal(_terminal) => {
                VarNo::MAX // Terminal node, so no variable.
            }
            Node::Inner(node) => node.level(),
        };
        plevel.min(slevel)
    };

    let result = if state_variables.contains(&lowest_level) {
        // Match paths on the level s_i, for irrelevant variables we take both paths.
        let (s_high, s_low) = {
            match manager.get_node(&signature) {
                Node::Inner(node) => {
                    if node.level() == lowest_level {
                        collect_children(node)
                    } else {
                        (signature.borrowed(), signature.borrowed())
                    }
                }
                _ => (signature.borrowed(), signature.borrowed()),
            }
        };
        let (p_high, p_low) = {
            match manager.get_node(&partition) {
                Node::Inner(node) => {
                    if node.level() == lowest_level {
                        collect_children(node)
                    } else {
                        (partition.borrowed(), partition.borrowed())
                    }
                }
                _ => unreachable!("Not a terminal node"),
            }
        };

        let low = refine_edge(
            manager,
            cache,
            signature_to_block,
            block_variables_bdds,
            state_variables,
            s_low,
            p_low,
        )?;
        let high = refine_edge(
            manager,
            cache,
            signature_to_block,
            block_variables_bdds,
            state_variables,
            s_high,
            p_high,
        )?;

        // 7. result := BDDnode(topVar, high, low)
        Ok(reduce(manager, lowest_level, high, low)?)
    } else {
        // 9. else:
        // \sigma (the signature function) now encodes the state signature (a, B)
        // P (the partition function) encodes the current block assignment

        // 10. B := decode_block(partition)
        let block_index = decode_block(manager, partition.borrowed());
        let signature = BDDFunction::from_edge(manager, manager.clone_edge(&signature));
        if let Some(block) = signature_to_block.get(&signature) {
            // 11. If blocks[B].signature == \bottom then
            // 12.     blocks[B].signature := signature
            // 13. if blocks[B].signature == signature then
            // 14.     return P
            if *block == block_index {
                trace!("Found existing signature for {block_index}");
                Ok(manager.clone_edge(&partition)) // The partition just encodes the current block.
            } else {
                // New partition needed
                trace!("Return existing block {block}");
                Ok(encode_block(manager, block_variables_bdds, *block)?)
            }
        } else {
            let new_block_index = signature_to_block.len() as u64;
            trace!("Creating new block {new_block_index}");
            signature_to_block.insert(signature, new_block_index);
            Ok(encode_block(manager, block_variables_bdds, new_block_index)?)
        }
    }?;

    cache.insert(
        (
            BDDFunction::from_edge(manager, manager.clone_edge(&signature)),
            BDDFunction::from_edge(manager, manager.clone_edge(&partition)),
        ),
        BDDFunction::from_edge(manager, manager.clone_edge(&result)),
    );

    Ok(result)
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
) -> Result<EdgeOfFunc<'id, BDDFunction>, OutOfMemory> {
    debug_assert!(
        variables.len() >= required_bits_64(block_no) as usize,
        "Not enough variables to encode block number {}",
        block_no
    );

    let mut result = EdgeDropGuard::new(manager, BDDFunction::t_edge(manager));
    let f_edge = EdgeDropGuard::new(manager, BDDFunction::f_edge(manager));

    for (i, var) in variables.iter().enumerate() {
        if block_no & (1 << i) != 0 {
            // bit is 1
            result = EdgeDropGuard::new(
                manager,
                BDDFunction::ite_edge(manager, var.as_edge(manager), &result, &f_edge)?,
            );
        } else {
            // bit is 0
            result = EdgeDropGuard::new(
                manager,
                BDDFunction::ite_edge(manager, var.as_edge(manager), &f_edge, &result)?,
            );
        }
    }

    Ok(result.into_edge())
}

/// Decodes the given block number from a BDD using the given variables as bits.
///
/// # Details
///
/// Should be the inverse of [encode_block].
fn decode_block<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    block: Borrowed<EdgeOfFunc<'id, BDDFunction>>,
) -> u64 {
    let mut result = 0u64;
    let mut mask = 1u64;
    let mut block = block.borrowed();

    let f_edge = EdgeDropGuard::new(manager, BDDFunction::f_edge(manager));
    while *block != *f_edge {
        match manager.get_node(&block) {
            Node::Inner(node) => {
                let (b_high, b_low) = collect_children(node);
                // For a cube: low satisfiable => bit 0, else => bit 1
                if *b_low != *f_edge {
                    block = b_low;
                } else {
                    result |= mask;
                    block = b_high;
                }
                mask <<= 1;
            }
            Node::Terminal(_) => break,
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

    use merc_ldd::Storage;
    use merc_utilities::Timing;
    use oxidd::BooleanFunction;
    use oxidd::Manager;
    use oxidd::ManagerRef;
    use oxidd::VarNo;
    use oxidd::bdd::BDDFunction;
    use oxidd::error::DuplicateVarName;
    use oxidd::util::Borrowed;
    use rand::RngExt;

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
    fn test_random_symbolic_sigref() {
        random_test(100, |rng| {
            let mut storage = merc_ldd::Storage::new();

            // We don't really check anything here, just ensure that reachability runs without errors.
            let lts = random_symbolic_lts(rng, &mut storage, 10, 5).unwrap();
            let manager_ref = oxidd::bdd::new_manager(2028, 2028, 1);
            let lts_bdd = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &lts).unwrap();

            sigref_symbolic(&manager_ref, &lts_bdd, &Timing::new(), false, false).unwrap();
        });
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
                let decoded = decode_block(manager, Borrowed::new(encoded));

                assert_eq!(
                    block_number, decoded,
                    "Decoding the block number did not yield the original"
                );
            });
        })
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_sigref() {
        random_test(100, |rng| {
            let mut storage = Storage::new();

            // We don't really check anything here, just ensure that reachability runs without errors.
            let lts = random_symbolic_lts(rng, &mut storage, 10, 5).unwrap();

            let manager_ref = oxidd::bdd::new_manager(2028, 2028, 1);
            let lts_bdd = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &lts).unwrap();

            let expected_partition = sigref_symbolic(&manager_ref, &lts_bdd, &mut Timing::new(), false, false).unwrap();

            // Create a separate manager since sigref_symbolic creates new block variables.
            let manager_ref_split = oxidd::bdd::new_manager(2028, 2028, 1);
            let lts_bdd_split = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref_split, &lts).unwrap();
            let split_partition =
                sigref_symbolic(&manager_ref_split, &lts_bdd_split, &mut Timing::new(), false, false).unwrap();

            // Apparently this works even when the BDDs are created in different managers.
            assert!(
                split_partition == expected_partition,
                "Split signature approach does not match actual signature refinement"
            );
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_szymanski_symbolic_refinement() {
        let mut storage = Storage::new();
        let lts_bdd = read_symbolic_lts(&mut storage, include_bytes!("../../../examples/lts/Szymanski_3-bit_lin_wait_alt.sym") as &[u8]).unwrap();

        let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);
        let lts_bdd = SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &lts_bdd).unwrap();

        let _partition = sigref_symbolic(&manager_ref, &lts_bdd, &Timing::new(), false, false).unwrap();
    }
}
