use std::cell::Cell;
use std::fmt;
use std::sync::Arc;

use log::debug;
use log::info;

use mcrl2::_aterm;
use mcrl2::ATerm;
use mcrl2::ATermList;
use mcrl2::ATermSend;
use mcrl2::DataExpression;
use mcrl2::DataExpressionRef;
use mcrl2::DataVariable;
use mcrl2::LearnSuccessorsContext;
use mcrl2::LinearProcessSpecification;
use mcrl2::LinearSummand;
use mcrl2::PreprocessOptions;
use mcrl2::Protected;
use mcrl2::free_variables_data_expression;
use mcrl2::is_variable;
use mcrl2::preprocess;
use mcrl2::pretty_print_multi_action;
use mcrl2::tau_multi_action;
use merc_explore::CacheLPS;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_explore::explore;
use merc_explore::explore_parallel;
use merc_io::TimeProgress;
use merc_lts::ConcurrentLtsBuilder;
use merc_lts::LtsBuilder;
use merc_lts::TransitionLabel;
use merc_unsafety::ConcurrentIndexedSet;
use merc_utilities::MercError;
use merc_utilities::ShardedCounter;
use merc_utilities::Timing;

use crate::cfg_lps::CfgLinearProcessSpecification;

/// Periodic progress reporter for LPS exploration, printing the number of
/// discovered states and transitions.
pub(crate) fn lps_progress() -> TimeProgress<(usize, usize)> {
    TimeProgress::new(
        |(states, transitions): (usize, usize)| {
            info!("Explored {states} states, {transitions} transitions...");
        },
        1,
    )
}

/// Explore the linear process specification explicitly, forwarding the
/// discovered transitions to `builder`.
///
/// When `control_flow` is set, a [`ControlFlowAnalysis`] is layered on top of
/// the explicit LPS to prune summands whose control flow guard cannot hold in
/// the current state (see [`CfgLinearProcessSpecification`]). The pruning never
/// changes the explored transition system.
///
/// [`ControlFlowAnalysis`]: crate::control_flow::ControlFlowAnalysis
pub fn explore_lps_explicit<B>(
    builder: &mut B,
    lps: &LinearProcessSpecification,
    caching: CachingStrategy,
    strategy: ExplorationStrategy,
    control_flow: bool,
    timing: &Timing,
) -> Result<B::LTS, MercError>
where
    B: LtsBuilder<Mcrl2MultiActionLabel>,
{
    if control_flow {
        let lps = CfgLinearProcessSpecification::new(lps)?;
        info!(
            "Control flow analysis identified {} control flow parameter(s)",
            lps.control_flow_parameters().len()
        );
        let result = explore_lps_explicit_impl(builder, &lps, caching, strategy, timing);
        debug!("{}", lps.metrics());
        result
    } else {
        let lps = ExplicitLinearProcessSpecification::new(lps)?;
        debug!("{lps:?}");
        explore_lps_explicit_impl(builder, &lps, caching, strategy, timing)
    }
}

/// Shared exploration driver over any explicit-state [`LPS`] view producing
/// mCRL2 multi-action labels, used by both the plain explicit explorer and the
/// control-flow-pruning variant.
fn explore_lps_explicit_impl<B, L>(
    builder: &mut B,
    lps: &L,
    caching: CachingStrategy,
    strategy: ExplorationStrategy,
    timing: &Timing,
) -> Result<B::LTS, MercError>
where
    B: LtsBuilder<Mcrl2MultiActionLabel>,
    L: LPS<Value = usize, Label = Mcrl2MultiActionLabel, StateInfo = (), Summand = ExplicitSummand>,
{
    // Only layer the enumeration cache on top of the LPS when a caching strategy
    // is actually requested; otherwise explore the bare LPS directly.
    match caching {
        CachingStrategy::None => run_explore_explicit(builder, lps, strategy, timing),
        _ => {
            let cached = CacheLPS::new(lps, caching);
            let result = run_explore_explicit(builder, &cached, strategy, timing)?;
            debug!("{}", cached.metrics());
            Ok(result)
        }
    }
}

