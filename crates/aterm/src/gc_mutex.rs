use std::{cell::{UnsafeCell}, ops::{Deref, DerefMut}};
use crate::{GlobalTermPoolGuard, GLOBAL_TERM_POOL};

/// Global mutex that prevents garbage collection.
pub struct GcMutex<T> {
    inner: UnsafeCell<T>,
}

unsafe impl<T> Send for GcMutex<T> {}
unsafe impl<T> Sync for GcMutex<T> {}

impl<T> GcMutex<T> {
    pub fn new(value: T) -> GcMutex<T> {
        GcMutex {
            inner: UnsafeCell::new(value),
        }
    }

    /// Provides mutable access to the underlying value.
    pub fn write(&self) -> GcMutexGuard<'_, T> {
        GcMutexGuard {
            mutex: &self,
            _guard: GLOBAL_TERM_POOL.lock(),
        }
    }

    /// Provides immutable access to the underlying value.
    pub fn read(&self) -> GcMutexGuard<'_, T> {
        GcMutexGuard {
            mutex: &self,
            _guard: GLOBAL_TERM_POOL.lock(),
        }
    }
}

pub struct GcMutexGuard<'a, T> {
    mutex: &'a GcMutex<T>,

    /// Only used to avoid garbage collection, will be released on drop.
    _guard: GlobalTermPoolGuard<'a>,
}

impl<'a, T> Deref for GcMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<'a, T> DerefMut for GcMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.inner.get() }
    }
}
