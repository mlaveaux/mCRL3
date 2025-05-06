use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::LazyLock;

use hashbrown::Equivalent;
use log::info;
use log::trace;
use mcrl3_unsafety::StablePointerSet;
use mcrl3_utilities::SimpleTimer;

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

use mcrl3_utilities::ProtectionSet;

use crate::ATerm;
use crate::ATermIndex;
use crate::ATermRef;
use crate::Markable;
use crate::Symb;
use crate::Symbol;
use crate::SymbolIndex;
use crate::SymbolPool;
use crate::SymbolRef;
use crate::THREAD_TERM_POOL;
use crate::Term;

/// This is the global set of protection sets that are managed by the ThreadTermPool
pub(crate) static GLOBAL_TERM_POOL: LazyLock<ReentrantMutex<RefCell<GlobalTermPool>>> =
    LazyLock::new(|| ReentrantMutex::new(RefCell::new(GlobalTermPool::new())));

/// Enables aggressive garbage collection, which is used for testing.
pub(crate) const AGRESSIVE_GC: bool = false;

/// A type alias for the global term pool guard
pub(crate) type GlobalTermPoolGuard<'a> = ReentrantMutexGuard<'a, RefCell<GlobalTermPool>>;

/// The single global (singleton) term pool.
pub(crate) struct GlobalTermPool {
    /// Unique table of all terms with stable pointers for references
    terms: StablePointerSet<SharedTerm>,
    /// The symbol pool for managing function symbols.
    symbol_pool: SymbolPool,
    /// The thread-specific protection sets.
    thread_pools: Vec<Option<Arc<Mutex<SharedTermProtection>>>>,

    /// A vector of terms that are used to store the arguments of a term for loopup.
    tmp_arguments: Vec<ATermRef<'static>>,

    // Data structures used for garbage collection
    /// Used to avoid reallocations for the markings of all terms - uses pointers as keys
    marked_terms: HashSet<ATermIndex>,
    /// Used to avoid reallocations for the markings of all symbols
    marked_symbols: HashSet<SymbolIndex>,
    /// A stack used to mark terms recursively.
    stack: Vec<ATermIndex>,

    /// Default terms
    int_symbol: SymbolRef<'static>,
    empty_list_symbol: SymbolRef<'static>,
    list_symbol: SymbolRef<'static>,
}

impl GlobalTermPool {
    fn new() -> GlobalTermPool {
        // Insert the default symbols.
        let mut symbol_pool = SymbolPool::new();
        let int_symbol = symbol_pool.create("Int", 0, |index| unsafe { SymbolRef::from_index(&index) });
        let list_symbol = symbol_pool.create("List", 2, |index| unsafe { SymbolRef::from_index(&index) });
        let empty_list_symbol = symbol_pool.create("[]", 0, |index| unsafe { SymbolRef::from_index(&index) });

        GlobalTermPool {
            terms: StablePointerSet::new(),
            symbol_pool,
            thread_pools: Vec::new(),
            tmp_arguments: Vec::new(),
            marked_terms: HashSet::new(),
            marked_symbols: HashSet::new(),
            stack: Vec::new(),
            int_symbol,
            list_symbol,
            empty_list_symbol,
        }
    }

    /// Returns the number of terms in the pool.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Returns the capacity of terms in the pool.
    pub fn capacity(&self) -> usize {
        self.terms.capacity()
    }

    /// Creates a term storing a single integer value.
    pub fn create_int<P>(&mut self, value: usize, protect: P) -> ATerm
    where
        P: FnOnce(&mut GlobalTermPool, &ATermIndex, bool) -> ATerm,
    {
        // We abuse the first argument of the term to store the integer value.
        self.tmp_arguments.clear();
        unsafe {
            self.tmp_arguments
                .push(ATermRef::from_index(&ATermIndex::from_index(NonZero::new_unchecked(
                    value + 1,
                ))));
        }

        let shared_term = SharedTermLookup {
            symbol: unsafe { SymbolRef::from_index(self.int_symbol.shared()) },
            arguments: &self.tmp_arguments,
        };

        let (index, inserted) = self.terms.insert_equiv(&shared_term);
        protect(self, &index, inserted)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term_iter<'a, 'b, 'c, 'd, I, T, P>(
        &mut self,
        symbol: &'b impl Symb<'a, 'b>,
        args: I,
        protect: P,
    ) -> ATerm
    where
        I: IntoIterator<Item = T>,
        T: Term<'c, 'd>,
        P: FnOnce(&mut GlobalTermPool, &ATermIndex, bool) -> ATerm,
    {
        self.tmp_arguments.clear();
        for arg in args {
            unsafe {
                self.tmp_arguments.push(ATermRef::from_index(arg.shared()));
            }
        }

        let shared_term = SharedTermLookup {
            symbol: SymbolRef::from_symbol(symbol),
            arguments: &self.tmp_arguments,
        };

        debug_assert_eq!(
            symbol.arity(),
            shared_term.arguments.len(),
            "The number of arguments does not match the arity of the symbol"
        );

        let (index, inserted) = self.terms.insert_equiv(&shared_term);
        protect(self, &index, inserted)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term<'a, 'b, P>(
        &mut self,
        symbol: &'b impl Symb<'a, 'b>,
        args: &'b [impl Term<'a, 'b>],
        protect: P,
    ) -> ATerm
    where
        P: FnOnce(&mut GlobalTermPool, &ATermIndex, bool) -> ATerm,
    {
        self.tmp_arguments.clear();
        for arg in args {
            unsafe {
                self.tmp_arguments.push(ATermRef::from_index(arg.shared()));
            }
        }

        let shared_term = SharedTermLookup {
            symbol: SymbolRef::from_symbol(symbol),
            arguments: &self.tmp_arguments,
        };

        debug_assert_eq!(
            symbol.shared().arity(),
            shared_term.arguments.len(),
            "The number of arguments does not match the arity of the symbol"
        );

        let (index, inserted) = self.terms.insert_equiv(&shared_term);
        protect(self, &index, inserted)
    }

    /// Create a function symbol
    pub fn create_symbol<P>(&mut self, name: impl Into<String> + AsRef<str>, arity: usize, protect: P) -> Symbol
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
        info!("Removed thread_local protection set(s) {}", index);
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

            for (root, symbol) in pool.symbol_protection_set.iter() {
                trace!("Marking root {root} symbol {symbol:?}");
                // Remove all symbols that are not protected
                marker.marked_symbols.insert(symbol.copy());
            }

            for (root, term) in pool.protection_set.iter() {
                trace!("Marking root {root} term {term:?}");
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
                trace!("Dropping term: {:?}", term);
                return false;
            }

            true
        });

