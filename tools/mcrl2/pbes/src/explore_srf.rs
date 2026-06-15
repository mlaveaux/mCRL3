use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;

use log::debug;
use log::info;

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
use mcrl2::Protected;
use mcrl2::SrfPbes;
use mcrl2::free_variables_data_expression;
use mcrl2::make_data_assignment_list;
use mcrl2::tau_multi_action;
use merc_explore::CacheLPS;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_explore::explore;
use merc_explore::explore_parallel;
use merc_io::TimeProgress;
use merc_lts::StateIndex;
use merc_unsafety::ConcurrentIndexedSet;
use merc_utilities::MercError;
use merc_utilities::Timing;
use merc_vpg::ParityGame;
use merc_vpg::ParityGameBuilder;
use merc_vpg::Player;
use merc_vpg::Priority;
use merc_vpg::VertexIndex;

/// Periodic progress reporter for PBES exploration. A PBES state is a BES
/// equation (parity-game vertex), so the count is reported as BES equations.
fn bes_progress() -> TimeProgress<(usize, usize)> {
    TimeProgress::new(
        |(equations, edges): (usize, usize)| {
            info!("Explored {equations} BES equations, {edges} edges...");
        },
        1,
    )
}

/// Builds a [`ParityGame`] by exploring the given PBES in SRF format.
pub fn parity_game_from_pbes(
    pbes: &Pbes,
    strategy: ExplorationStrategy,
    caching: CachingStrategy,
) -> Result<ParityGame, MercError> {
    let lps = PbesSrfLps::new(pbes)?;
    let timing = Timing::new();

    let mut builder = ParityGameBuilder::new(VertexIndex::new(0));

    // Count BES equations (vertices) and edges in the exploration closures,
    // driving the periodic progress reporter from `on_transition`.
    let progress = bes_progress();
    let equations = Cell::new(0usize);
    let edges = Cell::new(0usize);

    let cached = CacheLPS::new(lps, caching);
    let _initial = explore(
        &cached,
        strategy,
        &timing,
        &mut builder,
        |b: &mut ParityGameBuilder, state: StateIndex, info: &(Player, Priority)| {
            equations.set(equations.get() + 1);
            b.add_vertex(VertexIndex::new(state.value()), info.0, info.1);
            Ok(())
        },
        |b: &mut ParityGameBuilder, from: StateIndex, _label: &(), to: StateIndex| {
            edges.set(edges.get() + 1);
            progress.print((equations.get(), edges.get()));
            b.add_edge(VertexIndex::new(from.value()), VertexIndex::new(to.value()));
            Ok(())
        },
    )?;
    info!(
        "Exploration complete: {} BES equations, {} edges",
        equations.get(),
        edges.get(),
    );

    debug!("{}", cached.metrics());
    Ok(builder.finish(true, true))
}

/// Per-worker output partition for [`parity_game_from_pbes_parallel`].
///
/// Holds the vertices and edges discovered by one worker. All values are dense
/// `usize`-backed indices (and plain `Player`/`Priority`), so the partition is
/// `Send` and the partitions merge by concatenation without any remapping.
#[derive(Default)]
struct PbesPartition {
    vertices: Vec<(VertexIndex, Player, Priority)>,
    edges: Vec<(VertexIndex, VertexIndex)>,
}

