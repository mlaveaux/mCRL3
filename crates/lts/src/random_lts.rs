use rand::Rng;
use rustc_hash::FxHashSet;

use crate::LabelledTransitionSystem;

/// Generates a monolithic LTS with the desired number of states, labels, out
/// degree and in degree for all the states.
pub fn random_lts(num_of_states: usize, num_of_labels: u32, outdegree: usize) -> LabelledTransitionSystem {
    // Introduce lower case letters for the labels.
    let tau_label = "tau".to_string();

    let mut labels: Vec<String> = vec![tau_label.clone()];
    for i in 0..num_of_labels {
        labels.push(char::from_digit(i + 10, 36).unwrap().to_string());
    }

    let mut rng = rand::rng();
    let mut transitions: FxHashSet<(usize, usize, usize)> = FxHashSet::default();

    for state_index in 0..num_of_states {
        // Introduce outgoing transitions for this state based on the desired out degree.
        for _ in 0..rng.random_range(0..outdegree) {
            // Pick a random label and state.
            let label = rng.random_range(0..num_of_labels);
            let to = rng.random_range(0..num_of_states);

            transitions.insert((state_index, label as usize, to));
        }
    }

    LabelledTransitionSystem::new(
        0,
        Some(num_of_states),
        || transitions.iter().cloned(),
        labels,
        vec![tau_label],
    )
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test]
    fn test_random_lts() {
        let _lts = random_lts(10, 3, 3);
    }
}
