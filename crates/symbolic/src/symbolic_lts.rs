use std::fmt;

use merc_data::DataExpression;
use merc_data::DataSpecification;
use merc_data::DataVariable;
use merc_lts::LtsAction;
use merc_lts::LtsMultiAction;
use merc_utilities::MercError;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;

use crate::SymbolicLTS;
use crate::TransitionGroup;

/// Represents a symbolic LTS encoded by a disjunctive transition relation and a set of states.
pub struct SymbolicLts {
    data_specification: DataSpecification,
    states: LDDFunction,

    /// A singleton LDD representing the initial state.
    initial_state: LDDFunction,
    summand_groups: Vec<SummandGroup>,

    /// The action labels of the LTS, stored as their string representation,
    /// their position corresponds to the LDD values.
    action_labels: Vec<String>,

    /// The possible values for each process parameter, their position
    /// corresponds to the LDD values.
    parameter_values: Vec<Vec<DataExpression>>,
}

impl SymbolicLts {
    /// Creates a new symbolic LTS.
    ///
    /// `parameter_values` contains, for each process parameter, the list of
    /// possible values it can take. Their indices correspond to the values stored
    /// in the LDDs.
    pub fn new(
        data_specification: DataSpecification,
        states: LDDFunction,
        initial_state: LDDFunction,
        summand_groups: Vec<SummandGroup>,
        action_labels: Vec<LtsMultiAction<LtsAction>>,
        parameter_values: Vec<Vec<DataExpression>>,
    ) -> Self {
        let action_labels = action_labels
            .into_iter()
            .map(|aterm| format!("{}", aterm))
            .collect::<Vec<String>>();

        Self {
            data_specification,
            states,
            initial_state,
            summand_groups,
            action_labels,
            parameter_values,
        }
    }

    /// Returns the data specification of the LTS.
    pub fn data_specification(&self) -> &DataSpecification {
        &self.data_specification
    }

    pub fn parameter_values(&self) -> &[Vec<DataExpression>] {
        &self.parameter_values
    }

    /// Replaces the set of states of the LTS, for example after running reachability.
    pub fn set_states(&mut self, states: LDDFunction) {
        self.states = states;
    }
}

impl SymbolicLTS for SymbolicLts {
    fn states(&self) -> &LDDFunction {
        &self.states
    }

    fn initial_state(&self) -> &LDDFunction {
        &self.initial_state
    }

    fn transition_groups(&self) -> &[impl TransitionGroup] {
        &self.summand_groups
    }

    fn transition_groups_mut(&mut self) -> &mut [impl TransitionGroup] {
        &mut self.summand_groups
    }

    fn action_labels(&self) -> &[String] {
        &self.action_labels
    }

    fn parameter_values(&self) -> &[Vec<DataExpression>] {
        &self.parameter_values
    }
}

/// Represents a short vector transition relation for a group of summands.
///
/// # Details
///
/// A short transition vector is part of a transition relation T -> U, where we
/// store T' -> U' with T' being the projection of T on the read parameters and
/// U' the projection of U on the write parameters, as a LDD. Formally,
///
/// (t, u) in (T -> U)  iff  (t', u') in (T' -> U') where t' and u' are the projections
///     of t and u on the read and write parameters respectively.
pub struct SummandGroup {
    /// The read parameters for this summand group.
    read_parameters: Vec<DataVariable>,
    read_parameter_indices: Vec<Value>,

    /// The write parameters for this summand group.
    write_parameters: Vec<DataVariable>,
    write_parameter_indices: Vec<Value>,

    /// The transition relation T' -> U' for this summand group.
    relation: LDDFunction,

    /// The meta information for this summand group.
    meta: LDDFunction,

    /// The action label index for this summand group.
    action_label_index: usize,
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
        // All states are already explored.
        Ok(())
    }
}

impl SummandGroup {
    /// Creates a new summand group.
    ///
    /// This can fail if one of the read or write parameters is not in the list of all parameters.
    pub fn new(
        manager: &LDDManagerRef,
        parameters: &[DataVariable],
        read_parameters: Vec<DataVariable>,
        write_parameters: Vec<DataVariable>,
        relation: LDDFunction,
    ) -> Result<Self, MercError> {
        // Find the position of every variable in the parameter list.
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

        // For later processing it is convenient if these indices are sorted.
        write_parameter_indices.sort();
        read_parameter_indices.sort();

        let meta = LDDFunction::relation_product_meta(manager, &read_parameter_indices, &write_parameter_indices)?.0;
        let action_label_index = read_parameters.len() + write_parameters.len(); // The action label is the last index

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

impl fmt::Debug for SummandGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} -> {:?}",
            self.read_parameter_indices, self.write_parameter_indices
        )
    }
}
