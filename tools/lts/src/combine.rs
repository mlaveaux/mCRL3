use log::info;
use merc_collections::IndexedSet;
use merc_collections::SetIndex;
use merc_collections::VecBag;
use merc_io::TimeProgress;
use merc_lts::LTS;
use merc_lts::LabelIndex;
use merc_lts::LtsAction;
use merc_lts::LtsBuilder;
use merc_lts::LtsMultiAction;
use merc_lts::StateIndex;
use merc_lts::Transition;
use merc_syntax::CommExpr;
use merc_syntax::MultiActionLabel;
use merc_utilities::MercError;
use merc_utilities::Timing;
use streaming_iterator::StreamingIterator;

/// Computes the parallel composition hide(allow(comm(L1 || ... || Ln))).
///
/// The `builder` is used to construct the resulting LTS, which can also be
/// stored immediately in a file.
///
/// We interpret empty hide, allow and comm sets as the operator not being present.
pub fn combine_lts<L: LTS<Label = LtsMultiAction>, B: LtsBuilder<L::Label>>(
    builder: &mut B,
    parallel_composition: Vec<L>,
    hide: &Vec<String>,
    allow: &Vec<MultiActionLabel>,
    comm: &Vec<CommExpr>,
    timing: &Timing,
) -> Result<(), MercError> {
    if parallel_composition.is_empty() {
        return Err("At least one LTS is required for composition.".into());
    }

    // Keep track of the discovered states in the combined LTS.
    let mut discovered: IndexedSet<Vec<StateIndex>> = IndexedSet::new();
    let (index, _) = discovered.insert(
        parallel_composition
            .iter()
            .map(|lts| lts.initial_state_index())
            .collect(),
    );

    let progress = TimeProgress::new(
        |(states, transitions): (usize, usize)| {
            info!("Explored {states} states, {transitions} transitions...");
        },
        1,
    );

    // Pre-sort the allow set for efficient matching.
    let sorted_allow: Vec<SortedMultiActionLabel> = allow.iter().map(SortedMultiActionLabel::new).collect();

    // Working refers to the state vectors in discovered.
    let mut working: Vec<SetIndex> = vec![index];
    timing.measure("compose", || -> Result<(), MercError> {
        while let Some(current) = working.pop() {
            // Clone the current state vector since discovered may be mutated below.
            let current_state_vector = discovered
                .get(current)
                .expect("State must be in the discovered set")
                .as_ref();

            // Loop over all subsets of LTSs and their outgoing transitions in the current state vector.
            let mut iter = ParallelTransitionIter::new(&parallel_composition, &current_state_vector);
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

                // Copy the target before advancing invalidates it.
                let target_vec = transition.target.to_vec();

                let (target_index, is_new) = discovered.insert(target_vec);
                let to = StateIndex::new(*target_index);
                builder.add_transition(StateIndex::new(*current), &multi_action, to)?;

                if is_new {
                    working.push(target_index);
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

    builder.finish(StateIndex::new(*index))?;
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
fn communicate(comm: &[CommExpr], action: LtsMultiAction) -> LtsMultiAction {
    let mut actions = action.into_actions().to_vec();

    for expr in comm {
        loop {
            // Try to find a matching sub-multiset for this communication expression.
            if let Some(replacement) = find_communication_match(&actions, expr) {
                actions = replacement;
            } else {
                break;
            }
        }
    }

    LtsMultiAction::new(VecBag::from_vec(actions))
}

/// Tries to find actions matching a single communication expression
/// $a_1 | \cdots | a_n \rightarrow c$ within the given action list.
///
/// Returns the resulting action list with the matched actions replaced by the
/// communicated action, or `None` if no match was found.
fn find_communication_match(actions: &[LtsAction], expr: &CommExpr) -> Option<Vec<LtsAction>> {
    // For each action name in the communication expression's left-hand side,
    // find a matching action with the same label. All matched actions must
    // have the same arguments.
    let mut matched_indices = Vec::new();
    let mut first_match: Option<usize> = None;

    for required_name in &expr.from.actions {
        let mut found = false;
        for (i, action) in actions.iter().enumerate() {
            if matched_indices.contains(&i) {
                continue;
            }
            if action.label() == required_name {
                // All matched actions must share the same arguments.
                if let Some(first) = first_match {
                    if action.arguments() != actions[first].arguments() {
                        continue;
                    }
                } else {
                    first_match = Some(i);
                }
                matched_indices.push(i);
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
    }

    // Build the result: remove matched actions and add the communicated action.
    let mut result: Vec<LtsAction> = actions
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_indices.contains(i))
        .map(|(_, a)| a.clone())
        .collect();

    let args = first_match.map(|i| actions[i].arguments().to_vec()).unwrap_or_default();
    result.push(LtsAction::new(expr.to.clone(), args));
    Some(result)
}

/// Returns true iff the given multi-action is allowed by the allow operator.
///
/// A multi-action is allowed if:
/// - The action is tau (always allowed), or
/// - The action label names, compared as sorted multisets, match one of the
///   entries in the allow set.
fn is_allowed(allow: &[SortedMultiActionLabel], action: &SortedLtsMultiAction) -> bool {
    if action.is_tau_label() {
        return true;
    }

    allow.iter().any(|allowed| {
        allowed.actions.len() == action.len()
            && allowed
                .actions
                .iter()
                .zip(action.labels())
                .all(|(name, label)| name == label)
    })
}

/// Applies the hide operator $\tau_I$ to a multi-action.
///
/// Removes all actions whose label is in the hide set $I$. If all actions are
/// hidden the result is the tau action (empty multi-action).
fn hide_action(hide: &[String], mut action: LtsMultiAction) -> LtsMultiAction {
    if !hide.is_empty() {
        action.retain(|a| !hide.iter().any(|h| h == a.label()));
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
        let done = lengths.iter().any(|&len| len == 0);
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

/// A view of an [`LtsMultiAction`] that provides label names in sorted order
/// (ignoring action parameters), for proper multiset comparison with
/// [`SortedMultiActionLabel`].
///
/// Since `VecBag<LtsAction>` sorts by `(label, arguments)`, the label names
/// are already in non-decreasing order, so no additional sorting is needed.
struct SortedLtsMultiAction<'a> {
    action: &'a LtsMultiAction,
}

impl<'a> SortedLtsMultiAction<'a> {
    fn new(action: &'a LtsMultiAction) -> Self {
        SortedLtsMultiAction { action }
    }

    fn is_tau_label(&self) -> bool {
        self.action.is_tau_label()
    }

    /// Returns an iterator over action label names in sorted order.
    fn labels(&self) -> impl Iterator<Item = &str> {
        self.action.actions().iter().map(|a| a.label())
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

/// A streaming iterator that lazily enumerates all parallel transitions from a
/// given state vector across multiple LTSs.
///
/// For each non-empty subset $J \subseteq \{0, \ldots, n-1\}$ of LTSs,
/// enumerates the Cartesian product of outgoing transitions from the LTSs in
/// $J$. The result buffers are updated in-place to avoid per-`next()` allocation.
pub struct ParallelTransitionIter {
    /// Pre-collected outgoing transitions for each LTS at the current state.
    all_outgoing: Vec<Vec<Transition>>,

    /// Current subset bitmask (1 to 2^n - 1).
    current_subset: usize,
    /// Upper bound for subset enumeration (2^n).
    max_subset: usize,

    /// Indices of LTSs participating in the current subset.
    subset_indices: Vec<usize>,
    /// The Cartesian product iterator for the current subset.
    product: CartesianProduct,

    /// The base target state vector (the current state for each LTS).
    base_target: Vec<StateIndex>,
    /// Output buffer, reused across `advance()` calls.
    result: ParallelTransition,

    done: bool,
    started: bool,
}

impl ParallelTransitionIter {
    /// Creates a new iterator over parallel transitions for the given LTSs
    /// and current state vector.
    pub fn new<L: LTS>(lts_list: &[L], current_states: &[StateIndex]) -> Self {
        assert!(
            lts_list.len() < usize::BITS as usize,
            "Number of LTSs exceeds maximum supported for subset enumeration"
        );

        let all_outgoing: Vec<Vec<Transition>> = lts_list
            .iter()
            .zip(current_states.iter())
            .map(|(lts, &state)| lts.outgoing_transitions(state).collect())
            .collect();

        ParallelTransitionIter {
            all_outgoing,
            current_subset: 1,
            max_subset: 1usize << lts_list.len(),
            subset_indices: Vec::new(),
            product: CartesianProduct::new(vec![]),
            base_target: current_states.to_vec(),
            result: ParallelTransition {
                subset_indices: Vec::new(),
                labels: Vec::new(),
                target: current_states.to_vec(),
            },
            done: false,
            started: false,
        }
    }

    /// Builds a `CartesianProduct` for the current subset bitmask.
    /// Returns `true` if the subset produces a non-empty Cartesian product.
    fn setup_subset(&mut self) -> bool {
        self.subset_indices.clear();

        let mut lengths = Vec::new();
        for i in 0..self.all_outgoing.len() {
            if self.current_subset & (1 << i) != 0 {
                if self.all_outgoing[i].is_empty() {
                    return false;
                }
                self.subset_indices.push(i);
                lengths.push(self.all_outgoing[i].len());
            }
        }

        self.product = CartesianProduct::new(lengths);
        true
    }

    /// Fills the result buffer from the product's current indices.
    fn fill_result(&mut self) {
        self.result.subset_indices.clear();
        self.result.subset_indices.extend_from_slice(&self.subset_indices);
        self.result.labels.clear();
        self.result.target.copy_from_slice(&self.base_target);

        for (k, &lts_idx) in self.subset_indices.iter().enumerate() {
            let transition = &self.all_outgoing[lts_idx][self.product.indices[k]];
            self.result.labels.push(transition.label);
            self.result.target[lts_idx] = transition.to;
        }
    }
}

impl StreamingIterator for ParallelTransitionIter {
    type Item = ParallelTransition;

    fn advance(&mut self) {
        if self.done {
            return;
        }

        // Try to advance within current Cartesian product.
        if self.started {
            self.product.advance();
            if self.product.get().is_some() {
                self.fill_result();
                return;
            }
            self.current_subset += 1;
        }
        self.started = true;

        // Find next subset with a non-empty Cartesian product.
        while self.current_subset < self.max_subset {
            if self.setup_subset() {
                self.product.advance();
                if self.product.get().is_some() {
                    self.fill_result();
                    return;
                }
            }
            self.current_subset += 1;
        }

        self.done = true;
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.done || !self.started {
            None
        } else {
            Some(&self.result)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::io::{self};
    use std::path::Path;
    use std::process::Command;

    use log::trace;
    use merc_lts::LTS;
    use merc_lts::LtsBuilderFast;
    use merc_lts::LtsMultiAction;
    use merc_lts::StateIndex;
    use merc_lts::random_lts;
    use merc_lts::read_lts;
    use merc_lts::write_aut;
    use merc_reduction::Equivalence;
    use merc_reduction::compare_lts;
    use merc_syntax::MultiActionLabel;
    use merc_utilities::MercError;
    use merc_utilities::Timing;
    use merc_utilities::random_test;
    use rand::RngExt;
    use rand::seq::IndexedRandom;
    use tempfile::TempDir;

    use crate::combine::combine_lts;

    /// Uses `MERC_DUMP` as the temporary directory if set, and otherwise the default temp directory.
    fn temp_dir(name: &str) -> Result<TempDir, MercError> {
        if let Ok(dump_dir) = std::env::var("MERC_DUMP") {
            // Check if the directory is an absolute path
            if !Path::new(dump_dir.as_str()).is_absolute() {
                panic!("MERC_DUMP must be an absolute path, because tests write relative to their source file.");
            }

            // If we are asking for MERC_DUMP, disable cleanup.
            let mut dir = tempfile::TempDir::with_prefix_in(name, dump_dir)?;
            dir.disable_cleanup(true);
            Ok(dir)
        } else {
            tempfile::tempdir().map_err(|e| e.into())
        }
    }

    fn traced_status(command: &mut Command) -> io::Result<std::process::ExitStatus> {
        trace!(
            "{} {}",
            command.get_program().to_string_lossy(),
            command
                .get_args()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );
        command.status()
    }

    #[test]
    fn test_mcrl2_ltscombine() {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl2_ltscombine = Path::new(&mcrl2_path).join("ltscombine");
        let mcrl2_mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");
        let mcrl2_ltsconvert = Path::new(&mcrl2_path).join("ltsconvert");

        // Write the random LTS to a temp file for ltsconvert to process.
        let temp_dir = temp_dir("test_mcrl2_ltscombine").unwrap();
        let left_path = temp_dir.path().join("left.aut");
        let right_path = temp_dir.path().join("right.aut");
        let left_lts_path = temp_dir.path().join("left.lts");
        let right_lts_path = temp_dir.path().join("right.lts");

        let spec_path = temp_dir.path().join("spec.mcrl2");
        let lps_path = temp_dir.path().join("spec.lps");
        let output_path = temp_dir.path().join("output.lts");
        let expected_path = temp_dir.path().join("expected.aut");
        let result_path = temp_dir.path().join("result.aut");

        // Generate a dummy linear process specification to convert the .aut files to .lts format.
        writeln!(&mut File::create(&spec_path).unwrap(), "act i, a, b, c; init delta;").unwrap();

        let status = traced_status(Command::new(&mcrl2_mcrl22lps).arg(&spec_path).arg(&lps_path))
            .expect("Failed to run ltsconvert");
        assert!(status.success(), "ltsconvert failed with status: {status}");

        random_test(100, |rng| {
            let left_lts = random_lts(rng, 10, 3, 3)
                .relabel(|label| LtsMultiAction::from_string(&label))
                .unwrap();
            let right_lts = random_lts(rng, 10, 3, 3)
                .relabel(|label| LtsMultiAction::from_string(&label))
                .unwrap();

            write_aut(&mut File::create(&left_path).unwrap(), &left_lts).unwrap();
            write_aut(&mut File::create(&right_path).unwrap(), &right_lts).unwrap();

            // For mCRL2's ltscombine we need to convert the inputs to the mCRL2 LTS format.
            let status = traced_status(
                Command::new(&mcrl2_ltsconvert)
                    .arg("-enone")
                    .arg(&left_path)
                    .arg(&left_lts_path)
                    .arg("-l")
                    .arg(&lps_path),
            )
            .expect("Failed to run ltsconvert");
            assert!(status.success(), "ltsconvert failed with status: {status}");

            let status = traced_status(
                Command::new(&mcrl2_ltsconvert)
                    .arg("-enone")
                    .arg(&right_path)
                    .arg(&right_lts_path)
                    .arg("-l")
                    .arg(&lps_path),
            )
            .expect("Failed to run ltsconvert");
            assert!(status.success(), "ltsconvert failed with status: {status}");

            // Use ltscombine to compute the combined LTS, which we will compare against our implementation's result
            let status = traced_status(
                Command::new(&mcrl2_ltscombine)
                    .arg(&left_lts_path)
                    .arg(&right_lts_path)
                    .arg(&output_path),
            )
            .expect("Failed to run ltscombine");
            assert!(status.success(), "ltscombine failed with status: {status}");

            let expected_lts = read_lts(&File::open(&output_path).unwrap(), false).unwrap();
            write_aut(&mut File::create(&expected_path).unwrap(), &expected_lts).unwrap();

            // Allow an arbitrary subset of labels
            let labels = left_lts
                .labels()
                .iter()
                .chain(right_lts.labels().iter())
                .map(|l| l.to_string())
                .collect::<Vec<_>>();

            let num_of_allowed = rng.random_range(0..=labels.len());
            let allow = labels
                .sample(rng, num_of_allowed)
                .cloned()
                .map(|s| MultiActionLabel::new(vec![s]))
                .collect::<Vec<_>>();

            let mut result: LtsBuilderFast<LtsMultiAction> = LtsBuilderFast::new(Vec::new(), Vec::new());
            combine_lts(
                &mut result,
                vec![left_lts, right_lts],
                &vec![],
                &vec![],
                &vec![],
                &mut Timing::new(),
            )
            .unwrap();
            let result_lts = result.finish(StateIndex::new(0), false);
            write_aut(&mut File::create(&result_path).unwrap(), &result_lts).unwrap();

            assert!(
                compare_lts(
                    Equivalence::StrongBisim,
                    expected_lts,
                    result_lts,
                    false,
                    &mut Timing::new(),
                ),
                "The resulting LTSs are not bisimilar."
            );
        });
    }
}
