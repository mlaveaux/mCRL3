use std::{cell::Cell, error::Error, mem, ops::Deref};

use crate::{BfSharedMutex, BfSharedMutexWriteGuard};



/// An extension of the `SharedMutex` that allows recursive read locking without deadlocks.
struct RecursiveLock<T> {
    mutex: BfSharedMutex<T>,

    /// The number of times the current thread has read locked the mutex.
    recursive_depth: Cell<usize>,
}

impl<T> RecursiveLock<T> {
    /// The same as `BfSharedMutex::write`.
    fn write<'a>(&'a self) -> Result<BfSharedMutexWriteGuard<'a, T>, Box<dyn Error + 'a>> {
        self.mutex.write()
    }

    /// Acquires a read lock on the mutex, allowing for recursive read locking.
    fn read_recursive<'a>(&'a self) -> Result<RecursiveLockReadGuard<'a, T>, Box<dyn Error + 'a>> {
        if self.recursive_depth.get() == 0 {
            // If we are not already holding a read lock, we acquire one.
            // Acquire the read guard, but forget it to prevent it from being dropped.
            mem::forget(self.mutex.read());
            Ok(RecursiveLockReadGuard {
                mutex: self,
            })
        } else {
            // If we are already holding a read lock, we just increment the depth.
            self.recursive_depth.set(self.recursive_depth.get() + 1);
            Ok(RecursiveLockReadGuard {
                mutex: self,
            })
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
            debug_assert!(!self.mutex.mutex.data_ptr().is_null(), "Data pointer should not be null");
            self.mutex.mutex.data_ptr().as_ref().unwrap_unchecked()
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
                self.mutex.mutex.create_read_guard_unchecked();
            }
        }
    }
}

