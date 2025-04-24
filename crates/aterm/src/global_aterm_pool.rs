use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::LazyLock;

use hashbrown::Equivalent;
use log::info;
use log::trace;
use mcrl3_utilities::SimpleTimer;

#[cfg(not(feature = "mcrl3_miri"))]
mod mutex {
    pub use parking_lot::Mutex;
    pub use parking_lot::MutexGuard;
    pub use parking_lot::ReentrantMutex;
    pub use parking_lot::ReentrantMutexGuard;

    /// Helper function used to unwrap the mutex guard
    pub fn mutex_unwrap<'a, T>(guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
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

use mcrl3_utilities::IndexedSet;
use mcrl3_utilities::ProtectionSet;

use crate::ATerm;
use crate::ATermRef;
use crate::Markable;
use crate::Symb;
use crate::Symbol;
use crate::SymbolPool;
use crate::SymbolRef;
use crate::Term;

/// This is the global set of protection sets that are managed by the ThreadTermPool
pub(crate) static GLOBAL_TERM_POOL: LazyLock<ReentrantMutex<RefCell<GlobalTermPool>>> =
    LazyLock::new(|| ReentrantMutex::new(RefCell::new(GlobalTermPool::new())));

/// A type alias for the global term pool guard
pub(crate) type GlobalTermPoolGuard<'a> = ReentrantMutexGuard<'a, RefCell<GlobalTermPool>>;

/// Check if the symbol is the default "Int" symbol
pub fn is_int<'a, 'b>(symbol: &'b impl Symb<'a, 'b>) -> bool {
    GLOBAL_TERM_POOL.lock().borrow().int_symbol == symbol.copy()
}

/// Check if the symbol is the default "List" symbol
pub fn is_list<'a, 'b>(symbol: &'b impl Symb<'a, 'b>) -> bool {
    GLOBAL_TERM_POOL.lock().borrow().list_symbol == symbol.copy()
}

/// Check if the symbol is the default "[]" symbol
pub fn is_empty_list<'a, 'b>(symbol: &'b impl Symb<'a, 'b>) -> bool {
    GLOBAL_TERM_POOL.lock().borrow().empty_list_symbol == symbol.copy()
}

/// The single global (singleton) term pool.
pub(crate) struct GlobalTermPool {
    /// Unique table of all terms
    terms: IndexedSet<SharedTerm>,
    /// The symbol pool for managing function symbols.
    symbol_pool: SymbolPool,
    /// The thread-specific protection sets.
    thread_pools: Vec<Option<Arc<Mutex<SharedTermProtection>>>>,

    /// A vector of terms that are used to store the arguments of a term for loopup.
    tmp_arguments: Vec<ATermRef<'static>>,

    // Data structures used for garbage collection
    /// Used to avoid reallocations for the markings of all terms
    marked_terms: Vec<bool>,
    /// Used to avoid reallocations for the markings of all symbols
    marked_symbols: Vec<bool>,
    /// A stack used to mark terms recursively.
    stack: Vec<usize>,

