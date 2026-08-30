use std::collections::BTreeMap;
use std::fmt;

use log::trace;

use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;

use merc_io::TimeProgress;
use merc_symbolic::TransitionGroup;
use merc_symbolic::fix_element;
use merc_symbolic::intersect;
use merc_symbolic::merge;
use merc_utilities::MercError;

use crate::Player;
use crate::Priority;

/// Progress reported by [`SymbolicParityGame::attractor`]: the iteration number and the size of
/// the attractor set built so far.
pub type AttractorProgress = TimeProgress<(usize, usize)>;

/// A max-parity game over sets of vertices represented as LDDs.
pub struct SymbolicParityGame {
    manager: LDDManagerRef,

    /// Every vertex of the game: the union of `owned[Even]` and `owned[Odd]`.
    vertices: LDDFunction,

    /// Vertices owned by each player.
    owned: [LDDFunction; 2],

    /// Vertices per priority, their order is important.
    priorities: BTreeMap<Priority, LDDFunction>,

    relations: Vec<SymbolicRelation>,

    /// Whether [`Self::attractor`]/[`Self::apply_strategy`] also compute a winning strategy. Set
    /// once for the whole instance so every operation on it behaves consistently.
    compute_strategy: bool,
}

/// A transition relation of a symbolic parity game, flattened from a [`TransitionGroup`].
struct SymbolicRelation {
    relation: LDDFunction,
    meta: LDDFunction,

    /// Positions in the interleaved global vector `[from_0, to_0, from_1, to_1, …]` this
    /// relation reads from and writes to; needed by [`SymbolicParityGame::apply_strategy`].
    read_indices: Vec<Value>,
    write_indices: Vec<Value>,
}

