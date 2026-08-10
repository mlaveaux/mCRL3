use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use log::debug;
use mcrl2::_aterm;
use mcrl2::DataExpression;
use mcrl2::DataExpressionRef;
use mcrl2::DataSpecification;
use mcrl2::DataVariable;
use mcrl2::Pbes;
use mcrl2::PbesAndRef;
use mcrl2::PbesExpression;
use mcrl2::PbesExpressionRef;
use mcrl2::PbesExpressionVisitor;
use mcrl2::PbesOrRef;
use mcrl2::PbesPropositionalVariableInstantiationRef;
use mcrl2::PbesRewriteContext;
use mcrl2::Protected;
use mcrl2::flatten_pbes_and_into;
use mcrl2::flatten_pbes_or_into;
use mcrl2::is_pbes_and;
use mcrl2::is_pbes_false;
use mcrl2::is_pbes_or;
use mcrl2::is_pbes_propositional_variable_instantiation;
use mcrl2::is_pbes_true;
use mcrl2::is_variable;
use mcrl2::variable_occurrences_data_expression;
use merc_explore::CacheLPS;
use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_unsafety::ConcurrentIndexedSet;
use merc_utilities::MercError;
use merc_utilities::Timing;
use merc_vpg::ParityGame;
use merc_vpg::Player;
use merc_vpg::Priority;

use crate::explore_common::compute_priorities;
use crate::explore_common::explore_pbes_impl;
use crate::explore_common::explore_pbes_parallel_impl;

/// Tag values occupy the top 4 bits of a state[0] word so they never collide
/// with equation indices (which are small) or with usize::MAX (the sequence
/// forest's empty-slot sentinel).
const TAG_MASK: usize = 0xF << (usize::BITS - 4);
/// Source state is a true-sink (self-loop, Even wins).
const TRUE_SINK: usize = 0x1 << (usize::BITS - 4);
/// Source state is a false-sink (self-loop, Odd wins).
const FALSE_SINK: usize = 0x2 << (usize::BITS - 4);
/// Source state is an auxiliary AND node; full state: `[AND_OP, priority, aux_idx]`.
const AND_OP: usize = 0x3 << (usize::BITS - 4);
/// Source state is an auxiliary OR node; full state: `[OR_OP, priority, aux_idx]`.
const OR_OP: usize = 0x4 << (usize::BITS - 4);

type ValueMapping = ConcurrentIndexedSet<DataExpressionRef<'static>>;
type AuxMapping = ConcurrentIndexedSet<PbesExpressionRef<'static>>;

/// Parity-game LPS that directly explores a PBES without converting to SRF.
///
/// Applies the `enumerate_quantifiers_rewriter` on-the-fly to instantiate each
/// equation's right-hand side, then walks the resulting PBES expression to find
/// immediate successor states.  The method is equivalent to `pbesinst_lazy` /
/// the mCRL2 structure-graph algorithm.
///
/// State layout:
/// - PVI state:  `[eq_idx, intern(v0), …, intern(vn)]` — length `1 + num_params`
/// - TRUE sink:  `[TRUE_SINK]`  — length 1
/// - FALSE sink: `[FALSE_SINK]` — length 1
/// - Aux AND:    `[AND_OP, priority, aux_idx]` — length 3
/// - Aux OR:     `[OR_OP,  priority, aux_idx]` — length 3
pub(crate) struct PbesLps {
    /// The unified PBES; retained so the terms borrowed by the summands stay alive.
    _pbes: Pbes,

    /// Data specification used to build each per-thread [`PbesContext`].
    data_spec: DataSpecification,

    /// Flat list of summands: one per equation, plus the sink and aux summands.
    summands: Vec<PbesSummand>,

    /// For each equation index, the indices into [`PbesLps::summands`] of the
    /// summands belonging to that equation. A source state only explores the
    /// summands of its current equation (`state[0]`).
    equation_summands: Vec<Vec<usize>>,

    /// Indices into [`PbesLps::summands`] of the summands fired by a true sink state.
    true_sink_summands: Vec<usize>,

    /// Indices into [`PbesLps::summands`] of the summands fired by a false sink state.
    false_sink_summands: Vec<usize>,

    /// Indices into [`PbesLps::summands`] of the summands fired by an aux node state.
    aux_summands: Vec<usize>,

    /// The initial state vector.
    initial_state: Vec<usize>,

    /// Cached data-parameter variables (length `num_params`). All equations share
    /// the same parameter list after unification.
    process_parameters: Vec<*const _aterm>,

