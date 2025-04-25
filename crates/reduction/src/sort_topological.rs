use log::debug;
use log::trace;
use mcrl3_lts::LabelledTransitionSystem;
use mcrl3_utilities::MCRL3Error;

/// Returns a topological ordering of the states of the given LTS.
///
/// An error is returned if the LTS contains a cycle.
///     - reverse: If true, the topological ordering is reversed, i.e. successors before the incoming state.
pub fn sort_topological<F>(
    lts: &LabelledTransitionSystem,
    filter: F,
    reverse: bool,
) -> Result<Vec<usize>, MCRL3Error>
where
    F: Fn(usize, usize) -> bool,
{
    let start = std::time::Instant::now();
    trace!("{:?}", lts);

    // The resulting order of states.
    let mut stack = Vec::new();

    let mut visited = vec![false; lts.num_of_states()];
    let mut depth_stack = Vec::new();
    let mut marks = vec![None; lts.num_of_states()];

    for state_index in lts.iter_states() {
        if marks[state_index].is_none()
            && !sort_topological_visit(
                lts,
                &filter,
                state_index,
                &mut depth_stack,
                &mut marks,
                &mut visited,
                &mut stack,
            )
        {
            trace!("There is a cycle from state {state_index} on path {stack:?}");
            return Err("Labelled transition system contains a cycle".into());
        }
    }

    if !reverse {
        stack.reverse();
    }
    trace!("Topological order: {stack:?}");

    // Turn the stack into a permutation.
    let mut reorder = vec![0; lts.num_of_states()];
    for (i, &state_index) in stack.iter().enumerate() {
        reorder[state_index] = i;
    }

    debug_assert!(
        is_topologically_sorted(lts, filter, |i| reorder[i], reverse),
        "The permutation {reorder:?} is not a valid topological ordering of the states of the given LTS: {lts:?}"
    );
    debug!("Time sort_topological: {:.3}s", start.elapsed().as_secs_f64());

    Ok(reorder)
}

/// Reorders the states of the given LTS according to the given permutation.
pub fn reorder_states<P>(lts: &LabelledTransitionSystem, permutation: P) -> LabelledTransitionSystem
where
    P: Fn(usize) -> usize,
{
    let start = std::time::Instant::now();

    // We know that it is a permutation, so there won't be any duplicated transitions.
    let mut transitions: Vec<(usize, usize, usize)> = Vec::default();

    for state_index in lts.iter_states() {
        let new_state_index = permutation(state_index);

        for (label, to_index) in lts.outgoing_transitions(state_index) {
            let new_to_index = permutation(*to_index);
            transitions.push((new_state_index, *label, new_to_index));
        }
    }

    debug!("Time reorder_states: {:.3}s", start.elapsed().as_secs_f64());
    LabelledTransitionSystem::new(
        permutation(lts.initial_state_index()),
        Some(lts.num_of_states()),
        || transitions.iter().cloned(),
        lts.labels().into(),
        lts.hidden_labels().into(),
    )
}

// The mark of a state in the depth first search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Temporary,
    Permanent,
}

/// Visits the given state in a depth first search.
///
/// Returns false if a cycle is detected.
fn sort_topological_visit<F>(
    lts: &LabelledTransitionSystem,
    filter: &F,
    state_index: usize,
    depth_stack: &mut Vec<usize>,
    marks: &mut [Option<Mark>],
    visited: &mut [bool],
    stack: &mut Vec<usize>,
) -> bool
where
    F: Fn(usize, usize) -> bool,
{
    // Perform a depth first search.
    depth_stack.push(state_index);

    while let Some(state) = depth_stack.pop() {
        match marks[state] {
            None => {
                marks[state] = Some(Mark::Temporary);
                depth_stack.push(state); // Re-add to stack to mark as permanent later
                for (_, next_state) in lts
                    .outgoing_transitions(state)
                    .filter(|(label, to)| filter(*label, *to))
                {
                    // If it was marked temporary, then a cycle is detected.
                    if marks[*next_state] == Some(Mark::Temporary) {
                        return false;
                    }
                    if marks[*next_state].is_none() {
                        depth_stack.push(*next_state);
                    }
                }
            }
            Some(Mark::Temporary) => {
                marks[state] = Some(Mark::Permanent);
                visited[state] = true;
                stack.push(state);
            }
            Some(Mark::Permanent) => {}
        }
    }

    true
}

