use std::cell::UnsafeCell;
use std::fmt;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Instant;

use log::debug;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use merc_io::LargeFormatter;
use merc_sharedmutex::GlobalBfSharedMutex;
use merc_sharedmutex::RecursiveLockReadGuard;
use merc_unsafety::ProtectionSet;
use merc_unsafety::StablePointer;
use merc_utilities::debug_trace;

use crate::ATermIndex;
use crate::ATermRef;
use crate::Markable;
use crate::Symb;
use crate::Symbol;
use crate::SymbolIndex;
use crate::SymbolRef;
use crate::Term;
use crate::storage::ATermStorage;
use crate::storage::SharedTerm;
use crate::storage::SymbolPool;

/// This is the global set of protection sets that are managed by the [crate::storage::ThreadTermPool].
pub static GLOBAL_TERM_POOL: LazyLock<GlobalBfSharedMutex<GlobalTermPool>> =
    LazyLock::new(|| GlobalBfSharedMutex::new(GlobalTermPool::new()));

/// Enables aggressive garbage collection, which is used for testing.
pub(crate) const AGGRESSIVE_GC: bool = false;

/// A type alias for the global term pool guard
pub(crate) type GlobalTermPoolGuard<'a> = RecursiveLockReadGuard<'a, GlobalTermPool>;

/// The single global (singleton) term pool, accessed via [GLOBAL_TERM_POOL].
pub struct GlobalTermPool {
    /// Unique table of all terms with stable pointers for references
    terms: ATermStorage,
    /// The symbol pool for managing function symbols.
    symbol_pool: SymbolPool,
    /// The thread-specific protection sets.
    thread_pools: ThreadPoolList,
    /// A separate protection set for sendable terms, see [crate::ATermSend].
    send_term_protection_sets: Vec<Option<Arc<Mutex<ProtectionSet<ATermIndex>>>>>,
    /// Term roots adopted from thread-local protection sets when their owning thread exits while
    /// the terms are still reachable. Deduplicated so repeatedly leaking the same shared term
    /// (for example the default data symbols) does not grow this set unboundedly.
    orphan_term_protection_set: FxHashSet<ATermIndex>,
    /// Symbol roots adopted from thread-local protection sets during thread teardown, deduplicated.
    orphan_symbol_protection_set: FxHashSet<SymbolIndex>,
    /// Container roots adopted from thread-local protection sets during thread teardown, keyed by
    /// the container's [Arc] pointer address so the same container is retained at most once.
    orphan_container_protection_set: FxHashMap<usize, Arc<dyn Markable + Sync + Send>>,

    // Data structures used for garbage collection
    /// Used to avoid reallocations for the markings of all terms - uses pointers as keys
    marked_terms: FxHashSet<ATermIndex>,
    /// Used to avoid reallocations for the markings of all symbols
    marked_symbols: FxHashSet<SymbolIndex>,
    /// A stack used to mark terms recursively.
    stack: Vec<ATermIndex>,

    /// Indicates whether automatic garbage collection is enabled.
    garbage_collection: bool,

    /// Default terms
    int_symbol: SymbolRef<'static>,
    empty_list_symbol: SymbolRef<'static>,
    list_symbol: SymbolRef<'static>,
}

impl GlobalTermPool {
    fn new() -> GlobalTermPool {
        // Insert the default symbols, mirrors the symbols defined in mCRL2.
        // Only aterm_int has to be reserved since it behaves special with the
        // integer terms, the other two are just normal symbols.
        let mut symbol_pool = SymbolPool::new();

        // SAFETY: the default symbols are marked on every collection (see `collect_garbage`),
        // so these indices stay valid for the lifetime of the pool.
        let int_symbol = unsafe { SymbolRef::from_index(&symbol_pool.create_reserved("<aterm_int>", 0)) };
        let list_symbol = unsafe { SymbolRef::from_index(&&symbol_pool.create("<list_constructor>", 2)) };
        let empty_list_symbol = unsafe { SymbolRef::from_index(&symbol_pool.create("<empty_list>", 0)) };

        GlobalTermPool {
            terms: ATermStorage::new(),
            symbol_pool,
            thread_pools: ThreadPoolList(Vec::new()),
            send_term_protection_sets: Vec::new(),
            orphan_term_protection_set: FxHashSet::default(),
            orphan_symbol_protection_set: FxHashSet::default(),
            orphan_container_protection_set: FxHashMap::default(),
            marked_terms: FxHashSet::default(),
            marked_symbols: FxHashSet::default(),
            stack: Vec::new(),
            garbage_collection: true,
            int_symbol,
            list_symbol,
            empty_list_symbol,
        }
    }