    /// Default terms
    int_symbol: SymbolRef<'static>,
    empty_list_symbol: SymbolRef<'static>,
    list_symbol: SymbolRef<'static>,
}

impl GlobalTermPool {
    fn new() -> GlobalTermPool {
        // We should have no term with index zero, so insert bogus value.
        let mut terms = IndexedSet::new();
        terms.insert(SharedTerm {
            symbol: SymbolRef::from_index(NonZero::<usize>::MAX),
            arguments: Vec::new(),
        });

        let mut symbol_pool = SymbolPool::new();
        let int_symbol = symbol_pool.create("Int", 0, |index| SymbolRef::from_index(index));
        let list_symbol = symbol_pool.create("List", 2, |index| SymbolRef::from_index(index));
        let empty_list_symbol = symbol_pool.create("[]", 0, |index| SymbolRef::from_index(index));

        GlobalTermPool {
            terms,
            symbol_pool,
            thread_pools: Vec::new(),
            tmp_arguments: Vec::new(),
            marked_terms: Vec::new(),
            marked_symbols: Vec::new(),
            stack: Vec::new(),
            int_symbol,
            list_symbol,
            empty_list_symbol,
        }
    }

    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn get_head_symbol<'t>(&self, term: &ATermRef<'t>) -> SymbolRef<'t> {
        SymbolRef::from_symbol(self.terms.get(term.index().get()).unwrap().symbol())
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn get_argument<'t>(&self, term: &ATermRef<'t>, i: usize) -> ATermRef<'t> {
        ATermRef::from_term(
            self.terms
                .get(term.index().get())
                .unwrap()
                .arguments()
                .get(i)
                .expect("Argument index out of bounds"),
        )
    }

    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn symbol_name<'a, 'b: 'a>(&'b self, symbol: &SymbolRef<'a>) -> Arc<String>  {
        self.symbol_pool.symbol_name_owned(symbol)
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn symbol_arity<'a, 'b>(&self, symbol: &'b impl Symb<'a, 'b>) -> usize {
        self.symbol_pool.symbol_arity(symbol)
    }

    /// Returns the number of terms in the pool.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Returns the capacity of terms in the pool.
    pub fn capacity(&self) -> usize {
        self.terms.len()
    }

    /// Creates a term storing a single integer value.
    pub fn create_int<P>(&mut self, value: usize, protect: P) -> ATerm
    where
        P: FnOnce(&mut GlobalTermPool, NonZero<usize>, bool) -> ATerm,
    {
        // Fill the arguments in the existing
        self.tmp_arguments.clear();
        unsafe {
            self.tmp_arguments
                .push(ATermRef::from_index(NonZero::new(value + 1).unwrap()));
        }

        let shared_term = SharedTermLookup {
            symbol: SymbolRef::from_index(self.int_symbol.index()),
            arguments: &self.tmp_arguments,
        };

        let (index, inserted) = self.terms.insert_equiv(&shared_term);
        protect(self, NonZero::new(index).unwrap(), inserted)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term_iter<'a, 'b, 'c, 'd, I, T, P>(&mut self, symbol: &'b impl Symb<'a, 'b>, args: I, protect: P) -> ATerm
    where
        I: IntoIterator<Item = T>,
        T: Term<'c, 'd>,
        P: FnOnce(&mut GlobalTermPool, NonZero<usize>, bool) -> ATerm,
    {
        self.tmp_arguments.clear();
        for arg in args {
            unsafe {
                self.tmp_arguments.push(ATermRef::from_index(arg.index()));
            }
        }

        let shared_term = SharedTermLookup {
            symbol: SymbolRef::from_symbol(symbol),
            arguments: &self.tmp_arguments,
        };

        debug_assert_eq!(
            self.symbol_arity(symbol),
            shared_term.arguments.len(),
            "The number of arguments does not match the arity of the symbol"
        );

        let (index, inserted) = self.terms.insert_equiv(&shared_term);
        protect(self, NonZero::new(index).unwrap(), inserted)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term<'a, 'b, P>(&mut self, symbol: &'b impl Symb<'a, 'b>, args: &'b [impl Term<'a, 'b>], protect: P) -> ATerm
    where
        P: FnOnce(&mut GlobalTermPool, NonZero<usize>, bool) -> ATerm,
    {
        self.tmp_arguments.clear();
        for arg in args {
            unsafe {
                self.tmp_arguments.push(ATermRef::from_index(arg.index()));
            }
        }

        let shared_term = SharedTermLookup {
            symbol: SymbolRef::from_symbol(symbol),
            arguments: &self.tmp_arguments,
        };

        debug_assert_eq!(
            self.symbol_arity(symbol),
            shared_term.arguments.len(),
            "The number of arguments does not match the arity of the symbol"
        );

        let (index, inserted) = self.terms.insert_equiv(&shared_term);
        protect(self, NonZero::new(index).unwrap(), inserted)
    }

    /// Create a function symbol
    pub fn create_symbol<P>(&mut self, name: impl Into<String> + AsRef<str>, arity: usize, protect: P) -> Symbol
    where
        P: FnOnce(NonZero<usize>) -> Symbol,
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
        self.len()
    }

    /// Collects garbage terms.
    fn collect_garbage(&mut self) {
        // Resize to fit all terms
        self.marked_terms.resize(self.capacity(), false);
        self.marked_symbols.resize(self.symbol_pool.capacity(), false);

        // Mark the default symbols
        self.marked_symbols[self.int_symbol.index().get()] = true;
        self.marked_symbols[self.list_symbol.index().get()] = true;
        self.marked_symbols[self.empty_list_symbol.index().get()] = true;

        self.marked_terms[0] = true; // The bogus term
        self.marked_symbols[0] = true; // The bogus symbol

        let mut marker = Marker {
            marked_terms: &mut self.marked_terms,
            marked_symbols: &mut self.marked_symbols,
            stack: &mut self.stack,
            terms: &self.terms,
        };

        let mut mark_time = SimpleTimer::new();

        // Loop through all protection sets and mark the terms.
        for pool in self.thread_pools.iter() {
            if let Some(pool) = pool {
                let pool = mutex_unwrap(pool.lock());

                for (term, _) in pool.protection_set.iter() {
                    // Remove all terms that are not protected
                    unsafe {
                        ATermRef::from_index(*term).mark(&mut marker);
                    }
                }

                for (container, _) in pool.container_protection_set.iter() {
                    container.mark(&mut marker);
                }
            }
        }

        mark_time.stop();
        let collect_time = SimpleTimer::new();

        let num_of_terms = self.len();

        // Delete all terms that are not marked
        self.terms.retain_mut(|index, _term| {
            trace!("Removing term {}", index);
            self.marked_terms[index]
        });

        self.symbol_pool.retain_mut(|index| {
            trace!("Removing symbol {}", index);
            self.marked_symbols[index]
        });

        info!(
            "Garbage collection: marking took {}ms, collection took {}ms, {} terms removed",
            mark_time.elapsed().as_millis(),
            collect_time.elapsed().as_millis(),
            num_of_terms - self.len()
        );

        info!("{:?}", self);

        for pool in self.thread_pools.iter() {
            if let Some(pool) = pool {
                let pool = pool.lock();

                // Remove all terms that are not protected
                info!("{:?}", pool);
            }
        }
    }
}

impl fmt::Debug for GlobalTermPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "There are {} terms, and {} symbols",
            self.terms.len(),
            self.symbol_pool.size()
        )
    }
}

