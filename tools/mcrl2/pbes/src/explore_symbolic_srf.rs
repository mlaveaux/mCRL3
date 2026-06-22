use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use log::debug;

use mcrl2::_aterm;
use mcrl2::ATerm;
use mcrl2::ATermList;
use mcrl2::DataExpression;
use mcrl2::DataExpressionRef;
use mcrl2::DataSpecification;
use mcrl2::DataVariable;
use mcrl2::LearnSuccessorsContext;
use mcrl2::Pbes;
use mcrl2::PbesPropositionalVariableInstantiation;
use mcrl2::SrfPbes;
use mcrl2::free_variables_data_expression;
use mcrl2::make_data_assignment_list;
use mcrl2::tau_multi_action;
use merc_collections::IndexedSet;

use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;
use streaming_iterator::StreamingIterator;

use merc_symbolic::SymbolicLPS;
use merc_symbolic::TransitionGroup;
use merc_symbolic::iter;
use merc_symbolic::reachability;
use merc_utilities::MercError;
use merc_utilities::Timing;

/// Explore the PBES in SRF normal form using symbolic LDD-based reachability.
///
/// Returns the LDD encoding the set of reachable states (BES equations). State
/// vectors have layout `[equation_index, param_0, …, param_{n-1}]`, where the
/// equation index is encoded directly as the LDD value and each parameter as an
/// index into the per-parameter interning [`IndexedSet`].
pub(crate) fn explore_pbes_symbolic(
    storage: &LDDManagerRef,
    pbes: &Pbes,
    timing: &Timing,
) -> Result<LDDFunction, MercError> {
    let mut symbolic = SymbolicPbesSrf::new(storage, pbes)?;

    debug!("{symbolic:?}");

    reachability(storage, &mut symbolic, timing)
}

/// Information shared between the [`SymbolicPbesSrf`] and all of its summands.
struct Shared {
    /// mCRL2 backend used to evaluate conditions and enumerate solutions.
    context: LearnSuccessorsContext,

    /// Per-data-parameter interning of data expressions to dense LDD values.
    mapping: RefCell<Vec<IndexedSet<DataExpression>>>,
}

/// Symbolic LDD view of a PBES in SRF normal form.
///
/// The control component (which SRF equation a state belongs to) is encoded as
/// state-vector position `0`.
struct SymbolicPbesSrf {
    /// The unified SRF PBES; retained so summand terms stay alive.
    _srf: SrfPbes,

    /// The symbolic summands, one per `(equation, srf_summand)` pair.
    summands: Vec<SymbolicPbesSrfSummand>,

    /// State shared with every summand (enumerator + interning).
    _shared: Rc<Shared>,

    /// The LDD encoding the initial state.
    initial_state: LDDFunction,
}

