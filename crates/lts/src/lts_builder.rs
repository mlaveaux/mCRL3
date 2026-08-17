#![forbid(unsafe_code)]

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Mutex;

use rustc_hash::FxHashSet;

use merc_collections::ByteCompressedVec;
use merc_collections::CompressedEntry;
use merc_collections::bytevec;
use merc_utilities::MercError;

use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::StateIndex;
use crate::TransitionLabel;

/// Below this many transitions in a single state's outgoing bucket, duplicates
/// are detected with a linear scan (cheap, no allocation). Above it, a reused
/// hash set scratch buffer is used instead, so a handful of very high
/// out-degree "hub" states cannot make [`LtsBuilderMem::remove_duplicates`]
/// quadratic in the bucket size.
const HASH_DEDUP_THRESHOLD: usize = 16;

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

    /// Removes duplicate `(from, label, to)` transitions in place.
    ///
    /// # Details
    ///
    /// Two transitions can only be duplicates of each other if they share the same
    /// `from` state, so this never needs a global sort of the transitions. Instead it
    /// reuses the same counting-sort/bucket-fill idiom that [`LabelledTransitionSystem::new`]
    /// already uses to group transitions by `from` state:
    ///
    /// 1. Count outgoing transitions per state, turn the counts into prefix-sum start
    ///    offsets.
    /// 2. Scatter every `(label, to)` pair into a fresh pair of vectors at its state's
    ///    bucket, preserving insertion order within the bucket.
    /// 3. Walk each state's now-contiguous bucket once, compacting it in place to keep
    ///    only the first occurrence of every `(label, to)` pair (small buckets use a
    ///    linear scan, large ones a reused hash set, see [`HASH_DEDUP_THRESHOLD`]),
    ///    rebuilding `transition_from` alongside it in the same pass.
    ///
    /// Every pass above is a purely sequential scan (no permutation, no random-access
    /// gather/scatter through the byte-compressed columns), which is what keeps this
    /// fast despite operating on the compressed representation. The per-state
    /// bookkeeping (`offsets` below) is a plain `Vec<usize>`, not a `ByteCompressedVec`:
    /// it is transient (dropped once this function returns, unlike the transitions
    /// themselves, which persist in the builder) and touched once per state
    /// regardless of that state's out-degree, so on a sparse LTS (many states, few
    /// transitions each) it dominates the cost - not worth paying byte encode/decode
    /// overhead for.
    fn remove_duplicates(&mut self) {
        let num_states = self.num_of_states;
        let num_transitions = self.transition_from.len();

        if num_transitions == 0 {
            return;
        }

        // Pass 1: count outgoing transitions per `from` state, then turn the counts
        // into prefix-sum start offsets.
        let mut offsets = vec![0usize; num_states];
        for from in self.transition_from.iter() {
            offsets[*from] += 1;
        }

        let mut running = 0usize;
        for count in &mut offsets {
            let bucket_len = *count;
            *count = running;
            running += bucket_len;
        }

        // Pass 2: scatter (label, to) into per-`from` contiguous buckets, preserving
        // the original insertion order within every bucket. `offsets` doubles as the
        // running cursor here, so afterwards `offsets[state]` holds the *end* of that
        // state's bucket (and thus the start of state+1's, since buckets are
        // contiguous) rather than its start.
        let mut grouped_labels = bytevec![LabelIndex::new(0); num_transitions];
        let mut grouped_to = bytevec![StateIndex::new(0); num_transitions];
        for ((from, label), to) in self
            .transition_from
            .iter()
            .zip(self.transition_labels.iter())
            .zip(self.transition_to.iter())
        {
            let pos = &mut offsets[*from];
            grouped_labels.set(*pos, label);
            grouped_to.set(*pos, to);
            *pos += 1;
        }

        // The old columns have now been fully copied into `grouped_labels`/`grouped_to`
        // above, so free them before the compaction pass below rather than holding
        // both the old and new copies of the (potentially large) transition data alive
        // at once - that transient doubling is otherwise the dominant contributor to
        // this function's peak memory usage.
        self.transition_from = ByteCompressedVec::new();
        self.transition_labels = ByteCompressedVec::new();
        self.transition_to = ByteCompressedVec::new();

        // Pass 3: compact every bucket in place, keeping only the first occurrence of
        // each (label, to) pair, and rebuild `transition_from` alongside it. The write
        // cursor never exceeds the read cursor, so this is a forward-only compaction,
        // not a permutation.
        let mut new_transition_from = ByteCompressedVec::with_capacity(num_transitions, num_states.bytes_required());
        let mut scratch: FxHashSet<(LabelIndex, StateIndex)> = FxHashSet::default();
        let mut write = 0usize;
        let mut start = 0usize;

        for (state, &end) in offsets.iter().enumerate() {
            let bucket_start = write;

            if end - start <= HASH_DEDUP_THRESHOLD {
                // Small bucket: a linear scan avoids any allocation.
                for read in start..end {
                    let label = grouped_labels.index(read);
                    let to = grouped_to.index(read);
                    let is_duplicate = (bucket_start..write)
                        .any(|seen| grouped_labels.index(seen) == label && grouped_to.index(seen) == to);

                    if !is_duplicate {
                        if write != read {
                            grouped_labels.set(write, label);
                            grouped_to.set(write, to);
                        }
                        new_transition_from.push(StateIndex::new(state));
                        write += 1;
                    }
                }
            } else {
                // Large ("hub state") bucket: a hash set keeps this from degrading
                // to O(bucket_len^2).
                scratch.clear();
                for read in start..end {
                    let label = grouped_labels.index(read);
                    let to = grouped_to.index(read);

                    if scratch.insert((label, to)) {
                        if write != read {
                            grouped_labels.set(write, label);
                            grouped_to.set(write, to);
                        }
                        new_transition_from.push(StateIndex::new(state));
                        write += 1;
                    }
                }
            }

            start = end;
        }

        grouped_labels.resize_with(write, || unreachable!("compaction never grows the vector"));
        grouped_to.resize_with(write, || unreachable!("compaction never grows the vector"));

        // `resize_with` above (like `Vec::truncate`) only reduces the logical length,
        // it does not release the excess capacity allocated for the pre-dedup entry
        // count on its own - shrink explicitly so a duplicate-heavy builder actually
        // gives that memory back rather than merely reporting a smaller logical size.
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
    use std::collections::HashSet;

    use itertools::Itertools;
    use rand::RngExt;

    use merc_utilities::random_test;

    use crate::LTS;
    use crate::LabelIndex;
    use crate::LtsBuilder;
    use crate::LtsBuilderMem;
    use crate::StateIndex;

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

    /// Checks `LtsBuilderMem`'s bucket-local dedup against a trivial, independently
    /// implemented reference (deduplicating the raw transitions with a `HashSet`),
    /// so the cheaper bucket-local approach is not silently missing cases a naive
    /// approach would catch - i.e. this checks *completeness* (no unique transition
    /// is accidentally dropped), complementing [`test_random_remove_duplicates`]'s
    /// *soundness* check (no duplicate survives).
    #[test]
    fn test_random_remove_duplicates_matches_naive_reference() {
        random_test(100, |rng| {
            let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
            let mut builder = LtsBuilderMem::new(labels.clone(), Vec::new());

            let max_state_count = rng.random_range(1..20);
            let mut expected: HashSet<(StateIndex, String, StateIndex)> = HashSet::new();
            for _ in 0..rng.random_range(0..50) {
                let from = StateIndex::new(rng.random_range(0..max_state_count));
                let label = &labels[rng.random_range(0..2)];
                let to = StateIndex::new(rng.random_range(0..max_state_count));

                builder.add_transition(from, label, to).unwrap();
                expected.insert((from, label.clone(), to));
            }

            let initial_state = StateIndex::new(rng.random_range(0..max_state_count));
            let lts = builder.finish(initial_state, true);

            let mut actual: HashSet<(StateIndex, String, StateIndex)> = HashSet::new();
            for from in lts.iter_states() {
                for t in lts.outgoing_transitions(from) {
                    actual.insert((from, lts.labels()[t.label].clone(), t.to));
                }
            }

            assert_eq!(
                actual, expected,
                "Deduplicated transitions should match a naive HashSet-based reference"
            );
        });
    }

    /// A single high-out-degree "hub" state exercises the hash-set dedup path (as
    /// opposed to the linear-scan path used for small buckets).
    #[test]
    fn test_remove_duplicates_hub_state() {
        let labels: Vec<String> = (0..50).map(|i| format!("a{i}")).collect();
        let mut builder = LtsBuilderMem::new(labels.clone(), Vec::new());

        let hub = StateIndex::new(0);
        // Add every label twice, from the hub state to a handful of targets, so almost
        // every transition is a duplicate of one already placed in this bucket.
        for _ in 0..2 {
            for (index, label) in labels.iter().enumerate() {
                let to = StateIndex::new(1 + index % 5);
                builder.add_transition(hub, label, to).unwrap();
            }
        }

        let lts = builder.finish(hub, true);
        assert_eq!(lts.num_of_transitions(), labels.len());

        let transitions: Vec<_> = lts.outgoing_transitions(hub).map(|t| (t.label, t.to)).collect();
        assert!(
            transitions.iter().all_unique(),
            "Hub state transitions should be unique"
        );
    }
}