/// Runs the sequential explicit exploration loop over any [`LPS`] view producing
/// mCRL2 multi-action labels, counting states and transitions and finalising the
/// builder. The view is either the bare LPS or one wrapped in [`CacheLPS`].
fn run_explore_explicit<B, M>(
    builder: &mut B,
    lps: &M,
    strategy: ExplorationStrategy,
    timing: &Timing,
) -> Result<B::LTS, MercError>
where
    B: LtsBuilder<Mcrl2MultiActionLabel>,
    M: LPS<Value = usize, Label = Mcrl2MultiActionLabel, StateInfo = ()>,
{
    // Count states and transitions in the exploration closures, driving the
    // periodic progress reporter from `on_transition`.
    let progress = lps_progress();
    let states = Cell::new(0usize);
    let transitions = Cell::new(0usize);

    let initial = explore(
        lps,
        strategy,
        timing,
        builder,
        |_b: &mut B, _state, _info: &()| {
            states.set(states.get() + 1);
            Ok(())
        },
        |b: &mut B, from, label: &Mcrl2MultiActionLabel, to| {
            transitions.set(transitions.get() + 1);
            progress.print((states.get(), transitions.get()));
            b.add_transition(from, label, to)
        },
    )?;
    info!(
        "Exploration complete: {} states, {} transitions",
        states.get(),
        transitions.get(),
    );
    builder.require_num_of_states(states.get());

    builder.finish(initial)
}

/// Explores the linear process specification explicitly in parallel across
/// `threads` worker threads, streaming the discovered transitions into
/// `builder`.
pub fn explore_lps_explicit_parallel<B>(
    builder: &mut B,
    lps: &LinearProcessSpecification,
    caching: CachingStrategy,
    threads: usize,
    control_flow: bool,
    timing: &Timing,
) -> Result<B::LTS, MercError>
where
    B: ConcurrentLtsBuilder<Mcrl2MultiActionLabel>,
{
    if control_flow {
        let lps = CfgLinearProcessSpecification::new(lps)?;
        info!(
            "Control flow analysis identified {} control flow parameter(s)",
            lps.control_flow_parameters().len()
        );
        let result = explore_lps_explicit_parallel_impl(builder, &lps, caching, threads, timing);
        debug!("{}", lps.metrics());
        result
    } else {
        let lps = ExplicitLinearProcessSpecification::new(lps)?;
        explore_lps_explicit_parallel_impl(builder, &lps, caching, threads, timing)
    }
}

/// Shared parallel exploration driver over any explicit-state [`LPS`] view,
/// used by both the plain explicit explorer and the control-flow-pruning
/// variant.
fn explore_lps_explicit_parallel_impl<B, L>(
    builder: &mut B,
    lps: &L,
    caching: CachingStrategy,
    threads: usize,
    timing: &Timing,
) -> Result<B::LTS, MercError>
where
    B: ConcurrentLtsBuilder<Mcrl2MultiActionLabel>,
    L: LPS<Value = usize, Label = Mcrl2MultiActionLabel, StateInfo = (), Summand = ExplicitSummand> + Sync,
{
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|err| MercError::from(format!("Failed to build thread pool: {err}")))?;

    // Only layer the enumeration cache on top of the LPS when a caching strategy
    // is actually requested; otherwise explore the bare LPS directly.
    match caching {
        CachingStrategy::None => run_explore_explicit_parallel(builder, lps, &pool, timing),
        _ => {
            let cached = CacheLPS::new(lps, caching);
            let result = run_explore_explicit_parallel(builder, &cached, &pool, timing)?;
            debug!("{}", cached.metrics());
            Ok(result)
        }
    }
}

/// Runs the parallel explicit exploration loop over any [`LPS`] view producing
/// mCRL2 multi-action labels, streaming the discovered transitions into the
/// concurrent builder. The view is either the bare LPS or one wrapped in
/// [`CacheLPS`].
fn run_explore_explicit_parallel<B, M>(
    builder: &mut B,
    lps: &M,
    pool: &rayon::ThreadPool,
    timing: &Timing,
) -> Result<B::LTS, MercError>
where
    B: ConcurrentLtsBuilder<Mcrl2MultiActionLabel>,
    M: LPS<Value = usize, Label = Mcrl2MultiActionLabel, StateInfo = ()> + Sync,
    <M::Summand as Summand>::Context: Send,
{
    // Shared (immutable) reference to the builder used by the worker threads;
    // `ConcurrentLtsBuilder` requires `Sync`, so the workers can add transitions
    // concurrently while the builder synchronises internally.
    let builder_ref: &B = builder;

    // Shared state/transition counts.
    let states = ShardedCounter::new();
    let transitions = ShardedCounter::new();
    let progress = lps_progress();

    let initial = timing.measure("explore", || -> Result<_, MercError> {
        pool.install(|| {
            let timing = Timing::new();
            let (initial, _locals) = explore_parallel(
                lps,
                &timing,
                || (),
                |_local: &mut (), _state, _info: &()| {
                    states.increment();
                    Ok(())
                },
                |_local: &mut (), from, label: &Mcrl2MultiActionLabel, to| {
                    transitions.increment();
                    if progress.is_due() {
                        progress.print((states.get() as usize, transitions.get() as usize));
                    }
                    builder_ref.add_transition_shared(from, label, to)
                },
            )?;
            Ok(initial)
        })
    })?;

    let total_states = states.get() as usize;
    let total_transitions = transitions.get() as usize;
    info!("Exploration complete: {total_states} states, {total_transitions} transitions");

    // Finalise the builder, recording the total number of states so isolated
    // (deadlock) states are still reflected in the result.
    builder.require_num_of_states(total_states);

    builder.finish(initial)
}

