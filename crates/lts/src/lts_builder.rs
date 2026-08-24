#![forbid(unsafe_code)]

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Mutex;

use merc_collections::ByteCompressedVec;
use merc_collections::CompressedEntry;
use merc_collections::bytevec;
use merc_utilities::MercError;

use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::StateIndex;
use crate::TransitionLabel;

/// A trait for building labelled transition systems incrementally.
///
/// # Details
///
/// Depending on the implementation this can be done in a memory efficient way,
/// or in a way that is optimized for speed. Alternatively, the resulting LTS is
/// immediately written to disk. The builder accumulates transitions using
/// `add_transition`, and once all transitions have been added, the labelled
/// transition system can be constructed with `finish`. An initial state can
/// also be specified during finalization.
pub trait LtsBuilder<L: TransitionLabel> {
    /// The result type of the builder once finalized.
    type LTS;

    /// Adds a transition to the builder. For efficiency reasons, we can use
    /// another type `Q` for the label.
    fn add_transition<Q>(&mut self, from: StateIndex, label: &Q, to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash;

    /// Finalizes the builder and returns the constructed labelled transition system.
    fn finish(&mut self, initial_state: StateIndex) -> Result<Self::LTS, MercError>;

    /// Returns the number of transitions added to the builder.
    fn num_of_transitions(&self) -> usize;

    /// Returns the number of states added to the builder.
    fn num_of_states(&self) -> usize;

    /// Ensures the builder accounts for at least `num_states` states, so that
    /// states without incident transitions (such as an isolated initial state)
    /// are still reflected in the result.
    fn require_num_of_states(&mut self, num_states: usize);
}

/// Delegates to `B`'s own implementation, so a `&mut B` can stand in wherever an owned builder is
/// expected, without having to take ownership of it.
impl<L: TransitionLabel, B: LtsBuilder<L> + ?Sized> LtsBuilder<L> for &mut B {
    type LTS = B::LTS;

    fn add_transition<Q>(&mut self, from: StateIndex, label: &Q, to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash,
    {
        (**self).add_transition(from, label, to)
    }

    fn finish(&mut self, initial_state: StateIndex) -> Result<Self::LTS, MercError> {
        (**self).finish(initial_state)
    }

    fn num_of_transitions(&self) -> usize {
        (**self).num_of_transitions()
    }

    fn num_of_states(&self) -> usize {
        (**self).num_of_states()
    }

    fn require_num_of_states(&mut self, num_states: usize) {
        (**self).require_num_of_states(num_states)
    }
}

/// A builder that discards all transitions, producing no output. This is useful
/// when an LTS only needs to be explored but the result is not required.
impl<L: TransitionLabel> LtsBuilder<L> for () {
    type LTS = ();

    fn add_transition<Q>(&mut self, _from: StateIndex, _label: &Q, _to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash,
    {
        Ok(())
    }

    fn finish(&mut self, _initial_state: StateIndex) -> Result<Self::LTS, MercError> {
        Ok(())
    }

    fn num_of_transitions(&self) -> usize {
        0
    }

    fn num_of_states(&self) -> usize {
        0
    }

    fn require_num_of_states(&mut self, _num_states: usize) {}
}

/// A builder that additionally accepts transitions through a shared `&self`
/// reference, synchronising internally.
///
/// # Details
///
/// This is used by the parallel explorer, where several worker threads stream
/// transitions into a single builder at once. Implementors take care of their
/// own synchronisation, so callers need no external lock. The builder is
/// finalised through the [`LtsBuilder`] supertrait once exploration completes.
pub trait ConcurrentLtsBuilder<L: TransitionLabel>: LtsBuilder<L> + Sync {
    /// Adds a transition through a shared reference. See
    /// [`LtsBuilder::add_transition`].
    fn add_transition_shared<Q>(&self, from: StateIndex, label: &Q, to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash;
}

/// The discarding builder also discards concurrent transitions.
impl<L: TransitionLabel> ConcurrentLtsBuilder<L> for () {
    fn add_transition_shared<Q>(&self, _from: StateIndex, _label: &Q, _to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash,
    {
        Ok(())
    }
}

/// Adapts any [`LtsBuilder`] into a [`ConcurrentLtsBuilder`] by guarding it with
/// a `Mutex`.
///
/// Concurrent (`&self`) transitions simply lock the mutex and delegate to the
/// inner builder's [`LtsBuilder::add_transition`]; the single-threaded
/// (`&mut self`) operations access the builder without locking. This serialises
/// all writes, which is enough for builders that are cheap to write to (such as
/// [`crate::AutStream`]) while the expensive exploration happens outside the
/// lock.
pub struct MutexLtsBuilder<B> {
    inner: Mutex<B>,
}

impl<B> MutexLtsBuilder<B> {
    /// Wraps `builder` so it can be shared across worker threads.
    pub fn new(builder: B) -> MutexLtsBuilder<B> {
        MutexLtsBuilder {
            inner: Mutex::new(builder),
        }
    }

