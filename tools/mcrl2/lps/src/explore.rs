use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;
use log::debug;
use log::trace;
use streaming_iterator::StreamingIterator;

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
use merc_ldd::iterators::iter;
use merc_ldd::project;
use merc_ldd::singleton;
use merc_ldd::union;
use merc_symbolic::SymbolicLTS;
use merc_symbolic::TransitionGroup;
use merc_symbolic::reachability;
use merc_utilities::MercError;

use merc_utilities::Timing;
/// Explore the linear process specification using symbolic reachability.
pub fn explore_lps(
    storage: &mut Storage,
    lps: &LinearProcessSpecification,
    timing: &Timing,
) -> Result<usize, MercError> {
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
    _shared: Rc<RefCell<Shared>>,

    /// The initial state of the LPS.
    initial_state: Ldd,
}

impl SymbolicLinearProcessSpecification {
    pub fn new(storage: &mut Storage, lps: &LinearProcessSpecification) -> Result<Self, MercError> {
        let lps = preprocess(lps)?;

        let parameters = lps.parameters();
        let num_parameters = parameters.len();

        let shared = Rc::new(RefCell::new(Shared {
            context: LearnSuccessorsContext::new(&lps),
            mapping: (0..num_parameters).map(|_| IndexedSet::new()).collect(),
        }));

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
                let (index, _) = shared.borrow_mut().mapping[i].insert(param.clone());
                *index as u32
            })
            .collect::<Vec<u32>>();

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
    /// Context used by mCRL2 to perform the enumeration.
    context: LearnSuccessorsContext,

    /// Stores a bidirectional mapping between data expressions and indices.
    mapping: Vec<IndexedSet<DataExpression>>,
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

    /// The assignments of this summand.
    assignments: ATermList<ATerm>,

    /// The shared context containing the rewriter/enumerator.
    shared: Rc<RefCell<Shared>>,
}

impl SymbolicSummand {
    /// Extract the required information from the given action summand that is required for symbolic exploration.
    pub fn new(
        storage: &mut Storage,
        summand: &LinearSummand,
        parameters: &ATermList<DataVariable>,
        shared: Rc<RefCell<Shared>>,
    ) -> Self {
        // Collect free variables from the condition.
        let mut read_vars = free_variables_data_expression(&summand.condition().copy());
        let parameters = parameters.to_vec();

        // Collect free variables from the update expressions.
        let mut write_vars = Vec::new();
        for assignment in summand.assignments().iter() {
            let lhs: DataVariable = assignment.arg(0).protect().into();
            let rhs = assignment.arg(1);

            // The parameter is read if its RHS references any process parameter.
            let rhs_vars = free_variables_data_expression(&rhs.copy().into());
            read_vars.extend(rhs_vars);

            // The parameter is written if the RHS differs from the LHS variable.
            if DataExpressionRef::from(lhs.copy()) != DataExpressionRef::from(rhs.copy()) {
                write_vars.push(lhs);
            }
        }

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
        let assignments: ATermList<ATerm> = summand.assignments();

        let relation = storage.protect(storage.empty_set());
        let project_ldd = compute_proj(storage, &read_indices);

        let (meta, read_positions, write_positions) = compute_meta(storage, &read_indices, &write_indices);

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
            assignments,
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

    fn learn_successors(&mut self, storage: &mut Storage, todo: &Ldd) -> Result<Ldd, MercError> {
        let proj = project(storage, todo, &self.project_ldd);

        // The values to insert into the LDD.
        let mut values = Vec::new();
        let mut interleaved_values = vec![0; self.read_indices.len() + self.write_indices.len()];

        let mut proj_iter = iter(storage, &proj);
        while let Some(short_state) = proj_iter.next() {
            // Convert the LDD state values back to aterm pointers for the read parameters.
            let read_values: Vec<*const _aterm> = short_state
                .iter()
                .enumerate()
                .map(|(index, &val)| {
                    self.shared.borrow().mapping[self.read_indices[index] as usize]
                        .get_by_index(val as usize)
                        .expect("The value should be in the mapping")
                        .address()
                })
                .collect();

            for (offset, value) in self.read_positions.iter().zip(short_state.iter()) {
                interleaved_values[*offset as usize] = *value;
            }

            // TODO: The callback should be able to immediately deal with the values.
            let mut output = Vec::new();
            self.shared.borrow_mut().context.enumerate_raw(
                &self.condition,
                &self.summation_variables,
                &self.assignments,
                &self.read_parameters,
                &read_values,
                &mut |values: &[*const _aterm]| {
                    output.push(values.to_vec());
                },
            );

            for write in &output {
                trace!(
                    "written values {}",
                    write
                        .iter()
                        .format_with(", ", |value, f| f(&format_args!("{:?}", ATerm::from_ptr(*value))))
                );

                for (offset, (i, value)) in self.write_positions.iter().zip(write.iter().enumerate()) {
                    interleaved_values[*offset as usize] = *self.shared.borrow_mut().mapping
                        [self.write_indices[i] as usize]
                        .insert(DataExpression::from(ATerm::from_ptr(*value)))
                        .0 as Value;
                }

                values.push(interleaved_values.clone());
            }
        }

        // TODO: In oxidd we could actually immediately compute the union.
        let mut result = storage.protect(storage.empty_set());
        for values in values {
            let cube = singleton(storage, &values);
            result = union(storage, &result, &cube);
        }

        self.relation = union(storage, &self.relation, &result);

        Ok(result)
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
            self.assignments
                .iter()
                .format_with(", ", |assignment, f| f(&format_args!(
                    "{} := {}",
                    DataExpressionRef::from(assignment.arg(0)).pretty_print(),
                    DataExpressionRef::from(assignment.arg(1)).pretty_print()
                )))
        )?;

        writeln!(
            f,
            "\t\tread: {:?}",
            self.read_parameters
                .iter()
                .format_with(", ", |param, f| f(&format_args!("{:?}", ATerm::from_ptr(*param))))
        )?;
        writeln!(f, "\t\tread indices: {:?}", self.read_indices)?;
        writeln!(f, "\t\tread positions: {:?}", self.read_positions)?;
        writeln!(f, "\t\twrite indices: {:?}", self.write_indices)?;
        writeln!(f, "\t\twrite positions: {:?}", self.write_positions)
    }
}