/// A typed mCRL2 multi-action label backed by an [`ATermSend`].
///
/// We keep the term itself so labels remain maximally shared and can be used as
/// hash/ordering keys. Backing it with [`ATermSend`] (rather than [`ATerm`])
/// makes the label `Send + Sync`, so it can be created on one worker thread and
/// stored in (or dropped from) structures shared across threads — the
/// enumeration cache and the concurrent LTS builder. Display uses mCRL2's
/// pretty-printer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Mcrl2MultiActionLabel {
    term: ATermSend,
}

impl Mcrl2MultiActionLabel {
    fn from_multi_action_term(term: ATermSend) -> Self {
        debug_assert!(
            is_mcrl2_timed_multi_action_term(&term),
            "Expected TimedMultAct term as transition label"
        );
        Self { term }
    }

    /// Protects the multi-action term on the current thread for use with the
    /// mCRL2 FFI, which expects a thread-local [`ATerm`].
    fn as_aterm(&self) -> ATerm {
        self.term.protect_local()
    }
}

impl fmt::Display for Mcrl2MultiActionLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", pretty_print_multi_action(&self.as_aterm()))
    }
}

impl TransitionLabel for Mcrl2MultiActionLabel {
    fn tau_label() -> Self {
        Self::from_multi_action_term(ATermSend::from(&tau_multi_action()))
    }

    fn is_tau_label(&self) -> bool {
        self == &Self::tau_label()
    }

    fn matches_label(&self, label: &str) -> bool {
        pretty_print_multi_action(&self.as_aterm()) == label
    }

    fn from_index(_i: usize) -> Self {
        panic!("Mcrl2MultiActionLabel does not support synthetic label generation")
    }
}

fn is_mcrl2_timed_multi_action_term(term: &ATermSend) -> bool {
    let term = term.copy();
    let symbol = term.get_head_symbol();
    symbol.name() == "TimedMultAct" && symbol.arity() == 2
}

/// Explicit-state view of a [mcrl2::LinearProcessSpecification] that implements
/// the [merc_explore::LPS] trait.
///
/// State vectors are indices into a shared [`ValueMapping`]: a thread-safe set
/// interning the data expressions observed for every process parameter. The set
/// stores bare [`DataExpressionRef`]s kept alive by the garbage collector
/// through the [`Protected`] wrapper, so the mapping is globally consistent and
/// safe to populate from several worker threads at once. Labels are the printed
/// multi-actions of the summands.
pub(crate) struct ExplicitLinearProcessSpecification {
    /// The (preprocessed) underlying LPS. Retained because each per-thread
    /// [`ExplicitContext`] builds its own [`LearnSuccessorsContext`] from it.
    lps: LinearProcessSpecification,

    /// Cached process parameter variables in declaration order.
    process_parameters: Vec<*const _aterm>,

    /// The summands extracted from the LPS.
    summands: Vec<ExplicitSummand>,

    /// Concurrent value interning shared with every summand. Owned here through
    /// [`Protected`] so the garbage-collection protection is released when the
    /// LPS is dropped.
    value_mapping: Protected<ValueMapping>,

    /// The initial state vector.
    initial_state: Vec<usize>,
}

/// Shared interning of the data expressions observed as parameter values,
/// mapping each distinct expression to a dense `usize` used in state vectors.
type ValueMapping = ConcurrentIndexedSet<DataExpressionRef<'static>>;

// SAFETY: after construction the LPS is immutable except for `value_mapping`,
// whose backing [`ConcurrentIndexedSet`] is itself thread-safe. The cached
// `*const _aterm` parameter pointers are stable maximally shared term addresses,
// and mCRL2 garbage collection is stop-the-world, so no `&self` access can race
// with collection.
unsafe impl Sync for ExplicitLinearProcessSpecification {}

