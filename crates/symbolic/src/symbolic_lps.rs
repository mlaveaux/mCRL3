use std::fmt;

use merc_data::DataVariable;
use merc_utilities::MercError;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;

use crate::DependencyGraph;
use crate::Relation;

/// A generic trait for symbolic LDD-based state-space exploration.
///
/// Any type with an initial state and transition groups can be explored via [crate::reachability].
pub trait SymbolicLPS {
    /// Returns the LDD representing the initial state(s).
    fn initial_state(&self) -> &LDDFunction;

    /// Returns the transition groups.
    fn transition_groups(&self) -> &[impl TransitionGroup];

    /// Returns the transition groups (mutable), needed for on-the-fly learning.
    fn transition_groups_mut(&mut self) -> &mut [impl TransitionGroup];

    /// Computes the dependency graph from the read/write indices of each transition group.
    fn dependency_graph(&self) -> DependencyGraph {
        let relations = self
            .transition_groups()
            .iter()
            .map(|group| {
                Relation::new(
                    group.read_indices().iter().map(|i| *i as usize).collect(),
                    group.write_indices().iter().map(|i| *i as usize).collect(),
                )
            })
            .collect();

        DependencyGraph::new(relations)
    }
}

/// A single LDD-based transition group for a set of summands (short vector encoding).
pub trait TransitionGroup: fmt::Debug {
    /// Returns the transition relation T' -> U' for this summand group.
    fn relation(&self) -> &LDDFunction;

    /// Returns the read indices for this summand group.
    fn read_indices(&self) -> &[u32];

    /// Returns the write indices for this summand group.
    fn write_indices(&self) -> &[u32];

    /// Returns the action label index for this summand group, if any.
    fn action_label_index(&self) -> Option<usize>;

    /// Returns the meta LDD required to perform the relational product.
    fn meta(&self) -> &LDDFunction;

    /// Learns the successors of `todo` and incorporates them into [Self::relation].
    fn learn_successors(&mut self, manager: &LDDManagerRef, todo: &LDDFunction) -> Result<(), MercError>;
}

/// A short vector transition relation for a group of summands.
///
/// A short transition vector is part of a transition relation T -> U, where we
/// store T' -> U' with T' being the projection of T on the read parameters and
/// U' the projection of U on the write parameters, as a LDD. Formally,
///
/// (t, u) in (T -> U)  iff  (t', u') in (T' -> U') where t' and u' are the projections
///     of t and u on the read and write parameters respectively.
pub struct SummandGroup {
    read_parameters: Vec<DataVariable>,
    read_parameter_indices: Vec<Value>,

    write_parameters: Vec<DataVariable>,
    write_parameter_indices: Vec<Value>,

    relation: LDDFunction,
    meta: LDDFunction,
    action_label_index: usize,
}

impl SummandGroup {
    /// Creates a new summand group.
    ///
    /// Fails if a read or write parameter is not found in `parameters`.
    pub fn new(
        manager: &LDDManagerRef,
        parameters: &[DataVariable],
        read_parameters: Vec<DataVariable>,
        write_parameters: Vec<DataVariable>,
        relation: LDDFunction,
    ) -> Result<Self, MercError> {
        let mut read_parameter_indices: Vec<Value> = read_parameters
            .iter()
            .map(|var| {
                parameters
                    .iter()
                    .position(|p| p == var)
                    .ok_or(format!("Cannot find read parameter {var:?}"))
                    .map(|pos| pos as Value)
            })
            .collect::<Result<Vec<Value>, _>>()?;

        let mut write_parameter_indices: Vec<Value> = write_parameters
            .iter()
            .map(|var| {
                parameters
                    .iter()
                    .position(|p| p == var)
                    .ok_or(format!("Cannot find write parameter {var:?}"))
                    .map(|pos| pos as Value)
            })
            .collect::<Result<Vec<Value>, _>>()?;

        write_parameter_indices.sort();
        read_parameter_indices.sort();

        let meta = LDDFunction::relation_product_meta(manager, &read_parameter_indices, &write_parameter_indices)?.0;
        let action_label_index = read_parameters.len() + write_parameters.len();

        Ok(Self {
            read_parameters,
            read_parameter_indices,
            write_parameters,
            write_parameter_indices,
            relation,
            meta,
            action_label_index,
        })
    }

    /// Returns the transition relation LDD for this summand group.
    pub fn relation(&self) -> &LDDFunction {
        &self.relation
    }

    /// Returns the read parameters for this summand group.
    pub fn read_parameters(&self) -> &[DataVariable] {
        &self.read_parameters
    }

    /// Returns the write parameters for this summand group.
    pub fn write_parameters(&self) -> &[DataVariable] {
        &self.write_parameters
    }
}

impl TransitionGroup for SummandGroup {
    fn relation(&self) -> &LDDFunction {
        &self.relation
    }

    fn meta(&self) -> &LDDFunction {
        &self.meta
    }

    fn read_indices(&self) -> &[u32] {
        &self.read_parameter_indices
    }

    fn write_indices(&self) -> &[u32] {
        &self.write_parameter_indices
    }

    fn action_label_index(&self) -> Option<usize> {
        Some(self.action_label_index)
    }

    fn learn_successors(&mut self, _storage: &LDDManagerRef, _todo: &LDDFunction) -> Result<(), MercError> {
        Ok(())
    }
}

impl fmt::Debug for SummandGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} -> {:?}",
            self.read_parameter_indices, self.write_parameter_indices
        )
    }
}