    /// Unwraps and returns the inner builder.
    pub fn into_inner(self) -> B {
        self.inner.into_inner().expect("MutexLtsBuilder mutex poisoned")
    }
}

impl<L: TransitionLabel, B: LtsBuilder<L>> LtsBuilder<L> for MutexLtsBuilder<B> {
    type LTS = B::LTS;

    fn add_transition<Q>(&mut self, from: StateIndex, label: &Q, to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash,
    {
        // We hold `&mut self`, so the builder is exclusively ours and needs no lock.
        self.inner
            .get_mut()
            .expect("MutexLtsBuilder mutex poisoned")
            .add_transition(from, label, to)
    }

    fn finish(&mut self, initial_state: StateIndex) -> Result<Self::LTS, MercError> {
        self.inner
            .get_mut()
            .expect("MutexLtsBuilder mutex poisoned")
            .finish(initial_state)
    }

    fn num_of_transitions(&self) -> usize {
        self.inner
            .lock()
            .expect("MutexLtsBuilder mutex poisoned")
            .num_of_transitions()
    }

    fn num_of_states(&self) -> usize {
        self.inner
            .lock()
            .expect("MutexLtsBuilder mutex poisoned")
            .num_of_states()
    }

    fn require_num_of_states(&mut self, num_states: usize) {
        self.inner
            .get_mut()
            .expect("MutexLtsBuilder mutex poisoned")
            .require_num_of_states(num_states)
    }
}

impl<L: TransitionLabel, B: LtsBuilder<L> + Send> ConcurrentLtsBuilder<L> for MutexLtsBuilder<B> {
    fn add_transition_shared<Q>(&self, from: StateIndex, label: &Q, to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash,
    {
        self.inner
            .lock()
            .expect("MutexLtsBuilder mutex poisoned")
            .add_transition(from, label, to)
    }
}

/// One state's outgoing transitions, being accumulated so they can be deduplicated before being
/// forwarded on. Exposed directly for callers -- a sequential exploration driver flushing into
/// its own builder, or parallel exploration where each worker thread needs its own buffer rather
/// than one shared through a builder -- that need per-state buffering without going through the
/// [`LtsBuilder`] trait itself.
///
/// # Details
///
/// Exploration algorithms such as explicit LPS exploration and LTS combination (parallel
/// composition with hiding/allow/comm) can produce the same `(label, to)` pair more than once for
/// a given `from` state -- e.g. two different LPS summands instantiating to the same action and
/// successor, or two different underlying-LTS transition combinations that both reduce to the
/// same hidden `tau`-labelled successor. A duplicate transition is redundant in the resulting LTS
/// (semantically it changes nothing), but inflates the transition count and, unless the caller
/// deduplicates, leaves the output with literal repeated entries.
///
/// Deduplicating globally (as [`LtsBuilderMem::remove_duplicates`] does) needs the whole LTS
/// materialised first. This type avoids that by exploiting the fact that both algorithms above
/// add one state's outgoing transitions consecutively (all of `from`'s transitions, then all of
/// the next state's, and so on): it only ever needs to buffer *one* state's worth of `(label,
/// to)` pairs at a time, flushing them as soon as a different `from` state is seen.
pub struct PerStateDedup<L: TransitionLabel> {
    current_from: Option<StateIndex>,
    // Insertion-order (i.e. first-sighting-order), not a `HashSet`: a `HashSet`'s iteration order
    // depends on its random per-instance hasher seed, so draining one in `flush` would make the
    // output order (and hence e.g. a written file's exact bytes) vary from run to run for no
    // semantic reason. A linear `contains`-then-push scan is fine here because a bucket is one
    // state's out-degree, not the whole LTS -- the very thing that keeps this buffer small.
    seen: Vec<(L, StateIndex)>,
}

impl<L: TransitionLabel> Default for PerStateDedup<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: TransitionLabel> PerStateDedup<L> {
    /// Creates an empty buffer.
    pub fn new() -> Self {
        Self {
            current_from: None,
            seen: Vec::new(),
        }
    }

