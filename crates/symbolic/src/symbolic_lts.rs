use std::fmt;

use merc_aterm::ATerm;
use merc_data::DataSpecification;
use merc_data::DataVariable;
use merc_ldd::Ldd;
use merc_ldd::Storage;
use merc_ldd::Value;
use merc_ldd::compute_meta;
use merc_utilities::MercError;

use crate::SymbolicLTS;
use crate::TransitionGroup;

/// Represents a symbolic LTS encoded by a disjunctive transition relation and a set of states.
pub struct SymbolicLts {
    data_specification: DataSpecification,

    states: Ldd,

    /// A singleton LDD representing the initial state.
    initial_state: Ldd,

    summand_groups: Vec<SummandGroup>,

    action_labels: Vec<String>,
}

impl SymbolicLts {
    /// Creates a new symbolic LTS.
    pub fn new(
        data_specification: DataSpecification,
        states: Ldd,
        initial_state: Ldd,
        summand_groups: Vec<SummandGroup>,
        action_labels: Vec<ATerm>,
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
        }
    }

    /// Returns the data specification of the LTS.
    pub fn data_specification(&self) -> &DataSpecification {
        &self.data_specification
    }
}

impl SymbolicLTS for SymbolicLts {
    fn states(&self) -> &Ldd {
        &self.states
    }

    fn initial_state(&self) -> &Ldd {
        &self.initial_state
    }

    fn transition_groups(&self) -> &[impl TransitionGroup] {
        &self.summand_groups
    }

    fn action_labels(&self) -> &[String] {
        &self.action_labels
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
    relation: Ldd,

    /// The meta information for this summand group.
    meta: Ldd,

    /// The action label index for this summand group.
    action_label_index: usize,
}

impl TransitionGroup for SummandGroup {
    fn relation(&self) -> &Ldd {
        &self.relation
    }

    fn meta(&self) -> &Ldd {
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

    fn summand_count(&self) -> usize {
        1
    }
}

impl SummandGroup {
    /// Creates a new summand group.
    ///
    /// This can fail if one of the read or write parameters is not in the list of all parameters.
    pub fn new(
        storage: &mut Storage,
        parameters: &[DataVariable],
        read_parameters: Vec<DataVariable>,
        write_parameters: Vec<DataVariable>,
        relation: Ldd,
    ) -> Result<Self, MercError> {
        // Find the position of every variable in the parameter list.
        let read_parameter_indices: Vec<Value> = read_parameters
            .iter()
            .map(|var| {
                parameters
                    .iter()
                    .position(|p| p == var)
                    .ok_or(format!("Cannot find read parameter {var:?}"))
                    .map(|pos| pos as Value)
            })
            .collect::<Result<Vec<Value>, _>>()?;

        let write_parameter_indices: Vec<Value> = write_parameters
            .iter()
            .map(|var| {
                parameters
                    .iter()
                    .position(|p| p == var)
                    .ok_or(format!("Cannot find write parameter {var:?}"))
                    .map(|pos| pos as Value)
            })
            .collect::<Result<Vec<Value>, _>>()?;

        let meta = compute_meta(storage, &read_parameter_indices, &write_parameter_indices);
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
    pub fn relation(&self) -> &Ldd {
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
