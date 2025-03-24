use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::LazyLock;

use log::info;
use mcrl3_utilities::SimpleTimer;
use parking_lot::Mutex;

use mcrl3_utilities::IndexedSet;
use mcrl3_utilities::ProtectionSet;
use parking_lot::ReentrantMutex;
use parking_lot::ReentrantMutexGuard;

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

/// The single global (singleton) term pool.
pub(crate) struct GlobalTermPool {
    /// Unique table of all terms
    terms: IndexedSet<SharedTerm>,
    /// The symbol pool for managing function symbols.
    symbol_pool: SymbolPool,
    /// The thread-specific protection sets.
    thread_pools: Vec<Option<Arc<Mutex<SharedTermProtection>>>>,

    /// Used to avoid reallocations for the markings of all terms. Not stored in
    /// the term because of padding.
    marked: Vec<bool>,
}

impl GlobalTermPool {
    fn new() -> GlobalTermPool {
        // 0 Should be the default term.
        let mut terms = IndexedSet::new();
        terms.insert(SharedTerm {
            symbol: SymbolRef::from_index(0),
            arguments: Vec::new(),
        });

        GlobalTermPool {
            terms,
            symbol_pool: SymbolPool::new(),
            thread_pools: Vec::new(),
            marked: Vec::new(),
        }
    }

    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn get_head_symbol<'t>(&self, term: &ATermRef<'t>) -> SymbolRef<'t> {
        SymbolRef::from_symbol(
            self.terms.get(term.index()).unwrap().symbol(),
        )
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn get_argument<'t>(&self, term: &ATermRef<'t>, i: usize) -> ATermRef<'t> {
        ATermRef::from_term(
            self.terms
                .get(term.index())
                .unwrap()
                .arguments()
                .get(i)
                .expect("Argument index out of bounds"),
        )
    }

    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn symbol_name<'a, 'b: 'a>(&'b self, symbol: &SymbolRef<'a>) -> &'a str {
        self.symbol_pool.symbol_name(symbol)
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn symbol_arity<'a>(&self, symbol: &impl Symb<'a>) -> usize {
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

    pub fn create_int<P>(&mut self, value: usize, protect: P) -> ATerm 
        where P: FnOnce(&mut GlobalTermPool, usize, bool) -> ATerm
    {
        let shared_term = SharedTerm {
            symbol: SymbolRef::from_index(0),
            arguments: vec![ATermRef::from_index(value)],
        };

        let (index, inserted) = self.terms.insert(shared_term);
        protect(self, index, inserted)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term_iter<'a, I, T, P>(&mut self, symbol: &impl Symb<'a>, args: I, protect: P) -> ATerm
    where
        I: IntoIterator<Item = T>,
        T: Term<'a>,
        P: FnOnce(&mut GlobalTermPool, usize, bool) -> ATerm,
    {
        let shared_term = unsafe {
            SharedTerm {
                symbol: std::mem::transmute::<SymbolRef<'_>, SymbolRef<'static>>(SymbolRef::from_symbol(symbol)),
                arguments: args
                    .into_iter()
                    .map(|t| std::mem::transmute::<ATermRef<'_>, ATermRef<'static>>(t.copy()))
                    .collect(),
            }
        };
        
        debug_assert_eq!(
            self.symbol_arity(symbol),
            shared_term.arguments.len(),
            "The number of arguments does not match the arity of the symbol"
        );

        let (index, inserted) = self.terms.insert(shared_term);
        protect(self, index, inserted)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term<'a, 'b, P>(&mut self, symbol: &impl Symb<'a>, args: &[impl Term<'b>], protect: P) -> ATerm
    where
        P: FnOnce(&mut GlobalTermPool, usize, bool) -> ATerm,
    {
        // This is safe because we are responsible for garbage collection of shared terms.
        let shared_term = unsafe {
            SharedTerm {
                symbol: std::mem::transmute::<SymbolRef<'_>, SymbolRef<'static>>(SymbolRef::from_symbol(symbol)),
                arguments: args
                    .iter()
                    .map(|t| std::mem::transmute::<ATermRef<'_>, ATermRef<'static>>(t.copy()))
                    .collect(),
            }
        };

        debug_assert_eq!(
            self.symbol_arity(symbol),
            shared_term.arguments.len(),
            "The number of arguments does not match the arity of the symbol"
        );

        let (index, inserted) = self.terms.insert(shared_term);
        protect(self, index, inserted)
    }

    /// Create a function symbol
    pub fn create_symbol<P>(&mut self, name: impl Into<String>, arity: usize, protect: P) -> Symbol
    where
        P: FnOnce(usize) -> Symbol,
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

        self.thread_pools.push(Some(protection.clone()));

        protection
    }

    /// Deregisters a thread pool.
    pub(crate) fn deregister_thread_pool(&mut self, index: usize) {
        if let Some(entry) = self.thread_pools.get_mut(index) {
            *entry = None;
        }
    }

    /// Triggers garbage collection if necessary and returns an updated counter for the thread local pool.
    pub(crate) fn trigger_garbage_collection(&mut self) -> usize {
        self.collect_garbage();

        info!("Delay garbage collection by {}", self.capacity() - self.len());
        self.capacity() - self.len()
    }

    /// Collects garbage terms.
    fn collect_garbage(&mut self) {
        // Resize to fit all terms
        self.marked.resize(self.capacity(), false);

        let mut marker = Marker {
            marked: &mut self.marked,
        };        

        let mut mark_time = SimpleTimer::new();

        // Loop through all protection sets and mark the terms.
        for pool in self.thread_pools.iter() {
            if let Some(pool) = pool {
                let pool = pool.lock();

                for (term, _) in pool.protection_set.iter() {
                    // Remove all terms that are not protected
                    ATermRef::from_index(*term).mark(&mut marker);
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
        self.terms.retain_mut(|index, _term| self.marked[index]);

        info!("Garbage collection: marking took {}ms, collection took {}ms, {} terms removed", 
            mark_time.elapsed().as_millis(), 
            collect_time.elapsed().as_millis(),
            num_of_terms - self.len());
    }
}

pub(crate) struct SharedTermProtection {
    /// Protection set for terms
    pub(crate) protection_set: ProtectionSet<usize>,
    pub(crate) symbol_protection_set: ProtectionSet<usize>,
    /// Protection set to prevent garbage collection of symbols
    /// Protection set for containers
    pub(crate) container_protection_set: ProtectionSet<Arc<dyn Markable + Sync + Send>>,
    /// Index in global pool's thread pools list
    pub(crate) index: usize,
}

pub struct Marker<'a> {
    marked: &'a mut Vec<bool>,
}

impl Marker<'_> {
    // Marks the given term as being reachable.
    pub fn mark(&mut self, term: &ATermRef<'_>) {
        self.marked[term.index()] = true;
    }
}

impl Drop for SharedTermProtection {
    fn drop(&mut self) {
        assert!(self.protection_set.is_empty(), "Protection set must be empty on drop");

        GLOBAL_TERM_POOL.lock().borrow_mut().deregister_thread_pool(self.index);
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct SharedTerm {
    symbol: SymbolRef<'static>,
    arguments: Vec<ATermRef<'static>>,
}

impl Clone for SharedTerm {
    fn clone(&self) -> Self {
        SharedTerm {
            symbol: SymbolRef::from_index(self.symbol.index()),
            arguments: self.arguments.iter().map(|x| ATermRef::from_index(x.index())).collect(),
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
