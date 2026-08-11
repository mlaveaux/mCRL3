use std::cell::Cell;
use std::ops::Range;

use log::info;

use merc_explore::CacheLPS;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_explore::configure_rayon_thread_pool;
use merc_explore::explore;
use merc_explore::explore_parallel;
use merc_io::TimeProgress;
use merc_lts::StateIndex;
use merc_utilities::MercError;
use merc_utilities::Timing;
use merc_vpg::ParityGame;
use merc_vpg::ParityGameBuilder;
use merc_vpg::Player;
use merc_vpg::Priority;
use merc_vpg::VertexIndex;

use merc_explore::ExplorationStrategy;

/// Whether counter-example equations are excluded when unifying parameters.
///
/// Part of [`UNIFY_RESET_PARAMETERS`]'s contract: every caller must pass the same
/// pair of flags.
pub(crate) const UNIFY_IGNORE_CE_EQUATIONS: bool = false;

/// Whether a parameter that an equation does not declare is reset to a default
/// value rather than copied through.
///
/// Symmetry detection and exploration must unify parameters with *identical*
/// flags. This one changes the right-hand sides (see mCRL2's
/// `unify_parameters_replace_function`), so detecting symmetries on one PBES and
/// applying them while exploring a differently unified one is unsound.
pub(crate) const UNIFY_RESET_PARAMETERS: bool = true;

/// An [`LPS`] whose state vectors may carry a block of permutable data parameters.
///
/// Exploring a PBES into a parity game produces states of several shapes: a
/// propositional variable instantiation carries the parameter vector, while sinks
/// and subformula vertices carry a priority and an interned formula index instead.
/// A symmetry group acts on the parameters only, so a layer that permutes state
/// vectors has to be able to tell the shapes apart — permuting a subformula
/// vertex's payload silently corrupts it.
pub(crate) trait ParameterLayout: LPS {
    /// Returns the positions of `state` holding data parameters, or `None` when
    /// this state has no parameter block.
    fn parameter_range(&self, state: &[Self::Value]) -> Option<Range<usize>>;
}

impl<P: ParameterLayout> ParameterLayout for CacheLPS<P> {
    fn parameter_range(&self, state: &[Self::Value]) -> Option<Range<usize>> {
        self.inner().parameter_range(state)
    }
}

impl<P: ParameterLayout> ParameterLayout for &P {
    fn parameter_range(&self, state: &[Self::Value]) -> Option<Range<usize>> {
        (**self).parameter_range(state)
    }
}

/// What a parity-game vertex was created for.
///
/// mCRL2's `pbesinst_structure_graph` draws the same distinction: `SG0` creates
/// one vertex per propositional variable instantiation — the number its verbose
/// output reports as "Generated N BES equations" — while `SG1` creates an extra
/// vertex for every nested subformula that is not itself an instantiation. Both
/// kinds are vertices of the structure graph, so the total vertex count of the
/// generated parity game exceeds the equation count.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PbesVertexKind {
    /// A propositional variable instantiation, i.e. one BES equation.
    Instantiation,

    /// A nested and/or subformula that needs a vertex of its own because a
    /// parity-game vertex has a single owner, so a disjunction occurring under a
    /// conjunction (or vice versa) cannot be merged into its parent.
    Subformula,

    /// One of the two `true` / `false` sink vertices.
    Sink,
}

/// Owner, priority and provenance of a parity-game vertex, produced by
/// [`LPS::state_info`] of the PBES explorers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct PbesVertex {
    /// The player owning the vertex.
    pub(crate) player: Player,

    /// The vertex priority.
    pub(crate) priority: Priority,

    /// What the vertex was created for.
    pub(crate) kind: PbesVertexKind,
}

impl PbesVertex {
    pub(crate) fn new(player: Player, priority: Priority, kind: PbesVertexKind) -> Self {
        PbesVertex { player, priority, kind }
    }

    /// Shorthand for a vertex standing for a propositional variable instantiation.
    pub(crate) fn instantiation(player: Player, priority: Priority) -> Self {
        PbesVertex::new(player, priority, PbesVertexKind::Instantiation)
    }
}

/// Tally of the generated parity-game vertices, broken down by [`PbesVertexKind`].
#[derive(Clone, Copy, Default)]
pub(crate) struct VertexCounts {
    /// Vertices for propositional variable instantiations (BES equations).
    instantiations: usize,

    /// Vertices for nested subformulas.
    subformulas: usize,

    /// The `true` / `false` sink vertices, at most two.
    sinks: usize,
}

impl VertexCounts {
    /// Returns these counts with `kind` added.
    fn with(mut self, kind: PbesVertexKind) -> Self {
        match kind {
            PbesVertexKind::Instantiation => self.instantiations += 1,
            PbesVertexKind::Subformula => self.subformulas += 1,
            PbesVertexKind::Sink => self.sinks += 1,
        }
        self
    }

    /// Total number of vertices, i.e. the size of the structure graph.
    fn total(&self) -> usize {
        self.instantiations + self.subformulas + self.sinks
    }
}

/// Logs the final vertex breakdown, reporting the instantiation count (directly
/// comparable to mCRL2's "Generated N BES equations") separately from the
/// structure-graph vertices that surround it.
fn report_counts(counts: VertexCounts, edges: usize) {
    info!(
        "Exploration complete: {} BES equations, {} subformula vertices, {} sinks ({} vertices, {edges} edges)",
        counts.instantiations,
        counts.subformulas,
        counts.sinks,
        counts.total(),
    );
}

