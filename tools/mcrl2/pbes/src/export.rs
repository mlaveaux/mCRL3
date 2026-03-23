use std::collections::HashMap;
use std::io::Write;

use log::debug;
use mcrl2::Pbes;
use mcrl2::free_variables_data_expression;
use mcrl2::is_pbes_propositional_variable_instantiation;
use mcrl2::DataExpressionRef;
use mcrl2::DataVariable;
use mcrl2::PbesExpressionRef;
use mcrl2::PbesExpressionVisitor;
use mcrl2::PbesPropositionalVariableInstantiation;
use mcrl2::SrfEquation;
use mcrl2::SrfSummand;
use merc_utilities::MercError;

use crate::symmetry::SymmetryAlgorithm;
use crate::symmetry::variable_index;

/// Exports the information from the given PBES and its stategraph output in
/// JSON format to the given writer.
///
/// # Details
///
/// The output contains the identification of the control flow and data
/// parameters, and a mapping from clauses of each equation to the parameters
/// that are used-for, used-in and changed-by them.
pub fn export<W: Write>(write: &mut W, pbes: &Pbes) -> Result<(), MercError> {
    let symmetries = SymmetryAlgorithm::new(&pbes, false)?;

    let parameters = if let Some(equation) = symmetries.srf_pbes().equations().first() {
        equation.variable().parameters().to_vec()
    } else {
        // There are no equations, so no parameters.
        Vec::new()
    };

    // Figure out the control flow parameters.
    let all_control_flow_parameters = symmetries.state_graph()
        .control_flow_graphs()
        .iter()
        .map(variable_index)
        .collect::<Vec<_>>();

    // Figure out the data parameters by filtering out the control flow parameters.
    let data_parameters: Vec<usize> = parameters
        .iter()
        .enumerate()
        .filter_map(|(index, _param)| {
            if all_control_flow_parameters.contains(&index) {
                // Skip control flow parameters.
                None
            } else {
                Some(index)
            }
        })
        .collect();

    let mut mapping = HashMap::from_iter(
        parameters
            .iter()
            .enumerate()
            .map(|(index, param)| (index.to_string(), param.name().to_string())),
    );

    let mut clauses = HashMap::new();
    let mut unique_index = mapping.len();

    // Keep track of the variable mappings derived from the SRF pbes.
    let mut uf = HashMap::new();
    let mut ui = HashMap::new();
    let mut cb = HashMap::new();

    for equation in symmetries.srf_pbes().equations() {
        for (clause_index, clause) in equation.summands().iter().enumerate() {
            clauses.insert((equation.variable().name(), clause_index), unique_index);
            mapping.insert(
                unique_index.to_string(),
                format!("{}[{}]", equation.variable().name(), clause_index),
            );

            // Compute used-for and map the variables back to their position in the variables.
            let mut used_for_indices: Vec<usize> = used_for(clause)
                .iter()
                .map(|var| parameters.iter().position(|param| param.name() == var.name()).expect("variable must exist in unified parameters"))
                .collect();
            used_for_indices.sort_unstable();
            used_for_indices.dedup();

            uf.insert(unique_index.to_string(), used_for_indices);

            // Compute used-in and map the variables back to their position in the variables.
            ui.insert(
                unique_index.to_string(),
                used_in(equation, clause)
                    .iter()
                    .map(|var| parameters.iter().position(|param| param.name() == var.name()).expect("variable must exist in unified parameters"))
                    .collect(),
            );

            // Compute changed-by and map the variables back to their position in the variables.
            cb.insert(
                unique_index.to_string(),
                changed_by(equation, clause)
                    .iter()
                    .map(|var| parameters.iter().position(|param| param.name() == var.name()).expect("variable must exist in unified parameters"))
                    .collect(),
            );

            unique_index += 1;
        }
    }

    debug!("Clauses {:?}", clauses);

    // Keep track of the source or target, and copy variables for each clause.
    let mut src_tgt = HashMap::new();
    let mut copy = HashMap::new();

    for equation in symmetries.state_graph().equations() {
        for (clause_index, predicate) in equation.predicate_variables().iter().enumerate() {
            let clause_index = *clauses
                .get(&(equation.variable().name(), clause_index))
                .expect("Clause must have been added before");

            // Update the index for the source or target variables.
            for variable in predicate.source().iter().chain(predicate.target().iter()) {
                if data_parameters.contains(&variable) {
                    // This variable is a data parameter, so we are not interested in it for the source and target functions.
                    continue;
                }

                let vector = src_tgt
                    .entry(variable.to_string())
                    .or_insert_with(Vec::new);

                if !vector.contains(&clause_index) {
                    vector.push(clause_index);
                }
            }

            for variable in predicate.copy().iter() {
                if data_parameters.contains(&variable) {
                    // This variable is a data parameter, we not interested in it for the copy function.
                    continue;
                }

                let vector = copy.entry(variable.to_string())
                    .or_insert_with(Vec::new);

                if !vector.contains(&clause_index) {
                    vector.push(clause_index);
                }
            }
        }
    }

    let mut cliques = HashMap::new();
    for (clique_index, clique) in symmetries.cliques().iter().enumerate() {
        for parameter_index in clique.iter() {
            cliques.insert(all_control_flow_parameters[*parameter_index], format!("clique{}", clique_index));
        }
        
    }

    let output = Output {
        mapping,
        cliques,

        pars: (0..parameters.len()).collect(),

        clauses: clauses.values().cloned().collect(),

        cfp: all_control_flow_parameters.clone(),

        dp: data_parameters.clone(),

        uf,
        ui,
        cb,

        src_tgt,
        copy,
    };

    serde_json::to_writer_pretty(write, &output)?;
    Ok(())
}

