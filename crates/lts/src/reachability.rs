use crate::{LTS, StateIndex};

/// Performs a reachability analysis on the given LTS using a depth-first search
/// from the initial state.
///
/// Returns the list of reachable states, where the i-th element is true iff
/// state i is reachable.
pub fn reachability<L: LTS>(lts: &L, state: StateIndex) -> Vec<bool> {
    let mut reachable = vec![false; lts.num_of_states()];
    let mut stack = vec![state];

    while let Some(state) = stack.pop() {
        if reachable[state] {
            continue;
        }

        reachable[state] = true;

        for transition in lts.outgoing_transitions(state) {
            if !reachable[transition.to] {
                stack.push(transition.to);
            }
        }
    }

    reachable
}

/// Returns the number of states reachable from the given state of the LTS.
pub fn num_reachable_states<L: LTS>(lts: &L, state: StateIndex) -> usize {
    reachability(lts, state).iter().filter(|&&r| r).count()
}
