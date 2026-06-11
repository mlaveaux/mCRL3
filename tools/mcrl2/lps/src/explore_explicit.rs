use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use log::debug;

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
use mcrl2::is_variable;
use mcrl2::preprocess;
use mcrl2::pretty_print_multi_action;
use mcrl2::tau_multi_action;
use merc_collections::IndexedSet;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_explore::explore_to_lts;
use merc_lts::LtsBuilder;
use merc_lts::TransitionLabel;
use merc_utilities::MercError;
use merc_utilities::Timing;

/// Explore the linear process specification explicitly, forwarding the
/// discovered transitions to `builder`.
pub fn explore_lps_explicit<B>(
    builder: &mut B,
    lps: &LinearProcessSpecification,
    caching: CachingStrategy,
    strategy: ExplorationStrategy,
    timing: &Timing,
) -> Result<(), MercError>
where
    B: LtsBuilder<Mcrl2MultiActionLabel>,
{
    let lps = ExplicitLinearProcessSpecification::new(lps)?;
    debug!("{lps:?}");

    explore_to_lts(builder, &lps, strategy, caching, timing)?;
    Ok(())
}

/// A typed mCRL2 multi-action label backed by an [`ATerm`].
///
/// We keep the term itself so labels remain maximally shared and can be used
/// as hash/ordering keys. Display uses mCRL2's pretty-printer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Mcrl2MultiActionLabel {
    term: ATerm,
}

impl Mcrl2MultiActionLabel {
    fn from_multi_action_term(term: ATerm) -> Self {
        debug_assert!(
            is_mcrl2_timed_multi_action_term(&term),
            "Expected TimedMultAct term as transition label"
        );
        Self { term }
    }

    fn as_aterm(&self) -> &ATerm {
        &self.term
    }
}

impl fmt::Display for Mcrl2MultiActionLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", pretty_print_multi_action(&self.term))
    }
}

impl TransitionLabel for Mcrl2MultiActionLabel {
    fn tau_label() -> Self {
        Self::from_multi_action_term(tau_multi_action())
    }

    fn is_tau_label(&self) -> bool {
        self == &Self::tau_label()
    }

    fn matches_label(&self, label: &str) -> bool {
        pretty_print_multi_action(&self.term) == label
    }

    fn from_index(_i: usize) -> Self {
        panic!("Mcrl2MultiActionLabel does not support synthetic label generation")
    }
}

fn is_mcrl2_timed_multi_action_term(term: &ATerm) -> bool {
    let symbol = term.get_head_symbol();
    symbol.name() == "TimedMultAct" && symbol.arity() == 2
}

/// Explicit-state view of a [mcrl2::LinearProcessSpecification] that implements
/// the [merc_explore::LPS] trait.
///
/// State vectors are indexed into a per-parameter [IndexedSet] of the data
/// expressions observed for that parameter (see [Shared::mapping]). Labels are
/// the printed multi-actions of the summands.
struct ExplicitLinearProcessSpecification {
    /// The (preprocessed) underlying LPS.
    _lps: LinearProcessSpecification,

    /// Cached process parameter variables in declaration order.
    process_parameters: Vec<*const _aterm>,

    /// The summands extracted from the LPS.
    summands: Vec<ExplicitSummand>,

    /// Information shared between all summands.
    _shared: Rc<Shared>,

    /// The initial state vector.
    initial_state: Vec<usize>,
}

impl ExplicitLinearProcessSpecification {
    fn new(lps: &LinearProcessSpecification) -> Result<Self, MercError> {
        let lps = preprocess(lps)?;

        let parameters = lps.parameters();
        let parameter_terms: Vec<DataVariable> = parameters.to_vec();
        let process_parameters: Vec<*const _aterm> = parameter_terms.iter().map(|param| param.address()).collect();
        let num_parameters = parameters.len();

        let shared = Rc::new(Shared {
            context: LearnSuccessorsContext::new(&lps),
            mapping: RefCell::new((0..num_parameters).map(|_| IndexedSet::new()).collect()),
            parameter_values: RefCell::new(Vec::with_capacity(num_parameters)),
        });

        let mut summands = Vec::new();
        for index in 0..lps.num_summands() {
            summands.push(ExplicitSummand::new(
                &lps.action_summand(index)?,
                &parameters,
                Rc::clone(&shared),
            ));
        }

        let initial_state = lps
            .initial_process()
            .expressions()
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let (index, _) = shared.mapping.borrow_mut()[i].insert(param.clone());
                *index
            })
            .collect::<Vec<usize>>();

        debug_assert_eq!(
            initial_state.len(),
            num_parameters,
            "Initial state vector length must match number of parameters"
        );

        Ok(ExplicitLinearProcessSpecification {
            _lps: lps,
            summands,
            process_parameters,
            _shared: shared,
            initial_state,
        })
    }
}

