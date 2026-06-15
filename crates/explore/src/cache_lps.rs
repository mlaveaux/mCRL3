use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use merc_unsafety::ShardedHashMap;
use rustc_hash::FxBuildHasher;

use merc_utilities::MercError;

use crate::BTreeForest;
use crate::BTreeForestContext;
use crate::LPS;
use crate::Summand;
use crate::Tree;

/// Controls the enumeration caching behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum CachingStrategy {
    /// No caching; every enumeration is performed from scratch.
    None,
    /// Each summand maintains its own independent cache.
    Local,
}

/// Number of shards in each summand's local cache. Modest because the sharding
/// is already two-dimensional (one independent cache per summand) and the hot
/// lookup path only takes a shard read lock.
const CACHE_SHARDS: usize = 16;

/// A wrapper around an [`LPS`] that caches summand enumeration results.
///
/// The cache key for each summand is formed by projecting the state vector onto
/// the summand's read positions.
///
/// The cache is thread-safe: each summand's local cache, the shared forest and
/// the hit / miss counters are all updated through `&self`, so a single
/// `CacheLPS` can be shared (by `&`) across worker threads. Each thread threads
/// its own [`CacheContext`] through [`Summand::enumerate`] for the reusable
/// scratch buffers.
pub struct CacheLPS<P: LPS> {
    inner: Arc<P>,
    summands: Vec<CacheSummandWrapper<P>>,
}

/// A single cached enumeration result keyed by the read-position projection.
///
/// Stored as a bare element in a summand's sharded cache table; the hash and
/// equality are taken over `key` only. The captured results live behind an
/// [`Arc`] so cache hits clone the handle rather than the whole vector.
struct CacheEntry<L> {
    /// Hash-consed projection of the source state onto the read positions.
    key: Tree,
    /// Captured `(label, write-position values tree)` pairs for the key.
    results: Arc<Vec<(L, Tree)>>,
}

impl<L> Clone for CacheEntry<L> {
    fn clone(&self) -> Self {
        CacheEntry {
            key: self.key,
            results: Arc::clone(&self.results),
        }
    }
}

/// Per-thread reusable scratch buffers for [`CacheSummandWrapper::enumerate`].
///
/// Holding these per thread (rather than in shared `&self` state) is what makes
/// concurrent enumeration sound: each thread interns into the shared forest and
/// cache through `&self` but stages its keys and replay values in its own
/// context.
pub struct CacheContext<P: LPS> {
    /// Buffer projecting the state vector onto read (then write) positions.
    key_buf: Vec<P::Value>,
    /// Buffer reconstructing a next-state from a cached write-position tree.
    replay_buf: Vec<P::Value>,
    /// Scratch buffers reused when interning into the forest.
    forest_context: BTreeForestContext,
    /// Enumeration context for the wrapped inner summand (cache misses).
    inner: <P::Summand as Summand>::Context,
}

impl<P: LPS> Default for CacheContext<P> {
    fn default() -> Self {
        CacheContext {
            key_buf: Vec::new(),
            replay_buf: Vec::new(),
            forest_context: BTreeForestContext::new(),
            inner: Default::default(),
        }
    }
}

/// Thin metadata wrapper for a single summand in a [`CacheLPS`].
pub struct CacheSummandWrapper<P: LPS> {
    /// Index of the summand in the LPS, used for cache lookups.
    index: usize,

    /// Positions in the state vector that are read by this summand; these form the cache key.
    read_positions: Vec<usize>,
    /// Positions in the state vector that are written by this summand; these are stored in the cache values.
    write_positions: Vec<usize>,

    /// Caching strategy in effect for this summand.
    strategy: CachingStrategy,

    /// This summand's local enumeration cache, keyed by the read-position
    /// projection tree. One independent cache per summand, as intended by the
    /// `Local` strategy.
    cache: ShardedHashMap<CacheEntry<P::Label>, FxBuildHasher>,

    /// Hash-consed forest holding the keys and captured values. Shared across
    /// all summands so equal sequences are stored once.
    forest: Arc<BTreeForest<P::Value, 2>>,

    /// Shared reference to the inner LPS for delegating cache misses.
    inner: Arc<P>,

    /// Number of enumerations served from the cache.
    hits: AtomicU64,
    /// Number of enumerations that had to be delegated to the inner summand.
    misses: AtomicU64,
}