    /// Buffers `(label, to)` under `from`. If `from` differs from whichever state is currently
    /// buffered, that state's buffered transitions are flushed (deduplicated) through
    /// `flush_one` first.
    ///
    /// # Requirements
    ///
    /// Every transition for a given `from` state must be added consecutively; if transitions for
    /// two different states are interleaved, only duplicates *within* each contiguous run are
    /// caught.
    pub fn add<Q, F>(&mut self, from: StateIndex, label: &Q, to: StateIndex, flush_one: F) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash,
        F: FnMut(StateIndex, &L, StateIndex) -> Result<(), MercError>,
    {
        if self.current_from != Some(from) {
            self.flush(flush_one)?;
            self.current_from = Some(from);
        }

        // Compares against the borrowed `label`/`to` first, so a duplicate -- the common case for
        // a label seen again from the same state.
        let already_seen = self
            .seen
            .iter()
            .any(|(seen_label, seen_to)| *seen_to == to && seen_label.borrow() == label);
        if !already_seen {
            self.seen.push((label.to_owned(), to));
        }
        Ok(())
    }

    /// Flushes any buffered transitions through `flush_one`, deduplicated, in the order they were
    /// first added.
    pub fn flush<F>(&mut self, mut flush_one: F) -> Result<(), MercError>
    where
        F: FnMut(StateIndex, &L, StateIndex) -> Result<(), MercError>,
    {
        if let Some(from) = self.current_from.take() {
            for (label, to) in self.seen.drain(..) {
                flush_one(from, &label, to)?;
            }
        }
        Ok(())
    }

    /// Returns the number of transitions currently buffered for the in-progress state.
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Returns true iff nothing is currently buffered.
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

/// This struct helps in building a labelled transition system by accumulating
/// transitions in a memory efficient way.
///
/// # Details
///
/// Transitions can be added with `add_transition`, and once all transitions
/// have been added, the labelled transition system can be constructed with
/// `finish`. An initial state can also be specified during finalization.
/// `finish(.., true)` additionally removes duplicate transitions; see
/// [`LtsBuilderMem::remove_duplicates`] for how it does so without ever fully
/// decompressing the accumulated transitions or sorting them globally.
pub struct LtsBuilderMem<L> {
    transition_from: ByteCompressedVec<StateIndex>,
    transition_labels: ByteCompressedVec<LabelIndex>,
    transition_to: ByteCompressedVec<StateIndex>,

    // This is used to keep track of the label to index mapping.
    labels_index: HashMap<L, LabelIndex>,
    labels: Vec<L>,

    /// The hidden labels that should be mapped to the hidden action.
    hidden_labels: Vec<String>,

    /// The number of states (derived from the transitions).
    num_of_states: usize,
}

impl<L: TransitionLabel> LtsBuilderMem<L> {
    /// Initializes a new empty builder.
    pub fn new(labels: Vec<L>, hidden_labels: Vec<String>) -> Self {
        Self::with_capacity(labels, hidden_labels, 0, 0, 0)
    }