    /// Number of data parameters shared by every equation.
    num_params: usize,

    /// Number of equations in the PBES.
    num_equations: usize,

    /// Maps a propositional variable name to its equation index.
    name_to_eq: Arc<HashMap<String, usize>>,

    /// Interning table for enumerated parameter values, shared with the summands.
    value_mapping: Protected<ValueMapping>,

    /// Interning table for auxiliary (and/or) sub-formulas, shared with the summands.
    aux_mapping: Protected<AuxMapping>,
}

// SAFETY: after construction, PbesLps is immutable except for the two
// ConcurrentIndexedSets (which are thread-safe) and the Pbes (read-only).
unsafe impl Sync for PbesLps {}

/// Determines which state shape a [`PbesSummand`] fires on and how it computes
/// its successors.
enum PbesSummandKind {
    /// Instantiates the right-hand side of an equation for the current parameter
    /// values; `priority` is the parity-game priority of that equation.
    Equation {
        formula: mcrl2::PbesExpression,
        priority: usize,
    },

    /// The true sink, which has itself as its only successor.
    TrueSink,

    /// The false sink, which has itself as its only successor.
    FalseSink,

    /// Expands an interned auxiliary sub-formula into its operands.
    AuxNode,
}

/// A single summand of a [`PbesLps`], pre-bound to the state shape it fires on.
pub(crate) struct PbesSummand {
    /// The state shape this summand fires on and how successors are derived.
    kind: PbesSummandKind,

    /// Handle to the enclosing LPS's value interning, used to intern enumerated
    /// next-state values from any worker thread.
    value_mapping: Arc<ValueMapping>,

    /// Handle to the enclosing LPS's auxiliary formula interning.
    aux_mapping: Arc<AuxMapping>,

    /// Maps a propositional variable name to its equation index.
    name_to_eq: Arc<HashMap<String, usize>>,

    /// Number of data parameters shared by every equation.
    num_params: usize,

    /// Positions of the source state read by this summand.
    read_positions: Vec<usize>,

    /// Positions of the next state written by this summand.
    write_positions: Vec<usize>,
}

/// The lookup tables needed to encode a PBES sub-formula as a parity-game state: equation
/// names to indices, plus the interning tables for parameter values and auxiliary formulas.
#[derive(Clone, Copy)]
struct TargetTables<'a> {
    name_to_eq: &'a HashMap<String, usize>,
    value_mapping: &'a ValueMapping,
    aux_mapping: &'a AuxMapping,
}

/// Per-thread enumeration context for a [`PbesLps`].
pub(crate) struct PbesContext {
    /// The worker's own quantifier-enumerating rewriter.
    rewrite: PbesRewriteContext,

    /// Scratch buffer holding the parameter values of the source state.
    parameter_values: Vec<*const _aterm>,

    /// Scratch buffer for the next state reported to the callback.
    next_state_buf: Vec<usize>,

    /// Scratch buffer for the flattened operands of an and/or chain.
    flat_children: Vec<PbesExpression>,

    /// The instantiated right-hand side of the last explored state.
    psi: Option<mcrl2::PbesExpression>,

    /// The owner and priority of the last explored state, reported by [`LPS::state_info`].
    player_priority: Option<(Player, Priority)>,
}

// SAFETY: PbesContext is owned by exactly one worker thread. The
// PbesRewriteContext wraps a per-worker C++ rewriter that no other thread
// touches.  The raw term pointers in parameter_values are stable addresses
// into the global term pool and are only read, never written.
unsafe impl Send for PbesContext {}

