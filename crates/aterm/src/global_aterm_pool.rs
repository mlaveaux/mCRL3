use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::LazyLock;

use mcrl3_sharedmutex::BfSharedMutex;
use parking_lot::Mutex;

use mcrl3_utilities::IndexedSet;
use mcrl3_utilities::ProtectionSet;

use crate::ATerm;
use crate::ATermRef;
use crate::Markable;
use crate::SymbolPool;
use crate::SymbolRef;

/// This is the global set of protection sets that are managed by the ThreadTermPool
pub(crate) static GLOBAL_TERM_POOL: LazyLock<Mutex<GlobalTermPool>> =
    LazyLock::new(|| Mutex::new(GlobalTermPool::new()));

pub(crate) struct SharedTermProtection {    
    /// Protection set for terms
    pub(crate) protection_set: ProtectionSet<usize>,
    /// Protection set for containers
    pub(crate) container_protection_set: ProtectionSet<Arc<dyn Markable + Sync + Send>>,
    /// Index in global pool's thread pools list
    index: usize,
}

pub(crate) struct Marker {

}

impl Marker {
    pub fn mark(&self, term: &ATermRef<'_>) {
        unimplemented!();
    }
}

impl Drop for SharedTermProtection {
    fn drop(&mut self) {
        assert!(
            self.protection_set.is_empty(),
            "Protection set must be empty on drop"
        );
        GLOBAL_TERM_POOL.lock().deregister_thread_pool(self.index);
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
    pub fn create_term_iter<'a, I, P>(&'a mut self, symbol: &SymbolRef<'_>, args: I, protect: P) -> ATerm
    where
        I: IntoIterator<Item = ATermRef<'a>>,
        P: FnOnce(usize) -> ATerm,
    {
        let shared_term = SharedTerm {
            symbol: SymbolRef::new(symbol.index()),
            arguments: args.into_iter().map(|t| {
                unsafe {
                    t.upgrade_unchecked()
                }
            }).collect(),
        };

        let index = self.terms.insert(shared_term);
        protect(index)
    }

    /// Create a term from a head symbol and an iterator over its arguments
    pub fn create_term<'a, I, P>(&'a mut self, symbol: &SymbolRef<'_>, args: &[I], protect: P) -> ATerm
    where
        I: Borrow<ATermRef<'a>>,
        P: FnOnce(usize) -> ATerm,
    {
        let shared_term = SharedTerm {
            symbol: SymbolRef::new(symbol.index()),
            arguments: args.into_iter().map(|t| {               
                unsafe {
                    t.borrow().upgrade_unchecked()
                }
            }).collect()
        };

        let index = self.terms.insert(shared_term);
        protect(index)
    }

    /// Registers a new thread term pool.
    pub fn register_thread_term_pool(
        &mut self,
    ) -> Arc<Mutex<SharedTermProtection>>
    {
        let protection = Arc::new(Mutex::new(SharedTermProtection{
            protection_set: ProtectionSet::new(),
            container_protection_set: ProtectionSet::new(),
            index: self.thread_pools.len(),
        }));
        
        self.thread_pools
            .push(Some(protection.clone()));

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
        // Implement garbage collection logic here
        for pool in self.thread_pools.iter() {
            let mut marker = Marker {};


            if let Some(pool) = pool {
                let mut pool = pool.lock();
                
                for item in pool.protection_set.iter() {
                    // Remove all terms that are not protected

                }

                for (container, _) in pool.container_protection_set.iter() {
                    container.mark(&mut marker);
                }
            }

        }
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