    /// Initializes the builder with pre-allocated capacity for states and transitions. The number of labels
    /// can be used when labels are added dynamically.
    pub fn with_capacity(
        mut labels: Vec<L>,
        hidden_labels: Vec<String>,
        num_of_labels: usize,
        num_of_states: usize,
        num_of_transitions: usize,
    ) -> Self {
        // Remove duplicates from the labels.
        labels.sort();
        labels.dedup();

        // Introduce the fixed 0-indexed tau label.
        if let Some(tau_pos) = labels.iter().position(|l| l.is_tau_label()) {
            labels.swap(0, tau_pos);
        } else {
            labels.insert(0, L::tau_label());
        }

        // Ensure that all hidden labels are mapped to the tau action.
        let mut labels_index = HashMap::new();
        labels_index.insert(L::tau_label(), LabelIndex::new(0));
        for (index, label) in labels.iter().enumerate() {
            if hidden_labels.iter().any(|l| label.matches_label(l)) {
                labels_index.insert(label.clone(), LabelIndex::new(0)); // Map hidden labels to tau
            } else {
                labels_index.insert(label.clone(), LabelIndex::new(index));
            }
        }

        Self {
            transition_from: ByteCompressedVec::with_capacity(num_of_transitions, num_of_states.bytes_required()),
            transition_labels: ByteCompressedVec::with_capacity(
                num_of_transitions,
                num_of_labels.max(labels.len()).bytes_required(),
            ),
            transition_to: ByteCompressedVec::with_capacity(num_of_transitions, num_of_states.bytes_required()),
            labels_index,
            labels,
            hidden_labels,
            num_of_states: 0,
        }
    }

    /// Returns an iterator over all transitions as (from, label, to) tuples.
    fn iter(&self) -> impl Iterator<Item = (StateIndex, LabelIndex, StateIndex)> {
        self.transition_from
            .iter()
            .zip(self.transition_labels.iter())
            .zip(self.transition_to.iter())
            .map(|((from, label), to)| (from, label, to))
    }

    /// Returns the approximate number of bytes used to store the accumulated
    /// transitions.
    pub fn memory_usage(&self) -> usize {
        self.transition_from.metrics().actual_memory
            + self.transition_labels.metrics().actual_memory
            + self.transition_to.metrics().actual_memory
    }

    /// Finalizes the builder and returns the constructed labelled transition system.
    ///
    /// If `remove_duplicates` is true, duplicate `(from, label, to)` transitions are
    /// removed first. See [`LtsBuilderMem::remove_duplicates`] for how this is done
    /// without fully decompressing the accumulated transitions.
    pub fn finish(&mut self, initial_state: StateIndex, remove_duplicates: bool) -> LabelledTransitionSystem<L> {
        if remove_duplicates {
            self.remove_duplicates();
        }

        LabelledTransitionSystem::new(
            initial_state,
            Some(self.num_of_states),
            || self.iter(),
            self.labels.clone(),
        )
    }

