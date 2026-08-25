use std::collections::BTreeMap;
use std::fmt;

use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;

use merc_symbolic::TransitionGroup;
use merc_symbolic::fix_element;
use merc_symbolic::intersect;
use merc_utilities::MercError;

use crate::Player;
use crate::Priority;

/// A transition relation of a symbolic parity game, flattened from a
/// [`TransitionGroup`].
///
/// Only `relation` and `meta` are required to compute predecessors.
struct SymbolicRelation {
    relation: LDDFunction,
    meta: LDDFunction,
}

/// A symbolic parity game: a max-parity game over sets of vertices represented
/// as LDDs, rather than as an explicit vertex/edge list.
pub struct SymbolicParityGame {
    manager: LDDManagerRef,

    /// Vertices owned by each player.
    vertices: [LDDFunction; 2],

    /// Vertices per priority. A [`BTreeMap`] so it can be walked highest-first (see
    /// [`Self::max_priority`]).
    priorities: BTreeMap<Priority, LDDFunction>,

    relations: Vec<SymbolicRelation>,
}

impl SymbolicParityGame {
    /// Constructs a symbolic parity game from an explicit owner/priority partition, given as
    /// `vertices[Player::Even.to_index()]`/`vertices[Player::Odd.to_index()]` (mCRL2's second,
    /// `Veven`-based constructor).
    pub fn new<G: TransitionGroup>(
        manager: &LDDManagerRef,
        groups: &[G],
        vertices: [LDDFunction; 2],
        priorities: BTreeMap<Priority, LDDFunction>,
    ) -> Result<Self, MercError> {
        let relations = groups
            .iter()
            .map(|group| SymbolicRelation {
                relation: group.relation().clone(),
                meta: group.meta().clone(),
            })
            .collect();

        let game = Self {
            manager: manager.clone(),
            vertices,
            priorities,
            relations,
        };

        #[cfg(debug_assertions)]
        game.assert_consistent()?;

        Ok(game)
    }

    /// Constructs a symbolic parity game where the owner and priority of every
    /// vertex is determined by the value at `level` of its state vector.
    ///
    /// Takes `level` because a permuted state vector does not necessarily put
    /// the discriminating value at level 0; see [`fix_element`].
    pub fn from_block_index<G: TransitionGroup>(
        manager: &LDDManagerRef,
        groups: &[G],
        all_vertices: LDDFunction,
        level: usize,
        blocks: &[(Value, Player, Priority)],
    ) -> Result<Self, MercError> {
        let mut vertices = [
            manager.with_manager_shared(LDDFunction::empty_set)?,
            manager.with_manager_shared(LDDFunction::empty_set)?,
        ];
        let mut priorities: BTreeMap<Priority, LDDFunction> = BTreeMap::new();

        for &(value, player, priority) in blocks {
            let block = fix_element(manager, &all_vertices, level, value)?;

            vertices[player.to_index()] = vertices[player.to_index()].union(&block)?;

            match priorities.get(&priority) {
                Some(existing) => {
                    let merged = existing.union(&block)?;
                    priorities.insert(priority, merged);
                }
                None => {
                    priorities.insert(priority, block);
                }
            }
        }

        Self::new(manager, groups, vertices, priorities)
    }

    /// Checks that the owner and priority maps are actually a partition of `all_vertices`.
    ///
    /// mCRL2's constructors silently produce a partial partition if `equation_info` misses a
    /// value; this is the one invariant worth paying for even in debug builds only.
    ///
    /// The union of `vertices` (the owner partition) is recomputed here as the reference "all
    /// vertices" set, since the game does not otherwise keep one around.
    #[cfg(debug_assertions)]
    fn assert_consistent(&self) -> Result<(), MercError> {
        let all_vertices = self.vertices[0].union(&self.vertices[1])?;

        let mut covered = self.manager.with_manager_shared(LDDFunction::empty_set)?;
        for block in self.priorities.values() {
            covered = covered.union(block)?;
        }
        debug_assert!(covered == all_vertices, "priorities must partition all vertices");

        Ok(())
    }

    /// Returns the LDD manager this game was built in.
    pub fn manager(&self) -> &LDDManagerRef {
        &self.manager
    }

