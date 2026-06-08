use log::info;
use merc_collections::VecBag;
use merc_io::TimeProgress;
use merc_lts::LTS;
use merc_lts::LabelIndex;
use merc_lts::LtsAction;
use merc_lts::LtsBuilder;
use merc_lts::LtsMultiAction;
use merc_lts::SimpleAction;
use merc_lts::StateIndex;
use merc_lts::Transition;
use merc_lts::TransitionLabel;
use merc_syntax::CommExpr;
use merc_syntax::MultiActionLabel;
use merc_utilities::MercError;
use merc_utilities::Timing;
use streaming_iterator::StreamingIterator;

use crate::DiscoveredSet;
use crate::StateRef;

/// The trait for labels used in the composition operator.
pub trait CombineLabel: TransitionLabel + Ord {
    /// Returns true iff the data arguments of two actions are compatible for communication.
    /// Defaults to `true` for label types without structured arguments (e.g., `String`).
    fn comm_args_compatible(&self, _other: &Self) -> bool {
        true
    }

    /// Creates a new action as the result of a communication expression with the given `label`.
    /// `representative` is one of the matched input actions and provides argument values.
    fn from_comm(label: String, representative: &Self) -> Self;
}

impl CombineLabel for SimpleAction {
    fn comm_args_compatible(&self, other: &Self) -> bool {
        self.arguments() == other.arguments()
    }

    fn from_comm(label: String, representative: &Self) -> Self {
        SimpleAction::new(label, representative.arguments().to_vec())
    }
}

impl CombineLabel for LtsAction {
    fn comm_args_compatible(&self, other: &Self) -> bool {
        self.arguments() == other.arguments()
    }

    fn from_comm(label: String, representative: &Self) -> Self {
        LtsAction::new(label, representative.arguments().to_vec())
    }
}