impl PbesLps {
    pub(crate) fn new(mut pbes: Pbes) -> Result<Self, MercError> {
        pbes.unify_parameters(false, true)?;

        let equations = pbes.equations();
        let num_equations = equations.len();
        if num_equations == 0 {
            return Err("PBES has no equations".into());
        }

        let num_params = equations[0].variable().parameters().len();
        let is_mu: Vec<bool> = equations.iter().map(|e| e.is_mu()).collect();
        let priorities = compute_priorities(&is_mu);

        let name_to_eq: HashMap<String, usize> = equations
            .iter()
            .enumerate()
            .map(|(i, eq)| (eq.variable().name().to_string(), i))
            .collect();
        let name_to_eq = Arc::new(name_to_eq);

        // Raw pointers to the unified parameter variables (all equations share
        // the same list after unify_parameters).
        let process_parameters: Vec<*const _aterm> = equations[0]
            .variable()
            .parameters()
            .iter()
            .map(|v: DataVariable| v.address())
            .collect();

        let value_mapping = Protected::new(ValueMapping::new());
        let aux_mapping = Protected::new(AuxMapping::new());
        let data_spec = pbes.data_specification();

        // Building a rewriter normalises the data specification *in place*: it
        // imports the system-defined sorts and appends their constructors, which
        // reallocates vectors of aterms. Doing that from several rayon workers at
        // once corrupts the term-protection sets (the aterms are registered in
        // one thread's pool and relocated by another), so normalise once here on
        // the constructing thread. Afterwards `create_context` only reads the
        // specification and workers can build their rewriters concurrently.
        drop(PbesRewriteContext::from_data_spec(&data_spec));

        let initial_pvi = pbes.initial_state();
        let initial_eq_name = initial_pvi.name().to_string();
        let initial_eq_idx = *name_to_eq
            .get(&initial_eq_name)
            .ok_or_else(|| MercError::from(format!("Unknown initial equation: {initial_eq_name}")))?;

        let mut initial_state = Vec::with_capacity(1 + num_params);
        initial_state.push(initial_eq_idx);
        for arg in initial_pvi.arguments().iter() {
            // SAFETY: the term is interned into `value_mapping`, a `Protected`
            // container that keeps every interned term live through GC marking
            // for as long as the mapping exists.
            let (idx, _) = value_mapping.insert(unsafe { DataExpressionRef::from_address(arg.address()) });
            initial_state.push(idx);
        }

        let true_summand_idx = num_equations;
        let false_summand_idx = num_equations + 1;
        let aux_summand_idx = num_equations + 2;

        let mut summands: Vec<PbesSummand> = Vec::with_capacity(num_equations + 3);

        for (eq_idx, eq) in equations.iter().enumerate() {
            let formula = eq.formula();
            let (read_positions, write_positions) = formula_positions(&formula, &process_parameters);
            summands.push(PbesSummand {
                kind: PbesSummandKind::Equation {
                    formula,
                    priority: priorities[eq_idx],
                },
                value_mapping: value_mapping.handle(),
                aux_mapping: aux_mapping.handle(),
                name_to_eq: name_to_eq.clone(),
                num_params,
                read_positions,
                // Empty write_positions signals full-state caching to CacheLPS
                // (equation summands can produce PVI, sink, or aux-node outputs
                // with different lengths).
                write_positions,
            });
        }

        summands.push(PbesSummand {
            kind: PbesSummandKind::TrueSink,
            value_mapping: value_mapping.handle(),
            aux_mapping: aux_mapping.handle(),
            name_to_eq: name_to_eq.clone(),
            num_params,
            read_positions: vec![0],
            write_positions: vec![0],
        });
        summands.push(PbesSummand {
            kind: PbesSummandKind::FalseSink,
            value_mapping: value_mapping.handle(),
            aux_mapping: aux_mapping.handle(),
            name_to_eq: name_to_eq.clone(),
            num_params,
            read_positions: vec![0],
            write_positions: vec![0],
        });
        summands.push(PbesSummand {
            kind: PbesSummandKind::AuxNode,
            value_mapping: value_mapping.handle(),
            aux_mapping: aux_mapping.handle(),
            name_to_eq: name_to_eq.clone(),
            num_params,
            read_positions: vec![0, 1, 2],
            // Empty: aux→PVI/sink transitions change all positions.
            write_positions: vec![],
        });

        let equation_summands: Vec<Vec<usize>> = (0..num_equations).map(|i| vec![i]).collect();

        Ok(PbesLps {
            _pbes: pbes,
            data_spec,
            summands,
            equation_summands,
            true_sink_summands: vec![true_summand_idx],
            false_sink_summands: vec![false_summand_idx],
            aux_summands: vec![aux_summand_idx],
            initial_state,
            process_parameters,
            num_params,
            num_equations,
            name_to_eq,
            value_mapping,
            aux_mapping,
        })
    }
    pub(crate) fn num_params(&self) -> usize {
        self.num_params
    }
}

impl LPS for PbesLps {
    type Value = usize;
    type Label = ();
    type StateInfo = (Player, Priority);
    type Summand = PbesSummand;

    fn initial_state(&self) -> Vec<usize> {
        self.initial_state.clone()
    }

    fn summands(&self) -> &[PbesSummand] {
        &self.summands
    }

    fn create_context(&self) -> PbesContext {
        PbesContext {
            rewrite: PbesRewriteContext::from_data_spec(&self.data_spec),
            parameter_values: Vec::with_capacity(self.num_params),
            next_state_buf: Vec::new(),
            flat_children: Vec::new(),
            psi: None,
            player_priority: None,
        }
    }

