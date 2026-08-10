use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use mcrl2::DataExpression;
use mcrl2::DataExpressionRef;
use mcrl2::DataSpecification;
use mcrl2::DataVariable;
use mcrl2::Pbes;
use mcrl2::PbesAndRef;
use mcrl2::PbesExpressionRef;
use mcrl2::PbesExpressionVisitor;
use mcrl2::PbesOrRef;
use mcrl2::PbesPropositionalVariableInstantiationRef;
use mcrl2::PbesRewriteContext;
use mcrl2::Protected;
use mcrl2::_aterm;
use mcrl2::is_pbes_and;
use mcrl2::is_pbes_false;
use mcrl2::is_pbes_or;
use mcrl2::is_pbes_propositional_variable_instantiation;
use mcrl2::is_pbes_true;
use mcrl2::is_variable;
use mcrl2::variable_occurrences_data_expression;
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
    _pbes: Pbes,
    data_spec: DataSpecification,
    summands: Vec<PbesSummand>,
    equation_summands: Vec<Vec<usize>>,
    true_sink_summands: Vec<usize>,
    false_sink_summands: Vec<usize>,
    aux_summands: Vec<usize>,
    initial_state: Vec<usize>,
    process_parameters: Vec<*const _aterm>,
    num_params: usize,
    num_equations: usize,
    name_to_eq: Arc<HashMap<String, usize>>,
    value_mapping: Protected<ValueMapping>,
    aux_mapping: Protected<AuxMapping>,
}

// SAFETY: after construction, PbesLps is immutable except for the two
// ConcurrentIndexedSets (which are thread-safe) and the Pbes (read-only).
unsafe impl Sync for PbesLps {}

enum PbesSummandKind {
    Equation { formula: mcrl2::PbesExpression, priority: usize },
    TrueSink,
    FalseSink,
    AuxNode,
}

pub(crate) struct PbesSummand {
    kind: PbesSummandKind,
    value_mapping: Arc<ValueMapping>,
    aux_mapping: Arc<AuxMapping>,
    name_to_eq: Arc<HashMap<String, usize>>,
    num_params: usize,
    read_positions: Vec<usize>,
    write_positions: Vec<usize>,
}