    /// Returns the mapping from priorities to vertex sets.
    pub fn priorities(&self) -> &BTreeMap<Priority, LDDFunction> {
        &self.priorities
    }

    /// Returns the vertices owned by `player`.
    pub fn vertices_owned_by(&self, player: Player) -> &LDDFunction {
        &self.vertices[player.to_index()]
    }

    /// Returns `{ intersect(v, Even vertices), intersect(v, Odd vertices) }`, indexed by
    /// [`Player::to_index`].
    pub fn players(&self, v: &LDDFunction) -> Result<[LDDFunction; 2], MercError> {
        Ok([
            intersect(v, &self.vertices[Player::Even.to_index()])?,
            intersect(v, &self.vertices[Player::Odd.to_index()])?,
        ])
    }

    /// Returns the highest priority occurring in `v`, and the vertices of `v` at that priority.
    ///
    /// This is mCRL2's `get_min_rank`, ported under the priority-direction inversion described
    /// in the module documentation: mCRL2 scans its rank map ascending and stops at the first
    /// non-empty intersection (the *outermost*, i.e. numerically smallest, block present); the
    /// equivalent under merc's max-parity encoding is to scan the `BTreeMap` **descending**.
    pub fn max_priority(&self, v: &LDDFunction) -> Result<Option<(Priority, LDDFunction)>, MercError> {
        for (&priority, block) in self.priorities.iter().rev() {
            let restricted = intersect(v, block)?;
            if !restricted.is_empty() {
                return Ok(Some((priority, restricted)));
            }
        }
        Ok(None)
    }

    /// Returns `{ u ∈ u | ∃ v_elem ∈ v : u -> v_elem }`, the vertices of `u` with an edge into
    /// `v`.
    ///
    /// Groups are visited in reverse (as mCRL2 does), keeping large groups first.
    pub fn predecessors(&self, u: &LDDFunction, v: &LDDFunction) -> Result<LDDFunction, MercError> {
        let mut result = self.manager.with_manager_shared(LDDFunction::empty_set)?;
        for relation in self.relations.iter().rev() {
            result = result.union(&self.predecessors_group(relation, u, v)?)?;
        }
        Ok(result)
    }

    /// `predecessors(u, v)` restricted to a single transition relation.
    fn predecessors_group(
        &self,
        relation: &SymbolicRelation,
        u: &LDDFunction,
        v: &LDDFunction,
    ) -> Result<LDDFunction, MercError> {
        Ok(v.relational_predecessor(&relation.relation, &relation.meta, u)?)
    }

    /// Returns `{ w | ∃ elem ∈ u : elem -> w }`, the one-step successors of `u`.
    ///
    /// The dual of [`Self::predecessors`]; not needed by the solver itself (which only ever
    /// walks edges backwards), but used by [`crate::convert_symbolic_parity_game`] to
    /// decode a symbolic game into an explicit one for testing and debugging.
    pub fn successors(&self, u: &LDDFunction) -> Result<LDDFunction, MercError> {
        let mut result = self.manager.with_manager_shared(LDDFunction::empty_set)?;
        for relation in &self.relations {
            result = result.union(&u.relational_product(&relation.relation, &relation.meta)?)?;
        }
        Ok(result)
    }

    /// Returns the vertices of `u` that have no outgoing edge into `v` (i.e. are sinks w.r.t.
    /// `v`).
    pub fn sinks(&self, u: &LDDFunction, v: &LDDFunction) -> Result<LDDFunction, MercError> {
        Ok(u.minus(&self.predecessors(u, v)?)?)
    }