    /// Removes duplicate `(from, label, to)` transitions in place, deduplicating via a
    /// bucket (counting) sort on `from` followed by [`merc_collections::dedup_grouped`]
    /// (see its docs for the within-bucket algorithm).
    ///
    /// The counting sort itself goes through [`merc_collections::scatter_into_buckets`],
    /// which scatters `(label, to)` directly into the (byte-compressed) grouped columns via
    /// a write callback, rather than building an explicit `position -> original_index`
    /// permutation for the caller to gather through: for a large LTS, a full-width `usize`
    /// permutation (8 bytes per transition, regardless of how few bits
    /// `StateIndex`/`LabelIndex` actually need) would be the dominant term in this
    /// function's peak memory footprint, on top of the old and new transition columns it
    /// would sit alongside; scattering directly needs only the returned `bucket_ends`, of
    /// length `num_states` -- typically far smaller than `num_transitions`.
    fn remove_duplicates(&mut self) {
        let num_states = self.num_of_states;
        let num_transitions = self.transition_from.len();

        if num_transitions == 0 {
            return;
        }

        let mut grouped_labels = bytevec![LabelIndex::new(0); num_transitions];
        let mut grouped_to = bytevec![StateIndex::new(0); num_transitions];
        let bucket_ends = merc_collections::scatter_into_buckets(
            num_states,
            num_transitions,
            |i| self.transition_from.index(i).value(),
            |position, i| {
                grouped_labels.set(position, self.transition_labels.index(i));
                grouped_to.set(position, self.transition_to.index(i));
            },
        );

        // Free the old columns now that they're copied into `grouped_labels`/`grouped_to`,
        // rather than holding both alive at once during the compaction below.
        self.transition_from = ByteCompressedVec::new();
        self.transition_labels = ByteCompressedVec::new();
        self.transition_to = ByteCompressedVec::new();

        // `key_of` (reading) and the `Keep` arm (writing, to compact in place) both
        // need access to `grouped_labels`/`grouped_to`, so they're wrapped in a
        // `RefCell`: the two never actually borrow at the same time, just at a point
        // the borrow checker can't see through two separate closures.
        let mut new_transition_from = ByteCompressedVec::with_capacity(num_transitions, num_states.bytes_required());
        let mut write = 0usize;
        let grouped_labels = RefCell::new(grouped_labels);
        let grouped_to = RefCell::new(grouped_to);

        merc_collections::dedup_grouped(
            &bucket_ends,
            |position| {
                (
                    grouped_labels.borrow().index(position),
                    grouped_to.borrow().index(position),
                )
            },
            |position, state, outcome| {
                if let merc_collections::DedupOutcome::Keep { .. } = outcome {
                    if write != position {
                        let label = grouped_labels.borrow().index(position);
                        let to = grouped_to.borrow().index(position);
                        grouped_labels.borrow_mut().set(write, label);
                        grouped_to.borrow_mut().set(write, to);
                    }
                    new_transition_from.push(StateIndex::new(state));
                    write += 1;
                }
            },
        );

        let mut grouped_labels = grouped_labels.into_inner();
        let mut grouped_to = grouped_to.into_inner();

        grouped_labels.resize_with(write, || unreachable!("compaction never grows the vector"));
        grouped_to.resize_with(write, || unreachable!("compaction never grows the vector"));

        // `resize_with` doesn't release excess capacity on its own, so shrink
        // explicitly to actually give the memory back.
        new_transition_from.shrink_to_fit();
        grouped_labels.shrink_to_fit();
        grouped_to.shrink_to_fit();

        self.transition_from = new_transition_from;
        self.transition_labels = grouped_labels;
        self.transition_to = grouped_to;
    }
}

impl<L: TransitionLabel> LtsBuilder<L> for LtsBuilderMem<L> {
    type LTS = LabelledTransitionSystem<L>;