    /// Returns the number of terms in the pool.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Returns whether the term pool is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a term storing a single integer value.
    ///
    /// Crate-private: the returned pointer is unprotected, so the caller must protect it
    /// (or hand it to a [crate::Return]) before the read guard it was created under drops.
    pub(crate) fn create_int(&self, value: usize) -> (StablePointer<SharedTerm>, bool) {
        // SAFETY: `int_symbol` is one of the default symbols, which are marked on every
        // collection, so the index stays valid for the pool's lifetime.
        let (index, inserted) = unsafe {
            self.terms
                .insert_int_term(SymbolRef::from_index(self.int_symbol.shared()), value)
        };

        (index, inserted)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    ///
    /// Crate-private: the returned pointer is unprotected, see [Self::create_int].
    pub(crate) fn create_term_array<'a, 'b, 'c, S: Symb<'a, 'b>>(
        &'c self,
        symbol: &'b S,
        args: &'c [ATermRef<'c>],
    ) -> (StablePointer<SharedTerm>, bool) {
        self.terms.insert(symbol, args)
    }

    /// Create a function symbol
    ///
    /// Crate-private: `protect` receives an unprotected index, see [Self::create_int].
    ///
    /// # Panics
    ///
    /// Panics if `name`/`arity` collide with one of the pool's own reserved
    /// marker symbols (e.g. `<aterm_int>`) — this should never happen for a
    /// legitimate caller, since those names are internal implementation details.
    pub(crate) fn create_symbol<P, N>(&self, name: N, arity: usize, protect: P) -> Symbol
    where
        P: FnOnce(SymbolIndex) -> Symbol,
        N: Into<String> + AsRef<str>,
    {
        protect(self.symbol_pool.create(name, arity))
    }

    /// Registers a new thread term pool.
    ///
    /// # Safety
    ///
    /// Note that the returned `Arc<UnsafeCell<...>>` is not Send or Sync, so it
    /// *must* be protected through other means.
    #[allow(clippy::arc_with_non_send_sync)]
    pub(crate) fn register_thread_term_pool(
        &mut self,
    ) -> (
        Arc<UnsafeCell<SharedTermProtection>>,
        Arc<Mutex<ProtectionSet<ATermIndex>>>,
    ) {
        let protection = Arc::new(UnsafeCell::new(SharedTermProtection {
            term_protection_set: ProtectionSet::new(),
            symbol_protection_set: ProtectionSet::new(),
            container_protection_set: ProtectionSet::new(),
            index: self.thread_pools.len(),
        }));

        debug!("Registered thread_local protection set(s) {}", self.thread_pools.len());
        self.thread_pools.push(Some(protection.clone()));

        let protection_set = Arc::new(Mutex::new(ProtectionSet::new()));
        self.send_term_protection_sets.push(Some(protection_set.clone()));

        (protection, protection_set)
    }

    /// Deregisters a thread pool.
    ///
    /// The `send_term_protection_sets` slot is deliberately left in place: a
    /// still-live ATermSend created on this thread may outlive it and
    /// must keep being marked.
    pub(crate) fn deregister_thread_pool(&mut self, index: usize) {
        debug!("Removed thread_local protection set(s) {index}");
        if let Some(entry) = self.thread_pools.get_mut(index) {
            *entry = None;
        }
    }

    /// Adopts the term, symbol, and container roots that were still protected in a thread-local
    /// set when its thread exited. Deduplicated, so repeatedly leaking the same shared roots does
    /// not grow the orphan sets unboundedly.
    pub(crate) fn protect_orphan_roots(&mut self, protection: &SharedTermProtection) {
        for (_root, term) in protection.term_protection_set.iter() {
            // SAFETY: the value is copied from a currently protected root; the copy is owned by
            // the orphan set, which keeps the term alive as an additional GC root.
            self.orphan_term_protection_set.insert(unsafe { term.copy() });
        }

        for (_root, symbol) in protection.symbol_protection_set.iter() {
            // SAFETY: the value is copied from a currently protected symbol root; the copy is
            // owned by the orphan set, which keeps the symbol alive as an additional GC root.
            self.orphan_symbol_protection_set.insert(unsafe { symbol.copy() });
        }

        for (_root, container) in protection.container_protection_set.iter() {
            let key = Arc::as_ptr(container) as *const () as usize;
            self.orphan_container_protection_set
                .entry(key)
                .or_insert_with(|| container.clone());
        }
    }

    /// Triggers garbage collection if necessary and returns an updated counter for the thread local pool.
    pub(crate) fn trigger_garbage_collection(&mut self) -> usize {
        if self.garbage_collection {
            // Garbage collection is enabled.
            self.collect_garbage();
        }

        if AGGRESSIVE_GC {
            return 1;
        }

        self.len()
    }

    /// Enables or disables automatic garbage collection.
    pub fn automatic_garbage_collection(&mut self, enabled: bool) {
        self.garbage_collection = enabled;
    }

    /// Collects garbage terms.
    pub fn collect_garbage(&mut self) {
        // Mark the default symbols
        // SAFETY: mark-set entries only live for the duration of this collection pass
        // (the sets are drained by the sweep below), and a marked symbol is by
        // definition retained by the sweep.
        unsafe {
            self.marked_symbols.insert(self.int_symbol.shared().copy());
            self.marked_symbols.insert(self.list_symbol.shared().copy());
            self.marked_symbols.insert(self.empty_list_symbol.shared().copy());
        }

        let mut marker = Marker {
            marked_terms: &mut self.marked_terms,
            marked_symbols: &mut self.marked_symbols,
            stack: &mut self.stack,
        };

        let mark_time = Instant::now();

        // Loop through all protection sets and mark the terms.
        for pool in self.thread_pools.iter().flatten() {
            // SAFETY: We have exclusive access to the global term pool, so no other thread can modify the protection sets.
            let pool = unsafe { &mut *pool.get() };

            for (_root, symbol) in pool.symbol_protection_set.iter() {
                debug_trace!("Marking root {_root} symbol {symbol:?}");
                // Remove all symbols that are not protected
                // SAFETY: the protection set keeps the symbol alive, and the mark-set
                // entry is dropped when the sweep below finishes.
                marker.marked_symbols.insert(unsafe { symbol.copy() });
            }

            for (_root, term) in pool.term_protection_set.iter() {
                debug_trace!("Marking root {_root} term {term:?}");
                unsafe {
                    ATermRef::from_index(term).mark(&mut marker);
                }
            }

            // Marking a `Protected` container goes through `GcMutex::lock`, which takes a
            // `read_recursive` on the same lock we already hold for writing here.
            for (_, container) in pool.container_protection_set.iter() {
                container.mark(&mut marker);
            }
        }

        for pool in self.send_term_protection_sets.iter().flatten() {
            let pool = pool.lock().expect("Lock poisoned!");
            for (_root, term) in pool.iter() {
                debug_trace!("Marking sendable term {term:?}");
                unsafe {
                    ATermRef::from_index(term).mark(&mut marker);
                }
            }
        }

        for term in &self.orphan_term_protection_set {
            debug_trace!("Marking orphaned term {term:?}");
            unsafe {
                ATermRef::from_index(term).mark(&mut marker);
            }
        }

        for symbol in &self.orphan_symbol_protection_set {
            debug_trace!("Marking orphaned symbol {symbol:?}");
            // SAFETY: the orphan set keeps the symbol alive, and the mark-set entry is dropped
            // when the sweep below finishes.
            marker.marked_symbols.insert(unsafe { symbol.copy() });
        }

        for container in self.orphan_container_protection_set.values() {
            debug_trace!("Marking orphaned container");
            container.mark(&mut marker);
        }

        // Reclaim send-term protection sets whose owning thread has exited and that hold no
        // outstanding ATermSend.
        for slot in self.send_term_protection_sets.iter_mut() {
            if slot.as_ref().is_some_and(|set| Arc::strong_count(set) == 1) {
                *slot = None;
            }
        }

        let mark_time_elapsed = mark_time.elapsed();
        let collect_time = Instant::now();

        let num_of_terms = self.len();
        let num_of_symbols = self.symbol_pool.len();

        // Delete all terms that are not marked.
        // SAFETY: Marking visited every root in every protection set while holding the exclusive
        // lock, so unmarked terms have no live references that could be dereferenced afterwards.
        unsafe {
            self.terms.retain(|term| {
                if !self.marked_terms.contains(term) {
                    debug_trace!("Dropping term: {:?}", term);
                    return false;
                }

                true
            });
        }

        // We ensure that every removed symbol is not used anymore.
        // SAFETY: Unmarked symbols are not referenced by any marked term or protection set root.
        unsafe {
            self.symbol_pool.retain(|symbol| {
                if !self.marked_symbols.contains(symbol) {
                    debug_trace!("Dropping symbol: {:?}", symbol);
                    return false;
                }

                true
            });
        }

        debug!(
            "Garbage collection: marking took {}ms, collection took {}ms, {} terms and {} symbols removed",
            mark_time_elapsed.as_millis(),
            collect_time.elapsed().as_millis(),
            num_of_terms - self.len(),
            num_of_symbols - self.symbol_pool.len()
        );

        debug!("{}", self.metrics());

        // Print information from the protection sets.
        for pool in self.thread_pools.iter().flatten() {
            // SAFETY: We have exclusive access to the global term pool, so no other thread can modify the protection sets.
            let pool = unsafe { &mut *pool.get() };
            debug!("{}", pool.metrics());
        }

        // Clear marking data structures
        self.marked_terms.clear();
        self.marked_symbols.clear();
        self.stack.clear();
    }

    /// Returns the metrics of the term pool, can be formatted and written to output.
    pub fn metrics(&self) -> TermPoolMetrics<'_> {
        TermPoolMetrics(self)
    }

    /// Marks the given term as being reachable.
    ///
    /// # Safety
    ///
    /// Should only be called during garbage collection.
    pub unsafe fn mark_term(&mut self, term: &ATermRef<'_>) {
        // Ensure that the global term pool is locked for writing.
        let mut marker = Marker {
            marked_terms: &mut self.marked_terms,
            marked_symbols: &mut self.marked_symbols,
            stack: &mut self.stack,
        };
        term.mark(&mut marker);
    }

    /// Returns integer function symbol.
    pub(crate) fn get_int_symbol(&self) -> &SymbolRef<'static> {
        &self.int_symbol
    }

    /// Returns integer function symbol.
    pub(crate) fn get_list_symbol(&self) -> &SymbolRef<'static> {
        &self.list_symbol
    }

    /// Returns integer function symbol.
    pub(crate) fn get_empty_list_symbol(&self) -> &SymbolRef<'static> {
        &self.empty_list_symbol
    }
}