/// Computes the parallel composition hide(H, allow(A, comm(C, L1 || ... || Ln))).
///
/// The `builder` is used to construct the resulting LTS, which can also be
/// stored immediately in a file.
pub fn combine_lts<L: LTS<Label = LtsMultiAction<A>>, A: CombineLabel, B: LtsBuilder<L::Label>>(
    builder: &mut B,
    parallel_composition: Vec<L>,
    hide: &[String],
    allow: &[MultiActionLabel],
    comm: &[CommExpr],
    timing: &Timing,
) -> Result<(), MercError> {
    if parallel_composition.is_empty() {
        return Err("At least one LTS is required for composition.".into());
    }

    if allow.iter().any(|allow| allow.is_tau_label()) {
        return Err("Allow set cannot contain the tau action.".into());
    }

    if comm.iter().any(|comm| comm.from.is_tau_label()) {
        return Err("Communication expressions cannot have tau actions on the left-hand side.".into());
    }

    for (i, comm_i) in comm.iter().enumerate() {
        if comm_i.from.actions.len() < 2 {
            return Err(format!(
                "Communication expressions must have at least two actions on the left-hand side, but {comm_i} does not."
            )
            .into());
        }

        // The left hand side of a communication cannot overlap with any other communication's left hand side.
        for (j, comm_j) in comm.iter().enumerate() {
            if i != j && comm_i.from.actions.iter().any(|a| comm_j.from.actions.contains(a)) {
                return Err(format!(
                    "Communication expressions cannot have overlapping left-hand sides, i.e. {comm_i} and {comm_j}."
                )
                .into());
            }
        }

        // The right hand side of a communication cannot be in another communication's left hand side.
        for (j, comm_j) in comm.iter().enumerate() {
            if i != j && comm_j.from.actions.contains(&comm_i.to) {
                return Err(
                    format!("Communication expressions cannot have right-hand side in another communication's left-hand side, i.e. {comm_i} and {comm_j}.").into(),
                );
            }
        }
    }

    // Keep track of the discovered states in the combined LTS. State vectors
    // are stored as maximally shared sequences of raw `usize` indices in the
    // discovered set; we convert to/from `StateIndex` at the boundary.
    let mut discovered: DiscoveredSet<usize> = DiscoveredSet::new();
    let initial_vector: Vec<usize> = parallel_composition
        .iter()
        .map(|lts| lts.initial_state_index().value())
        .collect();
    let (initial_ref, _) = discovered.insert(&initial_vector);

    let progress = TimeProgress::new(
        |(states, transitions): (usize, usize)| {
            info!("Explored {states} states, {transitions} transitions...");
        },
        1,
    );

    // Pre-sort the allow set for efficient matching.
    let sorted_allow: Vec<SortedMultiActionLabel> = allow.iter().map(SortedMultiActionLabel::new).collect();

    // Working refers to the state vectors in discovered.
    let mut working: Vec<StateRef> = vec![initial_ref];
    // Reusable scratch buffers for the parallel transition iterator.
    let mut iter_context = ParallelTransitionContext::new();
    // Reusable buffers for reconstructing the current state vector from the
    // discovered set and for building target vectors prior to insertion.
    let mut current_state_raw: Vec<usize> = Vec::new();
    let mut current_state_vector: Vec<StateIndex> = Vec::new();
    let mut target_raw: Vec<usize> = Vec::new();
    timing.measure("compose", || -> Result<(), MercError> {
        while let Some(current) = working.pop() {
            // Reconstruct the current state vector from the discovered set.
            debug_assert!(
                discovered.get_into(current, &mut current_state_raw),
                "StateRef from working set must be valid"
            );
            current_state_vector.clear();
            current_state_vector.extend(current_state_raw.iter().copied().map(StateIndex::new));

            // Loop over all subsets of LTSs and their outgoing transitions in the current state vector.
            let mut iter = ParallelTransitionIter::new(&mut iter_context, &parallel_composition, &current_state_vector);
            loop {
                iter.advance();
                let Some(transition) = iter.get() else {
                    break;
                };

                // Build the combined multi-action alpha = alpha_{j_0} | ... | alpha_{j_m}.
                let mut actions = VecBag::new();
                for (k, &lts_idx) in transition.subset_indices.iter().enumerate() {
                    let label_idx = transition.labels[k];
                    let label = &parallel_composition[lts_idx].labels()[label_idx];
                    for action in label.actions() {
                        actions.insert(action.clone());
                    }
                }
                let multi_action = LtsMultiAction::new(actions);

                // Apply communication: alpha = gamma_C(alpha).
                let multi_action = communicate(comm, multi_action);

                // Check allow: alpha in A ∪ {tau}.
                if !is_allowed(&sorted_allow, &SortedLtsMultiAction::new(&multi_action)) {
                    continue;
                }

                // Apply hide: alpha = tau_I(alpha).
                let multi_action = hide_action(hide, multi_action);

                // Convert the target vector into raw indices for the discovered set.
                target_raw.clear();
                target_raw.extend(transition.target.iter().map(|s| s.value()));

                let (target_ref, is_new) = discovered.insert(&target_raw);
                let to = StateIndex::new(target_ref.index());
                builder.add_transition(StateIndex::new(current.index()), &multi_action, to)?;

                if is_new {
                    working.push(target_ref);
                }
            }

            progress.print((discovered.len(), builder.num_of_transitions()));
        }

        Ok(())
    })?;

    info!(
        "Composition complete: {} states, {} transitions",
        discovered.len(),
        builder.num_of_transitions()
    );

    builder.finish(StateIndex::new(initial_ref.index()))?;
    Ok(())
}

/// Applies the communication operator to a multi-action according to the given
/// communication expressions.
///
/// # Details
/// Applies the communication operator $\gamma_C$ to a multi-action.
///
/// For each communication expression $a_1 | \cdots | a_n \rightarrow c$ in $C$,
/// repeatedly finds matching sub-multisets of actions (with equal arguments)
/// and replaces them with the result action $c$.
fn communicate<L: CombineLabel>(comm: &[CommExpr], action: LtsMultiAction<L>) -> LtsMultiAction<L> {
    let mut actions = action.into_actions().to_vec();

    for expr in comm {
        // Try to find a matching sub-multiset for this communication expression.
        while let Some(replacement) = find_communication_match(&actions, expr) {
            actions = replacement;
        }
    }

    LtsMultiAction::new(VecBag::from_vec(actions))
}