pub(crate) struct SharedTermProtection {
    /// Protection set for terms
    pub(crate) protection_set: ProtectionSet<NonZero<usize>>,
    /// Protection set to prevent garbage collection of symbols
    pub(crate) symbol_protection_set: ProtectionSet<NonZero<usize>>,
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
    marked_terms: &'a mut Vec<bool>,
    marked_symbols: &'a mut Vec<bool>,
    terms: &'a IndexedSet<SharedTerm>,
    stack: &'a mut Vec<usize>,
}

impl Marker<'_> {
    // Marks the given term as being reachable.
    pub fn mark(&mut self, term: &ATermRef<'_>) {
        let term_index: usize = term.index().get();
        if !self.marked_terms[term_index] {
            self.stack.push(term_index);

            while let Some(index) = self.stack.pop() {
                // Each term should be marked.
                self.marked_terms[index] = true;

                let term = self.terms.get(index).unwrap();

                // Mark the function symbol.
                self.marked_symbols[term.symbol().index().get()] = true;

                for arg in term.arguments() {
                    let arg_index: usize = arg.index().get();
                    // Skip if unnecessary, otherwise mark before pushing to stack since it can be shared.
                    if !self.marked_terms[arg_index] {
                        self.marked_terms[arg_index] = true;
                        self.stack.push(arg_index);
                    }
                }
            }
        }
    }
}

/// The underlying type of terms that are actually shared.
#[derive(Debug, Eq, PartialEq)]
pub struct SharedTerm {
    symbol: SymbolRef<'static>,
    arguments: Vec<ATermRef<'static>>,
}

impl Clone for SharedTerm {
    fn clone(&self) -> Self {
        SharedTerm {
            symbol: SymbolRef::from_index(self.symbol.index()),
            arguments: self
                .arguments
                .iter()
                .map(|x| unsafe { ATermRef::from_index(x.index()) })
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
            symbol: SymbolRef::from_index(lookup.symbol.index()),
            arguments: lookup
                .arguments
                .iter()
                .map(|x| unsafe { ATermRef::from_index(x.index()) })
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
