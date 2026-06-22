use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use merc_collections::IndexedSet;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_utilities::MercError;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;
use streaming_iterator::StreamingIterator;

use crate::SymbolicLPS;
use crate::TransitionGroup;
use crate::iter;

/// The per-thread enumeration context produced by an [LPS].
type Context<L> = <<L as LPS>::Summand as Summand>::Context;

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
    groups: Vec<SymbolicLpsGroup<L>>,
    initial_state: LDDFunction,
}

/// State shared between a [`SymbolicLps`] and all of its groups.
struct Shared<L: LPS> {
    /// The wrapped `LPS` definition; the single source of summands and the
    /// `prepare`/`enumerate` machinery.
    lps: L,

    /// The enumeration backend, created once via [`LPS::create_context`].
    context: RefCell<Context<L>>,

    /// Per state-vector position interning of `LPS` values into dense LDD values.
    columns: RefCell<Vec<IndexedSet<L::Value>>>,
}

impl<L: LPS> SymbolicLps<L> {
    /// Wraps `lps` into a symbolic LDD view, encoding its initial state.
    pub fn new(manager: &LDDManagerRef, lps: L) -> Result<Self, MercError> {
        let initial_values = lps.initial_state();
        let num_positions = initial_values.len();

        let mut columns: Vec<IndexedSet<L::Value>> = (0..num_positions).map(|_| IndexedSet::new()).collect();

        // Encode the initial state, interning each value into its column.
        let mut initial_vector: Vec<Value> = Vec::with_capacity(num_positions);
        for (position, value) in initial_values.iter().enumerate() {
            let (index, _) = columns[position].insert(*value);
            initial_vector.push(*index as Value);
        }
        let initial_state = LDDFunction::singleton(manager, &initial_vector)?;

        let context = lps.create_context();
        let shared = Rc::new(Shared {
            lps,
            context: RefCell::new(context),
            columns: RefCell::new(columns),
        });

        // Build one symbolic group per summand.
        let mut groups = Vec::with_capacity(shared.lps.summands().len());
        for (index, summand) in shared.lps.summands().iter().enumerate() {
            let mut read_indices: Vec<Value> = summand.read_positions().iter().map(|&p| p as Value).collect();
            let mut write_indices: Vec<Value> = summand.write_positions().iter().map(|&p| p as Value).collect();
            // The short-vector encoding requires sorted read/write indices.
            read_indices.sort_unstable();
            write_indices.sort_unstable();

            let project_ldd = LDDFunction::projection_meta(manager, &read_indices)?;
            let (meta, read_positions, write_positions) =
                LDDFunction::relation_product_meta(manager, &read_indices, &write_indices)?;
            let relation = LDDFunction::empty_set(manager)?;

            groups.push(SymbolicLpsGroup {
                index,
                read_indices,
                write_indices,
                read_positions,
                write_positions,
                project_ldd,
                meta,
                relation,
                shared: Rc::clone(&shared),
            });
        }

        Ok(SymbolicLps { groups, initial_state })
    }
}

impl<L: LPS> SymbolicLPS for SymbolicLps<L> {
    fn initial_state(&self) -> &LDDFunction {
        &self.initial_state
    }

    fn transition_groups(&self) -> &[impl TransitionGroup] {
        &self.groups
    }

    fn transition_groups_mut(&mut self) -> &mut [impl TransitionGroup] {
        &mut self.groups
    }
}

/// A single summand of a [`SymbolicLps`], encoded as a short-vector LDD
/// transition relation that is learned on the fly during reachability.
struct SymbolicLpsGroup<L: LPS> {
    /// Index of this summand within `shared.lps.summands()`.
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

    /// Shared `LPS`, enumeration context and per-position interning.
    shared: Rc<Shared<L>>,
}

impl<L: LPS> TransitionGroup for SymbolicLpsGroup<L> {
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

    fn learn_successors(&mut self, storage: &LDDManagerRef, todo: &LDDFunction) -> Result<(), MercError> {
        let proj = todo.project(&self.project_ldd)?;

        // Reusable full-length state buffer. Non-read positions keep the initial
        // state's values: they are never read by this summand (guaranteed by the
        // read-positions contract), so any valid value works; the read positions
        // are overlaid from each short state below.
        let mut full_state = self.shared.lps.initial_state();
        debug_assert_eq!(full_state.len(), self.shared.columns.borrow().len());

        // Reusable interleaved short vector for the relation singletons.
        let mut interleaved: Vec<Value> = vec![0; self.read_indices.len() + self.write_indices.len()];

        // Accumulate into a local relation so the enumeration callback does not
        // have to borrow `self`.
        let mut relation = self.relation.clone();

        let mut states = iter(&proj);
        while let Some(short_state) = states.next() {
            debug_assert_eq!(
                short_state.len(),
                self.read_indices.len(),
                "Projected state must have one value per read index"
            );

            // Overlay the short read values into the full state buffer and the
            // interleaved read positions.
            {
                let columns = self.shared.columns.borrow();
                for (k, &value) in short_state.iter().enumerate() {
                    let full_pos = self.read_indices[k] as usize;
                    full_state[full_pos] = *columns[full_pos]
                        .get_by_index(value as usize)
                        .expect("read value must already be interned");
                    interleaved[self.read_positions[k]] = value;
                }
            }

            // `prepare` sets the backend assignments for this state and returns
            // the summands that may fire; skip when this group is not among them.
            // For plain LPSs every summand fires; for PBESs this is the
            // equation-index gate.
            let fires = {
                let mut context = self.shared.context.borrow_mut();
                self.shared
                    .lps
                    .prepare(&mut context, &full_state)
                    .any(|index| index == self.index)
            };
            if !fires {
                continue;
            }

            // Enumerate the successors, building the write side of each short
            // transition vector and unioning it into the relation.
            let mut context = self.shared.context.borrow_mut();
            let summand = &self.shared.lps.summands()[self.index];
            let shared = &self.shared;
            let write_indices = &self.write_indices;
            let write_positions = &self.write_positions;
            let interleaved = &mut interleaved;
            let relation = &mut relation;

            summand.enumerate(&mut context, &full_state, |_label, next_state| {
                {
                    let mut columns = shared.columns.borrow_mut();
                    for (m, &full_pos) in write_indices.iter().enumerate() {
                        let (index, _) = columns[full_pos as usize].insert(next_state[full_pos as usize]);
                        interleaved[write_positions[m]] = *index as Value;
                    }
                }

                let cube = LDDFunction::singleton(storage, interleaved.as_slice())?;
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
