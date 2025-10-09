use std::cell::Cell;
use std::error::Error;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

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
        }
    }

    /// Acquires a write lock on the mutex.
    pub fn write(&self) -> Result<RecursiveLockWriteGuard<'_, T>, Box<dyn Error + '_>> {
        debug_assert!(self.recursive_depth.get() == 0, "Cannot call write() inside a read section");
        self.recursive_depth.set(1);
        Ok(RecursiveLockWriteGuard { mutex: self, guard: self.inner.write()? })
    }
    
    /// Acquires a read lock on the mutex.
    pub fn read(&self) -> Result<BfSharedMutexReadGuard<'_, T>, Box<dyn Error + '_>> {
        debug_assert!(self.recursive_depth.get() == 0, "Cannot call read() inside a read section");
        self.inner.read()        
    }

    /// Acquires a read lock on the mutex, allowing for recursive read locking.
    pub fn read_recursive<'a>(&'a self) -> Result<RecursiveLockReadGuard<'a, T>, Box<dyn Error + 'a>> {
        if self.recursive_depth.get() == 0 {
            // If we are not already holding a read lock, we acquire one.
            // Acquire the read guard, but forget it to prevent it from being dropped.
            self.recursive_depth.set(1);
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


#[must_use = "Dropping the guard unlocks the recursive lock immediately"]
pub struct RecursiveLockWriteGuard<'a, T> {
    mutex: &'a RecursiveLock<T>,
    guard: BfSharedMutexWriteGuard<'a, T>,
}

/// Allow dereferences the underlying object.
impl<T> Deref for RecursiveLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be shared guards, which only provide immutable access to the object.
        self.guard.deref()
    }
}

/// Allow dereferences the underlying object.
impl<T> DerefMut for RecursiveLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // There can only be shared guards, which only provide immutable access to the object.
        self.guard.deref_mut()
    }
}

impl<T> Drop for RecursiveLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.recursive_depth.set(self.mutex.recursive_depth.get() - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_mutex() {
        let mutex = BfSharedMutex::new(100);
        let lock = RecursiveLock::from_mutex(mutex);
        assert_eq!(*lock.read().unwrap(), 100);
    }

    #[test]
    fn test_single_recursive_read() {
        let lock = RecursiveLock::new(42);
        let guard = lock.read_recursive().unwrap();
        assert_eq!(*guard, 42);
        assert_eq!(lock.recursive_depth.get(), 1);
    }

    #[test]
    fn test_nested_recursive_reads() {
        let lock = RecursiveLock::new(42);

        let guard1 = lock.read_recursive().unwrap();
        assert_eq!(*guard1, 42);
        assert_eq!(lock.recursive_depth.get(), 1);

        let guard2 = lock.read_recursive().unwrap();
        assert_eq!(*guard2, 42);
        assert_eq!(lock.recursive_depth.get(), 2);

        let guard3 = lock.read_recursive().unwrap();
        assert_eq!(*guard3, 42);
        assert_eq!(lock.recursive_depth.get(), 3);

        drop(guard3);
        assert_eq!(lock.recursive_depth.get(), 2);

        drop(guard2);
        assert_eq!(lock.recursive_depth.get(), 1);

        drop(guard1);
        assert_eq!(lock.recursive_depth.get(), 0);
    }
}