/// Tries to find actions matching a single communication expression
/// $a_1 | \cdots | a_n \rightarrow c$ within the given action list.
///
/// Returns the resulting action list with the matched actions replaced by the
/// communicated action, or `None` if no match was found.
fn find_communication_match<L: CombineLabel>(actions: &[L], expr: &CommExpr) -> Option<Vec<L>> {
    // For each action name in the communication expression's left-hand side,
    // find a matching action with the same label. All matched actions must
    // have the same arguments.
    let mut matched = vec![false; actions.len()];
    let mut first_match: Option<usize> = None;

    for required_name in &expr.from.actions {
        let mut found = false;
        for (i, action) in actions.iter().enumerate() {
            if matched[i] {
                continue;
            }
            if action.matches_label(required_name) {
                // All matched actions must share the same arguments.
                if let Some(first) = first_match {
                    if !action.comm_args_compatible(&actions[first]) {
                        continue;
                    }
                } else {
                    first_match = Some(i);
                }
                matched[i] = true;
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
    }

    // Build the result: remove matched actions and add the communicated action.
    let mut result: Vec<L> = actions
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched[*i])
        .map(|(_, a)| a.clone())
        .collect();

    let representative = &actions[first_match.expect("first_match is always Some when all actions are matched")];
    result.push(L::from_comm(expr.to.clone(), representative));
    Some(result)
}

/// Returns true iff the given multi-action is allowed by the allow operator.
///
/// A multi-action is allowed if:
/// - The action is tau (always allowed), or
/// - The action label names, compared as sorted multisets, match one of the
///   entries in the allow set.
fn is_allowed<L: CombineLabel>(allow: &[SortedMultiActionLabel], action: &SortedLtsMultiAction<L>) -> bool {
    if action.is_tau_label() {
        return true;
    }

    allow.iter().any(|allowed| {
        allowed.actions.len() == action.len()
            && allowed
                .actions
                .iter()
                .zip(action.iter_actions())
                .all(|(name, a)| a.matches_label(name))
    })
}

/// Applies the hide operator $\tau_I$ to a multi-action.
///
/// Removes all actions whose label is in the hide set $I$. If all actions are
/// hidden the result is the tau action (empty multi-action).
fn hide_action<L: CombineLabel>(hide: &[String], mut action: LtsMultiAction<L>) -> LtsMultiAction<L> {
    if !hide.is_empty() {
        action.retain(|a| !hide.iter().any(|h| a.matches_label(h)));
    }
    action
}

/// A streaming iterator over the Cartesian product of sequences with given
/// lengths.
///
/// Each call to `advance` updates an internal index buffer (odometer) that
/// represents one element of the Cartesian product. The buffer is reused
/// across calls to avoid allocation. The yielded `&[usize]` contains the
/// current index into each factor.
pub struct CartesianProduct {
    /// The length of each factor.
    lengths: Vec<usize>,
    /// Current index into each factor (odometer).
    indices: Vec<usize>,

    done: bool,
    started: bool,
}

impl CartesianProduct {
    /// Creates a new Cartesian product iterator for factors with the given
    /// lengths.
    ///
    /// If any length is zero the iterator will yield no elements.
    pub fn new(lengths: Vec<usize>) -> Self {
        let done = lengths.contains(&0);
        let n = lengths.len();
        CartesianProduct {
            lengths,
            indices: vec![0; n],
            done,
            started: false,
        }
    }

    /// Returns `true` if a next combination exists, `false` if exhausted.
    fn advance_impl(&mut self) -> bool {
        for i in (0..self.indices.len()).rev() {
            self.indices[i] += 1;
            if self.indices[i] < self.lengths[i] {
                return true;
            }
            self.indices[i] = 0;
        }
        false
    }
}

impl StreamingIterator for CartesianProduct {
    type Item = [usize];

    fn advance(&mut self) {
        if self.done {
            return;
        }

        if self.started {
            if !self.advance_impl() {
                self.done = true;
            }
        } else {
            self.started = true;
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.done || !self.started {
            None
        } else {
            Some(&self.indices)
        }
    }
}

/// A multi-action label with action names sorted, for proper multiset
/// comparison with [`SortedLtsMultiAction`].
struct SortedMultiActionLabel {
    /// Action names in sorted order.
    actions: Vec<String>,
}

impl SortedMultiActionLabel {
    fn new(label: &MultiActionLabel) -> Self {
        let mut actions = label.actions.clone();
        actions.sort();
        SortedMultiActionLabel { actions }
    }
}

/// A view of an [`LtsMultiAction`] for multiset comparison with [`SortedMultiActionLabel`].
///
/// Since `VecBag<L>` iterates in sorted order, the actions are already in
/// non-decreasing order, so no additional sorting is needed here.
struct SortedLtsMultiAction<'a, L: Ord> {
    action: &'a LtsMultiAction<L>,
}

impl<'a, L: TransitionLabel> SortedLtsMultiAction<'a, L> {
    fn new(action: &'a LtsMultiAction<L>) -> Self {
        SortedLtsMultiAction { action }
    }

    fn is_tau_label(&self) -> bool {
        self.action.is_tau_label()
    }

    fn iter_actions(&self) -> impl Iterator<Item = &L> {
        self.action.actions().iter()
    }

    fn len(&self) -> usize {
        self.action.actions().len()
    }
}

/// The output of a parallel transition step: label indices from participating
/// LTSs and the combined target state vector.
pub struct ParallelTransition {
    /// Indices of LTSs participating in this transition's subset.
    pub subset_indices: Vec<usize>,
    /// Label indices from each participating LTS in the current subset.
    pub labels: Vec<LabelIndex>,
    /// Target state vector: for LTSs in the subset, the transition target;
    /// for others, the current state is retained.
    pub target: Vec<StateIndex>,
}

/// Reusable scratch buffers for [`ParallelTransitionIter`].
///
/// Holding these buffers in a long-lived context avoids re-allocating per
/// source state during a single `combine_lts` invocation.
pub struct ParallelTransitionContext {
    /// Indices of LTSs participating in the current subset.
    subset_indices: Vec<usize>,
    /// Snapshot of the current source state vector (one state per LTS).
    base_target: Vec<StateIndex>,
    /// Current transition for each LTS in `subset_indices`, in the same order.
    current: Vec<Transition>,
    /// Current offset into the outgoing-transition iterator of each LTS in
    /// `subset_indices`, in the same order.
    offsets: Vec<usize>,
    /// Length of the outgoing-transition iterator of each LTS in
    /// `subset_indices`, in the same order.
    lengths: Vec<usize>,
    /// Output buffer surfaced via `StreamingIterator::get`.
    result: ParallelTransition,
}

impl ParallelTransitionContext {
    /// Creates a fresh context with empty buffers.
    pub fn new() -> Self {
        Self {
            subset_indices: Vec::new(),
            base_target: Vec::new(),
            current: Vec::new(),
            offsets: Vec::new(),
            lengths: Vec::new(),
            result: ParallelTransition {
                subset_indices: Vec::new(),
                labels: Vec::new(),
                target: Vec::new(),
            },
        }
    }
}

impl Default for ParallelTransitionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A streaming iterator that lazily enumerates all parallel transitions from a
/// given state vector across multiple LTSs.
///
/// For each non-empty subset $J \subseteq \{0, \ldots, n-1\}$ of LTSs,
/// enumerates the Cartesian product of outgoing transitions from the LTSs in
/// $J$ without materialising all outgoing transitions ahead of time. Internal
/// buffers live in a [`ParallelTransitionContext`] that can be reused across
/// many source states.
pub struct ParallelTransitionIter<'ctx, 'a, L: LTS> {
    ctx: &'ctx mut ParallelTransitionContext,
    lts_list: &'a [L],
    /// Current subset bitmask (1 to 2^n - 1).
    current_subset: usize,
    /// Upper bound for subset enumeration (2^n).
    max_subset: usize,
    done: bool,
    started: bool,
}

impl<'ctx, 'a, L: LTS> ParallelTransitionIter<'ctx, 'a, L> {
    /// Creates a new iterator over parallel transitions for the given LTSs
    /// and current state vector, reusing the buffers held in `ctx`.
    pub fn new(ctx: &'ctx mut ParallelTransitionContext, lts_list: &'a [L], current_states: &[StateIndex]) -> Self {
        assert!(
            lts_list.len() < usize::BITS as usize,
            "Number of LTSs exceeds maximum supported for subset enumeration"
        );
        assert_eq!(
            lts_list.len(),
            current_states.len(),
            "LTS slice and state vector must have the same length"
        );

        // Reset all reusable buffers while keeping their allocated capacity.
        ctx.subset_indices.clear();
        ctx.current.clear();
        ctx.offsets.clear();
        ctx.lengths.clear();

        ctx.base_target.clear();
        ctx.base_target.extend_from_slice(current_states);

        ctx.result.subset_indices.clear();
        ctx.result.labels.clear();
        ctx.result.target.clear();
        ctx.result.target.extend_from_slice(current_states);

        Self {
            ctx,
            lts_list,
            current_subset: 1,
            max_subset: 1usize << lts_list.len(),
            done: false,
            started: false,
        }
    }

    /// Sets up offsets, lengths and initial transitions for the current subset bitmask.
    ///
    /// Returns `true` if the subset has a non-empty Cartesian product (every
    /// participating LTS has at least one outgoing transition from its
    /// current state).
    fn setup_subset(&mut self) -> bool {
        let ctx = &mut *self.ctx;
        ctx.subset_indices.clear();
        ctx.current.clear();
        ctx.offsets.clear();
        ctx.lengths.clear();

        for i in 0..self.lts_list.len() {
            if self.current_subset & (1 << i) != 0 {
                let state = ctx.base_target[i];
                let mut iter = self.lts_list[i].outgoing_transitions(state);
                let Some(first) = iter.next() else {
                    return false;
                };
                let len = 1 + iter.count();
                ctx.subset_indices.push(i);
                ctx.current.push(first);
                ctx.offsets.push(0);
                ctx.lengths.push(len);
            }
        }
        true
    }

    /// Advances the Cartesian product odometer for the current subset.
    ///
    /// Returns `true` if a new combination is available; `false` if the
    /// subset is exhausted.
    fn advance_product(&mut self) -> bool {
        let ctx = &mut *self.ctx;
        for k in (0..ctx.subset_indices.len()).rev() {
            let next_offset = ctx.offsets[k] + 1;
            if next_offset < ctx.lengths[k] {
                let lts_idx = ctx.subset_indices[k];
                let state = ctx.base_target[lts_idx];
                let t = self.lts_list[lts_idx]
                    .outgoing_transitions(state)
                    .nth(next_offset)
                    .expect("Offset is within the iterator's exact length");
                ctx.offsets[k] = next_offset;
                ctx.current[k] = t;
                return true;
            }
            // Inner factor exhausted: restart it and carry to the next outer factor.
            let lts_idx = ctx.subset_indices[k];
            let state = ctx.base_target[lts_idx];
            let first = self.lts_list[lts_idx]
                .outgoing_transitions(state)
                .next()
                .expect("Outgoing transitions cannot become empty after a successful setup");
            ctx.offsets[k] = 0;
            ctx.current[k] = first;
        }
        false
    }

    /// Materialises the next result into the context's output buffer.
    fn fill_result(&mut self) {
        let ctx = &mut *self.ctx;
        let subset = &ctx.subset_indices;
        let current = &ctx.current;
        let base = &ctx.base_target;
        let result = &mut ctx.result;

        result.subset_indices.clear();
        result.subset_indices.extend_from_slice(subset);
        result.labels.clear();
        result.target.clear();
        result.target.extend_from_slice(base);

        for (k, &lts_idx) in subset.iter().enumerate() {
            let t = &current[k];
            result.labels.push(t.label);
            result.target[lts_idx] = t.to;
        }
    }
}

impl<'a, L: LTS> StreamingIterator for ParallelTransitionIter<'_, 'a, L> {
    type Item = ParallelTransition;

    fn advance(&mut self) {
        if self.done {
            return;
        }

        // Try to advance within the current Cartesian product.
        if self.started {
            if self.advance_product() {
                self.fill_result();
                return;
            }
            self.current_subset += 1;
        }
        self.started = true;

        // Find the next subset with a non-empty Cartesian product.
        while self.current_subset < self.max_subset {
            if self.setup_subset() {
                self.fill_result();
                return;
            }
            self.current_subset += 1;
        }

        self.done = true;
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.done || !self.started {
            None
        } else {
            Some(&self.ctx.result)
        }
    }
}
