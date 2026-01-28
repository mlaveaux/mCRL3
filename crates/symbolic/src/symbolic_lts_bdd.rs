use std::ops::Range;

use log::debug;
use log::info;
use oxidd::BooleanFunction;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::VarNo;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::error::DuplicateVarName;

use merc_data::DataExpression;
use merc_ldd::singleton;
use merc_ldd::Storage;
use merc_ldd::Value;
use merc_utilities::MercError;

use crate::SymbolicLTS;
use crate::TransitionGroup;
use crate::compute_bits;
use crate::compute_highest;
use crate::ldd_to_bdd;
use crate::required_bits;

/// The BDD representing the support variables of a BDD function.
pub type BDDSupport = BDDFunction;

/// A symbolic LTS that uses BDDs for the symbolic representation, instead of
/// LDDs as done in [crate::SymbolicLts].
pub struct SymbolicLtsBdd {
    /// The BDD representing the set of states.
    states: BDDFunction,

    /// The BDD representing the set of initial states.
    initial_state: BDDFunction,

    /// The transition groups representing the disjunctive transition relation.
    transition_groups: Vec<SummandGroupBdd>,

    /// The number of bits used for each state variable.
    state_variable_num_of_bits: Vec<Value>,

    /// The variable numbers used to represent the state variables.
    state_variable_indices: Vec<VarNo>,

    /// The BDDs representing each state variable.
    state_variables: Vec<BDDFunction>,

    /// The set of BDD variables used to represent the state variables.
    state_variables_bdd: BDDSupport,

    /// The set of BDD variables used to represent the next state variables (or primed variables).
    next_state_variables_bdd: BDDSupport,

    /// The BDD variables used to represent the next state variables (or primed variables).
    next_state_variables: Vec<BDDFunction>,

    /// The variable numbers used to represent the next state variables.
    next_state_variables_indices: Vec<VarNo>,

    /// The action variable indices used to represent the action labels.
    action_variable_indices: Vec<VarNo>,

    /// The BDD representing the action label variables.
    action_variables_bdd: BDDSupport,

    /// The action labels of the LTS, stored as their string representation,
    /// their position corresponds to the LDD values.
    action_labels: Vec<String>,

    /// The possible values for each process parameter, their position
    /// corresponds to the LDD values.
    parameter_values: Vec<Vec<DataExpression>>,
}