pub(crate) struct PbesContext {
    rewrite: PbesRewriteContext,
    parameter_values: Vec<*const _aterm>,
    next_state_buf: Vec<usize>,
    psi: Option<mcrl2::PbesExpression>,
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
            let (idx, _) =
                value_mapping.insert(unsafe { DataExpressionRef::from_address(arg.address()) });
            initial_state.push(idx);
        }

        let true_summand_idx = num_equations;
        let false_summand_idx = num_equations + 1;
        let aux_summand_idx = num_equations + 2;

        let mut summands: Vec<PbesSummand> = Vec::with_capacity(num_equations + 3);

        for (eq_idx, eq) in equations.iter().enumerate() {
            let formula = eq.formula();
            let (read_positions, write_positions) =
                formula_positions(&formula, &process_parameters);
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
            write_positions: vec![0, 1, 2],
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
            psi: None,
            player_priority: None,
        }
    }

    fn prepare<'a>(
        &'a self,
        context: &mut PbesContext,
        state: &'a [usize],
    ) -> impl Iterator<Item = usize> + 'a {
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
            for &vi in &state[1..] {
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
                    .rewrite_formula(&self.summands[eq_idx].formula().unwrap())
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
            context.player_priority.expect("prepare must be called before state_info")
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

    fn enumerate<F>(
        &self,
        context: &mut PbesContext,
        state: &[usize],
        mut report: F,
    ) -> Result<(), MercError>
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
                // Need to split borrow: formula borrows from psi (in context), but
                // report closure also needs context.next_state_buf. Extract priority
                // before the call and pass the buf directly.
                let priority = *priority;
                let buf = &mut context.next_state_buf;
                enumerate_formula_children(
                    formula,
                    priority,
                    &self.name_to_eq,
                    &self.value_mapping,
                    &self.aux_mapping,
                    buf,
                    &mut report,
                )
            }
            PbesSummandKind::AuxNode => {
                let aux_idx = state[2];
                let priority = state[1];
                let pbes_ref = self
                    .aux_mapping
                    .get_by_index(aux_idx)
                    .expect("aux index must be valid");
                let formula = pbes_ref.copy();
                let buf = &mut context.next_state_buf;
                enumerate_formula_children(
                    formula,
                    priority,
                    &self.name_to_eq,
                    &self.value_mapping,
                    &self.aux_mapping,
                    buf,
                    &mut report,
                )
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
/// For `AND(l,r)` and `OR(l,r)` at the current level, both children are
/// emitted as transitions (each child becomes a new state via `emit_as_target`).
/// For a PVI, true, or false leaf, the leaf itself is the single successor.
fn enumerate_formula_children<F>(
    formula: PbesExpressionRef<'_>,
    priority: usize,
    name_to_eq: &HashMap<String, usize>,
    value_mapping: &Arc<ValueMapping>,
    aux_mapping: &Arc<AuxMapping>,
    buf: &mut Vec<usize>,
    report: &mut F,
) -> Result<(), MercError>
where
    F: FnMut(&(), &[usize]) -> Result<(), MercError>,
{
    if is_pbes_and(&formula.copy()) {
        let and = PbesAndRef::from(formula);
        emit_as_target(and.lhs(), priority, name_to_eq, value_mapping, aux_mapping, buf, report)?;
        emit_as_target(and.rhs(), priority, name_to_eq, value_mapping, aux_mapping, buf, report)?;
        Ok(())
    } else if is_pbes_or(&formula.copy()) {
        let or = PbesOrRef::from(formula);
        emit_as_target(or.lhs(), priority, name_to_eq, value_mapping, aux_mapping, buf, report)?;
        emit_as_target(or.rhs(), priority, name_to_eq, value_mapping, aux_mapping, buf, report)?;
        Ok(())
    } else {
        // PVI, true, or false: the formula itself is the single successor.
        emit_as_target(formula, priority, name_to_eq, value_mapping, aux_mapping, buf, report)
    }
}

/// Emits a transition to the parity-game state corresponding to `expr`.
///
/// PVI → concrete PVI state `[eq_idx, arg0, …]`.
/// AND/OR sub-formula → aux node `[AUX_AND/OR, priority, aux_idx]`.
/// `true` / `false` → respective sink state.
fn emit_as_target<F>(
    expr: PbesExpressionRef<'_>,
    priority: usize,
    name_to_eq: &HashMap<String, usize>,
    value_mapping: &Arc<ValueMapping>,
    aux_mapping: &Arc<AuxMapping>,
    buf: &mut Vec<usize>,
    report: &mut F,
) -> Result<(), MercError>
where
    F: FnMut(&(), &[usize]) -> Result<(), MercError>,
{
    if is_pbes_propositional_variable_instantiation(&expr.copy()) {
        let pvi = PbesPropositionalVariableInstantiationRef::from(expr);
        let target_eq = *name_to_eq
            .get(&pvi.name().to_string())
            .ok_or_else(|| MercError::from("Unknown equation name in PVI"))?;
        buf.clear();
        buf.push(target_eq);
        for arg in pvi.arguments().iter() {
            // SAFETY: term interned into the Protected value_mapping.
            let (idx, _) =
                value_mapping.insert(unsafe { DataExpressionRef::from_address(arg.address()) });
            buf.push(idx);
        }
        report(&(), buf)
    } else if is_pbes_and(&expr.copy()) {
        // SAFETY: term is a sub-expression of the rewritten psi still in context;
        // the aux_mapping (Protected) keeps it alive via GC marking.
        let (aux_idx, _) =
            aux_mapping.insert(unsafe { PbesExpressionRef::from_address(expr.address()) });
        buf.clear();
        buf.extend([AND_OP, priority, aux_idx]);
        report(&(), buf)
    } else if is_pbes_or(&expr.copy()) {
        // SAFETY: term is a sub-expression of the rewritten psi still in context;
        // the aux_mapping (Protected) keeps it alive via GC marking.
        let (aux_idx, _) =
            aux_mapping.insert(unsafe { PbesExpressionRef::from_address(expr.address()) });
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
) -> Result<ParityGame, MercError> {
    let lps = PbesLps::new(pbes)?;
    let timing = Timing::new();
    explore_pbes_impl(&lps, strategy, &timing)
}

/// Builds a [`ParityGame`] by exploring the given PBES directly in parallel.
///
/// Caching is not supported for this path: aux-node states have variable-length
/// vectors that break the cache's write-position replay.
pub(crate) fn explore_pbes_parallel(
    pbes: Pbes,
    threads: usize,
    pinned: bool,
) -> Result<ParityGame, MercError> {
    let lps = PbesLps::new(pbes)?;
    explore_pbes_parallel_impl(&lps, threads, CachingStrategy::None, pinned)
}

/// Computes tight read and write position vectors for an equation summand.
///
/// `read_positions`: `{0}` ∪ `{k+1 | param[k]` appears anywhere in `formula`}.
/// `write_positions`: `{0}` ∪ `{k+1 | some PVI argument at position k is not identity}`.
fn formula_positions(
    formula: &mcrl2::PbesExpression,
    params: &[*const _aterm],
) -> (Vec<usize>, Vec<usize>) {
    // Single-pass visitor: collect read-variable addresses and write-position
    // mask simultaneously, visiting each PVI argument exactly once.
    struct FormulaPositions<'p> {
        var_addrs: HashSet<*const _aterm>,
        write_mask: Vec<bool>,
        params: &'p [*const _aterm],
    }

    impl PbesExpressionVisitor for FormulaPositions<'_> {
        fn visit_propositional_variable_instantiation(
            &mut self,
            inst: &PbesPropositionalVariableInstantiationRef<'_>,
        ) -> Option<mcrl2::PbesExpression> {
            for (k, (arg, &param_addr)) in
                inst.arguments().iter().zip(self.params.iter()).enumerate()
            {
                for v in variable_occurrences_data_expression(&arg.copy()) {
                    self.var_addrs.insert(v.address());
                }
                if !(is_variable(&arg.copy()) && arg.address() == param_addr) {
                    self.write_mask[k] = true;
                }
            }
            None
        }

        fn visit_data_expression(
            &mut self,
            expr: &DataExpressionRef<'_>,
        ) -> Option<DataExpression> {
            for v in variable_occurrences_data_expression(expr) {
                self.var_addrs.insert(v.address());
            }
            None
        }
    }

    let mut collector = FormulaPositions {
        var_addrs: HashSet::new(),
        write_mask: vec![false; params.len()],
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

    let mut write_positions = vec![0usize];
    for (k, &written) in collector.write_mask.iter().enumerate() {
        if written {
            write_positions.push(k + 1);
        }
    }

    (read_positions, write_positions)
}