/// Builds a [`ParityGame`] by exploring the given PBES in SRF format in parallel
/// across `threads` worker threads.
///
/// Uses the level-synchronised parallel BFS of [`explore_parallel`]. The PBES has
/// no action labels, so each worker only records `usize`-backed vertices and
/// edges; the shared discovered set assigns globally consistent vertex indices,
/// so the per-worker partitions merge by concatenation.
pub fn parity_game_from_pbes_parallel(pbes: &Pbes, threads: usize) -> Result<ParityGame, MercError> {
    let lps = PbesSrfLps::new(pbes)?;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|err| MercError::from(format!("Failed to build thread pool: {err}")))?;

    let (_initial, partitions) = pool.install(|| {
        let timing = Timing::new();
        explore_parallel(
            &lps,
            &timing,
            PbesPartition::default,
            |partition: &mut PbesPartition, state: StateIndex, info: &(Player, Priority)| {
                partition.vertices.push((VertexIndex::new(state.value()), info.0, info.1));
                Ok(())
            },
            |partition: &mut PbesPartition, from: StateIndex, _label: &(), to: StateIndex| {
                partition
                    .edges
                    .push((VertexIndex::new(from.value()), VertexIndex::new(to.value())));
                Ok(())
            },
        )
    })?;

    // The per-thread progress trackers cannot be shared (`TimeProgress` is
    // `!Sync`), so the parallel path reports only the final summary, counted
    // from the merged partitions.
    let total_equations: usize = partitions.iter().map(|p| p.vertices.len()).sum();
    let total_edges: usize = partitions.iter().map(|p| p.edges.len()).sum();
    info!("Exploration complete: {total_equations} BES equations, {total_edges} edges");

    // Merge the per-worker partitions: every state is reported to `on_state`
    // exactly once, so each vertex is added once. Add all vertices before edges
    // since an edge may target a vertex discovered by another worker.
    let mut builder = ParityGameBuilder::new(VertexIndex::new(0));
    for partition in &partitions {
        for &(vertex, player, priority) in &partition.vertices {
            builder.add_vertex(vertex, player, priority);
        }
    }
    for partition in &partitions {
        for &(from, to) in &partition.edges {
            builder.add_edge(from, to);
        }
    }
    Ok(builder.finish(true, true))
}

/// Per-thread enumeration context for a [`PbesSrfLps`].
///
/// Owns the mCRL2 enumeration backend and the reusable scratch buffers, so the
/// LPS and its summands stay immutable and shareable by `&self` while each
/// worker thread drives its own context.
pub struct PbesSrfContext {
    /// Backend used to evaluate summand conditions and enumerate solutions,
    /// staged per source state by [`LPS::prepare`].
    context: LearnSuccessorsContext,

    /// Reusable scratch buffer holding the resolved parameter pointers for the
    /// current source state. Filled during [`LPS::prepare`] and consumed
    /// immediately by [`LearnSuccessorsContext::set_assignments`].
    parameter_values: Vec<*const _aterm>,

    /// Reusable next-state buffer, pre-sized to `1 + num_params` and fully
    /// overwritten for every enumerated solution.
    next_state_buf: Vec<usize>,
}

// SAFETY: a `PbesSrfContext` is owned by exactly one worker thread, which both
// creates and uses it. `parameter_values` is transient scratch holding stable,
// maximally shared term addresses (not protected `ATerm`s), and the
// `LearnSuccessorsContext` wraps a per-worker mCRL2 enumerator that no other
// thread touches. mCRL2 is built with multithreading enabled and its garbage
// collection is stop-the-world, so moving the context between threads cannot
// race with collection or with another worker.
unsafe impl Send for PbesSrfContext {}

/// Explicit-state view of a PBES in SRF normal form.
///
/// State vectors have layout `[equation_index, param_0, …, param_{n-1}]` where
/// `equation_index` is a flat index into [`SrfPbes::equations`] and each
/// `param_i` is an index into the shared [`ValueMapping`].
pub struct PbesSrfLps {
    /// The unified SRF PBES; retained so summand pointers stay alive.
    _srf: SrfPbes,

    /// Data specification used to build each per-thread [`PbesSrfContext`].
    data_spec: DataSpecification,

    /// Flat list of summands, one per `(equation, srf_summand)` pair.
    summands: Vec<PbesSrfSummand>,

    /// The initial state vector.
    initial_state: Vec<usize>,

    /// Per-equation (Player, Priority) used by [`LPS::state_info`].
    state_info: Vec<(Player, Priority)>,

    /// Cached data-parameter variables (length `num_params`). All equations
    /// share the same parameter list after [`SrfPbes::unify_parameters`].
    process_parameters: Vec<*const _aterm>,

    /// Number of data parameters per equation.
    num_params: usize,

    /// Concurrent value interning shared with every summand. Owned here through
    /// [`Protected`] so the garbage-collection protection is released when the
    /// LPS is dropped.
    value_mapping: Protected<ValueMapping>,
}