    /// One attractor step: the vertices outside the attractor so far (`v_outside`) that are
    /// pulled in by the vertices most recently added to it (`u`), for player `alpha`.
    ///
    /// Port of `safe_control_predecessors_impl`, specialised to how [`Self::attractor`] (the
    /// only in-scope caller) invokes it: mCRL2's `I` (incomplete vertices, partial solving) is
    /// always empty here, and its `W` parameter (restricting chaining) is unused unless chaining
    /// is enabled, which this port does not yet implement — so both are dropped rather than
    /// threaded through unused.
    ///
    /// `alpha`-owned vertices in `v_outside` with *any* edge into `u` are pulled in outright;
    /// `¬alpha`-owned vertices are pulled in only once *every* group-edge leaving them lands
    /// inside the attractor, checked by removing (for each group) whichever candidates still
    /// have an edge back out to `v_outside`.
    fn control_predecessors(
        &self,
        alpha: Player,
        u: &LDDFunction,
        v_outside: &LDDFunction,
        vplayer: &[LDDFunction; 2],
    ) -> Result<LDDFunction, MercError> {
        let candidates = self.predecessors(v_outside, u)?;

        let forced_owner = alpha.opponent();
        let pulled_in = intersect(&candidates, &vplayer[alpha.to_index()])?;
        let mut forced = intersect(&candidates, &vplayer[forced_owner.to_index()])?;

        for relation in &self.relations {
            let still_leaving = self.predecessors_group(relation, &forced, v_outside)?;
            forced = forced.minus(&still_leaving)?;
        }

        Ok(pulled_in.union(&forced)?)
    }

    /// Computes the attractor set of `u` for player `alpha` within `v`, i.e. the vertices from
    /// which `alpha` can force play into `u`.
    ///
    /// `vplayer` is `self.players(v)`, taken as a parameter (as mCRL2 does) since callers that
    /// invoke this repeatedly (`zielonka`, `compute_total_graph`) already have it. When `target`
    /// is given, the computation stops as soon as any vertex of `target` has entered the
    /// attractor — used by `solve` to terminate as soon as the initial vertex is won.
    pub fn attractor(
        &self,
        alpha: Player,
        u: &LDDFunction,
        v: &LDDFunction,
        vplayer: &[LDDFunction; 2],
        target: Option<&LDDFunction>,
    ) -> Result<LDDFunction, MercError> {
        let mut z = u.clone();
        let mut todo = u.clone();
        let mut z_outside = v.minus(&z)?;

        while !todo.is_empty() {
            if let Some(target) = target
                && !intersect(target, &z)?.is_empty()
            {
                return Ok(z);
            }

            let pred = self.control_predecessors(alpha, &todo, &z_outside, vplayer)?;
            todo = pred.minus(&z)?;
            z = z.union(&todo)?;
            z_outside = z_outside.minus(&todo)?;
        }

        Ok(z)
    }

    /// Removes the winning regions from `v`, growing `winning` in place, and returns the
    /// resulting total subgraph.
    ///
    /// Every deadlock in `sinks` is assigned to the *opponent* of its owner
    /// (`winning[Even] |= sinks ∩ Odd-owned`, `winning[Odd] |= sinks ∩ Even-owned`) — the correct
    /// PBES semantics, since a disjunctive equation with no enabled summand is `false`. This is
    /// the mCRL2 behaviour, and it deliberately does *not* match `ParityGame::from_edges`'s
    /// `make_total`, which resolves a deadlock with a self-loop (won by the parity of the sink's
    /// own priority) instead; see the plan's §9.3 for the explicit-path implication.
    pub fn compute_total_graph(
        &self,
        v: &LDDFunction,
        sinks: &LDDFunction,
        winning: &mut [LDDFunction; 2],
    ) -> Result<LDDFunction, MercError> {
        let vplayer = self.players(v)?;

        if !sinks.is_empty() {
            winning[Player::Even.to_index()] =
                winning[Player::Even.to_index()].union(&intersect(sinks, &self.vertices[Player::Odd.to_index()])?)?;
            winning[Player::Odd.to_index()] =
                winning[Player::Odd.to_index()].union(&intersect(sinks, &self.vertices[Player::Even.to_index()])?)?;
        }

        winning[Player::Even.to_index()] =
            self.attractor(Player::Even, &winning[Player::Even.to_index()], v, &vplayer, None)?;
        winning[Player::Odd.to_index()] =
            self.attractor(Player::Odd, &winning[Player::Odd.to_index()], v, &vplayer, None)?;

        Ok(v.minus(&winning[0])?.minus(&winning[1])?)
    }
}

/// Prints per-priority and per-owner vertex counts.
impl fmt::Display for SymbolicParityGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (priority, block) in &self.priorities {
            writeln!(f, "priority {priority}: {} vertices", block.len())?;
        }

        write!(
            f,
            "{} even vertices and {} odd vertices",
            self.vertices[Player::Even.to_index()].len(),
            self.vertices[Player::Odd.to_index()].len()
        )
    }
}