impl<P: LPS> CacheLPS<P> {
    pub fn new(inner: P, strategy: CachingStrategy) -> Self {
        let inner = Arc::new(inner);
        let forest = Arc::new(BTreeForest::new());

        let summands: Vec<_> = inner
            .summands()
            .iter()
            .enumerate()
            .map(|(i, s)| CacheSummandWrapper {
                index: i,
                read_positions: s.read_positions().to_vec(),
                write_positions: s.write_positions().to_vec(),
                strategy,
                cache: ShardedHashMap::with_shards(CACHE_SHARDS),
                forest: Arc::clone(&forest),
                inner: Arc::clone(&inner),
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
            })
            .collect();

        CacheLPS { inner, summands }
    }

    /// Collects per-summand cache metrics for this [`CacheLPS`].
    ///
    /// The returned [`CacheMetrics`] implements [`fmt::Display`] for a
    /// human-readable summary of cache hits, misses and occupancy.
    pub fn metrics(&self) -> CacheMetrics {
        let summands = self
            .summands
            .iter()
            .map(|s| {
                let entries = match s.strategy {
                    CachingStrategy::None => 0,
                    CachingStrategy::Local => s.cache.len(),
                };

                SummandCacheMetrics {
                    index: s.index,
                    strategy: s.strategy,
                    hits: s.hits.load(Ordering::Relaxed),
                    misses: s.misses.load(Ordering::Relaxed),
                    entries,
                }
            })
            .collect();

        CacheMetrics { summands }
    }
}

/// Cache metrics for a single summand.
#[derive(Clone, Copy, Debug)]
pub struct SummandCacheMetrics {
    /// Index of the summand in the LPS.
    pub index: usize,
    /// Caching strategy in effect for this summand.
    pub strategy: CachingStrategy,
    /// Number of enumerations served from the cache.
    pub hits: u64,
    /// Number of enumerations delegated to the inner summand.
    pub misses: u64,
    /// Number of distinct keys currently stored for this summand.
    pub entries: usize,
}

impl SummandCacheMetrics {
    /// Total number of enumeration lookups.
    pub fn lookups(&self) -> u64 {
        self.hits + self.misses
    }

    /// Fraction of lookups served from the cache, in `[0.0, 1.0]`.
    ///
    /// Returns `0.0` when there were no lookups.
    pub fn hit_rate(&self) -> f64 {
        let lookups = self.lookups();
        if lookups == 0 {
            0.0
        } else {
            self.hits as f64 / lookups as f64
        }
    }
}

/// Aggregated cache metrics for every summand of a [`CacheLPS`].
#[derive(Clone, Debug)]
pub struct CacheMetrics {
    /// Per-summand metrics, ordered by summand index.
    pub summands: Vec<SummandCacheMetrics>,
}

impl CacheMetrics {
    /// Total number of cache hits across all summands.
    pub fn total_hits(&self) -> u64 {
        self.summands.iter().map(|s| s.hits).sum()
    }

    /// Total number of cache misses across all summands.
    pub fn total_misses(&self) -> u64 {
        self.summands.iter().map(|s| s.misses).sum()
    }

    /// Total number of cached entries across all summands.
    pub fn total_entries(&self) -> usize {
        self.summands.iter().map(|s| s.entries).sum()
    }

    /// Fraction of lookups served from the cache across all summands.
    pub fn hit_rate(&self) -> f64 {
        let hits = self.total_hits();
        let lookups = hits + self.total_misses();
        if lookups == 0 {
            0.0
        } else {
            hits as f64 / lookups as f64
        }
    }
}

impl fmt::Display for CacheMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "summand cache metrics:")?;
        writeln!(
            f,
            "  {:>7}  {:>8}  {:>10}  {:>10}  {:>8}  {:>8}",
            "summand", "strategy", "hits", "misses", "hit%", "entries"
        )?;
        for s in &self.summands {
            let strategy = match s.strategy {
                CachingStrategy::None => "none",
                CachingStrategy::Local => "local",
            };
            writeln!(
                f,
                "  {:>7}  {:>8}  {:>10}  {:>10}  {:>7.1}%  {:>8}",
                s.index,
                strategy,
                s.hits,
                s.misses,
                s.hit_rate() * 100.0,
                s.entries
            )?;
        }
        write!(
            f,
            "  total: {} hits, {} misses ({:.1}% hit rate), {} entries",
            self.total_hits(),
            self.total_misses(),
            self.hit_rate() * 100.0,
            self.total_entries()
        )
    }
}