/// State shared between [ExplicitSummand]s and the enclosing LPS.
struct Shared {
    /// Context used by mCRL2 to perform the enumeration. Behind a `RefCell` so
    /// the enumeration callback can re-enter Rust code that touches
    /// [Shared::mapping] while a call into the context is in progress.
    context: LearnSuccessorsContext,

    /// Bidirectional mapping between data expressions and indices, one
    /// [IndexedSet] per process parameter.
    mapping: RefCell<Vec<IndexedSet<DataExpression>>>,

    /// Reusable scratch buffer holding the parameter values resolved from the
    /// current source state. Filled during [`LPS::prepare`] and consumed
    /// immediately by [`LearnSuccessorsContext::set_assignments`].
    parameter_values: RefCell<Vec<*const _aterm>>,
}

/// A single summand of the LPS, prepared for explicit enumeration.
struct ExplicitSummand {
    /// The indices of the parameters that this summand reads.
    read_indices: Vec<usize>,

    /// The indices of the parameters that this summand writes (non-identity
    /// assignments).
    write_indices: Vec<usize>,

    /// The condition of this summand.
    condition: DataExpression,

    /// The summation variables of this summand.
    summation_variables: ATermList<DataVariable>,

    /// Only the non-identity assignments (write parameters) of this summand.
    write_assignments: ATermList<ATerm>,

    /// The multi-action of this summand.
    multi_action: Mcrl2MultiActionLabel,

    /// Shared context owning the rewriter/enumerator and parameter mappings.
    shared: Rc<Shared>,

    /// Reusable scratch buffer for the next-state vector produced for each
    /// enumerated solution. Reset and refilled for every solution.
    next_state_buf: RefCell<Vec<usize>>,
}

impl ExplicitSummand {
    fn new(summand: &LinearSummand, parameters: &ATermList<DataVariable>, shared: Rc<Shared>) -> Self {
        // Collect free variables from the condition.
        let mut read_vars = free_variables_data_expression(&summand.condition().copy());
        let parameters = parameters.to_vec();

        // Collect free variables from the update expressions and identify
        // which parameters are actually written.
        let mut write_vars = Vec::new();
        let mut write_assignments = Vec::new();

        for assignment in summand.assignments().iter() {
            let lhs: DataVariable = assignment.arg(0).protect().into();
            let rhs = assignment.arg(1);

            let rhs_vars = free_variables_data_expression(&rhs.copy().into());
            read_vars.extend(rhs_vars);

            if DataExpressionRef::from(lhs.copy()) != DataExpressionRef::from(rhs.copy()) {
                write_vars.push(lhs);
                write_assignments.push(assignment.protect());
            }
        }

        let write_assignments = ATermList::from_double_iter(write_assignments.into_iter());

        // The multi-action's data arguments (and time) may reference process
        // parameters that occur neither in the condition nor in any next-state
        // update.
        for subterm in summand.multi_action().iter() {
            if is_variable(&subterm) {
                read_vars.push(subterm.protect().into());
            }
        }

        let read_indices: Vec<usize> = parameters
            .iter()
            .enumerate()
            .filter(|(_, param)| read_vars.contains(param))
            .map(|(i, _)| i)
            .collect();

        let write_indices: Vec<usize> = parameters
            .iter()
            .enumerate()
            .filter(|(_, param)| write_vars.contains(param))
            .map(|(i, _)| i)
            .collect();

        let condition: DataExpression = summand.condition();
        let summation_variables: ATermList<DataVariable> = summand.summation_variables();

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

        let multi_action = Mcrl2MultiActionLabel::from_multi_action_term(summand.multi_action());

        Self {
            read_indices,
            write_indices,
            condition,
            summation_variables,
            write_assignments,
            multi_action,
            shared,
            next_state_buf: RefCell::new(Vec::new()),
        }
    }
}