    fn prepare<'a>(&'a self, context: &mut PbesContext, state: &'a [usize]) -> impl Iterator<Item = usize> + 'a {
        let tag = state[0] & TAG_MASK;
        if tag == TRUE_SINK {
            self.true_sink_summands.iter().copied()
        } else if tag == FALSE_SINK {
            self.false_sink_summands.iter().copied()
        } else if tag == AND_OP || tag == OR_OP {
            self.aux_summands.iter().copied()
        } else {
            debug_assert!(tag == 0, "unexpected state tag {tag:#x}");
            let eq_idx = state[0];

            context.parameter_values.clear();
            for &vi in &state[1..=self.num_params] {
                context.parameter_values.push(
                    self.value_mapping
                        .get_by_index(vi)
                        .expect("parameter value must be in mapping")
                        .address(),
                );
            }

            // SAFETY: `process_parameters` and `parameter_values` are live term
            // pointers from the global pool; the rewriter produces a protected
            // result that is immediately stored in `context.psi`.
            let psi = unsafe {
                context
                    .rewrite
                    .set_assignments(&self.process_parameters, &context.parameter_values);
                context
                    .rewrite
                    .rewrite_formula(self.summands[eq_idx].formula().unwrap())
            };

            let priority = self.summands[eq_idx].priority().unwrap();
            let player = player_of(&psi);
            context.psi = Some(psi);
            context.player_priority = Some((player, Priority::new(priority)));

            self.equation_summands[eq_idx].iter().copied()
        }
    }

    fn state_info(&self, state: &[usize], context: &PbesContext) -> (Player, Priority) {
        let tag = state[0] & TAG_MASK;
        if tag == TRUE_SINK {
            (Player::Even, Priority::new(0))
        } else if tag == FALSE_SINK {
            (Player::Odd, Priority::new(1))
        } else if tag == AND_OP {
            (Player::Odd, Priority::new(state[1]))
        } else if tag == OR_OP {
            (Player::Even, Priority::new(state[1]))
        } else {
            context
                .player_priority
                .expect("prepare must be called before state_info")
        }
    }
}

impl PbesSummand {
    fn formula(&self) -> Option<&mcrl2::PbesExpression> {
        match &self.kind {
            PbesSummandKind::Equation { formula, .. } => Some(formula),
            _ => None,
        }
    }

    fn priority(&self) -> Option<usize> {
        match &self.kind {
            PbesSummandKind::Equation { priority, .. } => Some(*priority),
            _ => None,
        }
    }

    fn tables(&self) -> TargetTables<'_> {
        TargetTables {
            name_to_eq: &self.name_to_eq,
            value_mapping: &self.value_mapping,
            aux_mapping: &self.aux_mapping,
        }
    }
}

impl Summand for PbesSummand {
    type Value = usize;
    type Label = ();
    type Context = PbesContext;

    fn read_positions(&self) -> &[usize] {
        &self.read_positions
    }

    fn write_positions(&self) -> &[usize] {
        &self.write_positions
    }

    fn enumerate<F>(&self, context: &mut PbesContext, state: &[usize], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&(), &[usize]) -> Result<(), MercError>,
    {
        match &self.kind {
            PbesSummandKind::TrueSink => {
                context.next_state_buf.clear();
                context.next_state_buf.push(TRUE_SINK);
                report(&(), &context.next_state_buf)
            }
            PbesSummandKind::FalseSink => {
                context.next_state_buf.clear();
                context.next_state_buf.push(FALSE_SINK);
                report(&(), &context.next_state_buf)
            }
            PbesSummandKind::Equation { priority, .. } => {
                let psi = context.psi.as_ref().expect("prepare must precede enumerate");
                let formula = psi.copy();
                // Split borrows: formula from context.psi, buf and children_buf from separate fields.
                let priority = *priority;
                let buf = &mut context.next_state_buf;
                let children_buf = &mut context.flat_children;
                enumerate_formula_children(formula, priority, self.tables(), children_buf, buf, &mut report)
            }
            PbesSummandKind::AuxNode => {
                let aux_idx = state[2];
                let priority = state[1];
                let pbes_ref = self.aux_mapping.get_by_index(aux_idx).expect("aux index must be valid");
                let formula = pbes_ref.copy();
                let buf = &mut context.next_state_buf;
                let children_buf = &mut context.flat_children;
                enumerate_formula_children(formula, priority, self.tables(), children_buf, buf, &mut report)
            }
        }
    }
}