/// Returns the data variables that are used for the given clause, i.e. the data
/// variables that occur in the condition of the clause.
///
/// Given clause `j`, `used_for(j)` hold iff `d_k` in `fv(f_j)` for some data variable
/// `d_k`.
fn used_for(clause: &SrfSummand) -> Vec<DataVariable> {
    free_variables_pbes_expression(&clause.condition().copy())
}

/// returns the data variables that are used in a given clause, i.e., there is
/// an update that contains the variable.
///
/// A data variable `d_k` is used in a clause `j` iff there is some `l <= n` such that `d_k` in `fv(g_j,l(d,e_j))` where if `X = X_j` then `k != l`.
fn used_in(equation: &SrfEquation, clause: &SrfSummand) -> Vec<DataVariable> {
    debug_assert!(
        is_pbes_propositional_variable_instantiation(&clause.variable()),
        "The clause variable must always be a PVI"
    );

    let pvi: PbesPropositionalVariableInstantiation = clause.variable().into();

    // If `X != X_j` then we need to check all variables that are used in the update of the clause.
    let must_be_different = pvi.name() == equation.variable().name().copy();

    // We assume that all equations have the same parameters, so we can just use the parameters of the given equation.
    let params = equation.variable().parameters();
    let args = pvi.arguments();
    debug_assert_eq!(
        params.len(),
        args.iter().count(),
        "used_in: parameters and arguments must have the same length"
    );

    let mut result = Vec::new();
    for ((var_index, variable), (update_index, update)) in params
        .iter()
        .enumerate()
        .zip(args.iter().enumerate())
    {
        if must_be_different && var_index == update_index {
            // This variable is not used in the clause, since it is updated to itself.
            continue;
        }

        if free_variables_data_expression(&update.copy()).contains(&variable) {
            // This variable is used in the clause.
            result.push(variable);
        }
    }

    result
}

/// Returns the data variables that are changed by a given clause, i.e., there is an update that contains the variable and the variable is updated to a different value.
///
/// A data variable `d_k` is changed by a clause `j` if `X = X_j` and `d_k` != `g_j,k(d, e_j)`.
fn changed_by(equation: &SrfEquation, clause: &SrfSummand) -> Vec<DataVariable> {
    debug_assert!(
        is_pbes_propositional_variable_instantiation(&clause.variable()),
        "The clause variable must always be a PVI"
    );

    let pvi: PbesPropositionalVariableInstantiation = clause.variable().into();

    let mut result = Vec::new();
    if pvi.name() == equation.variable().name().copy() {
        // X = X_j, so we need to check which variables are changed by the clause.
        let params = equation.variable().parameters();
        let args = pvi.arguments();
        debug_assert_eq!(
            params.len(),
            args.iter().count(),
            "changed_by: parameters and arguments must have the same length"
        );

        for (variable, update) in params.iter().zip(args.iter()) {
            if Into::<DataExpressionRef<'_>>::into(variable.copy()) != update.copy() {
                // This variable is changed by the clause.
                result.push(variable);
            }
        }
    }

    result
}

/// Returns all the data variables occurring in the given PBES expression.
fn free_variables_pbes_expression(expr: &PbesExpressionRef<'_>) -> Vec<DataVariable> {
    let mut result = Vec::new();

    /// Local struct that is used to collect data variable occurrences.
    struct FreeVariableOccurrences<'a> {
        result: &'a mut Vec<DataVariable>,
    }

    impl PbesExpressionVisitor for FreeVariableOccurrences<'_> {
        fn visit_data_expression(&mut self, expr: &mcrl2::DataExpressionRef<'_>) -> Option<mcrl2::DataExpression> {
            self.result.extend(free_variables_data_expression(expr));
            None
        }
    }

    let mut occurrences = FreeVariableOccurrences { result: &mut result };
    occurrences.visit(expr);
    result
}

/// The output gathered to be exported in JSON format.
#[derive(serde::Serialize)]
struct Output {
    /// Stores the mapping from indices to parameters names.
    mapping: HashMap<String, String>,

    /// Stores the indices of parameters (used in the uf, ui and cb fields).
    pars: Vec<usize>,

    /// Stores indices for clauses
    clauses: Vec<usize>,

    /// The control flow parameter indices in `mapping`.
    cfp: Vec<usize>,

    /// The data parameter indices in `mapping`.
    dp: Vec<usize>,

    /// Maps from clause indices to the parameter indices that are used for.
    uf: HashMap<String, Vec<usize>>,

    /// Maps from clause indices to the parameter indices that are used in.
    ui: HashMap<String, Vec<usize>>,

    /// Maps from clause indices to the parameter indices that are changed by the clause.
    cb: HashMap<String, Vec<usize>>,

    /// Maps from parameter indices (as strings) to the clause indices where they occur as source or target variables.
    src_tgt: HashMap<String, Vec<usize>>,

    /// Maps from parameter indices (as strings) to the clause indices where they occur as copy variables.
    copy: HashMap<String, Vec<usize>>,

    /// a mapping from control flow parameter indices to the clique they belong to.
    cliques: HashMap<usize, String>,
}
