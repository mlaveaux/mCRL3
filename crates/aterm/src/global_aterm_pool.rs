use std::fmt::Debug;
use std::sync::Arc;
use std::sync::LazyLock;

use mcrl3_sharedmutex::BfSharedMutex;
use parking_lot::Mutex;

use mcrl3_utilities::IndexedSet;
use mcrl3_utilities::ProtectionSet;

use crate::ATerm;
use crate::ATermRef;
use crate::BfTermPool;
use crate::Markable;
use crate::SymbolPool;
use crate::SymbolRef;

/// This is the global set of protection sets that are managed by the ThreadTermPool
pub(crate) static GLOBAL_TERM_POOL: LazyLock<Mutex<GlobalTermPool>> =
    LazyLock::new(|| Mutex::new(GlobalTermPool::new()));

struct SharedTermProtection {
    (Arc<ProtectionSet<usize>>, Arc<ProtectionSet<Arc<dyn Markable + Sync + Send>>>)
}

/// The single global (singleton) term pool.
pub(crate) struct GlobalTermPool {
    /// Unique table of all terms
    terms: IndexedSet<SharedTerm>,
    /// The symbol pool for managing function symbols.
    symbol_pool: SymbolPool,
    /// The thread-specific protection sets.
    thread_pools: Vec<Option<>>,

    lock: BfSharedMutex<()>,
}

impl GlobalTermPool {
    fn new() -> GlobalTermPool {
        GlobalTermPool {
            terms: IndexedSet::new(),
            symbol_pool: SymbolPool::new(),
            thread_pools: Vec::new(),
            lock: BfSharedMutex::new(()),
        }
    }

    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn get_head_symbol(&self, term: &ATermRef<'_>) -> SymbolRef<'_> {
        self.terms.get(term.index()).unwrap().symbol()
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn get_argument(&self, term: &ATermRef<'_>, i: usize) -> ATermRef<'_> {
        self.terms.get(term.index()).unwrap().arguments().get(i).unwrap().copy()
    }

    /// Returns the number of terms in the pool.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Returns the number of terms in the pool.
    pub fn capacity(&self) -> usize {
        self.terms.len()
    }

    /// Expose the symbol pool
    pub fn symbol_pool(&self) -> &SymbolPool {
        &self.symbol_pool
    }

    /// Expose the symbol pool as mutable
    pub fn symbol_pool_mut(&mut self) -> &mut SymbolPool {
        &mut self.symbol_pool
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term<I, P>(&mut self, symbol: &SymbolRef<'_>, args: I, protect: P) -> ATerm
    where
        I: IntoIterator<Item = ATermRef<'static>>,
        P: FnOnce(usize) -> ATerm,
    {
        let shared_term = SharedTerm {
            symbol: SymbolRef::new(symbol.index()),
            arguments: args.into_iter().collect(),
        };

        let index = self.terms.insert(shared_term);
        protect(index)
    }

    /// Registers a new thread term pool.
    pub fn register_thread_term_pool(
        &mut self,
    ) -> (
        Arc<ProtectionSet<usize>>,
        Arc<ProtectionSet<Arc<dyn Markable + Sync + Send>>>,
        BfSharedMutex<()>,
        usize,
    ) {
        let index = self.thread_pools.len();
        let protection_set = Arc::new(ProtectionSet::new());
        let container_set = Arc::new(ProtectionSet::new());

        self.thread_pools
            .push(Some((protection_set.clone(), container_set.clone())));

        (protection_set, container_set, self.lock.clone(), index)
    }

    /// Deregisters a thread pool.
    pub fn deregister_thread_pool(&mut self, index: usize) {
        if let Some(entry) = self.thread_pools.get_mut(index) {
            *entry = None;
        }
    }

    /// Collects garbage terms.
    fn collect_garbage(&mut self) {
        // Implement garbage collection logic here
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
            symbol: SymbolRef::new(self.symbol.index()),
            arguments: self.arguments.iter().map(|x| ATermRef::new(x.index())).collect(),
        }
    }
}

impl SharedTerm {
    pub fn symbol(&self) -> SymbolRef<'_> {
        self.symbol.copy()
    }

    pub fn arguments(&self) -> &[ATermRef<'static>] {
        &self.arguments
    }
}