/// Returns the player for the given top-level PBES formula.
fn player_of(psi: &mcrl2::PbesExpression) -> Player {
    let r = psi.copy();
    if is_pbes_and(&r) || is_pbes_false(&r) {
        Player::Odd
    } else {
        // OR, PVI, true — player Even (existential / single outgoing edge)
        Player::Even
    }
}

/// Emits successor states for the given PBES formula.
///
/// AND/OR chains are flattened (like mCRL2's `split_and`/`split_or`) so that
/// `(A && B) && (C && D)` emits 4 direct edges instead of 2 aux nodes.
/// Cross-operator nesting still produces aux nodes via [`emit_as_target`].
/// `children_buf` is a reusable scratch buffer owned by the per-thread context.
fn enumerate_formula_children<F>(
    formula: PbesExpressionRef<'_>,
    priority: usize,
    tables: TargetTables<'_>,
    children_buf: &mut Vec<PbesExpression>,
    buf: &mut Vec<usize>,
    report: &mut F,
) -> Result<(), MercError>
where
    F: FnMut(&(), &[usize]) -> Result<(), MercError>,
{
    children_buf.clear();
    if is_pbes_and(&formula.copy()) {
        flatten_pbes_and_into(formula, children_buf);
    } else if is_pbes_or(&formula.copy()) {
        flatten_pbes_or_into(formula, children_buf);
    } else {
        // Single leaf (PVI, true, false): emit directly without buffering.
        return emit_as_target(formula, priority, tables, buf, report);
    }
    for child in children_buf.drain(..) {
        emit_as_target(child.copy(), priority, tables, buf, report)?;
    }
    Ok(())
}

/// Emits a transition to the parity-game state corresponding to `expr`.
///
/// PVI → concrete PVI state `[eq_idx, arg0, …]`.
/// AND/OR sub-formula → aux node `[AUX_AND/OR, priority, aux_idx]`.
/// `true` / `false` → respective sink state.
fn emit_as_target<F>(
    expr: PbesExpressionRef<'_>,
    priority: usize,
    tables: TargetTables<'_>,
    buf: &mut Vec<usize>,
    report: &mut F,
) -> Result<(), MercError>
where
    F: FnMut(&(), &[usize]) -> Result<(), MercError>,
{
    if is_pbes_propositional_variable_instantiation(&expr.copy()) {
        let pvi = PbesPropositionalVariableInstantiationRef::from(expr);
        let target_eq = *tables
            .name_to_eq
            .get(&pvi.name().to_string())
            .ok_or_else(|| MercError::from("Unknown equation name in PVI"))?;
        buf.clear();
        buf.push(target_eq);
        for arg in pvi.arguments().iter() {
            // SAFETY: term interned into the Protected value_mapping.
            let (idx, _) = tables
                .value_mapping
                .insert(unsafe { DataExpressionRef::from_address(arg.address()) });
            buf.push(idx);
        }
        report(&(), buf)
    } else if is_pbes_and(&expr.copy()) {
        // SAFETY: term is a sub-expression of the rewritten psi still in context;
        // the aux_mapping (Protected) keeps it alive via GC marking.
        let (aux_idx, _) = tables
            .aux_mapping
            .insert(unsafe { PbesExpressionRef::from_address(expr.address()) });
        buf.clear();
        buf.extend([AND_OP, priority, aux_idx]);
        report(&(), buf)
    } else if is_pbes_or(&expr.copy()) {
        // SAFETY: term is a sub-expression of the rewritten psi still in context;
        // the aux_mapping (Protected) keeps it alive via GC marking.
        let (aux_idx, _) = tables
            .aux_mapping
            .insert(unsafe { PbesExpressionRef::from_address(expr.address()) });
        buf.clear();
        buf.extend([OR_OP, priority, aux_idx]);
        report(&(), buf)
    } else if is_pbes_true(&expr.copy()) {
        buf.clear();
        buf.push(TRUE_SINK);
        report(&(), buf)
    } else if is_pbes_false(&expr.copy()) {
        buf.clear();
        buf.push(FALSE_SINK);
        report(&(), buf)
    } else {
        Err(MercError::from(format!(
            "Unexpected PBES formula shape after rewriting: {}",
            expr.copy()
        )))
    }
}

