use std::collections::VecDeque;
use std::sync::Mutex;

use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use rayon::slice::ParallelSlice;

use merc_lts::StateIndex;
use merc_utilities::MercError;
use merc_utilities::Timing;

use crate::BTreeForestContext;
use crate::DiscoveredSet;
use crate::LPS;
use crate::StateRef;
use crate::Summand;

/// Order in which discovered states are explored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Default)]
pub enum ExplorationStrategy {
    /// Depth-first exploration.
    #[default]
    Dfs,
    /// Breadth-first exploration.
    Bfs,
}

/// Explores the state space of `lps`, invoking caller-supplied closures for
/// each discovered state and transition.
///
/// # Closures
///
/// The mutable caller context `ctx` (typically a builder) is passed into
/// every closure invocation so the closures themselves can stay
/// non-capturing of the outer mutable state.
///
/// - `on_state(ctx, state_index, &state_info)` is called exactly once per
///   discovered state, in the order the state was popped from the working set,
///   after the implementation has been prepared for that state.
/// - `on_transition(ctx, from, &label, to)` is called for every transition
///   produced by the summands.
///
/// Returns the [`StateIndex`] assigned to the initial state of `lps`. The
/// caller is responsible for finalising any builder it owns; counting states
/// and transitions (and any progress reporting) is left to the closures.
pub fn explore<P, Ctx, OnState, OnTransition>(
    lps: &P,
    strategy: ExplorationStrategy,
    timing: &Timing,
    ctx: &mut Ctx,
    mut on_state: OnState,
    mut on_transition: OnTransition,
) -> Result<StateIndex, MercError>
where
    P: LPS,
    OnState: FnMut(&mut Ctx, StateIndex, &P::StateInfo) -> Result<(), MercError>,
    OnTransition: FnMut(&mut Ctx, StateIndex, &P::Label, StateIndex) -> Result<(), MercError>,
{
    let discovered: DiscoveredSet<P::Value> = DiscoveredSet::new();
    let (initial_ref, _) = discovered.insert(&lps.initial_state());

    let mut working: VecDeque<StateRef> = VecDeque::from([initial_ref]);
    // Reusable buffer holding the current state vector reconstructed from the
    // discovered set, avoiding an allocation per explored state.
    let mut current_state: Vec<P::Value> = Vec::new();
    // Reused interning scratch buffers, avoiding a reallocation per inserted
    // state. This loop is single-threaded, so one context suffices.
    let mut forest_context = BTreeForestContext::new();
    // Per-thread enumeration context owning the backend and scratch buffers.
    // Single-threaded loop, so one context suffices.
    let mut enumerate_context = lps.create_context();

    timing.measure("explore", || -> Result<(), MercError> {
        loop {
            let current = match strategy {
                ExplorationStrategy::Dfs => working.pop_back(),
                ExplorationStrategy::Bfs => working.pop_front(),
            };
            let Some(current) = current else { break };

            // Reconstruct the state vector into the reusable buffer so
            // `discovered` can be mutated by inserts inside the summand callback
            // below.
            let found = discovered.get_into(current, &mut current_state);
            debug_assert!(found, "StateRef from working queue must be valid");
            let from = StateIndex::new(current.index());
            let summands_to_explore = lps.prepare(&mut enumerate_context, &current_state);

            let info = lps.state_info(&current_state);
            on_state(ctx, from, &info)?;

            let summands = lps.summands();
            for index in summands_to_explore {
                summands[index].enumerate(&mut enumerate_context, &current_state, |label, next_state| {
                    let (target_ref, is_new) = discovered.insert_with(next_state, &mut forest_context);
                    let to = StateIndex::new(target_ref.index());
                    on_transition(ctx, from, label, to)?;
                    if is_new {
                        working.push_back(target_ref);
                    }
                    Ok(())
                })?;
            }
        }

        Ok(())
    })?;

    Ok(StateIndex::new(initial_ref.index()))
}

/// Per-worker scratch state and output accumulator for [`explore_parallel`].
///
/// Note that the mCRL2 enumeration backend is deliberately *not* stored here:
/// its terms are thread-affine (each `aterm` must be created and destroyed on
/// the same thread), while a [`Segment`] is migrated between physical worker
/// threads by rayon's work-stealing scheduler. The backend therefore lives in a
/// separate per-thread slot, see [`explore_parallel`].
struct Segment<P: LPS, Local> {
    /// Per-thread interning scratch reused across inserts.
    forest_context: BTreeForestContext,
    /// Reusable buffer holding the reconstructed source state vector.
    state_buf: Vec<P::Value>,
    /// The caller's per-thread output accumulator (e.g. a transition partition).
    local: Local,
    /// States first discovered by this segment, forming part of the next level.
    new_refs: Vec<StateRef>,
    /// First error raised by a caller closure, if any; processing stops after it.
    error: Option<MercError>,
}