/// Returns true if the given permutation is a topological ordering of the states of the given LTS.
fn is_topologically_sorted<F, P>(lts: &LabelledTransitionSystem, filter: F, permutation: P, reverse: bool) -> bool
where
    F: Fn(usize, usize) -> bool,
    P: Fn(usize) -> usize,
{
    debug_assert!(is_valid_permutation(&permutation, lts.num_of_states()));

    // Check that each vertex appears before its successors.
    for state_index in lts.iter_states() {
        let state_order = permutation(state_index);
        for (_, successor) in lts
            .outgoing_transitions(state_index)
            .filter(|(label, to)| filter(*label, *to))
        {
            if reverse {
                if state_order <= permutation(*successor) {
                    return false;
                }
            } else if state_order >= permutation(*successor) {
                return false;
            }
        }
    }

    true
}

/// Returns true if the given permutation is a valid permutation.
fn is_valid_permutation<P>(permutation: &P, max: usize) -> bool
where
    P: Fn(usize) -> usize,
{
    let mut visited = vec![false; max];

    for i in 0..max {
        // Out of bounds
        if permutation(i) >= max {
            return false;
        }

        if visited[permutation(i)] {
            return false;
        }

        visited[permutation(i)] = true;
    }

    // Check that all entries are visited.
    visited.iter().all(|&v| v)
}

#[cfg(test)]
mod tests {

    use mcrl3_lts::random_lts;
    use rand::seq::SliceRandom;
    use test_log::test;

    use super::*;

    #[test]
    fn test_sort_topological_with_cycles() {
        let lts = random_lts(10, 3, 2);
        match sort_topological(&lts, |_, _| true, false) {
            Ok(order) => assert!(is_topologically_sorted(&lts, |_, _| true, |i| order[i], false)),
            Err(_) => {}
        }
    }

    #[test]
    fn test_reorder_states() {
        let lts = random_lts(10, 3, 2);

        // Generate a random permutation.
        let mut rng = rand::rng();
        let order: Vec<usize> = {
            let mut order: Vec<usize> = (0..lts.num_of_states()).collect();
            order.shuffle(&mut rng);
            order
        };

        let new_lts = reorder_states(&lts, |i| order[i]);

        trace!("{:?}", lts);
        trace!("{:?}", new_lts);

        //assert_eq!(new_lts.num_of_states(), lts.num_of_states());
        assert_eq!(new_lts.num_of_labels(), lts.num_of_labels());

        for from in lts.iter_states() {
            // Check that the states are in the correct order.
            for &(label, to) in lts.outgoing_transitions(from) {
                let new_from = order[from];
                let new_to = order[to];
                assert!(
                    new_lts
                        .outgoing_transitions(new_from)
                        .any(|trans| *trans == (label, new_to))
                );
            }
        }
    }

    #[test]
    fn test_is_valid_permutation() {
        let lts = random_lts(10, 15, 2);

        // Generate a valid permutation.
        let mut rng = rand::rng();
        let valid_permutation: Vec<usize> = {
            let mut order: Vec<usize> = (0..lts.num_of_states()).collect();
            order.shuffle(&mut rng);
            order
        };

        assert!(is_valid_permutation(&|i| valid_permutation[i], valid_permutation.len()));

        // Generate an invalid permutation (duplicate entries).
        let invalid_permutation = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 8];
        assert!(!is_valid_permutation(
            &|i| invalid_permutation[i],
            invalid_permutation.len()
        ));

        // Generate an invalid permutation (missing entries).
        let invalid_permutation = vec![0, 1, 3, 4, 5, 6, 7, 8];
        assert!(!is_valid_permutation(
            &|i| invalid_permutation[i],
            invalid_permutation.len()
        ));
    }
}
