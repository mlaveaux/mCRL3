use std::ops::Range;

use itertools::Itertools;
use log::debug;
use log::info;
use oxidd::BooleanFunction;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::VarNo;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::error::DuplicateVarName;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;
use oxidd::util::OutOfMemory;

use merc_data::DataExpression;
use merc_utilities::MercError;
use oxidd::ldd::LDDFunction;

use crate::SymbolicLTS;
use crate::TransitionGroup;
use crate::compute_bits;
use crate::compute_highest;
use crate::ldd_to_bdd;
use crate::required_bits;

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

    /// The variable numbers used to represent the next state variables.
    next_state_variables_indices: Vec<VarNo>,

    /// The action variable indices used to represent the action labels.
    action_variable_indices: Vec<VarNo>,

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
    pub fn from_symbolic_lts<L: SymbolicLTS>(
        manager: &LDDManagerRef,
        manager_bdd: &BDDManagerRef,
        lts: &L,
    ) -> Result<Self, MercError> {
        info!("Converting symbolic LTS from LDD to BDD representation...");

        // Determine the highest values for every state variable.
        let mut state_highest = compute_highest(manager, lts.states());

        let mut action_label_highest = 0u32;
        for group in lts.transition_groups() {
            let highest = compute_highest(manager, group.relation());

            // Deal with the special empty case.
            if highest.is_empty() {
                continue;
            }

            action_label_highest =
                action_label_highest.max(highest[group.action_label_index().ok_or("Action label index not found")?]);

            // Also consider the highest values read or written for the state
            // variables in the relation. LDD levels follow the sorted merge of
            // read and write indices (read before write at the same variable).
            for (ldd_level, var) in group
                .read_indices()
                .iter()
                .merge(group.write_indices().iter())
                .enumerate()
            {
                let var = *var as usize;
                state_highest[var] = state_highest[var].max(highest[ldd_level]);
            }
        }

        let state_bits = compute_bits(&state_highest);
        debug!("Determined number of bits for state variables: {:?}", state_bits);

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
        if manager_bdd.with_manager_shared(|manager| manager.num_vars()) != 0 {
            return Err("BDD manager must not contain any variables yet".into());
        }

        // Ensure that the BDD manager is empty.
        manager_bdd.with_manager_exclusive(|manager| {
            debug_assert_eq!(
                manager.num_vars(),
                0,
                "A BDD manager can only hold the variables for a single symbolic LTS"
            )
        });

        // Create variables in the BDD manager
        let number_of_vars = vars.len();
        let variables = manager_bdd
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
        let bits_dd = LDDFunction::singleton(manager, &state_bits)?;
        let all_state_variables_bits: Vec<VarNo> = state_variables_bits.iter().flatten().cloned().collect();
        let states = ldd_to_bdd(manager, manager_bdd, lts.states(), &bits_dd, &all_state_variables_bits)?;
        let initial_state = ldd_to_bdd(
            manager,
            manager_bdd,
            lts.initial_state(),
            &bits_dd,
            &all_state_variables_bits,
        )?;

        let mut transition_groups = Vec::new();
        for (index, group) in lts.transition_groups().iter().enumerate() {
            // Determine the number of bits used for each layer.
            let mut relation_bits = Vec::new();

            // Determine all the variables used in this relation.
            let mut variables = Vec::new();

            let mut read_variable_indices: Vec<VarNo> = Vec::new();
            let mut write_variable_indices: Vec<VarNo> = Vec::new();

            for (var, bits) in state_bits.iter().enumerate() {
                if group.read_indices().contains(&(var as VarNo)) {
                    // The transition group reads this state variable
                    relation_bits.push(*bits);
                    variables.extend(state_variables_bits[var].iter());
                    read_variable_indices.extend(state_variables_bits[var].iter())
                }

                if group.write_indices().contains(&(var as VarNo)) {
                    // The transition group writes this state variable
                    relation_bits.push(*bits);
                    variables.extend(next_state_variables_bits[var].iter());
                    write_variable_indices.extend(next_state_variables_bits[var].iter())
                }
            }

            // Append action label bits (between read and write segments) if present
            if let Some(_action_index) = group.action_label_index() {
                // TODO: This currently assumes that action label bits are at the end.
                variables.extend(action_labels_vars.iter());
            }

            // Append action label bits
            relation_bits.push(action_label_bits);
            debug!(
                "Transition group {}, {:?} uses number of bits {:?}, and variables: {:?}",
                index, group, relation_bits, variables
            );

            let bits_dd = LDDFunction::singleton(manager, &relation_bits)?;
            let relation_bdd = ldd_to_bdd(manager, manager_bdd, group.relation(), &bits_dd, &variables)?;

            transition_groups.push(SummandGroupBdd::new(
                relation_bdd,
                read_variable_indices,
                write_variable_indices,
            ));
        }

        // Compute the BDDs representing the state variables and next state variables.
        let all_next_state_variables_bits = next_state_variables_bits
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<VarNo>>();

        debug!("State bits {all_state_variables_bits:?}, and next state bits {all_next_state_variables_bits:?}");

        info!("Finished conversion.");
        Ok(Self {
            action_variable_indices: action_labels_vars,
            states,
            initial_state,
            transition_groups,
            state_variable_num_of_bits: state_bits,
            state_variable_indices: all_state_variables_bits,
            next_state_variables_indices: all_next_state_variables_bits,
            action_labels: lts.action_labels().to_vec(),
            parameter_values: lts.parameter_values().to_vec(),
        })
    }

    /// Constructs a new symbolic LTS with the given transition groups.
    pub fn with_transition_groups(lts: &Self, transition_groups: Vec<SummandGroupBdd>) -> Self {
        Self {
            states: lts.states.clone(),
            initial_state: lts.initial_state.clone(),
            state_variable_num_of_bits: lts.state_variable_num_of_bits.clone(),
            state_variable_indices: lts.state_variable_indices.clone(),
            next_state_variables_indices: lts.next_state_variables_indices.clone(),
            action_variable_indices: lts.action_variable_indices.clone(),
            action_labels: lts.action_labels.clone(),
            parameter_values: lts.parameter_values.clone(),
            transition_groups,
        }
    }

    /// Constructs a quotient symbolic LTS that reuses the action labels of `lts`
    /// but encodes states and transitions over fresh state and next-state variables
    /// (typically block-index variables produced by sigref).
    pub fn with_quotient_state(
        lts: &Self,
        states: BDDFunction,
        initial_state: BDDFunction,
        transition_groups: Vec<SummandGroupBdd>,
        state_variable_indices: Vec<VarNo>,
        next_state_variables_indices: Vec<VarNo>,
        state_variable_num_of_bits: Vec<Value>,
    ) -> Self {
        Self {
            states,
            initial_state,
            transition_groups,
            state_variable_num_of_bits,
            state_variable_indices,
            next_state_variables_indices,
            action_variable_indices: lts.action_variable_indices.clone(),
            action_labels: lts.action_labels.clone(),
            parameter_values: Vec::new(),
        }
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
    pub fn state_variables(&self) -> &[VarNo] {
        &self.state_variable_indices
    }

    /// Returns the variable numbers used to represent the next state variables.
    pub fn next_state_variables(&self) -> &[VarNo] {
        &self.next_state_variables_indices
    }

    /// Returns the transition groups representing the disjunctive transition relation.
    pub fn transition_groups(&self) -> &[SummandGroupBdd] {
        &self.transition_groups
    }

    /// Returns the action variable indices used to represent the action labels.
    pub fn action_variables(&self) -> &[VarNo] {
        &self.action_variable_indices
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
    read_variables: Vec<VarNo>,

    /// The indices of the write variables for this summand group.
    write_variables: Vec<VarNo>,
}

impl SummandGroupBdd {
    /// Creates a new summand group with the given transition relation.
    ///
    /// Read variables are current state variables, and write variables are next state variables.
    pub fn new(relation: BDDFunction, read_variables: Vec<VarNo>, write_variables: Vec<VarNo>) -> Self {
        Self {
            relation,
            read_variables,
            write_variables,
        }
    }

    /// Returns the BDD representing the transition relation for this summand group.
    pub fn relation(&self) -> &BDDFunction {
        &self.relation
    }

    /// Returns the indices of the read variables for this summand group.
    pub fn read_variables(&self) -> &[VarNo] {
        &self.read_variables
    }

    /// Returns the indices of the write variables for this summand group.
    pub fn write_variables(&self) -> &[VarNo] {
        &self.write_variables
    }
}

/// Creates BDD of variables for the given variable numbers.
pub fn compute_vars_bdd(
    manager_ref: &BDDManagerRef,
    vars: &[VarNo],
) -> Result<(Vec<BDDFunction>, BDDFunction), OutOfMemory> {
    manager_ref.with_manager_shared(|manager| -> Result<_, OutOfMemory> {
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
    use merc_utilities::test_logger;

    use crate::SymbolicLtsBdd;
    use crate::read_symbolic_lts;

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_from_symbolic_lts_bdd_abp() {
        test_logger();

        let input = include_bytes!("../../../../examples/lts/abp.sym");

        let ldd_manager = oxidd::ldd::new_manager(2048, 1024, 1);
        let bdd_manager = oxidd::bdd::new_manager(2048, 1024, 1);
        let symbolic_lts = read_symbolic_lts(&ldd_manager, &input[..]).unwrap();

        // This only tests that the conversion does not panic.
        SymbolicLtsBdd::from_symbolic_lts(&ldd_manager, &bdd_manager, &symbolic_lts).unwrap();
    }
}