impl SymbolicParityGame {
    /// Constructs a symbolic parity game from an explicit owner/priority partition.
    pub fn new<G: TransitionGroup>(
        manager: &LDDManagerRef,
        groups: &[G],
        owned: [LDDFunction; 2],
        priorities: BTreeMap<Priority, LDDFunction>,
        compute_strategy: bool,
    ) -> Result<Self, MercError> {
        let relations = groups
            .iter()
            .map(|group| {
                // A parity game is a graph, not a labelled transition system: a group must not
                // carry a trailing action-label dimension.
                assert!(
                    group.action_label_index().is_none(),
                    "action labels are not supported in parity games"
                );

                Ok(SymbolicRelation {
                    relation: group.relation().clone(),
                    meta: group.meta().clone(),
                    read_indices: group.read_indices().to_vec(),
                    write_indices: group.write_indices().to_vec(),
                })
            })
            .collect::<Result<Vec<_>, MercError>>()?;

        let vertices = owned[0].union(&owned[1])?;
        let game = Self {
            manager: manager.clone(),
            vertices,
            owned,
            priorities,
            relations,
            compute_strategy,
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
    ///
    /// `blocks` is a list of `(value, owner, priority)` triples, one for each
    /// value that occurs at `level` in the state vectors of the vertices of the
    /// game.
    pub fn from_block_index<G: TransitionGroup>(
        manager: &LDDManagerRef,
        groups: &[G],
        all_vertices: LDDFunction,
        level: usize,
        blocks: &[(Value, Player, Priority)],
        compute_strategy: bool,
    ) -> Result<Self, MercError> {
        let mut owned = [
            manager.with_manager_shared(LDDFunction::empty_set)?,
            manager.with_manager_shared(LDDFunction::empty_set)?,
        ];
        let mut priorities: BTreeMap<Priority, LDDFunction> = BTreeMap::new();

        for &(value, player, priority) in blocks {
            let block = fix_element(manager, &all_vertices, level, value)?;

            owned[player.to_index()] = owned[player.to_index()].union(&block)?;

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

        Self::new(manager, groups, owned, priorities, compute_strategy)
    }

    /// Checks that the priority map partitions [`Self::vertices`].
    #[cfg(debug_assertions)]
    fn assert_consistent(&self) -> Result<(), MercError> {
        let mut covered = self.manager.with_manager_shared(LDDFunction::empty_set)?;
        for block in self.priorities.values() {
            covered = covered.union(block)?;
        }
        debug_assert!(covered == self.vertices, "priorities must partition all vertices");

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

    /// Returns every vertex of the game, i.e. the union of both players' vertex sets.
    pub fn vertices(&self) -> &LDDFunction {
        &self.vertices
    }

    /// Returns the vertices owned by `player`.
    pub fn vertices_owned_by(&self, player: Player) -> &LDDFunction {
        &self.owned[player.to_index()]
    }

    /// Returns whether this game was built to also compute a winning strategy (see
    /// [`Self::attractor`]/[`Self::apply_strategy`]).
    pub fn compute_strategy(&self) -> bool {
        self.compute_strategy
    }

    fn empty(&self) -> Result<LDDFunction, MercError> {
        Ok(self.manager.with_manager_shared(LDDFunction::empty_set)?)
    }

    /// Returns the partition of players for the given vertex set, indexed by
    /// player.
    pub fn players(&self, v: &LDDFunction) -> Result<[LDDFunction; 2], MercError> {
        Ok([
            intersect(v, &self.owned[Player::Even.to_index()])?,
            intersect(v, &self.owned[Player::Odd.to_index()])?,
        ])
    }

    /// Returns the highest priority occurring in `v`, and the vertices of `v` at that priority.
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
    pub fn predecessors(&self, u: &LDDFunction, v: &LDDFunction) -> Result<LDDFunction, MercError> {
        let mut result = self.empty()?;
        for relation in &self.relations {
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
    pub fn successors(&self, u: &LDDFunction) -> Result<LDDFunction, MercError> {
        let mut result = self.empty()?;
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

    /// One attractor step: the vertices of `search_space` pulled in by the vertices most
    /// recently added to the attractor (`u`), for player `alpha`, together with the strategy
    /// edges this step contributes (when [`Self::compute_strategy`] is set).
    ///
    /// `alpha`-owned vertices in `search_space` with *any* edge into `u` are pulled in outright;
    /// `¬alpha`-owned vertices are pulled in only once *every* edge leaving them lands inside the
    /// attractor, checked by removing (for each group) whichever candidates still have an edge
    /// into `outside` — except `incomplete` vertices, which can never be pulled in this way: an
    /// incomplete vertex might have an edge that hasn't been learned yet, so it can never be
    /// *proven* forced into the attractor.
    ///
    /// [`Self::attractor`] always passes the same set for `search_space` and `outside` (its
    /// `Zoutside`, the vertices not yet in the attractor); [`Self::monotone_attractor`] is the one
    /// caller that needs them to differ (it searches all of `V` for predecessors, but only checks
    /// the *forced* condition against `Zoutside \ U`).
    ///
    /// The strategy contribution is `merge(pulled_in \ u, u)`: the interleaved cartesian product
    /// of the newly pulled-in `alpha`-owned vertices with the whole target set `u`, *not* the
    /// exact edges each one uses — an overapproximation that [`Self::apply_strategy`] intersects
    /// back with each group's real relation, recovering exactly the real edges (possibly several
    /// per vertex, which is still a sound strategy: every one of them stays inside `u`, the
    /// region this step is attracting into).
    fn control_predecessors(
        &self,
        alpha: Player,
        u: &LDDFunction,
        search_space: &LDDFunction,
        outside: &LDDFunction,
        vplayer: &[LDDFunction; 2],
        incomplete: Option<&LDDFunction>,
    ) -> Result<(LDDFunction, Option<LDDFunction>), MercError> {
        let candidates = self.predecessors(search_space, u)?;

        let forced_owner = alpha.opponent();
        let pulled_in = intersect(&candidates, &vplayer[alpha.to_index()])?;
        let mut forced = intersect(&candidates, &vplayer[forced_owner.to_index()])?;
        if let Some(incomplete) = incomplete {
            forced = forced.minus(incomplete)?;
        }

        let strategy = if self.compute_strategy {
            Some(merge(&self.manager, &pulled_in.minus(u)?, u)?)
        } else {
            None
        };

        for relation in &self.relations {
            let still_leaving = self.predecessors_group(relation, &forced, outside)?;
            forced = forced.minus(&still_leaving)?;
        }

        Ok((pulled_in.union(&forced)?, strategy))
    }

    /// Computes the attractor set of `u` for player `alpha` within `v`, i.e.
    /// the vertices from which `alpha` can force play into `u`, together with a
    /// winning strategy for the vertices pulled in along the way.
    ///
    /// `vplayer` is `self.players(v)`, taken as a parameter since callers that
    /// invoke this repeatedly (`zielonka`, `compute_total_graph`) already have
    /// it. `incomplete` marks vertices whose outgoing edges are not (yet) fully known, from a
    /// partial exploration; pass `None` when solving a fully-known game.
    ///
    /// When `target` is given, the computation stops as soon as any vertex of
    /// `target` has entered the attractor. Carries a progress reporter that
    /// prints the iteration number and the size of the attractor set built so
    /// far.
    #[allow(clippy::too_many_arguments)]
    pub fn attractor(
        &self,
        alpha: Player,
        u: &LDDFunction,
        v: &LDDFunction,
        vplayer: &[LDDFunction; 2],
        incomplete: Option<&LDDFunction>,
        target: Option<&LDDFunction>,
        progress: &AttractorProgress,
    ) -> Result<(LDDFunction, Option<LDDFunction>), MercError> {
        let mut z = u.clone();
        let mut todo = u.clone();
        let mut z_outside = v.minus(&z)?;
        let mut strategy = if self.compute_strategy {
            Some(self.empty()?)
        } else {
            None
        };

        progress.reset();
        let mut iteration = 0usize;

        while !todo.is_empty() {
            if let Some(target) = target
                && !intersect(target, &z)?.is_empty()
            {
                return Ok((z, strategy));
            }

            progress.print((iteration, z.len()));
            trace!(
                "attractor: iteration {iteration}, |Z| = {}, |todo| = {}",
                z.len(),
                todo.len()
            );

            let (pred, step_strategy) =
                self.control_predecessors(alpha, &todo, &z_outside, &z_outside, vplayer, incomplete)?;
            todo = pred.minus(&z)?;
            if let Some(strategy) = &mut strategy {
                *strategy = strategy.union(
                    step_strategy
                        .as_ref()
                        .expect("control_predecessors returns Some when compute_strategy is set"),
                )?;
            }
            z = z.union(&todo)?;
            z_outside = z_outside.minus(&todo)?;
            iteration += 1;
        }

        Ok((z, strategy))
    }

    /// Removes the winning regions from `v`, growing `winning` (and `strategy`, when
    /// [`Self::compute_strategy`] is set) in place, and returns the resulting total subgraph.
    ///
    /// Every deadlock in `sinks` is assigned to the *opponent* of its owner
    /// (`winning[Even] |= sinks ∩ Odd-owned`, `winning[Odd] |= sinks ∩ Even-owned`) — the correct
    /// PBES semantics, since a disjunctive equation with no enabled summand is `false`.
    pub fn compute_total_graph(
        &self,
        v: &LDDFunction,
        sinks: &LDDFunction,
        winning: &mut [LDDFunction; 2],
        strategy: &mut [Option<LDDFunction>; 2],
        incomplete: Option<&LDDFunction>,
        progress: &AttractorProgress,
    ) -> Result<LDDFunction, MercError> {
        let vplayer = self.players(v)?;

        let complete_sinks = match incomplete {
            Some(incomplete) => sinks.minus(incomplete)?,
            None => sinks.clone(),
        };

        if !complete_sinks.is_empty() {
            winning[Player::Even.to_index()] = winning[Player::Even.to_index()]
                .union(&intersect(&complete_sinks, &self.owned[Player::Odd.to_index()])?)?;
            winning[Player::Odd.to_index()] = winning[Player::Odd.to_index()]
                .union(&intersect(&complete_sinks, &self.owned[Player::Even.to_index()])?)?;
        }

        let (w0, s0) = self.attractor(
            Player::Even,
            &winning[Player::Even.to_index()],
            v,
            &vplayer,
            incomplete,
            None,
            progress,
        )?;
        let (w1, s1) = self.attractor(
            Player::Odd,
            &winning[Player::Odd.to_index()],
            v,
            &vplayer,
            incomplete,
            None,
            progress,
        )?;
        winning[Player::Even.to_index()] = w0;
        winning[Player::Odd.to_index()] = w1;

        if self.compute_strategy {
            let s0 = s0.expect("compute_strategy is set");
            let s1 = s1.expect("compute_strategy is set");
            strategy[Player::Even.to_index()] = Some(match strategy[Player::Even.to_index()].take() {
                Some(existing) => existing.union(&s0)?,
                None => s0,
            });
            strategy[Player::Odd.to_index()] = Some(match strategy[Player::Odd.to_index()].take() {
                Some(existing) => existing.union(&s1)?,
                None => s1,
            });
        }

        Ok(v.minus(&winning[0])?.minus(&winning[1])?)
    }

    /// Returns the vertices of `v` with even priority and with odd priority (indexed by
    /// [`Player::to_index`]), excluding sinks — a sink has no priority-driven behaviour of its
    /// own.
    ///
    /// Used only by [`partial_solve::detect_solitair_cycles`] and
    /// [`partial_solve::detect_forced_cycles`] (fatal/solitair-cycle detection): a vertex can only
    /// ever be part of a cycle that is winning for it "by parity" if its own priority has the
    /// parity it needs.
    ///
    /// [`partial_solve::detect_solitair_cycles`]: super::partial_solve::detect_solitair_cycles
    /// [`partial_solve::detect_forced_cycles`]: super::partial_solve::detect_forced_cycles
    pub fn parity(&self, v: &LDDFunction) -> Result<[LDDFunction; 2], MercError> {
        let mut parity = [self.empty()?, self.empty()?];
        for (&priority, block) in &self.priorities {
            let i = Player::from_priority(priority).to_index();
            parity[i] = parity[i].union(block)?;
        }

        let non_sinks = v.minus(&self.sinks(v, v)?)?;
        Ok([intersect(&non_sinks, &parity[0])?, intersect(&non_sinks, &parity[1])?])
    }

    /// Returns the vertices of `v` whose priority is at most `c` (under merc's max-parity
    /// encoding, the *less* significant priorities).
    ///
    /// Used only by [`partial_solve::detect_fatal_attractors`]'s `safe_monotone_attractor` calls,
    /// restricting the search for a fatal attractor at priority `c` to vertices that cannot
    /// escape to something *more* significant.
    ///
    /// [`partial_solve::detect_fatal_attractors`]: super::partial_solve::detect_fatal_attractors
    pub fn vertices_with_priority_at_most(&self, v: &LDDFunction, c: Priority) -> Result<LDDFunction, MercError> {
        let mut below = self.empty()?;
        for (&priority, block) in &self.priorities {
            if priority <= c {
                below = below.union(block)?;
            }
        }
        Ok(intersect(v, &below)?)
    }

    /// Computes the monotone attractor set of `u` for player `alpha` at priority `c` within `v`:
    /// like [`Self::attractor`], but every vertex pulled in is additionally required to have
    /// priority `c` itself (`u`'s own vertices only, since `u` is assumed to already satisfy
    /// this) — used by fatal-attractor detection to find a set of priority-`c` vertices that
    /// player `alpha` can always force play to stay within.
    ///
    /// Never computes a strategy — [`partial_solve::detect_fatal_attractors`] instead records
    /// `merge(Z, Z)` for the fatal-attractor vertices it accepts, an overapproximate
    /// self-loop-like strategy that [`Self::apply_strategy`] cuts down to real edges.
    ///
    /// [`partial_solve::detect_fatal_attractors`]: super::partial_solve::detect_fatal_attractors
    #[allow(clippy::too_many_arguments)]
    pub fn monotone_attractor(
        &self,
        u: &LDDFunction,
        alpha: Player,
        c: Priority,
        v: &LDDFunction,
        vplayer: &[LDDFunction; 2],
        incomplete: Option<&LDDFunction>,
        target: Option<&LDDFunction>,
    ) -> Result<LDDFunction, MercError> {
        let vc = self.vertices_with_priority_at_most(v, c)?;

        let mut z = self.empty()?;
        let mut todo = u.clone();
        let mut z_outside = v.clone();

        while !todo.is_empty() {
            if let Some(target) = target
                && !intersect(target, &z)?.is_empty()
            {
                return Ok(z);
            }

            let search_target = todo.union(u)?;
            let outside = z_outside.minus(u)?;
            let (pred, _) = self.control_predecessors(alpha, &search_target, v, &outside, vplayer, incomplete)?;
            todo = intersect(&vc, &pred.minus(&z)?)?;
            z = z.union(&todo)?;
            z_outside = z_outside.minus(&todo)?;
        }

        Ok(z)
    }

    /// Returns the vertices of `v` for which partial solving is safe with respect to `alpha`: no
    /// matter how the unknown edges of `incomplete` turn out, `alpha`'s opponent cannot force
    /// play out of this set and into the unresolved region.
    ///
    /// The seed is `(incomplete ∩ opponent-owned) ∪ sinks(incomplete, v)`, not just the first
    /// term: an `incomplete` vertex is, by construction, a *structural* sink of `v` until it is
    /// actually explored (an unexplored vertex has no discovered successors yet, regardless of
    /// who owns it), so leaving the second term out would let `alpha`-owned incomplete sinks stay
    /// inside the returned safe set — the extra `sinks(incomplete, v)` term is what keeps the safe
    /// subgame actually total whenever `incomplete` is non-empty, which callers like
    /// [`crate::symbolic::partial_solve::partial_solve`] rely on.
    pub fn safe_vertices(
        &self,
        alpha: Player,
        v: &LDDFunction,
        incomplete: &LDDFunction,
        progress: &AttractorProgress,
    ) -> Result<LDDFunction, MercError> {
        let vplayer = self.players(v)?;
        let opponent = alpha.opponent();

        let sinks_of_incomplete = self.sinks(incomplete, v)?;
        let seed = intersect(incomplete, &vplayer[opponent.to_index()])?.union(&sinks_of_incomplete)?;

        let (attracted, _) = self.attractor(opponent, &seed, v, &vplayer, None, None, progress)?;
        Ok(v.minus(&attracted)?)
    }

    /// One-shot [`Self::control_predecessors`], with `outside` computed automatically as `v \
    /// u` and its strategy contribution discarded.
    ///
    /// Needed by [`partial_solve::detect_forced_cycles`]'s forced-cycle search, which records an
    /// overapproximate `merge(U, U)` for the cycle it accepts instead of a per-step strategy.
    ///
    /// [`partial_solve::detect_forced_cycles`]: super::partial_solve::detect_forced_cycles
    pub fn control_predecessors_within(
        &self,
        alpha: Player,
        u: &LDDFunction,
        v: &LDDFunction,
        vplayer: &[LDDFunction; 2],
        incomplete: Option<&LDDFunction>,
    ) -> Result<LDDFunction, MercError> {
        let outside = v.minus(u)?;
        Ok(self.control_predecessors(alpha, u, v, &outside, vplayer, incomplete)?.0)
    }

    /// Returns a copy of this game where every `alpha`-owned source vertex has
    /// had its outgoing edges restricted to `strategy`.
    ///
    /// `strategy` is expected over the doubled, interleaved global vector
    /// `[from_0, to_0, from_1, to_1, …]`. Restriction is per *source vertex*,
    /// not per transition group. A group's `alpha`-owned rows are restricted to
    /// `strategy`; every other row (including any non-`alpha`-owned rows the
    /// same group happens to also carry) passes through unchanged. A
    /// `alpha`-owned source with no strategy at all (the strategy is only ever
    /// partial for vertices *outside* the winning region requested) loses its
    /// outgoing edges entirely, which is what turns an unresolved vertex into a
    /// fresh sink for the caller
    /// ([`verify_symbolic_strategy`][crate::symbolic::verify_symbolic::verify_symbolic_strategy])
    /// to route through [`Self::compute_total_graph`] again.
    pub fn apply_strategy(&self, alpha: Player, strategy: &LDDFunction) -> Result<Self, MercError> {
        let all_vertices = self.vertices();
        let strategy_is_empty = strategy.is_empty();
        let alpha_vertices = &self.owned[alpha.to_index()];

        let mut relations = Vec::with_capacity(self.relations.len());
        for relation in &self.relations {
            let keep: Vec<Value> = relation
                .read_indices
                .iter()
                .map(|&r| 2 * r)
                .chain(relation.write_indices.iter().map(|&w| 2 * w + 1))
                .collect();
            let projection_meta = self
                .manager
                .with_manager_shared(|m| LDDFunction::projection_meta(m, &keep))?;

            // The rows of this group whose source is `alpha`-owned, in the group's own
            // read/write shape, found the same way `predecessors_group`'s callers restrict a
            // global vertex set to a relation's shape (via `merge` + `project`).
            let alpha_rows = merge(&self.manager, alpha_vertices, all_vertices)?.project(&projection_meta)?;
            let alpha_part = intersect(&relation.relation, &alpha_rows)?;
            let other_part = relation.relation.minus(&alpha_part)?;

            let restricted_alpha_part = if strategy_is_empty {
                self.empty()?
            } else {
                let projected_strategy = strategy.project(&projection_meta)?;
                intersect(&alpha_part, &projected_strategy)?
            };

            let new_relation = other_part.union(&restricted_alpha_part)?;

            relations.push(SymbolicRelation {
                relation: new_relation,
                meta: relation.meta.clone(),
                read_indices: relation.read_indices.clone(),
                write_indices: relation.write_indices.clone(),
            });
        }

        Ok(Self {
            manager: self.manager.clone(),
            vertices: self.vertices.clone(),
            owned: self.owned.clone(),
            priorities: self.priorities.clone(),
            relations,
            compute_strategy: self.compute_strategy,
        })
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
            self.owned[Player::Even.to_index()].len(),
            self.owned[Player::Odd.to_index()].len()
        )
    }
}