        // We ensure that every removed symbol is not used anymore.
        self.symbol_pool.retain(|symbol| {
            if !self.marked_symbols.contains(symbol) {
                trace!("Dropping symbol: {:?}", symbol);
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

        info!("{:?}", self);

        // Print information from the protection sets.
        for pool in self.thread_pools.iter().flatten() {
            let pool = pool.lock();
            info!("{:?}", pool);
        }
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

impl fmt::Debug for GlobalTermPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "There are {} terms, and {} symbols",
            self.terms.len(),
            self.symbol_pool.len()
        )
    }
}

pub(crate) struct SharedTermProtection {
    /// Protection set for terms
    pub(crate) protection_set: ProtectionSet<ATermIndex>,
    /// Protection set to prevent garbage collection of symbols
    pub(crate) symbol_protection_set: ProtectionSet<SymbolIndex>,
    /// Protection set for containers
    pub(crate) container_protection_set: ProtectionSet<Arc<dyn Markable + Sync + Send>>,
    /// Index in global pool's thread pools list
    pub(crate) index: usize,
}

impl fmt::Debug for SharedTermProtection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Protection set {} has {} roots, max {} and {} insertions",
            self.index,
            self.protection_set.len(),
            self.protection_set.maximum_size(),
            self.protection_set.number_of_insertions()
        )?;

        writeln!(
            f,
            "Containers: {} roots, max {} and {} insertions",
            self.container_protection_set.len(),
            self.container_protection_set.maximum_size(),
            self.container_protection_set.number_of_insertions(),
        )?;

        write!(
            f,
            "Symbols: {} roots, max {} and {} insertions",
            self.symbol_protection_set.len(),
            self.symbol_protection_set.maximum_size(),
            self.symbol_protection_set.number_of_insertions(),
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

/// The underlying type of terms that are actually shared.
#[derive(Eq, PartialEq)]
pub struct SharedTerm {
    symbol: SymbolRef<'static>,
    arguments: Vec<ATermRef<'static>>,
}

impl fmt::Debug for SharedTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        THREAD_TERM_POOL.with_borrow(|tp| {
            // This is not very nice, but we cannot print the argument of an integer term.
            if *tp.int_symbol() == self.symbol {
                write!(f, "Int({})", self.arguments[0].shared().address() - 1)
            } else {
                write!(
                    f,
                    "SharedTerm {{ symbol: {:?}, arguments: {:?} }}",
                    self.symbol, self.arguments
                )
            }
        })
    }
}

impl Clone for SharedTerm {
    fn clone(&self) -> Self {
        SharedTerm {
            symbol: unsafe { SymbolRef::from_index(self.symbol.shared()) },
            arguments: self
                .arguments
                .iter()
                .map(|x| unsafe { ATermRef::from_index(x.shared()) })
                .collect(),
        }
    }
}

impl SharedTerm {
    pub fn symbol(&self) -> &SymbolRef<'_> {
        &self.symbol
    }

    pub fn arguments(&self) -> &[ATermRef<'static>] {
        &self.arguments
    }
}

/// A cheap reference to the elements of a shared term that can be used for
/// lookup of terms without allocating.
struct SharedTermLookup<'a> {
    symbol: SymbolRef<'a>,
    arguments: &'a [ATermRef<'a>],
}

impl From<&SharedTermLookup<'_>> for SharedTerm {
    fn from(lookup: &SharedTermLookup<'_>) -> Self {
        SharedTerm {
            symbol: unsafe { SymbolRef::from_index(lookup.symbol.shared()) },
            arguments: lookup
                .arguments
                .iter()
                .map(|x| unsafe { ATermRef::from_index(x.shared()) })
                .collect(),
        }
    }
}

impl Equivalent<SharedTerm> for SharedTermLookup<'_> {
    fn equivalent(&self, other: &SharedTerm) -> bool {
        self.symbol == other.symbol && self.arguments == other.arguments
    }
}

impl Hash for SharedTermLookup<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.arguments.hash(state);
    }
}

impl Hash for SharedTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.arguments.hash(state);
    }
}
