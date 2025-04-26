use std::fmt::Debug;
use std::hash::Hash;

use mcrl3_lts::LabelledTransitionSystem;
use mcrl3_lts::StateIndex;
use rustc_hash::FxHashSet;

use crate::Partition;

use super::BlockPartition;
use super::IndexedPartition;
use super::quotient_lts;
use super::reorder_partition;
use super::reorder_states;
use super::sort_topological;
use super::tau_scc_decomposition;

/// The builder used to construct the signature.
pub type SignatureBuilder = Vec<(usize, usize)>;

/// The type of a signature. We use sorted vectors to avoid the overhead of hash
/// sets that might have unused values.
#[derive(Eq)]
pub struct Signature<'a>(&'a [(usize, usize)]);

impl<'a> Signature<'a> {
    pub fn new(slice: &'a [(usize, usize)]) -> Self {
        Signature(slice)
    }
}

impl Signature<'_> {
    pub fn as_slice(&self) -> &[(usize, usize)] {
        &*self.0
    }

    // Check if target is a subset of self, excluding a specific element
    pub fn is_subset_of(&self, other: &[(usize, usize)], exclude: (usize, usize)) -> bool {
        let mut self_iter = self.as_slice().iter();
        let mut other_iter = other.iter().filter(|&&x| x != exclude);

        let mut self_item: Option<&(usize, usize)> = self_iter.next();
        let mut other_item = other_iter.next();

        while let Some(&o) = other_item {
            match self_item {
                Some(&s) if s == o => {
                    // Match found, move both iterators forward
                    self_item = self_iter.next();
                    other_item = other_iter.next();
                }
                Some(&s) if s < o => {
                    // Move only self iterator forward
                    self_item = self_iter.next();
                }
                _ => {
                    // No match found in self for o
                    return false;
                }
            }
        }
        // If we finished self_iter without returning false, self is a subset
        true
    }
}

impl Default for Signature<'_> {
    fn default() -> Self {
        Signature(&[])
    }
}

impl PartialEq for Signature<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Hash for Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { (*self.0).hash(state) }
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

/// Returns the signature for strong bisimulation sig(s, pi) = { (a, pi(t)) | s -a-> t in T }
pub fn strong_bisim_signature(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    builder: &mut SignatureBuilder,
) {
    builder.clear();

    for (label, to) in lts.outgoing_transitions(state_index) {
        builder.push((*label, partition.block_number(*to)));
    }

    // Compute the flat signature, which has Hash and is more compact.
    builder.sort_unstable();
    builder.dedup();
}

/// Returns the branching bisimulation signature for branching bisimulation
/// sig(s, pi) = { (a, pi(t)) | s -[tau]-> s1 -> ... s_n -[a]-> t in T && pi(s) = pi(s_i) && ((a != tau) || pi(s) != pi(t)) }
pub fn branching_bisim_signature(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    builder: &mut SignatureBuilder,
    visited: &mut FxHashSet<usize>,
    stack: &mut Vec<usize>,
) {
    // Clear the builders and the list of visited states.
    builder.clear();
    visited.clear();

    // A stack used for depth first search of tau paths.
    debug_assert!(stack.is_empty(), "The stack should be empty");
    stack.push(state_index);

    while let Some(inner_state_index) = stack.pop() {
        visited.insert(inner_state_index);

        for (label_index, to_index) in lts.outgoing_transitions(inner_state_index) {
            if lts.is_hidden_label(*label_index) {
                if partition.block_number(state_index) == partition.block_number(*to_index) {
                    // Explore the outgoing state as well, still tau path in same block
                    if !visited.contains(to_index) {
                        visited.insert(*to_index);
                        stack.push(*to_index);
                    }
                } else {
                    //  pi(s) != pi(t)
                    builder.push((*label_index, partition.block_number(*to_index)));
                }
            } else {
                // (a != tau) This is a visible action only reachable from tau paths with equal signatures.
                builder.push((*label_index, partition.block_number(*to_index)));
            }
        }
    }

    // Compute the flat signature, which has Hash and is more compact.
    builder.sort_unstable();
    builder.dedup();
}

/// The input lts must contain no tau-cycles.
pub fn branching_bisim_signature_sorted(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    state_to_signature: &[Signature],
    builder: &mut SignatureBuilder,
) {
    builder.clear();

    for &(label_index, to) in lts.outgoing_transitions(state_index) {
        let to_block = partition.block_number(to);

        if partition.block_number(state_index) == to_block {
            if lts.is_hidden_label(label_index) {
                // Inert tau transition, take signature from the outgoing tau-transition.
                builder.extend(state_to_signature[to].as_slice());
            } else {
                builder.push((label_index, to_block));
            }
        } else {
            // Visible action, add to the signature.
            builder.push((label_index, to_block));
        }
    }

    // Compute the flat signature, which has Hash and is more compact.
    builder.sort_unstable();
    builder.dedup();
}

/// The input lts must contain no tau-cycles.
pub fn branching_bisim_signature_inductive(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &BlockPartition,
    state_to_key: &[usize],
    builder: &mut SignatureBuilder,
) {
    builder.clear();

    let num_act: usize = lts.num_of_labels(); //this label index does not occur.
    for &(label_index, to) in lts.outgoing_transitions(state_index) {
        let to_block = partition.block_number(to);

        if partition.block_number(state_index) == to_block {
            if lts.is_hidden_label(label_index) && partition.is_element_marked(to) {
                // Inert tau transition, take signature from the outgoing tau-transition.
                builder.push((num_act, state_to_key[to]));
            } else {
                builder.push((label_index, to_block));
            }
        } else {
            // Visible action, add to the signature.
            builder.push((label_index, to_block));
        }
    }

    // Compute the flat signature, which has Hash and is more compact.
    builder.sort_unstable();
    builder.dedup();
}

/// Perform the preprocessing necessary for branching bisimulation with the
/// sorted signature see `branching_bisim_signature_sorted`.
pub fn preprocess_branching(lts: &LabelledTransitionSystem) -> (LabelledTransitionSystem, IndexedPartition) {
    let scc_partition = tau_scc_decomposition(lts);
    let tau_loop_free_lts = quotient_lts(lts, &scc_partition, true);

    // Sort the states according to the topological order of the tau transitions.
    let topological_permutation = sort_topological(
        &tau_loop_free_lts,
        |label_index, _| tau_loop_free_lts.is_hidden_label(label_index),
        true,
    )
    .expect("After quotienting, the LTS should not contain cycles");

    (
        reorder_states(&tau_loop_free_lts, |i| topological_permutation[i]),
        reorder_partition(scc_partition, |i| topological_permutation[i]),
    )
}
