use std::fmt;
use std::rc::Rc;

use log::debug;

use merc_collections::IndexedSet;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_utilities::MercError;
use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::RelationProductMeta;
use oxidd::ldd::Value;
use streaming_iterator::StreamingIterator;

use crate::ReadWritePattern;
use crate::SummandGrouping;
use crate::SymbolicLPS;
use crate::TransitionGroup;
use crate::VariableOrder;
use crate::inverse_order;
use crate::iter;
use crate::print_read_write_patterns;
use crate::print_transition_groups;
use crate::transition_group_pattern;

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

    /// The symbolic transition groups, as determined by the [`SummandGrouping`].
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

/// The choices that shape the symbolic encoding of an [`merc_explore::LPS`], mirroring the `--groups`
/// and `--reorder` options of the mCRL2 symbolic tools.
///
/// Neither option changes the reachable set, only the size of the decision diagrams and the amount of
/// work per exploration step.
#[derive(Clone, Debug, Default)]
pub struct SymbolicLpsOptions {
    /// How the summands are distributed over the transition groups.
    pub grouping: SummandGrouping,

    /// The order in which the state vector positions are stored in the diagram.
    pub order: VariableOrder,
}

impl<L: LPS> SymbolicLps<L> {
    /// Wraps `lps` into a symbolic LDD view, encoding its initial state, with one transition group
    /// per summand.
    pub fn new(manager: &LDDManagerRef, lps: L) -> Result<Self, MercError> {
        SymbolicLps::with_options(manager, lps, &SymbolicLpsOptions::default())
    }

    /// Wraps `lps` into a symbolic LDD view, encoding its initial state, distributing its summands
    /// over the transition groups according to `grouping`.
    ///
    /// Joining summands into a single group trades the number of relational products per exploration
    /// step against the size of each transition relation; it does not change the reachable set.
    pub fn with_grouping(manager: &LDDManagerRef, lps: L, grouping: &SummandGrouping) -> Result<Self, MercError> {
        let options = SymbolicLpsOptions {
            grouping: grouping.clone(),
            ..SymbolicLpsOptions::default()
        };
        SymbolicLps::with_options(manager, lps, &options)
    }