/// Shared interning of the data expressions observed as parameter values,
/// mapping each distinct expression to a dense `usize` used in state vectors.
type ValueMapping = ConcurrentIndexedSet<DataExpressionRef<'static>>;

// SAFETY: after construction the LPS is immutable. Every `&self` method only
// reads the cached terms (mCRL2 terms are immutable and `ATermRef`s are
// `Send`/`Sync`) or interns into `value_mapping`, whose backing
// `ConcurrentIndexedSet` is itself thread-safe. The raw `*const _aterm`
// parameter pointers are stable maximally shared term addresses, and mCRL2
// garbage collection is stop-the-world, so no `&self` access can race with
// collection. The container's protection root stays on the constructing thread
// (`Protected` is `!Send`), which is why only `Sync` is asserted, not `Send`.
unsafe impl Sync for PbesSrfLps {}

/// A single SRF summand, pre-bound to the equation it belongs to and the
/// target equation it transitions into.
pub struct PbesSrfSummand {
    /// Source equation index; the summand fires only when `state[0]` equals it.
    equation_index: usize,

    /// Target equation index written to position 0 of the next state.
    target_equation_index: usize,

    /// The summand's data condition (after SRF conversion).
    condition: DataExpression,

    /// Existential summation variables of the summand.
    summation_variables: ATermList<DataVariable>,

    /// Pre-built assignment list `params := target_args(summation_vars, params)`
    /// passed verbatim to the enumerator.
    write_assignments: ATermList<ATerm>,

    /// Handle to the enclosing LPS's value interning, used to intern enumerated
    /// next-state values from any worker thread.
    mapping: Arc<ValueMapping>,

    /// Cached tau multi-action term; protected once at construction instead of
    /// on every summand enumeration.
    tau: ATerm,

    /// Number of data parameters, used to size the next-state buffer.
    num_params: usize,

    /// Positions in the state vector that fully determine this summand's
    /// enumeration result; position 0 (equation index) is always included.
    read_positions: Vec<usize>,

    /// Positions in the state vector that this summand may change; position 0
    /// (equation index) is always included.
    write_positions: Vec<usize>,
}

