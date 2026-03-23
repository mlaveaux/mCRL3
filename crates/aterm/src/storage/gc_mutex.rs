use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::storage::GlobalTermPoolGuard;
use crate::storage::THREAD_TERM_POOL;

/// A mutex that prevents garbage collection by holding a shared read lock on the [super::GlobalTermPool]
/// for the duration of the guard's lifetime. Returns a [GcMutexGuard] on access.
pub struct GcMutex<T> {
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for GcMutex<T> {}
unsafe impl<T: Send> Sync for GcMutex<T> {}

impl<T> GcMutex<T> {
    pub fn new(value: T) -> GcMutex<T> {
        GcMutex {
            inner: UnsafeCell::new(value),
        }
    }

    /// Provides mutable access to the underlying value, returning a [GcMutexGuard].
    pub fn write(&self) -> GcMutexGuard<'_, T> {
        GcMutexGuard {
            mutex: self,
            guard: ManuallyDrop::new(THREAD_TERM_POOL.with_borrow(|tp| unsafe {
                std::mem::transmute(tp.term_pool().read_recursive().expect("Lock poisoned!"))
            })),
        }
    }

    /// Provides immutable access to the underlying value, returning a [GcMutexGuard].
    pub fn read(&self) -> GcMutexGuard<'_, T> {
        GcMutexGuard {
            mutex: self,
            guard: ManuallyDrop::new(THREAD_TERM_POOL.with_borrow(|tp| unsafe {
                std::mem::transmute(tp.term_pool().read_recursive().expect("Lock poisoned!"))
            })),
        }
    }
}

pub struct GcMutexGuard<'a, T> {
    mutex: &'a GcMutex<T>,

    /// Only used to avoid garbage collection, will be released on drop.
    guard: ManuallyDrop<GlobalTermPoolGuard<'a>>,
}

impl<T> Deref for GcMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> DerefMut for GcMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<T> Drop for GcMutexGuard<'_, T> {
    fn drop(&mut self) {
        if self.guard.read_depth() == 1 {
            // If this is the last guard, we can trigger garbage collection when it was delayed earlier.
            THREAD_TERM_POOL.with_borrow(|tp| unsafe { tp.trigger_delayed_garbage_collection(&mut self.guard) })
        } else {
            // Just drop the guard
            unsafe { ManuallyDrop::drop(&mut self.guard) };
        }
    }
}
