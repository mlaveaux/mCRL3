use std::cell::Cell;

use log::info;

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

use merc_explore::CachingStrategy;
use merc_explore::ExplorationStrategy;

/// Periodic progress reporter for PBES exploration.
pub(crate) fn bes_progress() -> TimeProgress<(usize, usize)> {
    TimeProgress::new(
        |(equations, edges): (usize, usize)| {
            info!("Explored {equations} BES equations, {edges} edges...");
        },
        1,
    )
}

/// Builds a [`ParityGame`] by exploring any LPS that produces unit labels and
/// `(Player, Priority)` state info (i.e. a parity game vertex description).
pub(crate) fn run_explore_parity_game<M>(
    lps: &M,
    strategy: ExplorationStrategy,
    timing: &Timing,
) -> Result<ParityGame, MercError>
where
    M: LPS<Value = usize, Label = (), StateInfo = (Player, Priority)>,
{
    let mut builder = ParityGameBuilder::new(VertexIndex::new(0));

    let progress = bes_progress();
    let equations = Cell::new(0usize);
    let edges = Cell::new(0usize);

    let _initial = explore(
        lps,
        strategy,
        timing,
        &mut builder,
        |b: &mut ParityGameBuilder, state: StateIndex, info: &(Player, Priority)| {
            equations.set(equations.get() + 1);
            b.add_vertex(VertexIndex::new(state.value()), info.0, info.1);
            Ok(())
        },
        |b: &mut ParityGameBuilder, from: StateIndex, _label: &(), to: StateIndex| {
            edges.set(edges.get() + 1);
            progress.print((equations.get(), edges.get()));
            b.add_edge(VertexIndex::new(from.value()), VertexIndex::new(to.value()));
            Ok(())
        },
    )?;
    info!(
        "Exploration complete: {} BES equations, {} edges",
        equations.get(),
        edges.get(),
    );

    Ok(builder.finish(true, true))
}

/// Per-worker output partition for parallel parity-game exploration.
#[derive(Default)]
pub(crate) struct PbesPartition {
    pub(crate) vertices: Vec<(VertexIndex, Player, Priority)>,
    pub(crate) edges: Vec<(VertexIndex, VertexIndex)>,
}

/// Builds a [`ParityGame`] by exploring any sync-safe LPS in parallel.
pub(crate) fn run_explore_parity_game_parallel<M>(
    lps: &M,
    threads: usize,
    caching: CachingStrategy,
    pinned: bool,
) -> Result<ParityGame, MercError>
where
    M: LPS<Value = usize, Label = (), StateInfo = (Player, Priority)> + Sync,
    <M::Summand as Summand>::Context: Send,
{
    let pool = configure_rayon_thread_pool(threads, pinned)?;
    let timing = Timing::new();

    let (_initial, partitions) = timing.measure("explore", || {
        pool.install(|| {
            explore_parallel(
                lps,
                PbesPartition::default,
                |partition: &mut PbesPartition, state: StateIndex, info: &(Player, Priority)| {
                    partition
                        .vertices
                        .push((VertexIndex::new(state.value()), info.0, info.1));
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

    let total_equations: usize = partitions.iter().map(|p| p.vertices.len()).sum();
    let total_edges: usize = partitions.iter().map(|p| p.edges.len()).sum();
    info!("Exploration complete: {total_equations} BES equations, {total_edges} edges");

    let mut builder = ParityGameBuilder::new(VertexIndex::new(0));
    for partition in &partitions {
        for &(vertex, player, priority) in &partition.vertices {
            builder.add_vertex(vertex, player, priority);
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