impl SymbolicLtsBdd {
    /// Converts a symbolic LTS using LDDs into a symbolic LTS using BDDs.
    ///
    /// # Details
    ///
    /// The resulting BDD is assumed to only be valid for the reachable states
    /// of the LDD symbolic LTS, as unreachable states may not be representable
    /// with the number of bits assigned to each state variable.
    pub fn from_symbolic_lts(
        storage: &mut Storage,
        manager_ref: &BDDManagerRef,
        lts: &impl SymbolicLTS,
    ) -> Result<Self, MercError> {
        info!("Converting symbolic LTS from LDD to BDD representation...");

        // Determine the highest values for every layer in the LDD representing the states
        let state_bits = compute_bits(&compute_highest(storage, lts.states()));
        debug!("Determined number of bits for state variables: {:?}", state_bits);

        let mut action_label_highest = 0u32;
        for group in lts.transition_groups() {
            let highest = compute_highest(storage, group.relation());

            // Deal with the special empty case.
            if !highest.is_empty() {
                action_label_highest = action_label_highest
                    .max(highest[group.action_label_index().ok_or("Action label index not found")?]);
            }
        }

        let action_label_bits = required_bits(action_label_highest);
        debug!(
            "Highest action label: {}, bits: {}",
            action_label_highest, action_label_bits
        );

        // Create the state variables, with interleaved primed variables for write parameters
        let mut vars = Vec::new();

        // Keep track of the bits per state variable.
        let mut state_variables_bits: Vec<Vec<VarNo>> = Vec::new();
        let mut next_state_variables_bits: Vec<Vec<VarNo>> = Vec::new();
        for (i, &bits) in state_bits.iter().enumerate() {
            let mut state_var_bits = Vec::new();
            let mut next_state_var_bits = Vec::new();
            for k in 0..bits {
                // Add variable for the state variable
                state_var_bits.push(vars.len() as VarNo);
                vars.push(format!("s{}_{}", i, k));

                // Add a primed version for the write parameters
                next_state_var_bits.push(vars.len() as VarNo);
                vars.push(format!("s{}'_{}", i, k));
            }
            state_variables_bits.push(state_var_bits);
            next_state_variables_bits.push(next_state_var_bits);
        }

        // Create action label variables
        let mut action_labels_vars = Vec::new();
        for k in 0..action_label_bits {
            action_labels_vars.push(vars.len() as VarNo);
            vars.push(format!("a_{}", k));
        }

        // Check for existing variables.
        if manager_ref.with_manager_shared(|manager| manager.num_vars()) != 0 {
            return Err("BDD manager must not contain any variables yet".into());
        }

        // Ensure that the BDD manager is empty.
        manager_ref.with_manager_exclusive(|manager| {
            debug_assert_eq!(
                manager.num_vars(),
                0,
                "A BDD manager can only hold the variables for a single symbolic LTS"
            )
        });

        // Create variables in the BDD manager
        let number_of_vars = vars.len();
        let variables = manager_ref
            .with_manager_exclusive(|manager| -> Result<Range<VarNo>, DuplicateVarName> {
                manager.add_named_vars(vars)
            })
            .map_err(|e| format!("Failed to create variables: {e}"))?;

        assert!(variables.clone().is_sorted(), "Variables must be added in sorted order");
        assert!(
            variables.len() == number_of_vars,
            "Number of created variables does not match"
        );

        // Convert the states to a BDD representation.
        let bits_dd = singleton(storage, &state_bits);
        let all_state_variables_bits: Vec<VarNo> = state_variables_bits.iter().flatten().cloned().collect();
        let states = ldd_to_bdd(storage, manager_ref, lts.states(), &bits_dd, &all_state_variables_bits)?;
        let initial_state = ldd_to_bdd(
            storage,
            manager_ref,
            lts.initial_state(),
            &bits_dd,
            &all_state_variables_bits,
        )?;

        let mut transition_groups = Vec::new();
        for group in lts.transition_groups() {
            // Determine the number of bits used for each layer.
            let mut bits = Vec::new();

            // Determine all the variables used in this relation.
            let mut variables = Vec::new();

            let mut read_variable_indices: Vec<VarNo> = Vec::new();
            let mut write_variable_indices: Vec<VarNo> = Vec::new();

            for (i, state_var_bits) in state_variables_bits.iter().enumerate() {
                if group.read_indices().contains(&(i as u32)) {
                    // The transition group reads this state variable
                    bits.push(state_bits[i]);
                    variables.extend(state_var_bits.iter());
                    read_variable_indices.extend(state_variables_bits[i].iter())
                }

                if group.write_indices().contains(&(i as u32)) {
                    // The transition group writes this state variable
                    bits.push(state_bits[i]);
                    variables.extend(next_state_variables_bits[i].iter());
                    write_variable_indices.extend(next_state_variables_bits[i].iter())
                }
            }

            // Append action label bits (between read and write segments) if present
            if let Some(_action_index) = group.action_label_index() {
                // TODO: This currently assumes that action label bits are at the end.
                variables.extend(action_labels_vars.iter());
            }

            // Append action label bits
            bits.push(action_label_bits);
            debug!(
                "Transition group {:?} uses number of bits {:?}, and variables: {:?}",
                group, bits, variables
            );

            let bits_dd = singleton(storage, &bits);
            let relation_bdd = ldd_to_bdd(storage, manager_ref, group.relation(), &bits_dd, &variables)?;

            let (read_variables, read_variables_bdd) = compute_vars_bdd(manager_ref, &read_variable_indices)?;
            let (write_variables, write_variables_bdd) = compute_vars_bdd(manager_ref, &write_variable_indices)?;

            transition_groups.push(SummandGroupBdd::new(
                relation_bdd,
                read_variable_indices,
                read_variables,
                read_variables_bdd,
                write_variable_indices,
                write_variables,
                write_variables_bdd,
            ));
        }

        // Compute the BDDs representing the state variables and next state variables.
        let all_next_state_variables_bits = next_state_variables_bits
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<VarNo>>();

        debug!("State bits {all_state_variables_bits:?}, and next state bits {all_next_state_variables_bits:?}");

        let (state_variables, state_variables_bdd) = compute_vars_bdd(manager_ref, &all_state_variables_bits)?;
        let (next_state_variables_bdds, next_state_variables_bdd) =
            compute_vars_bdd(manager_ref, &all_next_state_variables_bits)?;
        let (_action_label_variables, action_variables_bdd) = compute_vars_bdd(manager_ref, &action_labels_vars)?;

        info!("Finished conversion.");
        Ok(Self {
            action_variable_indices: action_labels_vars,
            states,
            initial_state,
            transition_groups,
            state_variable_num_of_bits: state_bits,
            state_variable_indices: all_state_variables_bits,
            state_variables_bdd,
            state_variables,
            next_state_variables_bdd,
            next_state_variables: next_state_variables_bdds,
            next_state_variables_indices: all_next_state_variables_bits,
            action_variables_bdd,
            action_labels: lts.action_labels().to_vec(),
            parameter_values: lts.parameter_values().to_vec(),
        })
    }

    /// Returns the BDD representing the set of states.
    pub fn states(&self) -> &BDDFunction {
        &self.states
    }

    /// Returns the BDD representing the set of initial states.
    pub fn initial_state(&self) -> &BDDFunction {
        &self.initial_state
    }

    /// Returns the number of bits used for each state variable.
    pub fn state_variable_num_of_bits(&self) -> &[Value] {
        &self.state_variable_num_of_bits
    }

    /// Returns the BDD variables used to represent the state variables.
    pub fn state_variable_indices(&self) -> &[VarNo] {
        &self.state_variable_indices
    }

    /// Returns the BDDs representing each state variable.
    pub fn state_variables(&self) -> &[BDDFunction] {
        &self.state_variables
    }