impl PbesSrfLps {
    /// Constructs a new [`PbesSrfLps`] from a PBES by normalising it to SRF and
    /// unifying the parameter lists.
    pub fn new(pbes: &Pbes) -> Result<Self, MercError> {
        let mut srf = SrfPbes::from(pbes)?;
        srf.unify_parameters(false, true)?;

        if srf.equations().is_empty() {
            return Err("PBES has no equations".into());
        }

        let num_params = srf.equations()[0].variable().parameters().len();
        let priorities = compute_priorities(&srf);

        // Equation name -> equation index, used when resolving target PVIs.
        let name_to_eq: HashMap<String, usize> = srf
            .equations()
            .iter()
            .enumerate()
            .map(|(i, eq)| (eq.variable().name().to_string(), i))
            .collect();

        // (Player, Priority) per equation. PBES convention: conjunctive (∧)
        // is owned by ∀ (Odd), disjunctive (∨) is owned by ∃ (Even).
        let state_info: Vec<(Player, Priority)> = srf
            .equations()
            .iter()
            .enumerate()
            .map(|(i, eq)| {
                let player = if eq.is_conjunctive() { Player::Odd } else { Player::Even };
                (player, Priority::new(priorities[i]))
            })
            .collect();

        // Cached parameter-variable pointers. After `unify_parameters` all
        // equations share the same list, so we take the one of equation 0.
        let process_parameters: Vec<*const _aterm> = srf.equations()[0]
            .variable()
            .parameters()
            .iter()
            .map(|v: DataVariable| v.address())
            .collect();

        // Shared value interning, kept alive as a garbage-collection container
        // by `value_mapping`.
        let value_mapping = Protected::new(ValueMapping::new());

        let data_spec = pbes.data_specification();
        let tau = tau_multi_action();

        // Build the initial state vector from the initial PVI.
        // After `unify_parameters`, the SRF's initial PVI has `num_params`
        // arguments; the original `pbes.initial_state()` may have fewer.
        // We use `srf.initial_state()` directly to avoid `srf.to_pbes()`,
        // which constructs a PBES object with an empty global-variable set and
        // triggers a well-typedness assertion when `unify_parameters` has
        // introduced dc* don't-care variables that aren't declared globally.
        let initial_pvi = srf.initial_state();
        let initial_eq_name = initial_pvi.name().to_string();
        let initial_eq_idx = *name_to_eq
            .get(&initial_eq_name)
            .ok_or_else(|| MercError::from(format!("Unknown initial equation: {initial_eq_name}")))?;

        let mut initial_state = Vec::with_capacity(1 + num_params);
        initial_state.push(initial_eq_idx);
        for arg in initial_pvi.arguments().iter() {
            let (idx, _) = value_mapping.insert(DataExpressionRef::from_address(arg.address()));
            initial_state.push(idx);
        }

        // Flatten (equation, srf_summand) pairs into a single summand list.
        let mut summands = Vec::new();
        for (eq_idx, eq) in srf.equations().iter().enumerate() {
            // The parameters list of this equation (LHS of the assignment list).
            let eq_param_term: ATerm = eq.variable().parameters().into();
            let params_vec: Vec<DataVariable> = eq.variable().parameters().iter().collect();

            for srf_summand in eq.summands() {
                let target_pvi: PbesPropositionalVariableInstantiation = srf_summand.variable().into();
                let target_eq_name = target_pvi.name().to_string();
                let target_eq_idx = *name_to_eq
                    .get(&target_eq_name)
                    .ok_or_else(|| MercError::from(format!("Unknown target equation: {target_eq_name}")))?;

                let target_args: ATerm = target_pvi.arguments().protect().into();
                let assignments_term = make_data_assignment_list(&eq_param_term, &target_args);
                let write_assignments: ATermList<ATerm> = ATermList::new(assignments_term);

                // Collect free variables from the condition.
                let condition_de: DataExpression = srf_summand.condition().into();
                let mut read_vars = free_variables_data_expression(&condition_de.copy());

                // Position 0 (equation index) is always read; extend with positions
                // whose parameter appears free in the condition or any target argument.
                // Also compute write positions: position 0 is always written; position
                // k+1 is written iff the k-th target argument is not the identity (param_k).
                let mut read_positions = vec![0usize];
                let mut write_positions = vec![0usize];
                for (k, (param, arg)) in params_vec.iter().zip(target_pvi.arguments().iter()).enumerate() {
                    read_vars.extend(free_variables_data_expression(&arg.copy()));
                    if Into::<DataExpressionRef<'_>>::into(param.copy()) != arg.copy() {
                        write_positions.push(k + 1);
                    }
                }
                for (i, param) in params_vec.iter().enumerate() {
                    if read_vars.contains(param) {
                        read_positions.push(i + 1);
                    }
                }

                summands.push(PbesSrfSummand {
                    equation_index: eq_idx,
                    target_equation_index: target_eq_idx,
                    condition: srf_summand.condition().into(),
                    summation_variables: srf_summand.parameters(),
                    write_assignments,
                    mapping: value_mapping.handle(),
                    tau: tau.clone(),
                    num_params,
                    read_positions,
                    write_positions,
                });
            }
        }

        Ok(Self {
            _srf: srf,
            data_spec,
            summands,
            initial_state,
            state_info,
            process_parameters,
            num_params,
            value_mapping,
        })
    }
}

impl LPS for PbesSrfLps {
    type Value = usize;
    type Label = ();
    type StateInfo = (Player, Priority);
    type Summand = PbesSrfSummand;

    fn initial_state(&self) -> Vec<usize> {
        self.initial_state.clone()
    }

    fn summands(&self) -> &[Self::Summand] {
        &self.summands
    }

    fn create_context(&self) -> PbesSrfContext {
        PbesSrfContext {
            context: LearnSuccessorsContext::from_data_spec(&self.data_spec),
            parameter_values: Vec::with_capacity(self.num_params),
            next_state_buf: vec![0; 1 + self.num_params],
        }
    }

