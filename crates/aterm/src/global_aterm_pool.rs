use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::LazyLock;

use parking_lot::Mutex;

use mcrl3_utilities::IndexedSet;
use mcrl3_utilities::ProtectionSet;
use parking_lot::ReentrantMutex;

use crate::ATerm;
use crate::ATermRef;
use crate::Markable;
use crate::StrRef;
use crate::Symb;
use crate::Symbol;
use crate::SymbolPool;
use crate::SymbolRef;
use crate::Term;

/// This is the global set of protection sets that are managed by the ThreadTermPool
pub(crate) static GLOBAL_TERM_POOL: LazyLock<ReentrantMutex<RefCell<GlobalTermPool>>> =
    LazyLock::new(|| ReentrantMutex::new(RefCell::new(GlobalTermPool::new())));

pub(crate) struct SharedTermProtection {
    /// Protection set for terms
    pub(crate) protection_set: ProtectionSet<usize>,
    /// Protection set to prevent garbage collection of symbols
    pub(crate) symbol_protection_set: ProtectionSet<usize>,
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

/// The single global (singleton) term pool.
pub(crate) struct GlobalTermPool {
    /// Unique table of all terms
    terms: IndexedSet<SharedTerm>,
    /// The symbol pool for managing function symbols.
    symbol_pool: SymbolPool,
    /// The thread-specific protection sets.
    thread_pools: Vec<Option<Arc<Mutex<SharedTermProtection>>>>,

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
        unsafe {
            std::mem::transmute::<SymbolRef<'_>, SymbolRef<'t>>(SymbolRef::from_symbol(
                self.terms.get(term.index()).unwrap().symbol(),
            ))
        }
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
    pub fn symbol_name<'a>(&self, symbol: &SymbolRef<'a>) -> StrRef<'a> {
        self.symbol_pool.symbol_name(symbol)
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn symbol_arity(&self, symbol: &SymbolRef<'_>) -> usize {
        self.symbol_pool.symbol_arity(symbol)
    }

    /// Returns the number of terms in the pool.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Returns the number of terms in the pool.
    pub fn capacity(&self) -> usize {
        self.terms.len()
    }

    pub fn create_int(&mut self, value: usize, protect: impl FnOnce(usize) -> ATerm) -> ATerm {
        let shared_term = SharedTerm {
            symbol: SymbolRef::from_index(0),
            arguments: vec![ATermRef::from_index(value)],
        };

        let index = self.terms.insert(shared_term);
        protect(index)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term_iter<'a, I, T, P>(&mut self, symbol: &impl Symb<'a>, args: I, protect: P) -> ATerm
    where
        I: IntoIterator<Item = T>,
        T: Term<'a>,
        P: FnOnce(usize) -> ATerm,
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

        let index = self.terms.insert(shared_term);
        protect(index)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term<'a, 'b, P>(&mut self, symbol: &impl Symb<'a>, args: &[impl Term<'b>], protect: P) -> ATerm
    where
        P: FnOnce(usize) -> ATerm,
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

        let index = self.terms.insert(shared_term);
        protect(index)
    }

    /// Create a function symbol
    pub fn create_symbol<P>(&mut self, name: impl Into<String>, arity: usize, protect: P) -> Symbol
    where
        P: FnOnce(usize) -> Symbol,
    {
        self.symbol_pool.create(name, arity, protect)
    }

    /// Registers a new thread term pool.
    pub fn register_thread_term_pool(&mut self) -> Arc<Mutex<SharedTermProtection>> {
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
    pub fn deregister_thread_pool(&mut self, index: usize) {
        if let Some(entry) = self.thread_pools.get_mut(index) {
            *entry = None;
        }
    }

    /// Collects garbage terms.
    fn collect_garbage(&mut self) {
        let mut marker = Marker {
            marked: &mut self.marked,
        };

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

        // Delete all terms that are not marked
        self.terms.retain_mut(|index, _term| self.marked[index]);
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