    /// Returns the BDD variables used to represent the state variables.
    pub fn state_variables_bdd(&self) -> &BDDSupport {
        &self.state_variables_bdd
    }

    /// Returns the BDD variables used to represent the state variables.
    pub fn next_state_variables_bdd(&self) -> &BDDSupport {
        &self.next_state_variables_bdd
    }

    /// Returns the variable numbers used to represent the next state variables.
    pub fn next_state_variable_indices(&self) -> &[VarNo] {
        &self.next_state_variables_indices
    }

    /// Returns the variable numbers used to represent the next state variables.
    pub fn next_state_variables(&self) -> &[BDDFunction] {
        &self.next_state_variables
    }

    /// Returns the transition groups representing the disjunctive transition relation.
    pub fn transition_groups(&self) -> &Vec<SummandGroupBdd> {
        &self.transition_groups
    }

    /// Returns the action variable indices used to represent the action labels.
    pub fn action_variable_indices(&self) -> &[VarNo] {
        &self.action_variable_indices
    }

    /// Returns the BDD representing the action label variables.
    pub fn action_variables_bdd(&self) -> &BDDSupport {
        &self.action_variables_bdd
    }

    /// Returns the action labels of the LTS.
    pub fn action_labels(&self) -> &[String] {
        &self.action_labels
    }

    /// Returns the possible values for each process parameter.
    pub fn parameter_values(&self) -> &[Vec<DataExpression>] {
        &self.parameter_values
    }
}

pub struct SummandGroupBdd {
    /// The BDD representing the transition relation for this summand group.
    relation: BDDFunction,

    /// The indices of the read variables for this summand group.
    read_variable_indices: Vec<VarNo>,

    /// The BDDs representing the read variables for this summand group.
    read_variables: Vec<BDDFunction>,

    /// The BDD representing all the read variables for this summand group.
    read_variables_bdd: BDDFunction,

    /// The indices of the write variables for this summand group.
    write_variable_indices: Vec<VarNo>,

    /// The BDDs representing the write variables for this summand group.
    write_variables: Vec<BDDFunction>,

    /// The BDD representing all the write variables for this summand group.
    write_variables_bdd: BDDFunction,
}

impl SummandGroupBdd {
    /// Creates a new summand group with the given transition relation.
    pub fn new(
        relation: BDDFunction,
        read_variable_indices: Vec<VarNo>,
        read_variables: Vec<BDDFunction>,
        read_variables_bdd: BDDFunction,
        write_variable_indices: Vec<VarNo>,
        write_variables: Vec<BDDFunction>,
        write_variables_bdd: BDDFunction,
    ) -> Self {
        Self {
            relation,
            read_variable_indices,
            read_variables,
            read_variables_bdd,
            write_variable_indices,
            write_variables,
            write_variables_bdd,
        }
    }

    /// Returns the BDD representing the transition relation for this summand group.
    pub fn relation(&self) -> &BDDFunction {
        &self.relation
    }

    /// Returns the indices of the read variables for this summand group.
    pub fn read_variable_indices(&self) -> &Vec<VarNo> {
        &self.read_variable_indices
    }

    /// Returns the BDDs representing the read variables for this summand group.
    pub fn read_variables(&self) -> &Vec<BDDFunction> {
        &self.read_variables
    }

    /// Returns the BDD representing all the read variables for this summand group.
    pub fn read_variables_bdd(&self) -> &BDDFunction {
        &self.read_variables_bdd
    }

    /// Returns the indices of the write variables for this summand group.
    pub fn write_variable_indices(&self) -> &Vec<VarNo> {
        &self.write_variable_indices
    }

    /// Returns the BDDs representing the write variables for this summand group.
    pub fn write_variables_bdds(&self) -> &Vec<BDDFunction> {
        &self.write_variables
    }

    /// Returns the BDD representing all the write variables for this summand group.
    pub fn write_variables_bdd(&self) -> &BDDFunction {
        &self.write_variables_bdd
    }
}

/// Creates BDD of variables for the given variable numbers.
fn compute_vars_bdd(manager_ref: &BDDManagerRef, vars: &[VarNo]) -> Result<(Vec<BDDFunction>, BDDFunction), MercError> {
    manager_ref.with_manager_shared(|manager| -> Result<_, MercError> {
        let mut vector = Vec::new();
        let mut bdd: BDDFunction = BDDFunction::t(manager);

        for var in vars {
            let var = BDDFunction::var(manager, *var)?;
            vector.push(var.clone());
            bdd = bdd.and(&var)?;
        }

        Ok((vector, bdd))
    })
}

#[cfg(test)]
mod tests {
    use merc_ldd::Storage;
    use merc_utilities::test_logger;

    use crate::SymbolicLtsBdd;
    use crate::read_symbolic_lts;

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_symbolic_lts_bdd() {
        test_logger();

        let input = include_bytes!("../../../examples/lts/abp.sym");

        let mut storage = Storage::new();
        let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);
        let symbolic_lts = read_symbolic_lts(&mut storage, &input[..]).unwrap();

        SymbolicLtsBdd::from_symbolic_lts(&mut storage, &manager_ref, &symbolic_lts).unwrap();
    }
}