impl SymbolicPbesSrf {
    fn new(storage: &LDDManagerRef, pbes: &Pbes) -> Result<Self, MercError> {
        let mut srf = SrfPbes::from(pbes)?;
        srf.unify_parameters(false, true)?;

        if srf.equations().is_empty() {
            return Err("PBES has no equations".into());
        }

        // After `unify_parameters` every equation shares the same parameter list.
        let parameters: Vec<DataVariable> = srf.equations()[0].variable().parameters().iter().collect();
        let num_params = parameters.len();

        let data_spec: DataSpecification = pbes.data_specification();
        let shared = Rc::new(Shared {
            context: LearnSuccessorsContext::from_data_spec(&data_spec),
            mapping: RefCell::new((0..num_params).map(|_| IndexedSet::new()).collect()),
        });

        // Equation name -> equation index, used when resolving target PVIs.
        let name_to_eq: HashMap<String, usize> = srf
            .equations()
            .iter()
            .enumerate()
            .map(|(i, eq)| (eq.variable().name().to_string(), i))
            .collect();

        let tau = tau_multi_action();

        // Build one symbolic summand per (equation, srf_summand) pair.
        let mut summands = Vec::new();
        for eq in srf.equations().iter() {
            let eq_index = *name_to_eq
                .get(&eq.variable().name().to_string())
                .expect("equation name must be present");

            let eq_param_term: ATerm = eq.variable().parameters().into();

            for srf_summand in eq.summands() {
                let target_pvi: PbesPropositionalVariableInstantiation = srf_summand.variable().into();
                let target_eq_index = *name_to_eq
                    .get(&target_pvi.name().to_string())
                    .ok_or_else(|| MercError::from(format!("Unknown target equation: {}", target_pvi.name())))?;

                summands.push(SymbolicPbesSrfSummand::new(
                    storage,
                    &parameters,
                    eq_index,
                    target_eq_index,
                    &eq_param_term,
                    srf_summand.condition().into(),
                    srf_summand.parameters(),
                    &target_pvi,
                    tau.clone(),
                    Rc::clone(&shared),
                )?);
            }
        }

        // Build the initial state vector: [equation_index, interned params...].
        let initial_pvi = srf.initial_state();
        let initial_eq_index = *name_to_eq
            .get(&initial_pvi.name().to_string())
            .ok_or_else(|| MercError::from(format!("Unknown initial equation: {}", initial_pvi.name())))?;

        let mut initial_vector: Vec<Value> = Vec::with_capacity(1 + num_params);
        initial_vector.push(initial_eq_index as Value);
        for (i, arg) in initial_pvi.arguments().iter().enumerate() {
            // SAFETY: `arg` is an argument of the live initial-state PVI; the
            // pointer is immediately protected by `ATerm::from_ptr`.
            let term = unsafe { ATerm::from_ptr(arg.address()) };
            let (index, _) = shared.mapping.borrow_mut()[i].insert(DataExpression::from(term));
            initial_vector.push(*index as Value);
        }

        debug_assert_eq!(
            initial_vector.len(),
            1 + num_params,
            "Initial state vector length must match 1 + number of parameters"
        );

        let initial_state = LDDFunction::singleton(storage, &initial_vector)?;

        Ok(SymbolicPbesSrf {
            _srf: srf,
            summands,
            _shared: shared,
            initial_state,
        })
    }
}

/// A single symbolic SRF summand, encoded as an LDD short-vector transition
/// relation that is learned on the fly during reachability.
struct SymbolicPbesSrfSummand {
    /// Source equation index; the summand fires only from states whose position
    /// `0` equals this value.
    equation_index: usize,

    /// Target equation index written to position `0` of every successor.
    target_equation_index: usize,

    /// Full state-vector indices read by this summand (sorted, always starts
    /// with `0`, the equation index).
    read_indices: Vec<u32>,

    /// Full state-vector indices written by this summand (sorted, always starts
    /// with `0`, the equation index).
    write_indices: Vec<u32>,

    /// aterm pointers of the data parameters read by this summand, aligned with
    /// `read_indices[1..]`.
    read_parameters: Vec<*const _aterm>,

    /// Positions of `read_indices` in the interleaved short vector.
    read_positions: Vec<usize>,

    /// Interleaved short-vector position of the equation index (write side).
    eq_write_position: usize,

    /// For each data parameter `i` (`0..num_params`), the interleaved short-vector
    /// position to write if the parameter is written, otherwise `None`.
    data_write_positions: Vec<Option<usize>>,

    /// Projection of the state space on `read_indices`.
    project_ldd: LDDFunction,

    /// The learned transition relation `T' -> U'` for this summand.
    relation: LDDFunction,

    /// Relational-product metadata for `read_indices`/`write_indices`.
    meta: LDDFunction,

    /// The summand's data condition.
    condition: DataExpression,

    /// The summand's existential summation variables.
    summation_variables: ATermList<DataVariable>,

