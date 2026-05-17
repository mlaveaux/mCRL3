use std::fmt;

use log::debug;
use log::info;
use log::trace;
use merc_data::DataExpression;
use merc_io::TimeProgress;
use merc_ldd::Ldd;
use merc_ldd::LddDisplay;
use merc_ldd::Storage;
use merc_ldd::len;
use merc_ldd::minus;
use merc_ldd::relational_product;
use merc_ldd::union;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::DependencyGraph;
use crate::Relation;

/// A generic trait representing a symbolic LTS
pub trait SymbolicLTS {
    /// Returns the LDD representing the set of states.
    fn states(&self) -> &Ldd;

    /// Returns the LDD representing the initial state.
    fn initial_state(&self) -> &Ldd;

    /// Returns an iterator over the summand groups.
    fn transition_groups(&self) -> &[impl TransitionGroup];

    /// Returns a mutable iterator over the summand groups.
    fn transition_groups_mut(&mut self) -> &mut [impl TransitionGroup];

    /// Returns the action labels for the LTS.
    fn action_labels(&self) -> &[String];

    /// Returns the possible values for each process parameter.
    fn parameter_values(&self) -> &[Vec<DataExpression>];

    /// Computes the dependency graph of the LTS.
    fn dependency_graph(&self) -> DependencyGraph {
        let mut relations = Vec::new();

        for group in self.transition_groups() {
            relations.push(Relation::new(
                group.read_indices().iter().map(|i| *i as usize).collect(),
                group.write_indices().iter().map(|i| *i as usize).collect(),
            ));
        }

        DependencyGraph::new(relations)
    }
}

pub trait TransitionGroup: fmt::Debug {
    /// Returns the transition relation T' -> U' for this summand group.
    fn relation(&self) -> &Ldd;

    /// Returns the read indices for this summand group.
    fn read_indices(&self) -> &[u32];

    /// Returns the write indices for this summand group.
    fn write_indices(&self) -> &[u32];

    /// Returns the action label index for this summand group, if any.
    fn action_label_index(&self) -> Option<usize>;

    /// Returns the meta information for this summand group.
    fn meta(&self) -> &Ldd;

    /// Learns the successors of the given set of states.
    ///
    /// The `todo` the set of vectors for which successors should be learned.
    fn learn_successors(&mut self, storage: &mut Storage, todo: &Ldd) -> Result<Ldd, MercError>;
}

/// Performs reachability analysis using the given initial state and transitions from the symbolic LTS.
pub fn reachability<L: SymbolicLTS>(storage: &mut Storage, lts: &mut L, timing: &Timing) -> Result<usize, MercError> {
    let mut todo = lts.initial_state().clone();
    let mut states = lts.initial_state().clone(); // The state space.
    let mut iteration = 0;

    trace!("states = {}", LddDisplay::new(storage, &states));
    let progress = TimeProgress::new(
        |(iteration, num_of_states)| {
            info!("explored {} state(s) after {} iteration(s)", num_of_states, iteration);
        },
        1,
    );

    timing.measure("reachability", || {
        while todo != *storage.empty_set() {
            debug!("Iteration {}: todo size = {}", iteration, len(storage, &todo));

            // Learn successors for all the transition groups and compute the next todo set.
            let mut todo1 = storage.empty_set().clone();
            for (i, transition) in lts.transition_groups_mut().iter_mut().enumerate() {
                trace!(
                    "Learning successors for transition group {}:",
                    i
                );
                timing.measure(&format!("learn_successors_{}", i), || {
                    transition.learn_successors(storage, &todo)
                })?;

                let result = relational_product(storage, &todo, transition.relation(), transition.meta());
                todo1 = union(storage, &todo1, &result);
            }

            trace!("todo1 = {}", LddDisplay::new(storage, &todo1));

            todo = minus(storage, &todo1, &states);
            states = union(storage, &states, &todo);
            if progress.is_due() {
                progress.print((iteration, len(storage, &states)));
            }
            iteration += 1;
        }

        Ok(len(storage, &states))
    })
}