    fn prepare(&self, context: &mut PbesSrfContext, state: &[Self::Value]) {
        debug_assert_eq!(
            state.len(),
            1 + self.num_params,
            "State vector length must match 1 + number of parameters"
        );

        // Look up the *const _aterm representative for each parameter value.
        context.parameter_values.clear();
        for &value_index in state.iter().skip(1) {
            context.parameter_values.push(
                self.value_mapping
                    .get_by_index(value_index)
                    .expect("Parameter value must be in mapping")
                    .address(),
            );
        }

        context
            .context
            .set_assignments(&self.process_parameters, &context.parameter_values);
    }

    fn state_info(&self, state: &[Self::Value]) -> Self::StateInfo {
        self.state_info[state[0]]
    }
}

impl Summand for PbesSrfSummand {
    type Value = usize;
    type Label = ();
    type Context = PbesSrfContext;

    fn read_positions(&self) -> &[usize] {
        &self.read_positions
    }

    fn write_positions(&self) -> &[usize] {
        &self.write_positions
    }

    fn enumerate<F>(&self, context: &mut Self::Context, state: &[usize], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[usize]) -> Result<(), MercError>,
    {
        // PBES summands only fire from their owning equation.
        if state[0] != self.equation_index {
            return Ok(());
        }

        // Borrow the backend and the next-state scratch buffer as disjoint
        // fields so the enumeration callback can fill the buffer while the
        // backend call is in progress.
        let PbesSrfContext {
            context: learn,
            next_state_buf,
            ..
        } = context;

        learn.enumerate_raw_with_current_assignments(
            &self.condition,
            &self.summation_variables,
            &self.write_assignments,
            &self.tau,
            |next_values: &[*const _aterm], _multi_action| {
                debug_assert_eq!(
                    next_values.len(),
                    self.num_params,
                    "Enumerated values must match number of parameters"
                );

                next_state_buf[0] = self.target_equation_index;
                for (i, &ptr) in next_values.iter().enumerate() {
                    let (idx, _) = self.mapping.insert(DataExpressionRef::from_address(ptr));
                    next_state_buf[1 + i] = idx;
                }

                // The PBES has no actions; we pass a placeholder unit label.
                report(&(), next_state_buf).expect("Failed to report PBES transition");
            },
        );

        Ok(())
    }
}

/// Computes a priority for each equation for a **max** parity game.
///
/// Algorithm:
/// 1. Assign each equation an *alternation depth* (incremented on every
///    μ ↔ ν switch), so the outermost block has depth 0 and the innermost
///    has depth `max_depth`.
/// 2. Reverse: `priority = max_depth − depth`, making the outermost block
///    the highest-priority block.
/// 3. Shift all priorities by 1 when the outermost equation's current parity
///    does not match its fixpoint type (ν → even, μ → odd).
fn compute_priorities(srf: &SrfPbes) -> Vec<usize> {
    let equations = srf.equations();
    if equations.is_empty() {
        return Vec::new();
    }

    // Step 1: compute alternation depth per equation.
    let mut depths = vec![0usize; equations.len()];
    let mut current_depth = 0usize;
    let mut prev_is_mu = equations[0].is_mu();

    for (i, eq) in equations.iter().enumerate() {
        let is_mu = eq.is_mu();
        if i > 0 && is_mu != prev_is_mu {
            current_depth += 1;
        }
        depths[i] = current_depth;
        prev_is_mu = is_mu;
    }

    // Step 2: reverse so outermost (depth 0) → highest priority (max_depth).
    let max_depth = *depths.last().unwrap();
    let mut priorities: Vec<usize> = depths.iter().map(|&d| max_depth - d).collect();

    // Step 3: shift all priorities by 1 iff the outermost equation's priority
    // parity does not match its fixpoint type (ν needs even, μ needs odd).
    let first_is_mu = equations[0].is_mu();
    if first_is_mu == priorities[0].is_multiple_of(2) {
        for p in &mut priorities {
            *p += 1;
        }
    }

    debug_assert!(
        priorities
            .iter()
            .zip(equations.iter())
            .all(|(p, eq)| p.is_multiple_of(2) != eq.is_mu()),
        "Max parity game invariant violated: ν must have even priority and μ must have odd priority"
    );

    priorities
}
