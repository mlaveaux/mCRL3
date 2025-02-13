use log::trace;
use mcrl3_sharedmutex::BfSharedMutex;
use mcrl3_sharedmutex::BfSharedMutexReadGuard;
use mcrl3_sharedmutex::BfSharedMutexWriteGuard;
use mcrl3_utilities::ProtectionSet;
use std::cell::RefCell;
use std::sync::Arc;

use crate::aterm::ATerm;
use crate::aterm::ATermRef;
use crate::global_aterm_pool::GLOBAL_TERM_POOL;
use crate::Markable;

/// The number of times before garbage collection is tested again.
const TEST_GC_INTERVAL: usize = 100;

thread_local! {
    /// Thread-specific term pool that manages protection sets.
    pub static THREAD_TERM_POOL: RefCell<ThreadTermPool> = RefCell::new(ThreadTermPool::new());
}

/// Per-thread term pool managing local protection sets.
pub struct ThreadTermPool {
    /// Protection set for terms
    protection_set: Arc<ProtectionSet<usize>>,
    /// Protection set for containers
    container_protection_set: Arc<ProtectionSet<Arc<dyn Markable + Sync + Send>>>,
    /// Index in global pool's thread pools list
    index: usize,
    /// Counter for garbage collection tests
    gc_counter: usize,
    /// Thread local lock
    lock: BfSharedMutex<()>,
}

impl ThreadTermPool {
    /// Creates a new thread-local term pool.
    pub fn new() -> Self {
        // Register protection sets with global pool
        let (protection_set, container_protection_set, lock, index) =
            GLOBAL_TERM_POOL.lock().register_thread_term_pool();

        Self {
            protection_set,
            container_protection_set,
            index,
            lock,
            gc_counter: TEST_GC_INTERVAL,
        }
    }

    /// Protect the term by adding its index to the protection set
    pub fn protect(&mut self, term: ATermRef<'_>) -> ATerm {
        // Protect the term by adding its index to the protection set
        let root = self.protection_set.protect(term.index());

        // Return the protected term
        ATerm::new(term.index(), root)
    }


    /// Unprotects a term from this thread's protection set.
    pub fn unprotect(&mut self, term: &ATerm) {
        term.require_valid();
        
        trace!(
            "Unprotected term {:?}, index {}, protection set {}",
            term,
            term.root(),
            self.index
        );
        self.protection_set.unprotect(term.root());
    }

    /// Protects a container in this thread's container protection set.
    pub fn protect_container(&mut self, container: Arc<dyn Markable + Send + Sync>) -> usize {
        let root = self.container_protection_set.borrow_mut().protect(container);
        trace!("Protected container index {}, protection set {}", root, self.index);
        root
    }

    /// Unprotects a container from this thread's container protection set.
    pub fn unprotect_container(&mut self, root: usize) {
        trace!("Unprotected container index {}, protection set {}", root, self.index);
        self.container_protection_set.unprotect(root);
    }

    /// Locks the thread local lock in shared mode.
    pub fn lock_shared(&self) -> BfSharedMutexReadGuard<()> {
        self.lock.read().unwrap()
    }

    /// Locks the thread local lock in exclusive mode.
    pub fn lock_exclusive(&self) -> BfSharedMutexWriteGuard<()> {
        self.lock.write().unwrap()
    }
}

impl Drop for ThreadTermPool {
    fn drop(&mut self) {
        assert!(
            self.protection_set.read().is_empty(),
            "Protection set must be empty on drop"
        );
        GLOBAL_TERM_POOL.lock().deregister_thread_pool(self.index);
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
                        let term = GLOBAL_TERM_POOL.lock().create_term("test", &[]);
                        let protected = pool.protect(term.term());

                        // Verify protection
                        assert!(pool.protection_set.read().contains(&protected.root()));

                        // Unprotect
                        pool.unprotect(&protected);
                        assert!(!pool.protection_set.read().contains(&protected.root()));
                    });
                });
            }
        });
    }
}