impl<P: LPS> LPS for CacheLPS<P> {
    type Value = P::Value;
    type Label = P::Label;
    type StateInfo = P::StateInfo;
    type Summand = CacheSummandWrapper<P>;

    fn initial_state(&self) -> Vec<Self::Value> {
        self.inner.initial_state()
    }

    fn summands(&self) -> &[Self::Summand] {
        &self.summands
    }

    fn prepare(&self, state: &[Self::Value]) {
        self.inner.prepare(state);
    }

    fn state_info(&self, state: &[Self::Value]) -> Self::StateInfo {
        self.inner.state_info(state)
    }
}

impl<P: LPS> CacheSummandWrapper<P> {
    fn replay_cached(
        &self,
        context: &mut CacheContext<P>,
        state: &[P::Value],
        results: &[(P::Label, Tree)],
        report: &mut impl FnMut(&P::Label, &[P::Value]) -> Result<(), MercError>,
    ) -> Result<(), MercError> {
        // Each cached tree holds only the values at the write positions.
        // The remaining positions are passed through unchanged, so they
        // must be taken from the *live* source state rather than the state
        // for which the entry was originally computed.
        let replay_buf = &mut context.replay_buf;
        for (label, write_tree) in results {
            replay_buf.clear();
            replay_buf.extend_from_slice(state);
            for (&pos, value) in self.write_positions.iter().zip(self.forest.iter(*write_tree)) {
                replay_buf[pos] = value;
            }
            report(label, replay_buf)?;
        }
        Ok(())
    }
}

impl<P: LPS> Summand for CacheSummandWrapper<P> {
    type Value = P::Value;
    type Label = P::Label;
    type Context = CacheContext<P>;

    fn enumerate<F>(&self, context: &mut Self::Context, state: &[Self::Value], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[Self::Value]) -> Result<(), MercError>,
    {
        if self.strategy == CachingStrategy::None || self.read_positions.is_empty() {
            return self.inner.summands()[self.index].enumerate(&mut context.inner, state, report);
        }

        // Project the state vector onto the read positions to form the cache
        // key, interning it into the shared forest.
        let key_tree = {
            let CacheContext {
                key_buf,
                forest_context,
                ..
            } = &mut *context;
            key_buf.clear();
            for &pos in &self.read_positions {
                key_buf.push(state[pos]);
            }
            self.forest.insert_with(key_buf, forest_context)
        };

        // This summand owns its cache, so the key is the projection tree alone.
        let hash = self.cache.hash(&key_tree);
        let eq = |entry: &CacheEntry<P::Label>| entry.key == key_tree;

        // Fast path: a present entry is found under a read lock only.
        if let Some(entry) = self.cache.find(hash, eq) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            self.replay_cached(context, state, &entry.results, &mut report)?;
            return Ok(());
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        // Cache MISS: delegate to inner summand, capture results. Only the
        // values at the write positions are stored; on replay they are
        // scattered back onto the live source state (see the hit branch).
        let mut captured: Vec<(P::Label, Tree)> = Vec::new();
        {
            let inner_summand = &self.inner.summands()[self.index];
            let forest = &self.forest;
            let write_positions = &self.write_positions;
            let CacheContext {
                key_buf,
                forest_context,
                inner,
                ..
            } = &mut *context;

            inner_summand.enumerate(inner, state, |label, next_state| {
                key_buf.clear();
                for &pos in write_positions {
                    key_buf.push(next_state[pos]);
                }
                let tree = forest.insert_with(key_buf, forest_context);
                captured.push((label.clone(), tree));
                report(label, next_state)
            })?;
        }

        // Publish the captured results. A concurrent thread may have inserted
        // the same key meanwhile; `find_or_insert_with` keeps the resident
        // entry and our copy is dropped. Either way the transitions above were
        // already reported.
        let results = Arc::new(captured);
        self.cache.find_or_insert_with(
            hash,
            eq,
            |entry| self.cache.hash(&entry.key),
            || CacheEntry { key: key_tree, results },
        );

        Ok(())
    }

    fn read_positions(&self) -> &[usize] {
        &self.read_positions
    }

    fn write_positions(&self) -> &[usize] {
        &self.write_positions
    }
}