pub struct TermPoolMetrics<'a>(&'a GlobalTermPool);

impl fmt::Display for TermPoolMetrics<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "There are {} terms, and {} symbols",
            self.0.terms.len(),
            self.0.symbol_pool.len()
        )
    }
}

/// A newtype wrapping the per-thread protection-set list stored inside
/// [`GlobalTermPool`].
///
/// # Safety
///
/// Note that [`UnsafeCell`] is not [`Sync`], but we explicitly only use this in
/// `&mut self` contexts, so we can safely implement `Sync` for this wrapper.
struct ThreadPoolList(Vec<Option<Arc<UnsafeCell<SharedTermProtection>>>>);

// SAFETY: See the safety documentation on `ThreadPoolList`.
unsafe impl Sync for ThreadPoolList {}
unsafe impl Send for ThreadPoolList {}

impl std::ops::Deref for ThreadPoolList {
    type Target = Vec<Option<Arc<UnsafeCell<SharedTermProtection>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ThreadPoolList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A struct that contains the protection sets for a thread, as well as the
/// index of the thread pool in the global term pool.
pub struct SharedTermProtection {
    /// Protection set for terms
    pub term_protection_set: ProtectionSet<ATermIndex>,
    /// Protection set to prevent garbage collection of symbols
    pub symbol_protection_set: ProtectionSet<SymbolIndex>,
    /// Protection set for containers
    pub container_protection_set: ProtectionSet<Arc<dyn Markable + Sync + Send>>,
    /// Index in global pool's thread pools list
    pub index: usize,
}

impl SharedTermProtection {
    /// Returns the metrics of the term pool, can be formatted and written to output.
    pub fn metrics(&self) -> ProtectionMetrics<'_> {
        ProtectionMetrics(self)
    }
}