    /// Full assignment list `params := target_args` passed to the enumerator;
    /// the callback yields one value per parameter.
    write_assignments: ATermList<ATerm>,

    /// Cached tau multi-action term (PBESs have no actions).
    multi_action: ATerm,

    /// State shared with the enclosing PBES (enumerator + interning).
    shared: Rc<Shared>,
}

impl SymbolicPbesSrfSummand {
    #[allow(clippy::too_many_arguments)]
    fn new(
        storage: &LDDManagerRef,
        parameters: &[DataVariable],
        equation_index: usize,
        target_equation_index: usize,
        eq_param_term: &ATerm,
        condition: DataExpression,
        summation_variables: ATermList<DataVariable>,
        target_pvi: &PbesPropositionalVariableInstantiation,
        multi_action: ATerm,
        shared: Rc<Shared>,
    ) -> Result<Self, MercError> {
        let num_params = parameters.len();

        // Collect read variables: free variables of the condition and of every
        // target argument.
        let mut read_vars = free_variables_data_expression(&condition.copy());

        // A data parameter is written iff its target argument differs from the
        // parameter itself (a non-identity assignment).
        let mut data_written = vec![false; num_params];
        for (k, (param, arg)) in parameters.iter().zip(target_pvi.arguments().iter()).enumerate() {
            read_vars.extend(free_variables_data_expression(&arg.copy()));
            if Into::<DataExpressionRef<'_>>::into(param.copy()) != arg.copy() {
                data_written[k] = true;
            }
        }

        // Read indices: equation index (0) plus every data parameter that
        // appears free in the condition or a target argument.
        let mut read_indices: Vec<u32> = vec![0];
        let mut read_parameters: Vec<*const _aterm> = Vec::new();
        for (i, param) in parameters.iter().enumerate() {
            if read_vars.contains(param) {
                read_indices.push((i + 1) as u32);
                read_parameters.push(param.address());
            }
        }

        // Write indices: equation index (0) plus every written data parameter.
        let mut write_indices: Vec<u32> = vec![0];
        for (i, &written) in data_written.iter().enumerate() {
            if written {
                write_indices.push((i + 1) as u32);
            }
        }

        // Full assignment list (one assignment per parameter); the enumerator
        // returns one value per parameter, of which we keep the written ones.
        let target_args: ATerm = target_pvi.arguments().protect().into();
        let write_assignments: ATermList<ATerm> =
            ATermList::new(make_data_assignment_list(eq_param_term, &target_args));

        let project_ldd = LDDFunction::projection_meta(storage, &read_indices)?;
        let relation = LDDFunction::empty_set(storage)?;
        let (meta, read_positions, write_positions) =
            LDDFunction::relation_product_meta(storage, &read_indices, &write_indices)?;

        // Position of the equation index on the write side; it is index 0, the
        // smallest, so it is the first entry of the sorted `write_indices`.
        let eq_write_position = write_positions[0];

        // Map each written data parameter to its interleaved short-vector
        // position; `write_indices[1..]` are the written data parameters in
        // ascending order, aligned with `write_positions[1..]`.
        let mut data_write_positions = vec![None; num_params];
        for (write_index, position) in write_indices.iter().skip(1).zip(write_positions.iter().skip(1)) {
            data_write_positions[(*write_index - 1) as usize] = Some(*position);
        }

        debug_assert_eq!(
            read_indices.len() - 1,
            read_parameters.len(),
            "Data read indices must match data read parameters"
        );
        debug_assert!(read_indices.iter().is_sorted(), "Read indices must be sorted");
        debug_assert!(write_indices.iter().is_sorted(), "Write indices must be sorted");

        Ok(SymbolicPbesSrfSummand {
            equation_index,
            target_equation_index,
            read_indices,
            write_indices,
            read_parameters,
            read_positions,
            eq_write_position,
            data_write_positions,
            project_ldd,
            relation,
            meta,
            condition,
            summation_variables,
            write_assignments,
            multi_action,
            shared,
        })
    }
}

