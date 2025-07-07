use std::cell::Cell;
use std::error::Error;
use std::mem;
use std::ops::Deref;

use crate::BfSharedMutex;
use crate::BfSharedMutexReadGuard;
use crate::BfSharedMutexWriteGuard;

/// An extension of the `SharedMutex` that allows recursive read locking without deadlocks.
pub struct RecursiveLock<T> {
    inner: BfSharedMutex<T>,

    /// The number of times the current thread has read locked the mutex.
    recursive_depth: Cell<usize>,
}

impl<T> RecursiveLock<T> {
    /// Creates a new `RecursiveLock` with the given data.
    pub fn new(data: T) -> Self {
        RecursiveLock {
            inner: BfSharedMutex::new(data),
            recursive_depth: Cell::new(0),
        }
    }

    /// Creates a new `RecursiveLock` from an existing `BfSharedMutex`.
    pub fn from_mutex(mutex: BfSharedMutex<T>) -> Self {
        RecursiveLock {
            inner: mutex,
            recursive_depth: Cell::new(0),
        }
    }

    delegate::delegate! {
        to self.inner {
            pub fn data_ptr(&self) -> *const T;
            pub fn is_locked(&self) -> bool;
            pub fn is_locked_exclusive(&self) -> bool;
            pub fn write(&self) -> Result<BfSharedMutexWriteGuard<'_, T>, Box<dyn Error + '_>>;
            pub fn read(&self) -> Result<BfSharedMutexReadGuard<'_, T>, Box<dyn Error + '_>>;
        }
    }

    /// Acquires a read lock on the mutex, allowing for recursive read locking.
    pub fn read_recursive<'a>(&'a self) -> Result<RecursiveLockReadGuard<'a, T>, Box<dyn Error + 'a>> {
        if self.recursive_depth.get() == 0 {
            // If we are not already holding a read lock, we acquire one.
            // Acquire the read guard, but forget it to prevent it from being dropped.
            mem::forget(self.inner.read());
            Ok(RecursiveLockReadGuard { mutex: self })
        } else {
            // If we are already holding a read lock, we just increment the depth.
            self.recursive_depth.set(self.recursive_depth.get() + 1);
            Ok(RecursiveLockReadGuard { mutex: self })
        }
    }
}

#[must_use = "Dropping the guard unlocks the recursive lock immediately"]
pub struct RecursiveLockReadGuard<'a, T> {
    mutex: &'a RecursiveLock<T>,
}

/// Allow dereferences the underlying object.
impl<T> Deref for RecursiveLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be shared guards, which only provide immutable access to the object.
        unsafe {
            debug_assert!(
                !self.mutex.inner.data_ptr().is_null(),
                "Data pointer should not be null"
            );
            self.mutex.inner.data_ptr().as_ref().unwrap_unchecked()
        }
    }
}

impl<T> Drop for RecursiveLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.recursive_depth.set(self.mutex.recursive_depth.get() - 1);
        if self.mutex.recursive_depth.get() == 0 {
            // If we are not holding a read lock anymore, we release the mutex.
            // This will allow other threads to acquire a read lock.
            unsafe {
                self.mutex.inner.create_read_guard_unchecked();
            }
        }
    }
}