impl ExplicitLinearProcessSpecification {
    pub(crate) fn new(lps: &LinearProcessSpecification) -> Result<Self, MercError> {
        let lps = preprocess(lps, &PreprocessOptions::default())?;

        let parameters = lps.parameters();
        let parameter_terms: Vec<DataVariable> = parameters.to_vec();
        let process_parameters: Vec<*const _aterm> = parameter_terms.iter().map(|param| param.address()).collect();
        let num_parameters = parameters.len();

        // Shared value interning, kept alive as a garbage-collection container
        // by `value_mapping`.
        let value_mapping = Protected::new(ValueMapping::new());

        let mut summands = Vec::new();
        for index in 0..lps.num_summands() {
            summands.push(ExplicitSummand::new(
                &lps.action_summand(index)?,
                &parameters,
                value_mapping.handle(),
            ));
        }

        // Temporary enumeration context used to rewrite the initial state
        // expressions to normal form. Its substitution is seeded with the
        // constant assignments recorded during preprocessing.
        let context = LearnSuccessorsContext::new(&lps);

        let initial_state = lps
            .initial_process()
            .expressions()
            .iter()
            .map(|param| {
                // Rewrite the initial expression under a context with the
                // constant assignments produced by
                // `replace_constants_by_variables`.
                let expr = unsafe { DataExpressionRef::from_address(param.address()) };
                let rewritten = context.rewrite_under_sigma(&expr);

                // SAFETY: the rewritten term is interned into `value_mapping`.
                value_mapping
                    .insert(unsafe { DataExpressionRef::from_address(rewritten.address()) })
                    .0
            })
            .collect::<Vec<usize>>();

        debug_assert_eq!(
            initial_state.len(),
            num_parameters,
            "Initial state vector length must match number of parameters"
        );

        Ok(ExplicitLinearProcessSpecification {
            lps,
            summands,
            process_parameters,
            value_mapping,
            initial_state,
        })
    }

    /// The process parameter variables in declaration order.
    pub(crate) fn parameters(&self) -> Vec<DataVariable> {
        self.lps.parameters().to_vec()
    }

    /// The (preprocessed) underlying linear process specification.
    pub(crate) fn lps(&self) -> &LinearProcessSpecification {
        &self.lps
    }

    /// Rewrites `value` to normal form under `context` and interns it into the
    /// shared value mapping, returning its dense index.
    ///
    /// The rewriting and interning mirror the construction of the initial state
    /// vector, so the returned index can be compared directly against the
    /// entries of explored state vectors.
    pub(crate) fn intern_normal_form(&self, context: &LearnSuccessorsContext, value: &DataExpressionRef) -> usize {
        let rewritten = context.rewrite_under_sigma(value);

        // SAFETY: the rewritten term is interned into `self.value_mapping`, a
        // `Protected` container that keeps every interned term live through GC
        // marking for as long as the mapping exists.
        self.value_mapping
            .insert(unsafe { DataExpressionRef::from_address(rewritten.address()) })
            .0
    }
}

/// Per-thread enumeration context for an [`ExplicitLinearProcessSpecification`].
///
/// Owns the mCRL2 enumeration backend and the reusable scratch buffers, so the
/// LPS and its summands stay immutable and shareable by `&self` while each
/// worker thread drives its own context.
pub struct ExplicitContext {
    /// Backend used by mCRL2 to perform the enumeration, staged per source
    /// state by [`LPS::prepare`] and consumed by [`Summand::enumerate`].
    context: LearnSuccessorsContext,

    /// Reusable scratch buffer holding the `*const _aterm` parameter values
    /// resolved from the current source state. Filled during [`LPS::prepare`]
    /// and consumed immediately by [`LearnSuccessorsContext::set_assignments`].
    parameter_values: Vec<*const _aterm>,

    /// Reusable scratch buffer for the next-state vector produced for each
    /// enumerated solution. Reset and refilled for every solution.
    next_state_buf: Vec<usize>,
}

// SAFETY: an `ExplicitContext` is owned by exactly one worker thread.
unsafe impl Send for ExplicitContext {}

/// A single summand of the LPS, prepared for explicit enumeration.
pub(crate) struct ExplicitSummand {
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

    /// Handle to the enclosing LPS's value interning, used to intern enumerated
    /// next-state values from any worker thread.
    mapping: Arc<ValueMapping>,
}

