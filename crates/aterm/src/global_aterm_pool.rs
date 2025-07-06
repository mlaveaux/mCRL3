use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use std::sync::LazyLock;

use log::info;
use mcrl3_utilities::LargeFormatter;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use rustc_hash::FxBuildHasher;

use mcrl3_unsafety::StablePointerSet;
use mcrl3_utilities::debug_trace;
use mcrl3_utilities::ProtectionSet;
use mcrl3_utilities::SimpleTimer;

use crate::ATerm;
use crate::ATermIndex;
use crate::ATermRef;
use crate::Markable;
use crate::SharedTerm;
use crate::SharedTermLookup;
use crate::Symb;
use crate::Symbol;
use crate::SymbolIndex;
use crate::SymbolPool;
use crate::SymbolRef;
use crate::Term;

#[cfg(not(feature = "mcrl3_miri"))]
mod mutex {
    pub use parking_lot::Mutex;
    pub use parking_lot::MutexGuard;
    pub use parking_lot::ReentrantMutex;
    pub use parking_lot::ReentrantMutexGuard;

    /// Helper function used to unwrap the mutex guard
    pub fn mutex_unwrap<T>(guard: MutexGuard<'_, T>) -> MutexGuard<'_, T> {
        guard
    }
}

#[cfg(feature = "mcrl3_miri")]
mod mutex {
    pub use std::sync::Mutex;
    pub use std::sync::MutexGuard;
    pub use std::sync::ReentrantLock as ReentrantMutex;
    pub use std::sync::ReentrantLockGuard as ReentrantMutexGuard;

    pub fn mutex_unwrap<'a, T, E: std::fmt::Debug>(guard: Result<MutexGuard<'a, T>, E>) -> MutexGuard<'a, T> {
        guard.expect("Mutex lock has been poisoned")
    }
}

// Depends on the selection of the feature, the mutex type is either a parking_lot or std mutex.
pub use mutex::*;

/// This is the global set of protection sets that are managed by the ThreadTermPool
pub static GLOBAL_TERM_POOL: LazyLock<RwLock<GlobalTermPool>> =
    LazyLock::new(|| RwLock::new(GlobalTermPool::new()));

/// Enables aggressive garbage collection, which is used for testing.
pub(crate) const AGRESSIVE_GC: bool = false;

/// A type alias for the global term pool guard
pub(crate) type GlobalTermPoolGuard<'a> = RwLockReadGuard<'a, GlobalTermPool>;

/// The single global (singleton) term pool.
pub struct GlobalTermPool {
    /// Unique table of all terms with stable pointers for references
    terms: StablePointerSet<SharedTerm, FxBuildHasher>,
    /// The symbol pool for managing function symbols.
    symbol_pool: SymbolPool,
    /// The thread-specific protection sets.
    thread_pools: Vec<Option<Arc<Mutex<SharedTermProtection>>>>,

    // Data structures used for garbage collection
    /// Used to avoid reallocations for the markings of all terms - uses pointers as keys
    marked_terms: HashSet<ATermIndex>,
    /// Used to avoid reallocations for the markings of all symbols
    marked_symbols: HashSet<SymbolIndex>,
    /// A stack used to mark terms recursively.
    stack: Vec<ATermIndex>,

    /// Deletion hooks called whenever a term with the given head symbol is deleted.
    deletion_hooks: Vec<(Symbol, Box<dyn Fn(&ATermIndex) + Sync + Send>)>,

    /// Default terms
    int_symbol: SymbolRef<'static>,
    empty_list_symbol: SymbolRef<'static>,
    list_symbol: SymbolRef<'static>,
}

impl GlobalTermPool {
    fn new() -> GlobalTermPool {
        // Insert the default symbols.
        let symbol_pool = SymbolPool::new();
        let int_symbol = symbol_pool.create("Int", 0, |index| unsafe { SymbolRef::from_index(&index) });
        let list_symbol = symbol_pool.create("List", 2, |index| unsafe { SymbolRef::from_index(&index) });
        let empty_list_symbol = symbol_pool.create("[]", 0, |index| unsafe { SymbolRef::from_index(&index) });

        GlobalTermPool {
            terms: StablePointerSet::with_hasher(FxBuildHasher),
            symbol_pool,
            thread_pools: Vec::new(),
            marked_terms: HashSet::new(),
            marked_symbols: HashSet::new(),
            stack: Vec::new(),
            deletion_hooks: Vec::new(),
            int_symbol,
            list_symbol,
            empty_list_symbol,
        }
    }