impl SymbolicLPS for SymbolicPbesSrf {
    fn initial_state(&self) -> &LDDFunction {
        &self.initial_state
    }

    fn transition_groups(&self) -> &[impl TransitionGroup] {
        &self.summands
    }

    fn transition_groups_mut(&mut self) -> &mut [impl TransitionGroup] {
        &mut self.summands
    }
}

impl TransitionGroup for SymbolicPbesSrfSummand {
    fn relation(&self) -> &LDDFunction {
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

    fn meta(&self) -> &LDDFunction {
        &self.meta
    }

    fn learn_successors(&mut self, storage: &LDDManagerRef, todo: &LDDFunction) -> Result<(), MercError> {
        let proj = todo.project(&self.project_ldd)?;

        // Reused across short states to avoid per-iteration allocation.
        let mut read_values: Vec<*const _aterm> = Vec::with_capacity(self.read_parameters.len());
        let mut interleaved_values: Vec<Value> = vec![0; self.read_indices.len() + self.write_indices.len()];

        let mut states = iter(&proj);
        while let Some(short_state) = states.next() {
            debug_assert_eq!(
                short_state.len(),
                self.read_indices.len(),
                "Projected state must have one value per read index"
            );

            // Position 0 of `read_indices` is the equation index; only the
            // owning equation's states fire this summand.
            if short_state[0] as usize != self.equation_index {
                continue;
            }

            // Copy the read values into their interleaved positions.
            for (position, value) in self.read_positions.iter().zip(short_state.iter()) {
                interleaved_values[*position] = *value;
            }

            // Resolve the data read parameters (skip the equation index at 0).
            read_values.clear();
            {
                let mapping = self.shared.mapping.borrow();
                for (read_index, value) in self.read_indices.iter().skip(1).zip(short_state.iter().skip(1)) {
                    read_values.push(
                        mapping[(*read_index - 1) as usize]
                            .get_by_index(*value as usize)
                            .expect("The value should be in the mapping")
                            .address(),
                    );
                }
            }

            // The equation index always transitions to the target equation.
            interleaved_values[self.eq_write_position] = self.target_equation_index as Value;

            self.shared.context.enumerate_raw(
                &self.condition,
                &self.summation_variables,
                &self.write_assignments,
                &self.multi_action,
                &self.read_parameters,
                &read_values,
                |values: &[*const _aterm], _multi_action: *const _aterm| {
                    debug_assert_eq!(
                        values.len(),
                        self.data_write_positions.len(),
                        "Enumerated values must match number of parameters"
                    );

                    {
                        let mut mapping = self.shared.mapping.borrow_mut();
                        for (i, &value) in values.iter().enumerate() {
                            if let Some(position) = self.data_write_positions[i] {
                                // SAFETY: `value` is a live enumerated term handed
                                // to this callback by the mCRL2 enumerator.
                                let term = unsafe { ATerm::from_ptr(value) };
                                interleaved_values[position] =
                                    *mapping[i].insert(DataExpression::from(term)).0 as Value;
                            }
                        }
                    }

                    let cube =
                        LDDFunction::singleton(storage, &interleaved_values).expect("Failed to allocate LDD singleton");
                    self.relation = self.relation.union(&cube).expect("Failed to allocate LDD union");
                },
            );
        }

        Ok(())
    }
}

impl fmt::Debug for SymbolicPbesSrf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SymbolicPbesSrf:")?;
        writeln!(f, "  Summands:")?;
        for (i, summand) in self.summands.iter().enumerate() {
            writeln!(f, "    {i}: {summand:?}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for SymbolicPbesSrfSummand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "X{} -> X{}: {} (read {:?}, write {:?})",
            self.equation_index,
            self.target_equation_index,
            self.condition.pretty_print(),
            self.read_indices,
            self.write_indices
        )
    }
}