/// Periodic progress reporter for PBES exploration.
pub(crate) fn bes_progress() -> TimeProgress<(VertexCounts, usize)> {
    TimeProgress::new(
        |(counts, edges): (VertexCounts, usize)| {
            info!(
                "Explored {} BES equations, {} vertices, {edges} edges...",
                counts.instantiations,
                counts.total(),
            );
        },
        1,
    )
}

/// Builds a [`ParityGame`] by exploring any LPS that produces unit labels and
/// [`PbesVertex`] state info (i.e. a parity game vertex description).
pub(crate) fn explore_pbes_impl<M>(
    lps: &M,
    strategy: ExplorationStrategy,
    timing: &Timing,
) -> Result<ParityGame, MercError>
where
    M: LPS<Value = usize, Label = (), StateInfo = PbesVertex>,
{
    let mut builder = ParityGameBuilder::new(VertexIndex::new(0));

    let progress = bes_progress();
    let counts = Cell::new(VertexCounts::default());
    let edges = Cell::new(0usize);

    let _initial = explore(
        lps,
        strategy,
        timing,
        &mut builder,
        |b: &mut ParityGameBuilder, state: StateIndex, info: &PbesVertex| {
            counts.set(counts.get().with(info.kind));
            b.add_vertex(VertexIndex::new(state.value()), info.player, info.priority);
            Ok(())
        },
        |b: &mut ParityGameBuilder, from: StateIndex, _label: &(), to: StateIndex| {
            edges.set(edges.get() + 1);
            progress.print((counts.get(), edges.get()));
            b.add_edge(VertexIndex::new(from.value()), VertexIndex::new(to.value()));
            Ok(())
        },
    )?;
    report_counts(counts.get(), edges.get());

    Ok(builder.finish(true, true))
}

/// Per-worker output partition for parallel parity-game exploration.
#[derive(Default)]
pub(crate) struct PbesPartition {
    /// Vertices discovered by this worker, with their owner, priority and kind.
    pub(crate) vertices: Vec<(VertexIndex, PbesVertex)>,

    /// Edges discovered by this worker, as `(source, target)` pairs.
    pub(crate) edges: Vec<(VertexIndex, VertexIndex)>,
}

/// Builds a [`ParityGame`] by exploring any sync-safe LPS in parallel.
pub(crate) fn explore_pbes_parallel_impl<M>(lps: &M, threads: usize, pinned: bool) -> Result<ParityGame, MercError>
where
    M: LPS<Value = usize, Label = (), StateInfo = PbesVertex> + Sync,
    <M::Summand as Summand>::Context: Send,
{
    let pool = configure_rayon_thread_pool(threads, pinned)?;
    let timing = Timing::new();

    let (_initial, partitions) = timing.measure("explore", || {
        pool.install(|| {
            explore_parallel(
                lps,
                PbesPartition::default,
                |partition: &mut PbesPartition, state: StateIndex, info: &PbesVertex| {
                    partition.vertices.push((VertexIndex::new(state.value()), *info));
                    Ok(())
                },
                |partition: &mut PbesPartition, from: StateIndex, _label: &(), to: StateIndex| {
                    partition
                        .edges
                        .push((VertexIndex::new(from.value()), VertexIndex::new(to.value())));
                    Ok(())
                },
            )
        })
    })?;

    let counts = partitions
        .iter()
        .flat_map(|p| p.vertices.iter())
        .fold(VertexCounts::default(), |counts, (_, vertex)| counts.with(vertex.kind));
    let total_edges: usize = partitions.iter().map(|p| p.edges.len()).sum();
    report_counts(counts, total_edges);

    let mut builder = ParityGameBuilder::new(VertexIndex::new(0));
    for partition in &partitions {
        for &(index, vertex) in &partition.vertices {
            builder.add_vertex(index, vertex.player, vertex.priority);
        }
    }
    for partition in &partitions {
        for &(from, to) in &partition.edges {
            builder.add_edge(from, to);
        }
    }
    Ok(builder.finish(true, true))
}

/// Computes a priority for each equation for a **max** parity game.
///
/// `is_mu[i]` is `true` when equation `i` is a least fixpoint (μ), `false` for ν.
/// Equations must be in declaration order (outermost first).
///
/// Algorithm:
/// 1. Assign each equation an *alternation depth* (incremented on every μ ↔ ν switch).
/// 2. Reverse so outermost (depth 0) → highest priority (max_depth).
/// 3. Shift all priorities by 1 when the outermost equation's parity does not
///    match its fixpoint type (ν → even, μ → odd).
pub(crate) fn compute_priorities(is_mu: &[bool]) -> Vec<usize> {
    if is_mu.is_empty() {
        return Vec::new();
    }

    let mut depths = vec![0usize; is_mu.len()];
    let mut current_depth = 0usize;
    let mut prev_is_mu = is_mu[0];

    for (i, &mu) in is_mu.iter().enumerate() {
        if i > 0 && mu != prev_is_mu {
            current_depth += 1;
        }
        depths[i] = current_depth;
        prev_is_mu = mu;
    }

    let max_depth = *depths.last().unwrap();
    let mut priorities: Vec<usize> = depths.iter().map(|&d| max_depth - d).collect();

    let first_is_mu = is_mu[0];
    if first_is_mu == priorities[0].is_multiple_of(2) {
        for p in &mut priorities {
            *p += 1;
        }
    }

    debug_assert!(
        priorities
            .iter()
            .zip(is_mu.iter())
            .all(|(p, &mu)| p.is_multiple_of(2) != mu),
        "Max parity game invariant violated: ν must have even priority and μ must have odd priority"
    );

    priorities
}
