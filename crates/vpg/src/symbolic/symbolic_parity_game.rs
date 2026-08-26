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

    /// Vertices owned by each player, their union is the total vertex set.
    vertices: [LDDFunction; 2],

    /// Vertices per priority, their order is important.
    priorities: BTreeMap<Priority, LDDFunction>,

    relations: Vec<SymbolicRelation>,

    /// Whether the operations on this parity game compute a strategy. This is
    /// set here for the whole instance because it should be done consistently.
    compute_strategy: bool,
}

/// A transition relation of a symbolic parity game, flattened from a
/// [`TransitionGroup`].
///
/// # Details
///
/// The indices are interleaved global vector `[from_0, to_0, from_1, to_1, …]`
/// be projected down onto exactly the positions `relation` itself reads and
/// writes.
struct SymbolicRelation {
    relation: LDDFunction,
    meta: LDDFunction,

    /// The indices of the positions in the global vector that this relation reads from. Only needed for the
    /// apply strategy operation.
    read_indices: Vec<Value>,
    write_indices: Vec<Value>,
}

impl SymbolicParityGame {
    /// Constructs a symbolic parity game from an explicit owner/priority partition.
    pub fn new<G: TransitionGroup>(
        manager: &LDDManagerRef,
        groups: &[G],
        vertices: [LDDFunction; 2],
        priorities: BTreeMap<Priority, LDDFunction>,
        compute_strategy: bool,
    ) -> Result<Self, MercError> {
        let relations = groups
            .iter()
            .map(|group| {
                // A game is a graph, not a labelled transition system: its exploration groups must
                // not carry a trailing action-label dimension in the first place (the PBES/LPS
                // generator feeding `merc_symbolic`'s exploration sets `LPS::HAS_LABELS = false` for
                // that), rather than have this type project it away, since every other method here
                // (in particular `apply_strategy`, whose strategy has no notion of actions) assumes
                // `group.relation()` covers exactly `read_indices` and `write_indices`.
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

        let game = Self {
            manager: manager.clone(),
            vertices,
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

        Self::new(manager, groups, vertices, priorities, compute_strategy)
    }

    /// Checks that the owner and priority maps are actually a partition of `all_vertices`.
    #[cfg(debug_assertions)]
    fn assert_consistent(&self) -> Result<(), MercError> {
        let all_vertices = self.all_vertices()?;

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

    /// Returns whether this game was built to also compute a winning strategy (see
    /// [`Self::attractor`]/[`Self::apply_strategy`]).
    pub fn compute_strategy(&self) -> bool {
        self.compute_strategy
    }

    /// Returns the union of both owner partitions, i.e. every vertex the game was constructed
    /// over.
    fn all_vertices(&self) -> Result<LDDFunction, MercError> {
        Ok(self.vertices[0].union(&self.vertices[1])?)
    }

    fn empty(&self) -> Result<LDDFunction, MercError> {
        Ok(self.manager.with_manager_shared(LDDFunction::empty_set)?)
    }

    /// Returns the partition of players for the given vertex set, indexed by
    /// player.
    pub fn players(&self, v: &LDDFunction) -> Result<[LDDFunction; 2], MercError> {
        Ok([
            intersect(v, &self.vertices[Player::Even.to_index()])?,
            intersect(v, &self.vertices[Player::Odd.to_index()])?,
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

    /// One attractor step: the vertices of `search_space` that are pulled in by
    /// the vertices most recently added to the attractor (`u`), for player
    /// `alpha`, together with the strategy edges this step contributes (when
    /// [`Self::compute_strategy`] is set).
    ///
    /// Incomplete vertices whose outgoing edges are not (yet) fully known, from
    /// a partial exploration; pass `None` when solving a fully-known game
    /// (every caller except the partial solvers in `partial_solve.rs` does).
    ///
    /// [`Self::attractor`] always passes the same set for `search_space` and
    /// `outside` (its `Zoutside`, the vertices not yet in the attractor);
    /// [`Self::monotone_attractor`] is the one caller that needs them to differ
    /// (it searches all of `V` for predecessors, but only checks the *forced*
    /// condition against `Zoutside \ U`), which is why both are threaded
    /// through separately rather than the single set the plan's original sketch
    /// assumed.
    ///
    /// `alpha`-owned vertices in `search_space` with *any* edge into `u` are
    /// pulled in outright; `¬alpha`-owned vertices are pulled in only once
    /// *every* group-edge leaving them lands inside the attractor, checked by
    /// removing (for each group) whichever candidates still have an edge into
    /// `outside` — except `incomplete` vertices, which can never be pulled in
    /// this way: an incomplete vertex might have an edge that hasn't been
    /// learned yet, so it can never be *proven* forced into the attractor.
    ///
    /// The strategy contribution is `merge(pulled_in \ u, u)`: the interleaved
    /// cartesian product of the newly pulled-in `alpha`-owned vertices with the
    /// whole target set `u`, *not* the exact edges each one uses. This is
    /// deliberately an overapproximation — mCRL2's own
    /// `safe_control_predecessors_impl` does the same — because
    /// [`Self::apply_strategy`] intersects it back with each group's real
    /// relation, which recovers exactly the real edges (possibly several per
    /// vertex, which is still a sound strategy: every one of them stays inside
    /// `u`, the region this step is attracting into).
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
    /// it. `incomplete` is mCRL2's `I` (see [`Self::control_predecessors`]);
    /// pass `None` when solving a fully-known game.
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
    /// PBES semantics, since a disjunctive equation with no enabled summand is `false`. This is
    /// the mCRL2 behaviour, and it deliberately does *not* match `ParityGame::from_edges`'s
    /// `make_total`, which resolves a deadlock with a self-loop (won by the parity of the sink's
    /// own priority) instead; see the plan's §9.3 for the explicit-path implication.
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
                .union(&intersect(&complete_sinks, &self.vertices[Player::Odd.to_index()])?)?;
            winning[Player::Odd.to_index()] = winning[Player::Odd.to_index()]
                .union(&intersect(&complete_sinks, &self.vertices[Player::Even.to_index()])?)?;
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
    /// [`Player::to_index`]), excluding sinks (a sink has no real priority-driven behaviour of
    /// its own — the only thing it could still be "helping" its own parity by would be a
    /// self-loop this port never fabricates, so it is neither).
    ///
    /// Port of mCRL2's `parity`, used only by [`partial_solve::detect_solitair_cycles`] and
    /// [`partial_solve::detect_forced_cycles`] (fatal/solitair-cycle detection): a vertex can
    /// only ever be part of a cycle that is winning for it "by parity" if its own priority has
    /// the parity it needs.
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

    /// Returns the vertices of `v` whose priority is at most `c`.
    ///
    /// Port of mCRL2's `prio_above` (`rank >= c`, min-parity: the *less* significant priorities),
    /// inverted under merc's max-parity encoding to `priority <= c` — the analogous "less
    /// significant than `c`" side. Used only by [`partial_solve::detect_fatal_attractors`]'s
    /// `safe_monotone_attractor` calls, restricting the search for a fatal attractor at priority
    /// `c` to vertices that cannot escape to something *more* significant.
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
    /// Port of `safe_monotone_attractor`. Never computes a strategy (mCRL2's doesn't either —
    /// [`partial_solve::detect_fatal_attractors`] instead records `merge(Z, Z)` for the
    /// fatal-attractor vertices it accepts, an overapproximate self-loop-like strategy that
    /// [`Self::apply_strategy`] cuts down to real edges, the same trick every other strategy
    /// contribution here relies on).
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
    /// Port of `compute_safe_vertices`. Deliberately does *not* forward `incomplete` into its own
    /// internal attractor call, matching mCRL2's `safe_attractor(..., Vplayer)` (no `I` argument,
    /// i.e. its default of "fully known") — `incomplete` only seeds *which* vertices the
    /// opponent's attractor starts from, not how that attractor itself is computed.
    ///
    /// The seed is `(incomplete ∩ opponent-owned) ∪ sinks(incomplete, v)`, not just the first
    /// term — the paper's Definition 4 (`V \ Attr_ᾱ(⅁, V_ᾱ ∩ I)`) covers only `incomplete`
    /// vertices the opponent owns, but an `incomplete` vertex is also, by construction, a
    /// *structural* sink of `v` until it is actually explored (an unexplored vertex has no
    /// discovered successors yet, regardless of who owns it). Leaving those out of the seed would
    /// let `alpha`-owned incomplete sinks stay inside the returned safe set, which callers like
    /// [`crate::symbolic::partial_solve::partial_solve`] then solve with a plain [`zielonka`
    /// call][crate::symbolic::symbolic_zielonka::zielonka] that assumes totality — the extra
    /// `sinks(incomplete, v)` term is what keeps that safe subgame actually total whenever
    /// `incomplete` is non-empty (mCRL2's `compute_safe_vertices` does the same for the same
    /// reason, not literal Definition 4 either).
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
    /// Port of mCRL2's `safe_control_predecessors`, needed by
    /// [`partial_solve::detect_forced_cycles`]'s forced-cycle search, which records an
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

    /// Returns a copy of this game where every transition group owned by `alpha` has been
    /// restricted to `strategy`.
    ///
    /// Port of mCRL2's `apply_strategy`. `strategy` is expected over the doubled, interleaved
    /// global vector `[from_0, to_0, from_1, to_1, …]` — see the module documentation §6 in the
    /// implementation plan. A group is considered owned by `alpha` when *any* vertex it can fire
    /// from belongs to `alpha` (checked once here via [`Self::predecessors_group`] against
    /// [`Self::all_vertices`], rather than mCRL2's cheaper but harder-to-port per-group read
    /// projection, since a group's ownership never actually straddles both players in practice —
    /// same assumption mCRL2's own overlap test relies on). A group with no strategy at all (the
    /// strategy is only ever partial for vertices *outside* the winning region requested) is
    /// dropped to the empty relation, which is what turns an unresolved vertex into a fresh sink
    /// for the caller (`check_strategy`) to route through [`Self::compute_total_graph`] again.
    pub fn apply_strategy(&self, alpha: Player, strategy: &LDDFunction) -> Result<Self, MercError> {
        let all_vertices = self.all_vertices()?;
        let strategy_is_empty = strategy.is_empty();

        let mut relations = Vec::with_capacity(self.relations.len());
        for relation in &self.relations {
            let domain = self.predecessors_group(relation, &all_vertices, &all_vertices)?;
            let owner = if intersect(&domain, &self.vertices[Player::Even.to_index()])?.is_empty() {
                Player::Odd
            } else {
                Player::Even
            };

            let new_relation = if owner != alpha {
                relation.relation.clone()
            } else if strategy_is_empty {
                self.empty()?
            } else {
                let keep: Vec<Value> = relation
                    .read_indices
                    .iter()
                    .map(|&r| 2 * r)
                    .chain(relation.write_indices.iter().map(|&w| 2 * w + 1))
                    .collect();
                let projection_meta = self
                    .manager
                    .with_manager_shared(|m| LDDFunction::projection_meta(m, &keep))?;
                let projected_strategy = strategy.project(&projection_meta)?;
                intersect(&relation.relation, &projected_strategy)?
            };

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
            self.vertices[Player::Even.to_index()].len(),
            self.vertices[Player::Odd.to_index()].len()
        )
    }
}