/// Builds a [`ParityGame`] by exploring the given PBES directly (no SRF conversion).
pub(crate) fn explore_pbes(
    pbes: Pbes,
    strategy: ExplorationStrategy,
    caching: CachingStrategy,
) -> Result<ParityGame, MercError> {
    let lps = PbesLps::new(pbes)?;
    let timing = Timing::new();
    match caching {
        CachingStrategy::None => explore_pbes_impl(&lps, strategy, &timing),
        _ => {
            let cached = CacheLPS::new(&lps, caching);
            let game = explore_pbes_impl(&cached, strategy, &timing)?;
            debug!("{}", cached.metrics());
            Ok(game)
        }
    }
}

/// Builds a [`ParityGame`] by exploring the given PBES directly in parallel.
pub(crate) fn explore_pbes_parallel(
    pbes: Pbes,
    threads: usize,
    caching: CachingStrategy,
    pinned: bool,
) -> Result<ParityGame, MercError> {
    let lps = PbesLps::new(pbes)?;
    match caching {
        CachingStrategy::None => explore_pbes_parallel_impl(&lps, threads, pinned),
        _ => {
            let cached = CacheLPS::new(&lps, caching);
            let game = explore_pbes_parallel_impl(&cached, threads, pinned)?;
            debug!("{}", cached.metrics());
            Ok(game)
        }
    }
}

/// Computes read and write position vectors for an equation summand.
///
/// `read_positions`: `{0}` ∪ `{k+1 | param[k]` appears anywhere in `formula`}.
/// `write_positions`: `{0}` ∪ `{k+1 | some PVI argument at position k is not identity}`.
/// Returns empty `write_positions` when the formula contains AND/OR, signalling
/// `CacheLPS` to use full-state capture (equation summands can produce outputs of
/// different lengths: PVI, sink, or aux-node states).
fn formula_positions(formula: &mcrl2::PbesExpression, params: &[*const _aterm]) -> (Vec<usize>, Vec<usize>) {
    // Single-pass visitor: collect read-variable addresses and write-position
    // mask simultaneously, visiting each PVI argument exactly once.
    struct FormulaPositions<'p> {
        var_addrs: HashSet<*const _aterm>,
        write_mask: Vec<bool>,
        has_aux_outputs: bool,
        params: &'p [*const _aterm],
    }

    impl PbesExpressionVisitor for FormulaPositions<'_> {
        fn visit_and(&mut self, and: &PbesAndRef<'_>) -> Option<mcrl2::PbesExpression> {
            self.has_aux_outputs = true;
            self.visit(&and.lhs());
            self.visit(&and.rhs());
            None
        }

        fn visit_or(&mut self, or: &PbesOrRef<'_>) -> Option<mcrl2::PbesExpression> {
            self.has_aux_outputs = true;
            self.visit(&or.lhs());
            self.visit(&or.rhs());
            None
        }

        fn visit_propositional_variable_instantiation(
            &mut self,
            inst: &PbesPropositionalVariableInstantiationRef<'_>,
        ) -> Option<mcrl2::PbesExpression> {
            for (k, (arg, &param_addr)) in inst.arguments().iter().zip(self.params.iter()).enumerate() {
                // Skip identity pass-throughs: the replay already handles them,
                // so they don't need to be part of the cache key.
                if is_variable(&arg.copy()) && arg.address() == param_addr {
                    continue;
                }
                for v in variable_occurrences_data_expression(&arg.copy()) {
                    self.var_addrs.insert(v.address());
                }
                self.write_mask[k] = true;
            }
            None
        }

        fn visit_data_expression(&mut self, expr: &DataExpressionRef<'_>) -> Option<DataExpression> {
            for v in variable_occurrences_data_expression(expr) {
                self.var_addrs.insert(v.address());
            }
            None
        }
    }

    let mut collector = FormulaPositions {
        var_addrs: HashSet::new(),
        write_mask: vec![false; params.len()],
        has_aux_outputs: false,
        params,
    };
    collector.visit(&formula.copy());

    // Position 0 (eq_idx) is always read/written.
    let mut read_positions = vec![0usize];
    for (k, &param_addr) in params.iter().enumerate() {
        if collector.var_addrs.contains(&param_addr) {
            read_positions.push(k + 1);
        }
    }

    // Empty write_positions signals CacheLPS to use full-state capture.
    let write_positions = if collector.has_aux_outputs {
        vec![]
    } else {
        let mut write_positions = vec![0usize];
        for (k, &written) in collector.write_mask.iter().enumerate() {
            if written {
                write_positions.push(k + 1);
            }
        }
        write_positions
    };

    (read_positions, write_positions)
}
