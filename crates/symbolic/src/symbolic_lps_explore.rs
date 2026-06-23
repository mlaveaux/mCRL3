use std::fmt;
use std::rc::Rc;

use merc_collections::IndexedSet;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_utilities::MercError;
use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;
use streaming_iterator::StreamingIterator;

use crate::SymbolicLPS;
use crate::TransitionGroup;
use crate::iter;

/// A symbolic LDD view of any [`merc_explore::LPS`].
///
/// This adapts the explicit-exploration abstraction (an `LPS` exposing summands
/// with read/write positions, a `prepare` step selecting the summands that fire
/// from a state, and per-summand enumeration) into the disjunctive LDD
/// transition relation consumed by [`crate::reachability`]. A single generic
/// implementation therefore drives symbolic exploration for both LPSs and PBESs
/// in SRF form (and anything else implementing `LPS`).
///
/// The wrapped `LPS` interns parameter values in its own space; this adapter
/// keeps a separate per-position interning into dense LDD values so the decision
/// diagrams stay compact regardless of how the `LPS` numbers its values.
pub struct SymbolicLps<L: LPS> {
    /// The wrapped `LPS` definition; the single source of summands and the
    /// `prepare`/`enumerate` machinery. Shared immutably with every group.
    lps: Rc<L>,

    /// One symbolic transition group per summand.
    groups: Vec<SymbolicLpsGroup<L>>,

    /// The encoded initial state.
    initial_state: LDDFunction,
}

/// Per-run learning state threaded through [SymbolicLpsGroup::learn_successors].
///
/// Holds the mutable state shared by all groups of one [SymbolicLps], so the
/// groups themselves need no interior mutability.
pub struct SymbolicContext<L: LPS> {
    /// The enumeration backend created by [`LPS::create_context`].
    enumerate: <L::Summand as Summand>::Context,

    /// Per state-vector position interning of `LPS` values into dense LDD values.
    columns: Vec<IndexedSet<L::Value>>,
}

impl<L: LPS> SymbolicLps<L> {
    /// Wraps `lps` into a symbolic LDD view, encoding its initial state.
    pub fn new(manager: &LDDManagerRef, lps: L) -> Result<Self, MercError> {
        let lps = Rc::new(lps);

        let initial_values = lps.initial_state();
        let num_positions = initial_values.len();

        let mut columns: Vec<IndexedSet<L::Value>> = (0..num_positions).map(|_| IndexedSet::new()).collect();

        // Encode the initial state, interning each value into its column.
        let mut initial_vector: Vec<Value> = Vec::with_capacity(num_positions);
        for (position, value) in initial_values.iter().enumerate() {
            let (index, _) = columns[position].insert(*value);
            initial_vector.push(*index as Value);
        }
        let initial_state = manager.with_manager_shared(|m| LDDFunction::singleton(m, &initial_vector))?;

        // Build one symbolic group per summand.
        let mut groups = Vec::with_capacity(lps.summands().len());
        for (index, summand) in lps.summands().iter().enumerate() {
            let mut read_indices: Vec<Value> = summand.read_positions().iter().map(|&p| p as Value).collect();
            let mut write_indices: Vec<Value> = summand.write_positions().iter().map(|&p| p as Value).collect();
            // The short-vector encoding requires sorted read/write indices.
            read_indices.sort_unstable();
            write_indices.sort_unstable();

            let (project_ldd, meta, read_positions, write_positions, relation) = manager
                .with_manager_shared(|m| -> Result<_, MercError> {
                    let project_ldd = LDDFunction::projection_meta(m, &read_indices)?;
                    let (meta, read_positions, write_positions) =
                        LDDFunction::relation_product_meta(m, &read_indices, &write_indices)?;
                    let relation = LDDFunction::empty_set(m)?;
                    Ok((project_ldd, meta, read_positions, write_positions, relation))
                })?;

            groups.push(SymbolicLpsGroup {
                lps: Rc::clone(&lps),
                index,
                read_indices,
                write_indices,
                read_positions,
                write_positions,
                project_ldd,
                meta,
                relation,
            });
        }

        Ok(SymbolicLps {
            lps,
            groups,
            initial_state,
        })
    }

    /// Builds the per-position interning seeded with the initial-state values.
    ///
    /// Re-inserting `initial_state()` into fresh per-column sets reproduces the
    /// exact dense indices used when [Self::new] encoded the initial LDD, so the
    /// learned relations stay consistent with it.
    fn initial_columns(&self) -> Vec<IndexedSet<L::Value>> {
        let initial_values = self.lps.initial_state();
        let mut columns: Vec<IndexedSet<L::Value>> = (0..initial_values.len()).map(|_| IndexedSet::new()).collect();
        for (position, value) in initial_values.iter().enumerate() {
            columns[position].insert(*value);
        }
        columns
    }
}

impl<L: LPS> SymbolicLPS for SymbolicLps<L> {
    type Group = SymbolicLpsGroup<L>;