/// A struct that can be used to print the performance of the protection sets.
pub struct ProtectionMetrics<'a>(&'a SharedTermProtection);

impl fmt::Display for ProtectionMetrics<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Protection set {} has {} roots, max {} and {} insertions",
            self.0.index,
            LargeFormatter(self.0.term_protection_set.len()),
            LargeFormatter(self.0.term_protection_set.maximum_size()),
            LargeFormatter(self.0.term_protection_set.number_of_insertions())
        )?;

        writeln!(
            f,
            "Containers: {} roots, max {} and {} insertions",
            LargeFormatter(self.0.container_protection_set.len()),
            LargeFormatter(self.0.container_protection_set.maximum_size()),
            LargeFormatter(self.0.container_protection_set.number_of_insertions()),
        )?;

        write!(
            f,
            "Symbols: {} roots, max {} and {} insertions",
            LargeFormatter(self.0.symbol_protection_set.len()),
            LargeFormatter(self.0.symbol_protection_set.maximum_size()),
            LargeFormatter(self.0.symbol_protection_set.number_of_insertions()),
        )
    }
}

/// Helper struct to pass private data required to mark term recursively.
pub struct Marker<'a> {
    marked_terms: &'a mut FxHashSet<ATermIndex>,
    marked_symbols: &'a mut FxHashSet<SymbolIndex>,
    stack: &'a mut Vec<ATermIndex>,
}