/// Explores the state space of `lps` in parallel using a level-synchronized
/// breadth-first search driven by `rayon`.
///
/// The interface is similar to [`explore`], but an extra closure is supplied to
/// produce a fresh per-thread accumulator for each worker.
///
/// One [`Segment`] is allocated per worker before exploration begins and reused
/// across every BFS level, so each worker's LPS enumeration context, interning
/// scratch, and accumulator are created exactly once. Rayon passes each segment
/// into its job by mutable reference; no per-job or thread-local allocation is
/// performed inside the loop.
///
/// - `make_local()` produces a fresh accumulator for each worker segment.
///
/// The returned `Vec<Local>` holds one accumulator per worker (at most
/// `rayon::current_num_threads()` entries), each carrying the merged result of
/// every level that worker processed.
pub fn explore_parallel<P, Local, MakeLocal, OnState, OnTransition>(
    lps: &P,
    timing: &Timing,
    make_local: MakeLocal,
    on_state: OnState,
    on_transition: OnTransition,
) -> Result<(StateIndex, Vec<Local>), MercError>
where
    P: LPS + Sync,
    P::Value: Send + Sync,
    <P::Summand as Summand>::Context: Send,
    Local: Send,
    MakeLocal: Fn() -> Local + Sync,
    OnState: Fn(&mut Local, StateIndex, &P::StateInfo) -> Result<(), MercError> + Sync,
    OnTransition: Fn(&mut Local, StateIndex, &P::Label, StateIndex) -> Result<(), MercError> + Sync,
{
    let discovered: DiscoveredSet<P::Value> = DiscoveredSet::new();
    let (initial_ref, _) = discovered.insert(&lps.initial_state());

    // One segment per worker in the active rayon pool, allocated once and reused
    // across all levels so the scratch buffers are created a single time per
    // worker. The segments carry only thread-agnostic scratch (their values are
    // interned indices, not terms) and may freely migrate between physical
    // worker threads.
    let num_workers = rayon::current_num_threads().max(1);
    let mut segments: Vec<Segment<P, Local>> = (0..num_workers)
        .map(|_| Segment {
            forest_context: BTreeForestContext::new(),
            state_buf: Vec::new(),
            local: make_local(),
            new_refs: Vec::new(),
            error: None,
        })
        .collect();

    // Enumeration backends pinned to physical worker threads. mCRL2 terms are
    // thread-affine (each term must be created and destroyed on the same
    // thread), so the backend cannot live in a `Segment` that rayon migrates
    // between threads. Each slot is keyed by `rayon::current_thread_index()`,
    // lazily created on first use on its owning thread, reused across all levels,
    // and finally dropped on that same thread (see the `broadcast` below).
    let contexts: Vec<Mutex<Option<<P::Summand as Summand>::Context>>> =
        (0..num_workers).map(|_| Mutex::new(None)).collect();

    let mut frontier: Vec<StateRef> = vec![initial_ref];

    let result = timing.measure("explore", || -> Result<(), MercError> {
        while !frontier.is_empty() {
            // Split the current level across the persistent segments. Each
            // chunk is processed by one segment, reusing that segment's scratch
            // buffers and its physical worker thread's enumeration backend.
            let chunk_size = frontier.len().div_ceil(num_workers);
            segments
                .par_iter_mut()
                .zip(frontier.par_chunks(chunk_size))
                .for_each(|(seg, chunk)| {
                    // Borrow the fields disjointly so the enumeration callback
                    // can mutate the accumulator and discovery buffers while the
                    // backend reads the source state.
                    let Segment {
                        forest_context,
                        state_buf,
                        local,
                        new_refs,
                        error,
                    } = seg;

                    // Acquire the enumeration backend pinned to this physical
                    // worker thread, creating it on first use.
                    let thread_index = rayon::current_thread_index().expect("This must only be called from a worker thread");
                    let mut context_guard = contexts[thread_index].lock().expect("context slot poisoned");
                    let context = context_guard.get_or_insert_with(|| lps.create_context());

                    for &state_ref in chunk {
                        let result = (|| -> Result<(), MercError> {
                            let found = discovered.get_into(state_ref, state_buf);
                            debug_assert!(found, "StateRef from frontier must be valid");
                            let from = StateIndex::new(state_ref.index());
                            let summands_to_explore = lps.prepare(context, state_buf);

                            let info = lps.state_info(state_buf);
                            on_state(local, from, &info)?;

                            let summands = lps.summands();
                            for index in summands_to_explore {
                                summands[index].enumerate(context, state_buf, |label, next_state| {
                                    let (target_ref, is_new) = discovered.insert_with(next_state, forest_context);
                                    let to = StateIndex::new(target_ref.index());
                                    on_transition(local, from, label, to)?;
                                    if is_new {
                                        new_refs.push(target_ref);
                                    }
                                    Ok(())
                                })?;
                            }
                            Ok(())
                        })();

                        if let Err(err) = result {
                            *error = Some(err);
                            break;
                        }
                    }
                });

            // Barrier: surface the first error and gather the next level from
            // the per-segment discoveries.
            frontier.clear();
            for segment in &mut segments {
                if let Some(error) = segment.error.take() {
                    return Err(error);
                }
                frontier.append(&mut segment.new_refs);
            }
        }

        Ok(())
    });

    // Drop each pinned enumeration backend on its owning worker thread. mCRL2
    // terms are thread-affine, so a backend created on one worker must also be
    // destroyed there; dropping it on the calling thread (as part of dropping
    // `contexts`) would corrupt that worker's protection set and abort. This
    // runs on both the success and error paths, before `contexts` itself is
    // dropped on the calling thread (by then all slots are empty).
    rayon::broadcast(|ctx| {
        if let Some(slot) = contexts.get(ctx.index()) {
            let _ = slot.lock().expect("context slot poisoned").take();
        }
    });

    result?;

    let locals = segments.into_iter().map(|segment| segment.local).collect();
    Ok((StateIndex::new(initial_ref.index()), locals))
}
