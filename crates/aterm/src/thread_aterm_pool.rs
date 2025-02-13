use log::trace;
use mcrl3_sharedmutex::{BfSharedMutex, BfSharedMutexReadGuard, BfSharedMutexWriteGuard};
use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::sync::Arc;

use crate::aterm::{ATerm, ATermRef};
use crate::global_aterm_pool::GLOBAL_TERM_POOL;
use crate::{Markable, SharedContainerProtectionSet, SharedProtectionSet};

/// The number of times before garbage collection is tested again.
const TEST_GC_INTERVAL: usize = 100;

thread_local! {
    /// Thread-specific term pool that manages protection sets.
    pub static THREAD_TERM_POOL: RefCell<ThreadTermPool> = RefCell::new(ThreadTermPool::new());
}

/// Per-thread term pool managing local protection sets.
pub struct ThreadTermPool {
    /// Protection set for terms
    protection_set: SharedProtectionSet,
    /// Protection set for containers
    container_protection_set: SharedContainerProtectionSet,
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
        let (protection_set, container_protection_set, lock, index) = GLOBAL_TERM_POOL.lock().register_thread_term_pool();

        Self {
            protection_set,
            container_protection_set,
            index,
            lock,
            gc_counter: TEST_GC_INTERVAL,
        }
    }

    /// Protects a term in this thread's protection set.
    pub fn protect(&mut self, term: ATermRef<'_>) -> ATerm {
        let guard = self.protection_set.write();
        let term_index = GLOBAL_TERM_POOL.lock().get_or_create_index(term);
        let root = guard.protect(term_index);

        trace!(
            "Protected term {:?}, index {}, protection set {}",
            term,
            root,
            self.index
        );

        // Check garbage collection
        self.gc_counter = self.gc_counter.saturating_sub(1);
        if self.gc_counter == 0 {
            self.test_garbage_collection();
            self.gc_counter = TEST_GC_INTERVAL;
        }

        ATerm::new(term_index, root)
    }

    /// Protects a container in this thread's container protection set.
    pub fn protect_container(&mut self, container: Arc<dyn Markable + Send + Sync>) -> usize {
        let root = self.container_protection_set.write().protect(container);
        trace!("Protected container index {}, protection set {}", root, self.index);
        root
    }

    /// Unprotects a term from this thread's protection set.
    pub fn unprotect(&mut self, term: &ATerm) {
        term.require_valid();
        let mut guard = self.protection_set.write();
        trace!(
            "Unprotected term {:?}, index {}, protection set {}",
            term.term(),
            term.root(),
            self.index
        );
        guard.unprotect(&term.root());
    }

    /// Unprotects a container from this thread's container protection set.
    pub fn unprotect_container(&mut self, root: usize) {
        let mut guard = self.container_protection_set.write();
        trace!("Unprotected container index {}, protection set {}", root, self.index);
        guard.unprotect(root);
    }

    /// Tests if garbage collection should run.
    fn test_garbage_collection(&self) {
        GLOBAL_TERM_POOL.lock().test_garbage_collection();
    }

    /// Locks the thread local lock in shared mode.
    pub fn lock_shared(&self) -> BfSharedMutexReadGuard<()> {
        self.lock.read().unwrap()
    }

    /// Locks the thread local lock in exclusive mode.
    pub fn lock_exclusive(&self) -> BfSharedMutexWriteGuard<()>{
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
