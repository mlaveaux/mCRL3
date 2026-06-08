use std::cell::Cell;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use allocator_api2::alloc::Global;
use rustc_hash::FxHashMap;

use merc_utilities::MercError;

use crate::BTreeForest;
use crate::LPS;
use crate::Slot;
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
    /// All summands share a single global cache.
    Global,
}

/// A wrapper around an [`LPS`] that caches summand enumeration results.
///
/// The cache key for each summand is formed by projecting the state vector onto
/// the summand's read positions.
pub struct CacheLPS<P: LPS> {
    inner: Rc<P>,
    summands: Vec<CacheSummandWrapper<P>>,
}

struct CacheShared<V: Slot, L: Clone> {
    /// Stores the nodes for all the cached keys and values, ensuring that the cache is stored compactly.
    forest: RefCell<BTreeForest<V, Global, 2>>,

    /// An array of summand local caches.
    local_caches: RefCell<Vec<FxHashMap<Tree, Vec<(L, Tree)>>>>,

    /// A single global cache mapping (summand_index, key) to cached results.
    global_cache: RefCell<FxHashMap<(usize, Tree), Vec<(L, Tree)>>>,

    key_buf: RefCell<Vec<V>>,
    replay_buf: RefCell<Vec<V>>,
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

    /// Shared cache data structures.
    shared: Rc<CacheShared<P::Value, P::Label>>,

    /// Shared reference to the inner LPS for delegating cache misses.
    inner: Rc<P>,

    /// Number of enumerations served from the cache.
    hits: Cell<u64>,
    /// Number of enumerations that had to be delegated to the inner summand.
    misses: Cell<u64>,
}

impl<P: LPS> CacheLPS<P> {
    pub fn new(inner: P, strategy: CachingStrategy) -> Self {
        let inner = Rc::new(inner);
        let num_summands = inner.summands().len();
        let shared = Rc::new(CacheShared {
            forest: RefCell::new(BTreeForest::new()),
            local_caches: RefCell::new(vec![FxHashMap::default(); num_summands]),
            global_cache: RefCell::new(FxHashMap::default()),
            key_buf: RefCell::new(Vec::new()),
            replay_buf: RefCell::new(Vec::new()),
        });

        let summands: Vec<_> = inner
            .summands()
            .iter()
            .enumerate()
            .map(|(i, s)| CacheSummandWrapper {
                index: i,
                read_positions: s.read_positions().to_vec(),
                write_positions: s.write_positions().to_vec(),
                strategy,
                shared: Rc::clone(&shared),
                inner: Rc::clone(&inner),
                hits: Cell::new(0),
                misses: Cell::new(0),
            })
            .collect();

        CacheLPS { inner, summands }
    }

    /// Collects per-summand cache metrics for this [`CacheLPS`].
    ///
    /// The returned [`CacheMetrics`] implements [`fmt::Display`] for a
    /// human-readable summary of cache hits, misses and occupancy.
    ///
    /// # Cost
    ///
    /// For [`CachingStrategy::Global`] this scans the cache once per summand.
    pub fn metrics(&self) -> CacheMetrics {
        let summands = self
            .summands
            .iter()
            .map(|s| {
                let entries = match s.strategy {
                    CachingStrategy::None => 0,
                    CachingStrategy::Local => s.shared.local_caches.borrow()[s.index].len(),
                    CachingStrategy::Global => s
                        .shared
                        .global_cache
                        .borrow()
                        .keys()
                        .filter(|(index, _)| *index == s.index)
                        .count(),
                };

                SummandCacheMetrics {
                    index: s.index,
                    strategy: s.strategy,
                    hits: s.hits.get(),
                    misses: s.misses.get(),
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
                CachingStrategy::Global => "global",
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
        state: &[P::Value],
        results: &[(P::Label, Tree)],
        report: &mut impl FnMut(&P::Label, &[P::Value]) -> Result<(), MercError>,
    ) -> Result<(), MercError> {
        // Each cached tree holds only the values at the write positions.
        // The remaining positions are passed through unchanged, so they
        // must be taken from the *live* source state rather than the state
        // for which the entry was originally computed.
        let mut replay_buf = self.shared.replay_buf.borrow_mut();
        for (label, write_tree) in results {
            replay_buf.clear();
            replay_buf.extend_from_slice(state);
            for (&pos, value) in self
                .write_positions
                .iter()
                .zip(self.shared.forest.borrow().iter(*write_tree))
            {
                replay_buf[pos] = value;
            }
            report(label, &replay_buf)?;
        }
        Ok(())
    }
}

impl<P: LPS> Summand for CacheSummandWrapper<P> {
    type Value = P::Value;
    type Label = P::Label;

    fn enumerate<F>(&self, state: &[Self::Value], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[Self::Value]) -> Result<(), MercError>,
    {
        if self.strategy == CachingStrategy::None || self.read_positions.is_empty() {
            return self.inner.summands()[self.index].enumerate(state, report);
        }

        // Project state vector onto read positions to form the cache key.
        {
            let mut key_buf = self.shared.key_buf.borrow_mut();
            key_buf.clear();
            for &pos in &self.read_positions {
                key_buf.push(state[pos]);
            }
        }

        let key_tree = {
            let key_buf = self.shared.key_buf.borrow();
            self.shared.forest.borrow_mut().insert(&key_buf)
        };

        let hit = match self.strategy {
            CachingStrategy::Local => {
                let caches = self.shared.local_caches.borrow();
                if let Some(results) = caches[self.index].get(&key_tree) {
                    self.hits.set(self.hits.get() + 1);
                    self.replay_cached(state, results, &mut report)?;
                    true
                } else {
                    false
                }
            }
            CachingStrategy::Global => {
                let cache = self.shared.global_cache.borrow();
                if let Some(results) = cache.get(&(self.index, key_tree)) {
                    self.hits.set(self.hits.get() + 1);
                    self.replay_cached(state, results, &mut report)?;
                    true
                } else {
                    false
                }
            }
            CachingStrategy::None => unreachable!(),
        };

        if !hit {
            self.misses.set(self.misses.get() + 1);
            // Cache MISS: delegate to inner summand, capture results. Only the
            // values at the write positions are stored; on replay they are
            // scattered back onto the live source state (see the hit branch).
            let mut captured: Vec<(P::Label, Tree)> = Vec::new();

            self.inner.summands()[self.index].enumerate(state, |label, next_state| {
                let mut scratch = self.shared.key_buf.borrow_mut();
                scratch.clear();
                for &pos in &self.write_positions {
                    scratch.push(next_state[pos]);
                }
                let tree = self.shared.forest.borrow_mut().insert(&scratch);
                drop(scratch);
                captured.push((label.clone(), tree));
                report(label, next_state)
            })?;

            // Store results in cache.
            match self.strategy {
                CachingStrategy::Local => {
                    self.shared.local_caches.borrow_mut()[self.index].insert(key_tree, captured);
                }
                CachingStrategy::Global => {
                    self.shared
                        .global_cache
                        .borrow_mut()
                        .insert((self.index, key_tree), captured);
                }
                CachingStrategy::None => unreachable!(),
            }
        }

        Ok(())
    }

    fn read_positions(&self) -> &[usize] {
        &self.read_positions
    }

    fn write_positions(&self) -> &[usize] {
        &self.write_positions
    }
}
