//! Explicit-state exploration of a PBES in SRF (Simple Recursive Form) format.
//!
//! The PBES exploration is exposed as an [`merc_explore::LPS`] implementation
//! ([`PbesSrfLps`]), so the same generic exploration loop used for LPSs drives
//! it. The [`parity_game_from_pbes`] wrapper installs two closures that feed
//! the discovered states and transitions into a [`merc_vpg::ParityGameBuilder`].

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use mcrl2::ATerm;
use mcrl2::ATermList;
use mcrl2::DataExpression;
use mcrl2::DataVariable;
use mcrl2::LearnSuccessorsContext;
use mcrl2::Pbes;
use mcrl2::PbesPropositionalVariableInstantiation;
use mcrl2::SrfPbes;
use mcrl2::_aterm;
use mcrl2::make_data_assignment_list;
use mcrl2::tau_multi_action;
use merc_collections::IndexedSet;
use merc_explore::ExplorationStrategy;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_explore::explore;
use merc_lts::StateIndex;
use merc_utilities::MercError;
use merc_utilities::Timing;
use merc_vpg::ParityGame;
use merc_vpg::ParityGameBuilder;
use merc_vpg::Player;
use merc_vpg::Priority;
use merc_vpg::VertexIndex;

/// State data shared by every [`PbesSrfSummand`] of a [`PbesSrfLps`].
struct PbesSrfShared {
    /// Backend used to evaluate summand conditions and enumerate solutions.
    context: LearnSuccessorsContext,

    /// One [`IndexedSet`] per data parameter, mapping `DataExpression -> usize`.
    /// Behind a `RefCell` so summand enumeration callbacks can insert while
    /// the outer loop holds an immutable borrow of the LPS.
    mapping: RefCell<Vec<IndexedSet<DataExpression>>>,

    /// Reusable scratch buffer holding the resolved parameter pointers for
    /// the current source state. Filled during [`LPS::prepare`] and consumed
    /// immediately by [`LearnSuccessorsContext::set_assignments`].
    parameter_values: RefCell<Vec<*const _aterm>>,
}

/// Explicit-state view of a PBES in SRF normal form.
///
/// State vectors have layout `[equation_index, param_0, …, param_{n-1}]` where
/// `equation_index` is a flat index into [`SrfPbes::equations`] and each
/// `param_i` is an index into the corresponding [`IndexedSet<DataExpression>`]
/// in [`PbesSrfShared::mapping`].
pub struct PbesSrfLps {
    /// The unified SRF PBES; retained so summand pointers stay alive.
    _srf: SrfPbes,

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

