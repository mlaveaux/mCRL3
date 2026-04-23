#![forbid(unsafe_code)]

use rand::Rng;
use rand::RngExt;
use rand::distr::Uniform;

use merc_utilities::MercError;

use crate::LTS;
use crate::LabelledTransitionSystem;
use crate::LtsBuilderFast;
use crate::StateIndex;
use crate::TransitionLabel;
use crate::product_lts;

/// Generates a random LTS with the desired number of states, labels and out
/// degree by composing three smaller random LTSs using the synchronous product.
/// This is often a more realistic structure than fully random LTSs, but
/// otherwise see [`random_lts_monolithic`].
pub fn random_lts<R: Rng>(
    rng: &mut R,
    num_of_states: usize,
    num_of_labels: u32,
    outdegree: usize,
) -> LabelledTransitionSystem<String> {
    let components: Vec<LabelledTransitionSystem<String>> = (0..3)
        .map(|_| random_lts_monolithic(rng, num_of_states, num_of_labels, outdegree))
        .collect();

    // Synchronize on some of the labels.
    let synchronized_labels: Vec<String> = (0..rng.random_range(0..num_of_labels))
        .map(|i| String::from_index(i as usize))
        .collect();

    components
        .into_iter()
        .reduce(|acc, lts| product_lts(&acc, &lts, Some(synchronized_labels.clone())))
        .expect("At least one component should be present")
}

/// Generates a monolithic LTS with the desired number of states, labels, out
/// degree and in degree for all the states. Uses the given TransitionLabel type
/// to generate the transition labels.
///
/// # Details
///
/// The number of labels is limited to 26, since only singular alphabetic labels
/// are used, because those are easier to read and understand.
pub fn random_lts_monolithic<L: TransitionLabel, R: Rng>(
    rng: &mut R,
    num_of_states: usize,
    num_of_labels: u32,
    outdegree: usize,
) -> LabelledTransitionSystem<L> {
    assert!(
        num_of_labels < 26,
        "Too many labels requested, we only support alphabetic labels."
    );

    // Introduce lower case letters for the labels.
    let mut labels: Vec<L> = Vec::new();
    labels.push(L::tau_label()); // The initial hidden label, assumed to be index 0.
    for i in 0..(num_of_labels - 1) {
        labels.push(L::from_index(i as usize));
    }

    let mut builder = LtsBuilderFast::with_capacity(labels.clone(), Vec::new(), num_of_states);

    for state_index in 0..num_of_states {
        // Introduce outgoing transitions for this state based on the desired out degree.
        for _ in 0..rng.random_range(0..outdegree) {
            // Pick a random label and state.
            let label = rng.random_range(0..num_of_labels);
            let to = rng.random_range(0..num_of_states);

            builder.add_transition(
                StateIndex::new(state_index),
                &labels[label as usize],
                StateIndex::new(to),
            );
        }
    }

    // Ensure there is at least one state (otherwise it would be an LTS without initial state).
    builder.require_num_of_states(num_of_states.max(1));

    builder.finish(StateIndex::new(rng.random_range(0..num_of_states)), true)
}

/// Mutates the given LTS by randomly adding and removing transitions, and
/// changing the labels of some transitions. The number of mutations is
/// determined by the `num_of_mutations` parameter.
pub fn mutate_lts<L: LTS, R: Rng>(
    lts: &L,
    rng: &mut R,
    num_of_mutations: usize,
) -> Result<LabelledTransitionSystem<L::Label>, MercError> {
    let mut builder = LtsBuilderFast::new(lts.labels().to_vec(), Vec::new());
    builder.require_num_of_states(lts.num_of_states());

    let removed_transition = if num_of_mutations > 0 { rng.random_range(0..num_of_mutations) } else { 0 };

    // The indices of the transitions to remove.
    let to_remove = if lts.num_of_transitions() > 0 {
        let uniform = Uniform::new(0, lts.num_of_transitions())?;
        rng.sample_iter(uniform).take(removed_transition).collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut index = 0;
    for state in lts.iter_states() {
        for transition in lts.outgoing_transitions(state) {
            // Skip this transition if it is one of the transitions to remove.
            if !to_remove.contains(&index) {
                builder.add_transition(state, &lts.labels()[transition.label], transition.to);
            }

            index += 1;
        }
    }

    // Add new random transitions for the remaining mutations.
    let added_transitions = num_of_mutations - removed_transition;
    if lts.num_of_states() > 0 && !lts.labels().is_empty() {
        let state_uniform = Uniform::new(0, lts.num_of_states())?;
        let label_uniform = Uniform::new(0, lts.labels().len())?;
        for _ in 0..added_transitions {
            let from = StateIndex::new(rng.sample(state_uniform));
            let label = rng.sample(label_uniform);
            let to = StateIndex::new(rng.sample(state_uniform));
            builder.add_transition(from, &lts.labels()[label], to);
        }
    }

    Ok(builder.finish(lts.initial_state_index(), true))
}