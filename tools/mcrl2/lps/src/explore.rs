use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;
use log::debug;
use log::trace;

use mcrl2::_aterm;
use mcrl2::ATerm;
use mcrl2::ATermList;
use mcrl2::DataExpression;
use mcrl2::DataExpressionRef;
use mcrl2::DataVariable;
use mcrl2::LearnSuccessorsContext;
use mcrl2::LinearProcessSpecification;
use mcrl2::LinearSummand;
use mcrl2::free_variables_data_expression;
use mcrl2::preprocess;
use merc_collections::IndexedSet;

use merc_ldd::Ldd;
use merc_ldd::Storage;
use merc_ldd::Value;
use merc_ldd::compute_meta;
use merc_ldd::compute_proj;
use merc_ldd::iterators::for_each_mut;
use merc_ldd::project;
use merc_ldd::singleton;
use merc_ldd::union;
use merc_symbolic::SymbolicLTS;
use merc_symbolic::TransitionGroup;
use merc_symbolic::reachability;
use merc_utilities::MercError;

use merc_utilities::Timing;

/// Explore the linear process specification using symbolic reachability.
pub fn explore_lps(storage: &mut Storage, lps: &LinearProcessSpecification, timing: &Timing) -> Result<Ldd, MercError> {
    let mut symbolic_lts = SymbolicLinearProcessSpecification::new(storage, lps)?;

    debug!("{symbolic_lts:?}");

    reachability(storage, &mut symbolic_lts, timing)
}

/// This struct provides a [merc_symbolic::SymbolicLTS] interface to a [mcrl2::LinearProcessSpecification].
struct SymbolicLinearProcessSpecification {
    /// The underlying linear process specification.
    _lps: LinearProcessSpecification,

    /// The symbolic summands of the LPS, which are obtained by preprocessing the LPS.
    symbolic_summands: Vec<SymbolicSummand>,

    /// The actions that are discovered for this summand.
    action_labels: Vec<String>,

    /// Information shared between all summands and the LPS.
    _shared: Rc<Shared>,

    /// The initial state of the LPS.
    initial_state: Ldd,
}

impl SymbolicLinearProcessSpecification {
    pub fn new(storage: &mut Storage, lps: &LinearProcessSpecification) -> Result<Self, MercError> {
        let lps = preprocess(lps)?;

        let parameters = lps.parameters();
        let num_parameters = parameters.len();

        let shared = Rc::new(Shared {
            context: LearnSuccessorsContext::new(&lps),
            mapping: RefCell::new((0..num_parameters).map(|_| IndexedSet::new()).collect()),
        });

        let mut symbolic_summands = Vec::new();
        for index in 0..lps.num_summands() {
            symbolic_summands.push(SymbolicSummand::new(
                storage,
                &lps.action_summand(index)?,
                &parameters,
                Rc::clone(&shared),
            ));
        }

        let initial_state_vector = lps
            .initial_process()
            .expressions()
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let (index, _) = shared.mapping.borrow_mut()[i].insert(param.clone());
                *index as u32
            })
            .collect::<Vec<u32>>();

        debug_assert_eq!(
            initial_state_vector.len(),
            num_parameters,
            "Initial state vector length must match number of parameters"
        );

        let initial_state = singleton(storage, &initial_state_vector);

        Ok(SymbolicLinearProcessSpecification {
            _lps: lps,
            symbolic_summands,
            _shared: shared,
            initial_state,
            action_labels: Vec::new(),
        })
    }
}

/// Information that is shared between all [SymbolicSummand]s.
struct Shared {
    /// Context used by mCRL2 to perform the enumeration. Uses interior
    /// mutability so that the enumeration callback can access [Shared::mapping]
    /// while a call into the context is in progress.
    context: LearnSuccessorsContext,

    /// Stores a bidirectional mapping between data expressions and indices.
    mapping: RefCell<Vec<IndexedSet<DataExpression>>>,
}

/// Represents a symbolic summand of a [mcrl2::LinearProcessSpecification].
struct SymbolicSummand {
    /// The LDD encoding the projection of the state space on the read variables of this summand.
    project_ldd: Ldd,

    /// The relation encoding the transition relation of this summand.
    relation: Ldd,

    /// The parameters that are read by this summand.
    read_parameters: Vec<*const _aterm>,

    /// The indices of the parameters that are read by this summand, which is
    /// used to determine the projection of the state space for this summand.
    read_indices: Vec<Value>,

    /// The positions of `read_indices` in the short vector.
    read_positions: Vec<Value>,

    /// The indices of the parameters that are written by this summand.
    write_indices: Vec<Value>,

    /// The positions of `write_indices` in the short vector.
    write_positions: Vec<Value>,

    /// The meta information for this summand, which is required by the relational product.
    meta: Ldd,