    /// Returns the number of terms in the pool.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Creates a term storing a single integer value.
    pub fn create_int<P>(&self, value: usize, protect: P) -> (ATerm, bool)
    where
        P: FnOnce(&ATermIndex) -> ATerm,
    {
        let shared_term = SharedTermLookup {
            symbol: unsafe { SymbolRef::from_index(self.int_symbol.shared()) },
            arguments: &[],
            annotation: Some(value),
        };

        let (index, inserted) =
            self.terms
                .insert_equiv_dst(&shared_term, SharedTerm::length_for(&shared_term), |ptr, key| unsafe {
                    SharedTerm::construct(ptr, key)
                });

        (protect(&index), inserted)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term_array<'a, 'b, 'c, P>(
        &'c self,
        symbol: &'b impl Symb<'a, 'b>,
        args: &'c [ATermRef<'c>],
        protect: P,
    ) -> (ATerm, bool)
    where
        P: FnOnce(&ATermIndex) -> ATerm,
    {
        let shared_term = SharedTermLookup {
            symbol: SymbolRef::from_symbol(symbol),
            arguments: args,
            annotation: None,
        };

        debug_assert_eq!(
            symbol.shared().arity(),
            shared_term.arguments.len(),
            "The number of arguments does not match the arity of the symbol"
        );

        let (index, inserted) =
            self.terms
                .insert_equiv_dst(&shared_term, SharedTerm::length_for(&shared_term), |ptr, key| unsafe {
                    SharedTerm::construct(ptr, key)
                });

        (protect(&index), inserted)
    }

    /// Create a function symbol
    pub fn create_symbol<P>(&self, name: impl Into<String> + AsRef<str>, arity: usize, protect: P) -> Symbol
    where
        P: FnOnce(SymbolIndex) -> Symbol,
    {
        self.symbol_pool.create(name, arity, protect)
    }

    /// Registers a new thread term pool.
    pub(crate) fn register_thread_term_pool(&mut self) -> Arc<Mutex<SharedTermProtection>> {
        let protection = Arc::new(Mutex::new(SharedTermProtection {
            protection_set: ProtectionSet::new(),
            symbol_protection_set: ProtectionSet::new(),
            container_protection_set: ProtectionSet::new(),
            index: self.thread_pools.len(),
        }));

        info!("Registered thread_local protection set(s) {}", self.thread_pools.len());
        self.thread_pools.push(Some(protection.clone()));

        protection
    }

    /// Deregisters a thread pool.
    pub(crate) fn deregister_thread_pool(&mut self, index: usize) {
        info!("Removed thread_local protection set(s) {index}");
        if let Some(entry) = self.thread_pools.get_mut(index) {
            *entry = None;
        }
    }

    /// Triggers garbage collection if necessary and returns an updated counter for the thread local pool.
    pub(crate) fn trigger_garbage_collection(&mut self) -> usize {
        self.collect_garbage();

        if AGRESSIVE_GC {
            return 1;
        }

        self.len()
    }

    /// Register a deletion hook that is called whenever a term is deleted with the given symbol.
    pub fn register_deletion_hook<F>(&mut self, symbol: SymbolRef<'static>, hook: F) 
        where F: Fn(&ATermIndex) + Sync + Send + 'static 
    {
        self.deletion_hooks.push((symbol.protect(), Box::new(hook)));
    }

    /// Collects garbage terms.
    fn collect_garbage(&mut self) {
        // Clear marking data structures
        self.marked_terms.clear();
        self.marked_symbols.clear();
        self.stack.clear();

        // Mark the default symbols
        self.marked_symbols.insert(self.int_symbol.shared().copy());
        self.marked_symbols.insert(self.list_symbol.shared().copy());
        self.marked_symbols.insert(self.empty_list_symbol.shared().copy());

        let mut marker = Marker {
            marked_terms: &mut self.marked_terms,
            marked_symbols: &mut self.marked_symbols,
            stack: &mut self.stack,
        };

        let mark_time = SimpleTimer::new();

        // Loop through all protection sets and mark the terms.
        for pool in self.thread_pools.iter().flatten() {
            let pool = mutex_unwrap(pool.lock());

            for (_root, symbol) in pool.symbol_protection_set.iter() {
                debug_trace!("Marking root {_root} symbol {symbol:?}");
                // Remove all symbols that are not protected
                marker.marked_symbols.insert(symbol.copy());
            }

            for (_root, term) in pool.protection_set.iter() {
                debug_trace!("Marking root {_root} term {term:?}");
                unsafe {
                    ATermRef::from_index(term).mark(&mut marker);
                }
            }

            for (_, container) in pool.container_protection_set.iter() {
                container.mark(&mut marker);
            }
        }

        let collect_time = SimpleTimer::new();

        let num_of_terms = self.len();
        let num_of_symbols = self.symbol_pool.len();

        // Delete all terms that are not marked
        self.terms.retain(|term| {
            if !self.marked_terms.contains(term) {
                debug_trace!("Dropping term: {:?}", term);

                // Call the deletion hooks for the term
                for (symbol, hook) in &self.deletion_hooks {
                    if symbol == term.symbol() {
                        debug_trace!("Calling deletion hook for term: {:?}", term);
                        hook(term);
                    }
                }

                return false;
            }

            true
        });

        // We ensure that every removed symbol is not used anymore.
        self.symbol_pool.retain(|symbol| {
            if !self.marked_symbols.contains(symbol) {
                debug_trace!("Dropping symbol: {:?}", symbol);
                return false;
            }

            true
        });

        info!(
            "Garbage collection: marking took {}ms, collection took {}ms, {} terms and {} symbols removed",
            mark_time.elapsed().as_millis(),
            collect_time.elapsed().as_millis(),
            num_of_terms - self.len(),
            num_of_symbols - self.symbol_pool.len()
        );

        info!("{}", self.metrics());

        // Print information from the protection sets.
        for pool in self.thread_pools.iter().flatten() {
            let pool = pool.lock();
            info!("{}", pool.metrics());
        }
    }

    /// Returns the metrics of the term pool, can be formatted and written to output.
    pub fn metrics(&self) -> TermPoolMetrics<'_> {
        TermPoolMetrics(self)
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

pub struct SharedTermProtection {
    /// Protection set for terms
    pub protection_set: ProtectionSet<ATermIndex>,
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
            LargeFormatter(self.0.protection_set.len()),
            LargeFormatter(self.0.protection_set.maximum_size()),
            LargeFormatter(self.0.protection_set.number_of_insertions())
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
    marked_terms: &'a mut HashSet<ATermIndex>,
    marked_symbols: &'a mut HashSet<SymbolIndex>,
    stack: &'a mut Vec<ATermIndex>,
}

impl Marker<'_> {
    // Marks the given term as being reachable.
    pub fn mark(&mut self, term: &ATermRef<'_>) {
        if !self.marked_terms.contains(term.shared()) {
            self.stack.push(term.shared().copy());

            while let Some(term) = self.stack.pop() {
                // Each term should be marked.
                self.marked_terms.insert(term.copy());

                // Mark the function symbol.
                self.marked_symbols.insert(term.symbol().shared().copy());

                // For some terms, such as ATermInt, we must ONLY consider the valid arguments (indicated by the arity)
                for arg in term.arguments()[0..term.symbol().arity()].iter() {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mcrl3_utilities::random_test;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;

    use crate::random_term;
    use crate::Term;

    #[test]
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
    fn test_parallel_iterator() {
        let mut rng = rand::rng();

        let mut terms = Vec::new();
        terms.resize_with(100, || {
            random_term(&mut rng, &[("f".into(), 2), ("g".into(), 1)], &["a".to_string()], 10)
        });

        let total: usize = terms.par_iter().fold(|| 0, |value, t| value + t.iter().count()).sum();

        println!("{:?}", total);
    }
}