    /// Wraps `lps` into a symbolic LDD view, encoding its initial state, with the given grouping of
    /// its summands and order of its state vector positions.
    pub fn with_options(manager: &LDDManagerRef, lps: L, options: &SymbolicLpsOptions) -> Result<Self, MercError> {
        let lps = Rc::new(lps);

        let initial_values = lps.initial_state();
        let num_positions = initial_values.len();

        let patterns = read_write_patterns(&*lps, num_positions)?;
        debug!(
            "Read/write matrix of the summands:\n{}",
            print_read_write_patterns(&patterns)
        );

        // `order[level]` is the state vector position stored at `level` of the diagram, and
        // `level_of[position]` its inverse. The `columns` and the states of the wrapped `LPS` stay in
        // position space; only the short vectors and their metas live in level space. The order is
        // used here only: every group stores the positions its short vectors carry, so learning needs
        // no translation between the two spaces.
        let order = options.order.compute(&patterns, num_positions)?;
        let level_of = inverse_order(&order);

        let mut columns: Vec<IndexedSet<L::Value>> = (0..num_positions).map(|_| IndexedSet::new()).collect();

        // Encode the initial state, interning each value into its column.
        let mut initial_vector: Vec<Value> = Vec::with_capacity(num_positions);
        for &position in order.iter() {
            let (index, _) = columns[position].insert(initial_values[position]);
            initial_vector.push(*index as Value);
        }
        let initial_state = manager.with_manager_shared(|m| LDDFunction::singleton(m, &initial_vector))?;

        // Distribute the summands over the transition groups, and derive the read/write pattern each
        // group has to cover.
        let group_indices = options.grouping.compute(&patterns)?;
        let group_patterns = group_indices
            .iter()
            .map(|indices| transition_group_pattern(&patterns, indices))
            .collect::<Result<Vec<_>, MercError>>()?;

        if !matches!(options.grouping, SummandGrouping::None) || !matches!(options.order, VariableOrder::None) {
            // Shown in the variable order, i.e. the way the transition relations store it.
            let permuted = group_patterns
                .iter()
                .map(|pattern| pattern.permute(&order))
                .collect::<Result<Vec<_>, MercError>>()?;

            debug!(
                "Read/write matrix of the transition groups (grouping {}):\n{}",
                options.grouping,
                print_transition_groups(&group_indices, &permuted)
            );
        }

        let mut groups = Vec::with_capacity(group_indices.len());
        for (indices, pattern) in group_indices.into_iter().zip(group_patterns) {
            // The short-vector encoding requires sorted read/write indices, in level space. Sort
            // (level, position) pairs so that the state vector position of every short vector entry
            // is kept alongside the level it is stored at.
            let mut read_pairs: Vec<(Value, usize)> =
                pattern.read_positions().map(|p| (level_of[p] as Value, p)).collect();
            let mut write_pairs: Vec<(Value, usize)> =
                pattern.write_positions().map(|p| (level_of[p] as Value, p)).collect();
            read_pairs.sort_unstable();
            write_pairs.sort_unstable();

            let (read_indices, read_full_positions): (Vec<Value>, Vec<usize>) = read_pairs.into_iter().unzip();
            let (write_indices, write_full_positions): (Vec<Value>, Vec<usize>) = write_pairs.into_iter().unzip();

            let (project_ldd, meta, read_positions, write_positions, relation, domain) =
                manager.with_manager_shared(|m| -> Result<_, MercError> {
                    let project_ldd = LDDFunction::projection_meta(m, &read_indices)?;
                    let RelationProductMeta {
                        meta,
                        read_positions,
                        write_positions,
                    } = LDDFunction::relation_product_meta(m, &read_indices, &write_indices)?;
                    let relation = LDDFunction::empty_set(m)?;
                    let domain = LDDFunction::empty_set(m)?;
                    Ok((project_ldd, meta, read_positions, write_positions, relation, domain))
                })?;

            groups.push(SymbolicLpsGroup {
                lps: Rc::clone(&lps),
                indices,
                read_indices,
                write_indices,
                read_full_positions,
                write_full_positions,
                read_positions,
                write_positions,
                project_ldd,
                meta,
                relation,
                domain,
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

/// Returns the read/write pattern of every summand of `lps` over the `num_positions` positions of its
/// state vector.
fn read_write_patterns<L: LPS>(lps: &L, num_positions: usize) -> Result<Vec<ReadWritePattern>, MercError> {
    lps.summands()
        .iter()
        .enumerate()
        .map(|(index, summand)| {
            // A symbolic transition relation is inherently positional: it stores only the read and
            // write columns, so there is no encoding for a summand whose next states change shape.
            let effect = summand.effect();
            let write_positions = effect.positions().ok_or_else(|| {
                MercError::from(format!(
                    "summand {index} has an opaque state effect, which symbolic exploration cannot encode"
                ))
            })?;

            ReadWritePattern::from_indices(num_positions, summand.read_positions(), write_positions)
                .map_err(|error| MercError::from(format!("summand {index}: {error}")))
        })
        .collect()
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

/// A group of summands of a [`SymbolicLps`], encoded as a single short-vector LDD
/// transition relation that is learned on the fly during reachability.
pub struct SymbolicLpsGroup<L: LPS> {
    /// The wrapped `LPS`, shared immutably with [SymbolicLps].
    lps: Rc<L>,

    /// Indices of the summands in this group within `lps.summands()`, in increasing order.
    indices: Vec<usize>,

    /// Diagram levels read by the group (sorted).
    read_indices: Vec<Value>,

    /// Diagram levels written by the group (sorted).
    write_indices: Vec<Value>,

    /// State vector positions of `read_indices`, i.e. the position whose value the `k`-th entry of a
    /// short read vector holds. This is where the variable order enters the group; it needs no other
    /// knowledge of it.
    read_full_positions: Vec<usize>,

    /// State vector positions of `write_indices`, see [Self::read_full_positions].
    write_full_positions: Vec<usize>,

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

    /// The domain of [Self::relation]: the projections on `read_indices` whose successors have
    /// already been learned. Only grown when learning is requested with caching, and empty otherwise.
    domain: LDDFunction,
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
        cached: bool,
    ) -> Result<(), MercError> {
        let projection = todo.project(&self.project_ldd)?;

        // With caching only the projections that have not been learned from before are enumerated;
        // the ones in the domain already contributed all their transitions to the relation.
        let proj = if cached {
            projection.minus(&self.domain)?
        } else {
            projection.clone()
        };

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

        // Reused buffer for the summands of this group that fire from the current state.
        let mut applicable: Vec<usize> = Vec::with_capacity(self.indices.len());

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
                let full_pos = self.read_full_positions[k];
                full_state[full_pos] = *columns[full_pos]
                    .get_by_index(value as usize)
                    .expect("read value must already be interned");
                interleaved[self.read_positions[k]] = value;
            }

            // Prepare the backend assignments for this state and collect the summands of this group
            // that may fire from it.
            applicable.clear();
            applicable.extend(
                self.lps
                    .prepare(enumerate, &full_state)
                    .filter(|i| self.indices.binary_search(i).is_ok()),
            );

            // Enumerate the successors of every applicable summand, building the write side of each
            // short transition vector and unioning it into the relation.
            //
            // A summand that does not write one of the group's write positions leaves it untouched
            // (the [`StateEffect::Positions`] contract), so `next_state` carries the source value
            // there, which the group reads by construction of its read positions.
            for &index in &applicable {
                let write_full_positions = &self.write_full_positions;
                let write_positions = &self.write_positions;
                let interleaved = &mut interleaved;
                let relation = &mut relation;

                self.lps.summands()[index].enumerate(enumerate, &full_state, |_label, next_state| {
                    for (m, &full_pos) in write_full_positions.iter().enumerate() {
                        let (value, _) = columns[full_pos].insert(next_state[full_pos]);
                        interleaved[write_positions[m]] = *value as Value;
                    }

                    let cube = storage.with_manager_shared(|m| LDDFunction::singleton(m, interleaved.as_slice()))?;
                    *relation = relation.union(&cube)?;
                    Ok(())
                })?;
            }
        }

        self.relation = relation;
        if cached {
            // Only extend the domain once every projection above was enumerated successfully,
            // otherwise a failed learning call would hide the states it never reached.
            self.domain = self.domain.union(&projection)?;
        }
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
            "summands {:?} (read {:?}, write {:?})",
            self.indices, self.read_indices, self.write_indices
        )
    }
}
