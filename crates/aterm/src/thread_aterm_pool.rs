use log::trace;
use mcrl3_sharedmutex::BfSharedMutex;
use mcrl3_sharedmutex::BfSharedMutexReadGuard;
use mcrl3_sharedmutex::BfSharedMutexWriteGuard;
use parking_lot::Mutex;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::sync::Arc;

use crate::aterm::ATerm;
use crate::aterm::ATermRef;
use crate::global_aterm_pool::GLOBAL_TERM_POOL;
use crate::Markable;
use crate::SharedTermProtection;
use crate::Symbol;
use crate::SymbolRef;

/// The number of times before garbage collection is tested again.
const TEST_GC_INTERVAL: usize = 100;

thread_local! {
    /// Thread-specific term pool that manages protection sets.
    pub static THREAD_TERM_POOL: RefCell<ThreadTermPool> = RefCell::new(ThreadTermPool::new());
}

/// Per-thread term pool managing local protection sets.
pub struct ThreadTermPool {
    /// Counter for garbage collection tests
    gc_counter: usize,
    ///
    protection_set: Arc<Mutex<SharedTermProtection>>,
    /// Thread local lock
    lock: BfSharedMutex<()>,
}

impl ThreadTermPool {
    /// Creates a new thread-local term pool.
    pub fn new() -> Self {
        // Register protection sets with global pool
        let protection_set =
            GLOBAL_TERM_POOL.lock().register_thread_term_pool();

        Self {
            protection_set,
            gc_counter: TEST_GC_INTERVAL,
            lock: BfSharedMutex::new(()),
        }
    }

    pub fn create_constant<'a>(&mut self, symbol: &SymbolRef<'_>) -> ATerm {
        unimplemented!();
        // GLOBAL_TERM_POOL.lock().create_term(symbol, children, |index| {
        //     let protected = self.protect(&ATermRef::new(index));

        //     ATerm::new(protected.index(), protected.root())
        // })
    }

    /// Create a term
    pub fn create_term<'a>(&mut self, symbol: &SymbolRef<'_>, children: &[impl Borrow<ATermRef<'a>>]) -> ATerm {
        unimplemented!();
        // GLOBAL_TERM_POOL.lock().create_term(symbol, children, |index| {
        //     let protected = self.protect(&ATermRef::new(index));

        //     ATerm::new(protected.index(), protected.root())
        // })
    }

    /// Create a function symbol
    pub fn create_symbol(&mut self, name: &str, arity: usize) -> Symbol {
        GLOBAL_TERM_POOL.lock().symbol_pool_mut().create(name, arity)
    }

    /// Protect the term by adding its index to the protection set
    pub fn protect(&mut self, term: &ATermRef<'_>) -> ATerm {
        // Protect the term by adding its index to the protection set
        let root = self.protection_set.lock().protection_set.protect(term.index());

        // Return the protected term
        ATerm::new_interal(term.index(), root)
    }


    /// Unprotects a term from this thread's protection set.
    pub fn unprotect(&mut self, term: &ATerm) {
        term.require_valid();
        
        trace!(
            "Unprotected term {:?}, index {}, protection set {}",
            term,
            term.root(),
            self.index()
        );

        self.protection_set.lock().protection_set.unprotect(term.root());
    }

    /// Protects a container in this thread's container protection set.
    pub fn protect_container(&mut self, container: Arc<dyn Markable + Send + Sync>) -> usize {
        let root = self.protection_set.lock().container_protection_set.protect(container);
        trace!("Protected container index {}, protection set {}", root, self.index());
        root
    }

    /// Unprotects a container from this thread's container protection set.
    pub fn drop_container(&mut self, root: usize) {
        trace!("Unprotected container index {}, protection set {}", root, self.index());
        self.protection_set.lock().container_protection_set.unprotect(root);
    }

    /// 
    pub fn from_string(&mut self, term: &str) -> Result<ATerm, Box<dyn Error>> {
        unimplemented!();

    }

    /// Locks the thread local lock in shared mode.
    pub fn lock_shared(&self) -> BfSharedMutexReadGuard<()> {
        self.lock.read().unwrap()
    }

    /// Locks the thread local lock in exclusive mode.
    pub fn lock_exclusive(&self) -> BfSharedMutexWriteGuard<()> {
        self.lock.write().unwrap()
    }

    fn index(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_thread_local_protection() {
        thread::scope(|scope| {
            for _ in 0..3 {
                scope.spawn(|| {
                    THREAD_TERM_POOL.with(|pool| {
                        let mut pool = pool.borrow_mut();

                        // Create and protect some terms
                        let symbol = pool.create_symbol("test", 0);
                        let term = pool.create_constant(&symbol);
                        let protected = pool.protect(&term);

                        // Verify protection
                        //assert!(pool.protection_set.lock().protection_set.contains(&protected.root()));

                        // Unprotect
                        pool.unprotect(&protected);
                        //assert!(!pool.protection_set.lock().protection_set.contains(&protected.root()));
                    });
                });
            }
        });
    }
}