impl LPS for ExplicitLinearProcessSpecification {
    type Value = usize;
    type Label = Mcrl2MultiActionLabel;
    type StateInfo = ();
    type Summand = ExplicitSummand;

    fn initial_state(&self) -> Vec<usize> {
        self.initial_state.clone()
    }

    fn summands(&self) -> &[Self::Summand] {
        &self.summands
    }

    fn prepare(&self, state: &[Self::Value]) {
        debug_assert_eq!(
            state.len(),
            self.process_parameters.len(),
            "State vector length must match number of process parameters"
        );

        let mut parameter_values = self._shared.parameter_values.borrow_mut();
        parameter_values.clear();
        let mapping = self._shared.mapping.borrow();
        for (i, value_index) in state.iter().enumerate() {
            parameter_values.push(
                mapping[i]
                    .get_by_index(*value_index)
                    .expect("Value must be in the mapping")
                    .address(),
            );
        }
        drop(mapping);

        self._shared
            .context
            .set_assignments(&self.process_parameters, &parameter_values);
    }

    fn state_info(&self, _state: &[Self::Value]) -> Self::StateInfo {}
}

impl Summand for ExplicitSummand {
    type Value = usize;
    type Label = Mcrl2MultiActionLabel;

    fn read_positions(&self) -> &[usize] {
        &self.read_indices
    }

    fn write_positions(&self) -> &[usize] {
        &self.write_indices
    }

    fn enumerate<F>(&self, state: &[usize], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[usize]) -> Result<(), MercError>,
    {
        self.shared.context.enumerate_raw_with_current_assignments(
            &self.condition,
            &self.summation_variables,
            &self.write_assignments,
            self.multi_action.as_aterm(),
            |values: &[*const _aterm], multi_action: *const _aterm| {
                debug_assert_eq!(
                    values.len(),
                    self.write_indices.len(),
                    "Enumerated values must match number of write indices"
                );

                // Build the next-state vector in the cached buffer instead of
                // allocating a fresh `Vec` per enumerated transition.
                let mut next_state = self.next_state_buf.borrow_mut();
                next_state.clear();
                next_state.extend_from_slice(state);
                {
                    let mut mapping = self.shared.mapping.borrow_mut();
                    for (i, &value) in values.iter().enumerate() {
                        let param_index = self.write_indices[i];
                        let new_index = mapping[param_index]
                            .insert(DataExpression::from(ATerm::from_ptr(value)))
                            .0;
                        next_state[param_index] = *new_index;
                    }
                }

                // Wrap the rewritten multi-action term as a typed label. The
                // ATerm protects the term beyond the temporary handed to the
                // callback, and aterms are maximally shared so equal
                // multi-actions still share storage.
                let label = Mcrl2MultiActionLabel::from_multi_action_term(ATerm::from_ptr(multi_action));

                // We cannot propagate errors from the C callback, so we panic on error and catch it in the caller.
                report(&label, &next_state).expect("Failed to report successor state");
            },
        );

        Ok(())
    }
}

impl fmt::Debug for ExplicitLinearProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ExplicitLinearProcessSpecification:")?;

        writeln!(f, "  Parameters:")?;
        for (i, param) in self._lps.parameters().iter().enumerate() {
            writeln!(f, "    {:?}: {:?}", i, param)?;
        }

        writeln!(f, "  Summands:")?;
        for (i, summand) in self.summands.iter().enumerate() {
            writeln!(f, "    {:?}: {:?}", i, summand)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ExplicitSummand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} -> {}", self.condition.pretty_print(), self.multi_action)?;
        writeln!(f, "\t\tread indices: {:?}", self.read_indices)?;
        writeln!(f, "\t\twrite indices: {:?}", self.write_indices)
    }
}