    /// Resources shared with every summand.
    shared: Rc<PbesSrfShared>,
}

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

    /// Shared backend and parameter mapping.
    shared: Rc<PbesSrfShared>,

    /// Number of data parameters, used to size the next-state buffer.
    num_params: usize,

    /// Reusable buffer holding the next-state vector.
    next_state_buf: RefCell<Vec<usize>>,
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

        let shared = Rc::new(PbesSrfShared {
            context: LearnSuccessorsContext::from_data_spec(&pbes.data_specification()),
            mapping: RefCell::new((0..num_params).map(|_| IndexedSet::new()).collect()),
            parameter_values: RefCell::new(Vec::with_capacity(num_params)),
        });

        // Build the initial state vector from the initial PVI.
        let initial_pvi = pbes.initial_state();
        let initial_eq_name = initial_pvi.name().to_string();
        let initial_eq_idx = *name_to_eq
            .get(&initial_eq_name)
            .ok_or_else(|| MercError::from(format!("Unknown initial equation: {initial_eq_name}")))?;

        let mut initial_state = Vec::with_capacity(1 + num_params);
        initial_state.push(initial_eq_idx);
        {
            let mut mapping = shared.mapping.borrow_mut();
            for (i, arg) in initial_pvi.arguments().iter().enumerate() {
                let (idx, _) = mapping[i].insert(arg.into());
                initial_state.push(*idx);
            }
        }

        // Flatten (equation, srf_summand) pairs into a single summand list.
        let mut summands = Vec::new();
        for (eq_idx, eq) in srf.equations().iter().enumerate() {
            // The parameters list of this equation (LHS of the assignment list).
            let eq_param_term: ATerm = eq.variable().parameters().into();
            for srf_summand in eq.summands() {
                let target_pvi: PbesPropositionalVariableInstantiation = srf_summand.variable().into();
                let target_eq_name = target_pvi.name().to_string();
                let target_eq_idx = *name_to_eq.get(&target_eq_name).ok_or_else(|| {
                    MercError::from(format!("Unknown target equation: {target_eq_name}"))
                })?;

                let target_args: ATerm = target_pvi.arguments().protect().into();
                let assignments_term = make_data_assignment_list(&eq_param_term, &target_args);
                let write_assignments: ATermList<ATerm> = ATermList::new(assignments_term);

                summands.push(PbesSrfSummand {
                    equation_index: eq_idx,
                    target_equation_index: target_eq_idx,
                    condition: srf_summand.condition().into(),
                    summation_variables: srf_summand.parameters(),
                    write_assignments,
                    shared: Rc::clone(&shared),
                    num_params,
                    next_state_buf: RefCell::new(vec![0; 1 + num_params]),
                });
            }
        }

        Ok(Self {
            _srf: srf,
            summands,
            initial_state,
            state_info,
            process_parameters,
            num_params,
            shared,
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

    fn prepare(&self, state: &[Self::Value]) {
        debug_assert_eq!(
            state.len(),
            1 + self.num_params,
            "State vector length must match 1 + number of parameters"
        );

        // Look up the *const _aterm representative for each parameter value.
        let mut parameter_values = self.shared.parameter_values.borrow_mut();
        parameter_values.clear();
        let mapping = self.shared.mapping.borrow();
        for (i, &value_index) in state.iter().skip(1).enumerate() {
            parameter_values.push(
                mapping[i]
                    .get_by_index(value_index)
                    .expect("Parameter value must be in mapping")
                    .address(),
            );
        }
        drop(mapping);

        self.shared
            .context
            .set_assignments(&self.process_parameters, &parameter_values);
    }

    fn state_info(&self, state: &[Self::Value]) -> Self::StateInfo {
        self.state_info[state[0]]
    }
}

impl Summand for PbesSrfSummand {
    type Value = usize;
    type Label = ();

    fn enumerate<F>(&self, state: &[usize], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[usize]) -> Result<(), MercError>,
    {
        // PBES summands only fire from their owning equation.
        if state[0] != self.equation_index {
            return Ok(());
        }

        let tau = tau_multi_action();

        self.shared.context.enumerate_raw_with_current_assignments(
            &self.condition,
            &self.summation_variables,
            &self.write_assignments,
            &tau,
            |next_values: &[*const _aterm], _multi_action| {
                debug_assert_eq!(
                    next_values.len(),
                    self.num_params,
                    "Enumerated values must match number of parameters"
                );

                let mut next_state = self.next_state_buf.borrow_mut();
                next_state[0] = self.target_equation_index;
                {
                    let mut mapping = self.shared.mapping.borrow_mut();
                    for (i, &ptr) in next_values.iter().enumerate() {
                        let expr = DataExpression::from(ATerm::from_ptr(ptr));
                        let (idx, _) = mapping[i].insert(expr);
                        next_state[1 + i] = *idx;
                    }
                }

                // The PBES has no actions; we pass a placeholder unit label.
                report(&(), &next_state).expect("Failed to report PBES transition");
            },
        );

        // PBES has no read/write position tracking that the cache would use,
        // so we silently ignore `state` beyond the equation-index check above.
        let _ = state;
        Ok(())
    }
}

/// Computes a priority for each equation using alternation-depth.
///
/// Scans equations from outermost (index 0) to innermost (index n-1). Each
/// alternation of the fixpoint symbol (μ ↔ ν) bumps the priority. After the
/// scan the parities are shifted so μ-equations receive **odd** and
/// ν-equations receive **even** priorities, as required by the standard
/// PBES-to-parity-game translation.
fn compute_priorities(srf: &SrfPbes) -> Vec<usize> {
    let equations = srf.equations();
    if equations.is_empty() {
        return Vec::new();
    }

    let mut priorities = vec![0usize; equations.len()];
    let mut current_priority = 0usize;
    let mut prev_is_mu = equations[0].is_mu();

    for (i, eq) in equations.iter().enumerate() {
        let is_mu = eq.is_mu();
        if i > 0 && is_mu != prev_is_mu {
            current_priority += 1;
        }
        priorities[i] = current_priority;
        prev_is_mu = is_mu;
    }

    // Ensure parity invariant: μ → odd, ν → even.
    let first_is_mu = equations[0].is_mu();
    let first_priority_is_even = priorities[0].is_multiple_of(2);
    if first_is_mu && first_priority_is_even {
        for p in &mut priorities {
            *p += 1;
        }
    }

    priorities
}

/// Builds a [`ParityGame`] by exploring the given PBES in SRF format.
///
/// The exploration drives a generic [`merc_explore::explore`] loop over a
/// [`PbesSrfLps`] and uses two closures — both receiving the builder as their
/// caller context — to feed each discovered state and transition into the
/// [`ParityGameBuilder`]. The builder is then finalised with deduplication
/// and the make-total fixup enabled.
pub fn parity_game_from_pbes(pbes: &Pbes, strategy: ExplorationStrategy) -> Result<ParityGame, MercError> {
    let lps = PbesSrfLps::new(pbes)?;
    let timing = Timing::new();

    let mut builder = ParityGameBuilder::new(VertexIndex::new(0));

    explore(
        &lps,
        strategy,
        &timing,
        &mut builder,
        |b: &mut ParityGameBuilder, state: StateIndex, info: &(Player, Priority)| {
            b.add_vertex(VertexIndex::new(state.value()), info.0, info.1);
            Ok(())
        },
        |b: &mut ParityGameBuilder, from: StateIndex, _label: &(), to: StateIndex| {
            b.add_edge(VertexIndex::new(from.value()), VertexIndex::new(to.value()));
            Ok(())
        },
    )?;

    Ok(builder.finish(true, true))
}