    /// The condition of this summand.
    condition: DataExpression,

    /// The summation variables of this summand.
    summation_variables: ATermList<DataVariable>,

    /// Only the non-identity assignments (write parameters) of this summand.
    write_assignments: ATermList<ATerm>,

    /// The shared context containing the rewriter/enumerator.
    shared: Rc<Shared>,
}

impl SymbolicSummand {
    /// Extract the required information from the given action summand that is required for symbolic exploration.
    pub fn new(
        storage: &mut Storage,
        summand: &LinearSummand,
        parameters: &ATermList<DataVariable>,
        shared: Rc<Shared>,
    ) -> Self {
        // Collect free variables from the condition.
        let mut read_vars = free_variables_data_expression(&summand.condition().copy());
        let parameters = parameters.to_vec();

        // Collect free variables from the update expressions and identify write assignments.
        let mut write_vars = Vec::new();
        let mut write_assignments = Vec::new();

        for assignment in summand.assignments().iter() {
            let lhs: DataVariable = assignment.arg(0).protect().into();
            let rhs = assignment.arg(1);

            // The parameter is read if its RHS references any process parameter.
            let rhs_vars = free_variables_data_expression(&rhs.copy().into());
            read_vars.extend(rhs_vars);

            // The parameter is written if the RHS differs from the LHS variable.
            if DataExpressionRef::from(lhs.copy()) != DataExpressionRef::from(rhs.copy()) {
                write_vars.push(lhs);
                write_assignments.push(assignment.protect());
            }
        }

        // Convert write assignments to an aterm list.
        let write_assignments = ATermList::from_double_iter(write_assignments.into_iter());

        // Convert read variables to parameter indices.
        let read_indices: Vec<u32> = parameters
            .iter()
            .enumerate()
            .filter(|(_, param)| read_vars.contains(param))
            .map(|(i, _)| i as u32)
            .collect();

        let read_parameters: Vec<*const _aterm> = read_indices
            .iter()
            .map(|&index| parameters[index as usize].address())
            .collect();

        // Convert write variables to parameter indices.
        let write_indices: Vec<u32> = parameters
            .iter()
            .enumerate()
            .filter(|(_, param)| write_vars.contains(param))
            .map(|(i, _)| i as u32)
            .collect();

        // Store the condition, summation variables, and assignments for enumeration.
        let condition: DataExpression = summand.condition();
        let summation_variables: ATermList<DataVariable> = summand.summation_variables();

        let relation = storage.protect(storage.empty_set());
        let project_ldd = compute_proj(storage, &read_indices);

        let (meta, read_positions, write_positions) = compute_meta(storage, &read_indices, &write_indices);

        debug_assert_eq!(
            read_indices.len(),
            read_parameters.len(),
            "Number of read indices must match number of read parameters"
        );
        debug_assert_eq!(
            read_indices.len(),
            read_positions.len(),
            "Number of read indices must match number of read positions"
        );
        debug_assert_eq!(
            write_indices.len(),
            write_positions.len(),
            "Number of write indices must match number of write positions"
        );
        debug_assert_eq!(
            write_indices.len(),
            write_assignments.iter().count(),
            "Number of write indices must match number of write assignments"
        );
        debug_assert!(read_indices.iter().is_sorted(), "Read indices must be strictly sorted");
        debug_assert!(
            write_indices.iter().is_sorted(),
            "Write indices must be strictly sorted"
        );

        Self {
            project_ldd,
            relation,
            read_indices,
            write_indices,
            meta,
            read_positions,
            write_positions,
            condition,
            summation_variables,
            write_assignments,
            shared,
            read_parameters,
        }
    }
}

impl SymbolicLTS for SymbolicLinearProcessSpecification {
    fn states(&self) -> &Ldd {
        unreachable!("The SymbolicLTS interface can only be explored");
    }

    fn initial_state(&self) -> &Ldd {
        &self.initial_state
    }

    fn transition_groups(&self) -> &[impl TransitionGroup] {
        &self.symbolic_summands
    }

    fn transition_groups_mut(&mut self) -> &mut [impl TransitionGroup] {
        &mut self.symbolic_summands
    }

    fn action_labels(&self) -> &[String] {
        &self.action_labels
    }

    fn parameter_values(&self) -> &[Vec<merc_data::DataExpression>] {
        &[]
    }
}

impl TransitionGroup for SymbolicSummand {
    fn relation(&self) -> &Ldd {
        &self.relation
    }

    fn read_indices(&self) -> &[u32] {
        &self.read_indices
    }

    fn write_indices(&self) -> &[u32] {
        &self.write_indices
    }

    fn action_label_index(&self) -> Option<usize> {
        None
    }

    fn meta(&self) -> &Ldd {
        &self.meta
    }