    fn add_transition<Q>(&mut self, from: StateIndex, label: &Q, to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash,
    {
        let label_index = if let Some(&index) = self.labels_index.get(label) {
            index
        } else {
            // Label was not yet added, so add it to the labels and the index.
            let label = label.to_owned();
            let index = if self.hidden_labels.iter().any(|l| label.matches_label(l)) {
                LabelIndex::new(0) // Map hidden labels to tau
            } else {
                let idx = LabelIndex::new(self.labels.len());
                self.labels.push(label.clone());
                idx
            };
            self.labels_index.insert(label, index);
            index
        };

        self.transition_from.push(from);
        self.transition_labels.push(label_index);
        self.transition_to.push(to);

        // Update the number of states.
        self.num_of_states = self.num_of_states.max(from.value() + 1).max(to.value() + 1);
        Ok(())
    }

    fn finish(&mut self, initial_state: StateIndex) -> Result<Self::LTS, MercError> {
        Ok(self.finish(initial_state, false))
    }

    fn num_of_transitions(&self) -> usize {
        self.transition_from.len()
    }

    fn num_of_states(&self) -> usize {
        self.num_of_states
    }

    fn require_num_of_states(&mut self, num_of_states: usize) {
        if num_of_states > self.num_of_states {
            self.num_of_states = num_of_states;
        }
    }
}

impl<Label: TransitionLabel> fmt::Debug for LtsBuilderMem<Label> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Transitions:")?;
        for (from, label, to) in self.iter() {
            writeln!(f, "    {:?} --[{:?}]-> {:?}", from, label, to)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rand::RngExt;

    use merc_utilities::random_test;

    use crate::LTS;
    use crate::LabelIndex;
    use crate::LabelledTransitionSystem;
    use crate::LtsBuilder;
    use crate::LtsBuilderMem;
    use crate::PerStateDedup;
    use crate::StateIndex;
    use crate::TransitionLabel;

    // The underlying dedup algorithm (matching a naive `HashSet`-based reference,
    // its high-out-degree "hub" hash-map path, and the in-place compaction this
    // builder specifically relies on) is covered once, generically, by
    // `merc_collections::remove_duplicates`'s own tests. This test only checks
    // that `LtsBuilderMem` wires its own columns into it correctly.
    #[test]
    fn test_random_remove_duplicates() {
        random_test(100, |rng| {
            let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
            let mut builder = LtsBuilderMem::new(labels.clone(), Vec::new());

            for _ in 0..rng.random_range(0..10) {
                let from = StateIndex::new(rng.random_range(0..10));
                let label = LabelIndex::new(rng.random_range(0..2));
                let to = StateIndex::new(rng.random_range(0..10));
                builder.add_transition(from, &labels[label], to).unwrap();
            }

            builder.remove_duplicates();

            let transitions = builder.iter().collect::<Vec<_>>();
            debug_assert!(
                transitions.iter().all_unique(),
                "Transitions should be unique after removing duplicates"
            );
        });
    }

    /// Collects `(from, label, to)` for every transition in `lts`, in no particular order.
    fn all_transitions<L: TransitionLabel>(
        lts: &LabelledTransitionSystem<L>,
    ) -> Vec<(StateIndex, LabelIndex, StateIndex)> {
        lts.iter_states()
            .flat_map(|state| lts.outgoing_transitions(state).map(move |t| (state, t.label, t.to)))
            .collect()
    }

    #[test]
    fn test_per_state_dedup_removes_within_state_duplicates() {
        let labels = vec!["a".to_string(), "b".to_string()];
        let mut inner = LtsBuilderMem::new(labels.clone(), Vec::new());
        let mut dedup = PerStateDedup::new();

        // State 0 has a duplicate (0, "b", 1) transition, plus a genuinely distinct (0, "b", 2).
        dedup
            .add(StateIndex::new(0), &labels[1], StateIndex::new(1), |from, label, to| {
                inner.add_transition(from, label, to)
            })
            .unwrap();
        dedup
            .add(StateIndex::new(0), &labels[1], StateIndex::new(1), |from, label, to| {
                inner.add_transition(from, label, to)
            })
            .unwrap();
        dedup
            .add(StateIndex::new(0), &labels[1], StateIndex::new(2), |from, label, to| {
                inner.add_transition(from, label, to)
            })
            .unwrap();
        // State 1 happens to have the exact same (label, to) pair as one of state 0's; it must
        // NOT be removed, since it belongs to a different `from` state.
        dedup
            .add(StateIndex::new(1), &labels[1], StateIndex::new(1), |from, label, to| {
                inner.add_transition(from, label, to)
            })
            .unwrap();
        dedup
            .flush(|from, label, to| inner.add_transition(from, label, to))
            .unwrap();

        let result = inner.finish(StateIndex::new(0), false);

        assert_eq!(
            all_transitions(&result).len(),
            3,
            "the repeated (0, b, 1) transition must be removed, but not the unrelated (1, b, 1)"
        );
    }

    #[test]
    fn test_random_per_state_dedup() {
        random_test(100, |rng| {
            let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
            let mut inner = LtsBuilderMem::new(labels.clone(), Vec::new());
            let mut dedup = PerStateDedup::new();

            // Add every state's transitions consecutively, as `PerStateDedup` requires, with
            // deliberate repeats within each state to exercise the deduplication itself.
            for from in 0..10 {
                for _ in 0..rng.random_range(0..8) {
                    let label = LabelIndex::new(rng.random_range(0..2));
                    let to = StateIndex::new(rng.random_range(0..10));
                    dedup
                        .add(StateIndex::new(from), &labels[label], to, |from, label, to| {
                            inner.add_transition(from, label, to)
                        })
                        .unwrap();
                }
            }
            dedup
                .flush(|from, label, to| inner.add_transition(from, label, to))
                .unwrap();

            let result = inner.finish(StateIndex::new(0), false);

            assert!(
                all_transitions(&result).iter().all_unique(),
                "Transitions should be unique after going through PerStateDedup"
            );
        });
    }
}