impl ExplicitSummand {
    fn new(summand: &LinearSummand, parameters: &ATermList<DataVariable>, mapping: Arc<ValueMapping>) -> Self {
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

        let multi_action = Mcrl2MultiActionLabel::from_multi_action_term(ATermSend::from(&summand.multi_action()));

        Self {
            read_indices,
            write_indices,
            condition,
            summation_variables,
            write_assignments,
            multi_action,
            mapping,
        }
    }

    /// The condition (guard) of this summand.
    pub(crate) fn condition(&self) -> &DataExpression {
        &self.condition
    }

    /// The non-identity assignments (write parameters) of this summand.
    pub(crate) fn write_assignments(&self) -> &ATermList<ATerm> {
        &self.write_assignments
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

    fn create_context(&self) -> ExplicitContext {
        ExplicitContext {
            context: LearnSuccessorsContext::new(&self.lps),
            parameter_values: Vec::with_capacity(self.process_parameters.len()),
            next_state_buf: Vec::new(),
        }
    }

    fn prepare<'a>(
        &'a self,
        context: &mut ExplicitContext,
        state: &'a [Self::Value],
    ) -> impl Iterator<Item = usize> + 'a {
        debug_assert_eq!(
            state.len(),
            self.process_parameters.len(),
            "State vector length must match number of process parameters"
        );

        context.parameter_values.clear();
        for value_index in state.iter() {
            context.parameter_values.push(
                self.value_mapping
                    .get_by_index(*value_index)
                    .expect("Value must be in the mapping")
                    .address(),
            );
        }

        context
            .context
            .set_assignments(&self.process_parameters, &context.parameter_values);

        // Every summand is a candidate for every source state.
        0..self.summands.len()
    }

    fn state_info(&self, _state: &[Self::Value]) -> Self::StateInfo {}
}

impl Summand for ExplicitSummand {
    type Value = usize;
    type Label = Mcrl2MultiActionLabel;
    type Context = ExplicitContext;

    fn read_positions(&self) -> &[usize] {
        &self.read_indices
    }

    fn write_positions(&self) -> &[usize] {
        &self.write_indices
    }

    fn enumerate<F>(&self, context: &mut Self::Context, state: &[usize], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[usize]) -> Result<(), MercError>,
    {
        // Borrow the backend and the next-state scratch buffer as disjoint
        // fields so the enumeration callback can fill the buffer while the
        // backend call is in progress.
        let ExplicitContext {
            context: learn,
            next_state_buf,
            ..
        } = context;

        // The mCRL2 backend wants a thread-local `ATerm` template; protect the
        // shared label term on this thread for the duration of the call.
        let multi_action_template = self.multi_action.as_aterm();

        learn.enumerate_raw_with_current_assignments(
            &self.condition,
            &self.summation_variables,
            &self.write_assignments,
            &multi_action_template,
            |values: &[*const _aterm], multi_action: *const _aterm| {
                debug_assert_eq!(
                    values.len(),
                    self.write_indices.len(),
                    "Enumerated values must match number of write indices"
                );

                // Build the next-state vector in the cached buffer instead of
                // allocating a fresh `Vec` per enumerated transition.
                next_state_buf.clear();
                next_state_buf.extend_from_slice(state);
                for (i, &value) in values.iter().enumerate() {
                    let param_index = self.write_indices[i];
                    // SAFETY: the term is interned into `self.mapping`, a
                    // `Protected` container that keeps every interned term live
                    // through GC marking for as long as the mapping exists.
                    let (new_index, _) = self.mapping.insert(unsafe { DataExpressionRef::from_address(value) });
                    next_state_buf[param_index] = new_index;
                }

                // Wrap the rewritten multi-action term as a typed label. The
                // `ATermSend` protects the term beyond the temporary handed to
                // the callback (via the global send protection set) and aterms
                // are maximally shared, so equal multi-actions share storage and
                // the label is safe to store across threads.
                let label = Mcrl2MultiActionLabel::from_multi_action_term(ATermSend::from_ptr(multi_action));

                // We cannot propagate errors from the C callback, so we panic on error and catch it in the caller.
                report(&label, next_state_buf).expect("Failed to report successor state");
            },
        );

        Ok(())
    }
}

impl fmt::Debug for ExplicitLinearProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ExplicitLinearProcessSpecification:")?;

        writeln!(f, "  Parameters:")?;
        for (i, param) in self.lps.parameters().iter().enumerate() {
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