    fn learn_successors(&mut self, storage: &mut Storage, todo: &Ldd) -> Result<(), MercError> {
        let proj = project(storage, todo, &self.project_ldd);

        // Reused across short states to avoid per-iteration allocation.
        let mut read_values: Vec<*const _aterm> = Vec::with_capacity(self.read_indices.len());
        let mut interleaved_values: Vec<Value> = vec![0; self.read_indices.len() + self.write_indices.len()];

        for_each_mut(storage, &proj, |storage, short_state| {
            debug_assert_eq!(
                short_state.len(),
                self.read_indices.len(),
                "Projected state must have one value per read index"
            );

            // Convert the LDD state values back to aterm pointers for the read parameters.
            read_values.clear();
            {
                let mapping = self.shared.mapping.borrow();
                for (index, &val) in short_state.iter().enumerate() {
                    read_values.push(
                        mapping[self.read_indices[index] as usize]
                            .get_by_index(val as usize)
                            .expect("The value should be in the mapping")
                            .address(),
                    );
                }
            }

            debug_assert_eq!(
                read_values.len(),
                self.read_parameters.len(),
                "Number of read values must match number of read parameters"
            );

            for (offset, value) in self.read_positions.iter().zip(short_state.iter()) {
                interleaved_values[*offset as usize] = *value;
            }

            self.shared.context.enumerate_raw(
                &self.condition,
                &self.summation_variables,
                &self.write_assignments,
                &self.read_parameters,
                &read_values,
                |values: &[*const _aterm]| {
                    debug_assert_eq!(
                        values.len(),
                        self.write_indices.len(),
                        "Enumerated values must match number of write indices"
                    );

                    {
                        let mut mapping = self.shared.mapping.borrow_mut();
                        for (&offset, (i, value)) in self.write_positions.iter().zip(values.iter().enumerate()) {
                            interleaved_values[offset as usize] = *mapping[self.write_indices[i] as usize]
                                .insert(DataExpression::from(ATerm::from_ptr(*value)))
                                .0 as Value;
                        }
                    }

                    trace!(
                        "[{}] -> [{}]",
                        short_state.iter().join(", "),
                        self.write_positions
                            .iter()
                            .map(|&pos| interleaved_values[pos as usize])
                            .join(", ")
                    );

                    let cube = singleton(storage, &interleaved_values);
                    self.relation = union(storage, &self.relation, &cube);
                },
            );
        });

        Ok(())
    }
}

impl fmt::Debug for SymbolicLinearProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SymbolicLinearProcessSpecification:")?;

        writeln!(f, "  Parameters:")?;
        for (i, param) in self._lps.parameters().iter().enumerate() {
            writeln!(f, "    {:?}: {:?}", i, param)?;
        }

        writeln!(f, "  Summands:")?;
        for (i, summand) in self.symbolic_summands.iter().enumerate() {
            writeln!(f, "    {:?}: {:?}", i, summand)?;
        }
        Ok(())
    }
}

impl fmt::Debug for SymbolicSummand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} -> {}",
            self.condition.pretty_print(),
            self.write_assignments
                .iter()
                .format_with(", ", |assignment, f| f(&format_args!(
                    "{} := {}",
                    DataExpressionRef::from(assignment.arg(0)).pretty_print(),
                    DataExpressionRef::from(assignment.arg(1)).pretty_print()
                )))
        )?;

        writeln!(f, "\t\tread indices: {:?}", self.read_indices)?;
        writeln!(f, "\t\twrite indices: {:?}", self.write_indices)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use mcrl2::read_lps;
    use merc_ldd::Storage;
    use merc_ldd::len;
    use merc_utilities::Timing;

    use super::explore_lps;

    #[test]
    fn test_mcrl2_explore_symbolic_abp() {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");

        let temp_dir = tempfile::tempdir().unwrap();
        let lps_path = temp_dir.path().join("abp.lps");

        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/mCRL2/academic/abp/abp.mcrl2");

        // Run mcrl22lps on the ABP example to get an LPS file.
        let status = Command::new(&mcrl22lps)
            .arg(&spec_path)
            .arg(&lps_path)
            .status()
            .expect("Failed to execute mcrl22lps");
        assert!(status.success(), "mcrl22lps failed with status: {status}");

        let lps = read_lps(lps_path.to_str().expect("LPS path is valid UTF-8")).expect("Failed to read LPS");

        let mut storage = Storage::new();
        let timing = Timing::new();

        let states = explore_lps(&mut storage, &lps, &timing).expect("Failed to explore LPS");
        let num_of_states = len(&mut storage, &states);

        assert_eq!(
            num_of_states, 74,
            "ABP should have 74 reachable states (see examples/lts/abp.aut)"
        );
    }
}
