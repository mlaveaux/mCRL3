use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use hashbrown::HashMap;
use merc_utilities::MercError;

use crate::BTreeForest;
use crate::LPS;
use crate::Slot;
use crate::Summand;
use crate::Tree;

/// Controls the enumeration caching behaviour of [`CacheLPS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum CachingStrategy {
    /// No caching; every enumeration is performed from scratch.
    None,
    /// Each summand maintains its own independent cache.
    Local,
    /// All summands share a single global cache keyed by (summand_index, key).
    Global,
}

/// A wrapper around an [`LPS`] that caches summand enumeration results.
///
/// The cache key for each summand is formed by projecting the state vector
/// onto the summand's [`Summand::read_positions`]. Keys and next-state vectors
/// are interned in a [`BTreeForest`] for compact, hash-consed storage.
pub struct CacheLPS<P: LPS> {
    inner: P,
    summands: Vec<CacheSummandWrapper<P>>,
}

struct CacheShared<V: Slot, L: Clone> {
    forest: RefCell<BTreeForest<V>>,
    local_caches: RefCell<Vec<HashMap<Tree, Vec<(L, Tree)>>>>,
    global_cache: RefCell<HashMap<(usize, Tree), Vec<(L, Tree)>>>,
    key_buf: RefCell<Vec<V>>,
    replay_buf: RefCell<Vec<V>>,
}

/// Thin metadata wrapper for a single summand in a [`CacheLPS`].
pub struct CacheSummandWrapper<P: LPS> {
    index: usize,
    read_positions: Vec<usize>,
    strategy: CachingStrategy,
    shared: Rc<CacheShared<P::Value, P::Label>>,
    _marker: PhantomData<P>,
}

/// Exploration context for [`CacheLPS`], wrapping the inner LPS context.
pub struct CacheLPSContext<P: LPS> {
    inner_context: P::Context,
    // SAFETY: valid for the entire `explore()` borrow scope because CacheLPS
    // owns `inner` and is not moved while exploration is in progress.
    inner_summands: *const [P::Summand],
}

impl<P: LPS> CacheLPS<P> {
    pub fn new(inner: P, strategy: CachingStrategy) -> Self {
        let num_summands = inner.summands().len();
        let shared = Rc::new(CacheShared {
            forest: RefCell::new(BTreeForest::new()),
            local_caches: RefCell::new(vec![HashMap::new(); num_summands]),
            global_cache: RefCell::new(HashMap::new()),
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
                strategy,
                shared: Rc::clone(&shared),
                _marker: PhantomData,
            })
            .collect();

        CacheLPS { inner, summands }
    }
}

impl<P: LPS> LPS for CacheLPS<P> {
    type Value = P::Value;
    type Label = P::Label;
    type Context = CacheLPSContext<P>;
    type Summand = CacheSummandWrapper<P>;

    fn initial_state(&self) -> Vec<Self::Value> {
        self.inner.initial_state()
    }

    fn summands(&self) -> &[Self::Summand] {
        &self.summands
    }

    fn create_context(&self) -> Self::Context {
        CacheLPSContext {
            inner_context: self.inner.create_context(),
            inner_summands: self.inner.summands() as *const [P::Summand],
        }
    }

    fn prepare_context(&self, state: &[Self::Value], context: &mut Self::Context) {
        context.inner_summands = self.inner.summands() as *const [P::Summand];
        self.inner.prepare_context(state, &mut context.inner_context);
    }
}

impl<P: LPS> Summand for CacheSummandWrapper<P> {
    type Value = P::Value;
    type Label = P::Label;
    type Context = CacheLPSContext<P>;

    fn enumerate<F>(&self, state: &[Self::Value], context: &mut Self::Context, mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[Self::Value]) -> Result<(), MercError>,
    {
        // SAFETY: pointer is valid for the duration of exploration.
        let inner_summands = unsafe { &*context.inner_summands };

        if self.strategy == CachingStrategy::None || self.read_positions.is_empty() {
            return inner_summands[self.index].enumerate(state, &mut context.inner_context, report);
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
            self.shared.forest.borrow_mut().build(&key_buf)
        };

        // Look up in the appropriate cache.
        let cached = match self.strategy {
            CachingStrategy::Local => {
                let caches = self.shared.local_caches.borrow();
                caches[self.index].get(&key_tree).is_some()
            }
            CachingStrategy::Global => {
                let cache = self.shared.global_cache.borrow();
                cache.get(&(self.index, key_tree)).is_some()
            }
            CachingStrategy::None => unreachable!(),
        };

        if cached {
            // Cache HIT: replay stored results.
            let results = match self.strategy {
                CachingStrategy::Local => {
                    let caches = self.shared.local_caches.borrow();
                    caches[self.index].get(&key_tree).unwrap().clone()
                }
                CachingStrategy::Global => {
                    let cache = self.shared.global_cache.borrow();
                    cache.get(&(self.index, key_tree)).unwrap().clone()
                }
                CachingStrategy::None => unreachable!(),
            };

            let mut replay_buf = self.shared.replay_buf.borrow_mut();
            for (label, state_tree) in &results {
                replay_buf.clear();
                replay_buf.extend(self.shared.forest.borrow().iter(*state_tree));
                report(label, &replay_buf)?;
            }
        } else {
            // Cache MISS: delegate to inner summand, capture results.
            let mut captured: Vec<(P::Label, Tree)> = Vec::new();

            inner_summands[self.index].enumerate(state, &mut context.inner_context, |label, next_state| {
                let tree = self.shared.forest.borrow_mut().build(next_state);
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
}
