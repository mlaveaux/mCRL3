use log::trace;

use crate::LTS;
use crate::StateIndex;

/// Performs a reachability analysis on the given LTS using a depth-first search
/// from the initial state.
///
/// Returns the list of reachable states, where the i-th element is true iff
/// state i is reachable.
pub fn reachability<P, L: LTS>(lts: &L, state: StateIndex, mut visit: P)
where
    P: FnMut(StateIndex),
{
    let mut reachable = vec![false; lts.num_of_states()];
    visit(state);
    reachable[state] = true;

    let mut stack = vec![state];

    while let Some(state) = stack.pop() {
        debug_assert!(reachable[state], "State {} must already be marked as reachable", state);
        trace!("Visiting {}", state);

        for transition in lts.outgoing_transitions(state) {
            if !reachable[transition.to] {
                trace!("Transition -[{}]-> {}", transition.label, transition.to);
                reachable[transition.to] = true;
                visit(transition.to);
                stack.push(transition.to);
            }
        }
    }

    trace!("Finished reachability");
}

/// Returns the number of states reachable from the given state of the LTS.
pub fn num_reachable_states<L: LTS>(lts: &L, state: StateIndex) -> usize {
    let mut count = 0;
    reachability(lts, state, |_| count += 1);
    count
}