impl Marker<'_> {
    // Marks the given term as being reachable.
    pub fn mark(&mut self, term: &ATermRef<'_>) {
        // SAFETY: all copies below go into the mark sets and the work stack, which are
        // drained before the collection pass ends, and a marked term or symbol is by
        // definition retained by the sweep; `term` itself is alive for the borrow.
        unsafe {
            if !self.marked_terms.contains(term.shared()) {
                self.stack.push(term.shared().copy());

                while let Some(term) = self.stack.pop() {
                    // Each term should be marked.
                    self.marked_terms.insert(term.copy());

                    // Reconstruct the wide reference once; each `deref` rebuilds it from the
                    // symbol header, so we reuse the borrow for both the symbol and arguments.
                    let shared = term.deref();

                    // Mark the function symbol.
                    self.marked_symbols.insert(shared.symbol().shared().copy());

                    // The arguments slice already has exactly the symbol's arity as its length
                    // (integer terms reconstruct to an empty slice), so iterating it is correct.
                    for arg in shared.arguments().iter() {
                        // Skip if unnecessary, otherwise mark before pushing to stack since it can be shared.
                        if !self.marked_terms.contains(arg.shared()) {
                            self.marked_terms.insert(arg.shared().copy());
                            self.marked_symbols.insert(arg.get_head_symbol().shared().copy());
                            self.stack.push(arg.shared().copy());
                        }
                    }
                }
            }
        }
    }

    /// Marks the given symbol as being reachable.
    pub fn mark_symbol(&mut self, symbol: &SymbolRef<'_>) {
        // SAFETY: see `mark`; the entry is dropped when the sweep finishes and a marked
        // symbol is retained by it.
        self.marked_symbols.insert(unsafe { symbol.shared().copy() });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use merc_utilities::random_test;

    use crate::ATerm;
    use crate::Symbol;
    use crate::Term;
    use crate::random_term;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_maximal_sharing() {
        random_test(100, |rng| {
            let mut terms = HashMap::new();

            for _ in 0..1000 {
                let term = random_term(rng, &[("f".into(), 2), ("g".into(), 1)], &["a".to_string()], 10);

                let representation = format!("{}", term);
                if let Some(entry) = terms.get(&representation) {
                    assert_eq!(term, *entry, "There is another term with the same representation");
                } else {
                    terms.insert(representation, term);
                }
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_term_out_of_bound_arity() {
        let c = ATerm::constant(&Symbol::new("a", 0));

        let t = ATerm::with_args(&Symbol::new("f", 1), &[c.copy(), c.copy()]);

        // Currently we check on access
        let _ = t.arg(1);
    }
}
