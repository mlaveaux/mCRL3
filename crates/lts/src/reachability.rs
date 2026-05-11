use log::trace;

use crate::LTS;
use crate::LabelIndex;
use crate::StateIndex;

/// Performs a reachability analysis on the given LTS using a depth-first search
/// from the initial state.
///
/// Only follows transitions for which `filter` returns `true`.
/// Calls `visit` for every newly reached state, including the start state.
pub fn reachability<P, F, L: LTS>(lts: &L, state: StateIndex, mut filter: F, mut visit: P)
where
    P: FnMut(StateIndex),
    F: FnMut(LabelIndex) -> bool,
{
    let mut reachable = vec![false; lts.num_of_states()];
    visit(state);
    reachable[state] = true;

    let mut stack = vec![state];

    while let Some(state) = stack.pop() {
        debug_assert!(reachable[state], "State {} must already be marked as reachable", state);
        trace!("Visiting {}", state);

        for transition in lts.outgoing_transitions(state) {
            if filter(transition.label) && !reachable[transition.to] {
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
    reachability(lts, state, |_| true, |_| count += 1);
    count
}