    fn initial_state(&self) -> &LDDFunction {
        &self.initial_state
    }

    fn transition_groups(&self) -> &[Self::Group] {
        &self.groups
    }

    fn transition_groups_mut(&mut self) -> &mut [Self::Group] {
        &mut self.groups
    }

    fn create_context(&self) -> SymbolicContext<L> {
        SymbolicContext {
            enumerate: self.lps.create_context(),
            columns: self.initial_columns(),
        }
    }
}

/// A single summand of a [`SymbolicLps`], encoded as a short-vector LDD
/// transition relation that is learned on the fly during reachability.
pub struct SymbolicLpsGroup<L: LPS> {
    /// The wrapped `LPS`, shared immutably with [SymbolicLps].
    lps: Rc<L>,

    /// Index of this summand within `lps.summands()`.
    index: usize,

    /// Full state-vector positions read by the summand (sorted).
    read_indices: Vec<Value>,

    /// Full state-vector positions written by the summand (sorted).
    write_indices: Vec<Value>,

    /// Interleaved short-vector positions of `read_indices`.
    read_positions: Vec<usize>,

    /// Interleaved short-vector positions of `write_indices`.
    write_positions: Vec<usize>,

    /// Projection meta selecting `read_indices` from a full state.
    project_ldd: LDDFunction,

    /// Relational-product meta for `read_indices`/`write_indices`.
    meta: LDDFunction,

    /// The learned transition relation `T' -> U'`, grown on the fly.
    relation: LDDFunction,
}

impl<L: LPS> TransitionGroup for SymbolicLpsGroup<L> {
    type Context = SymbolicContext<L>;

    fn relation(&self) -> &LDDFunction {
        &self.relation
    }

    fn read_indices(&self) -> &[u32] {
        &self.read_indices
    }

    fn write_indices(&self) -> &[u32] {
        &self.write_indices
    }

    fn action_label_index(&self) -> Option<usize> {
        None
    }

    fn meta(&self) -> &LDDFunction {
        &self.meta
    }

    fn learn_successors(
        &mut self,
        context: &mut SymbolicContext<L>,
        storage: &LDDManagerRef,
        todo: &LDDFunction,
    ) -> Result<(), MercError> {
        let proj = todo.project(&self.project_ldd)?;

        // Borrow the backend and the interning as disjoint fields so the
        // enumeration callback can grow the interning while the backend call is
        // in progress.
        let SymbolicContext { enumerate, columns } = context;

        // Reusable full-length state buffer. Non-read positions keep the initial
        // state's values: they are never read by this summand (guaranteed by the
        // read-positions contract), so any valid value works; the read positions
        // are overlaid from each short state below.
        let mut full_state = self.lps.initial_state();

        // Reusable interleaved short vector for the relation singletons.
        let mut interleaved: Vec<Value> = vec![0; self.read_indices.len() + self.write_indices.len()];

        // Accumulate into a local relation so the enumeration callback does not
        // have to borrow `self`.
        let mut relation = self.relation.clone();

        let summand = &self.lps.summands()[self.index];

        let mut states = iter(&proj);
        while let Some(short_state) = states.next() {
            debug_assert_eq!(
                short_state.len(),
                self.read_indices.len(),
                "Projected state must have one value per read index"
            );

            // Overlay the short read values into the full state buffer and the
            // interleaved read positions.
            for (k, &value) in short_state.iter().enumerate() {
                let full_pos = self.read_indices[k] as usize;
                full_state[full_pos] = *columns[full_pos]
                    .get_by_index(value as usize)
                    .expect("read value must already be interned");
                interleaved[self.read_positions[k]] = value;
            }

            // Prepare the backend assignments for this state.
            let _ = self.lps.prepare(enumerate, &full_state);

            // Enumerate the successors, building the write side of each short
            // transition vector and unioning it into the relation.
            let write_indices = &self.write_indices;
            let write_positions = &self.write_positions;
            let interleaved = &mut interleaved;
            let relation = &mut relation;

            summand.enumerate(enumerate, &full_state, |_label, next_state| {
                for (m, &full_pos) in write_indices.iter().enumerate() {
                    let (index, _) = columns[full_pos as usize].insert(next_state[full_pos as usize]);
                    interleaved[write_positions[m]] = *index as Value;
                }

                let cube = storage.with_manager_shared(|m| LDDFunction::singleton(m, interleaved.as_slice()))?;
                *relation = relation.union(&cube)?;
                Ok(())
            })?;
        }

        self.relation = relation;
        Ok(())
    }
}

impl<L: LPS> fmt::Debug for SymbolicLps<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SymbolicLps:")?;
        for (i, group) in self.groups.iter().enumerate() {
            writeln!(f, "  {i}: {group:?}")?;
        }
        Ok(())
    }
}

impl<L: LPS> fmt::Debug for SymbolicLpsGroup<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "group {} (read {:?}, write {:?})",
            self.index, self.read_indices, self.write_indices
        )
    }
}
