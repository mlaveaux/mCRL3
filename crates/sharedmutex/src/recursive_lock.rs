//! Authors: Maurice Laveaux, Flip van Spaendonck and Jan Friso Groote

use std::cell::Cell;
use std::error::Error;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::BfSharedMutex;
use crate::BfSharedMutexReadGuard;
use crate::BfSharedMutexWriteGuard;

/// An extension of the [BfSharedMutex] that allows recursive read locking without deadlocks.
pub struct RecursiveLock<T> {
    inner: BfSharedMutex<T>,

    /// The number of times the current thread has read locked the mutex.
    recursive_depth: Cell<usize>,

    /// The number of calls to the write() method.
    write_calls: Cell<usize>,

    /// The number of calls to the read_recursive() method.
    read_recursive_calls: Cell<usize>,
}

impl<T> RecursiveLock<T> {
    /// Creates a new `RecursiveLock` with the given data.
    pub fn new(data: T) -> Self {
        RecursiveLock {
            inner: BfSharedMutex::new(data),
            recursive_depth: Cell::new(0),
            write_calls: Cell::new(0),
            read_recursive_calls: Cell::new(0),
        }
    }

    /// Creates a new `RecursiveLock` from an existing `BfSharedMutex`.
    pub fn from_mutex(mutex: BfSharedMutex<T>) -> Self {
        RecursiveLock {
            inner: mutex,
            recursive_depth: Cell::new(0),
            write_calls: Cell::new(0),
            read_recursive_calls: Cell::new(0),
        }
    }

    delegate::delegate! {
        to self.inner {
            #[cfg(not(loom))]
            pub fn data_ptr(&self) -> *const T;
            #[cfg(loom)]
            pub fn data_ptr(&self) -> loom::cell::ConstPtr<T>;
            pub fn is_locked(&self) -> bool;
            pub fn is_locked_exclusive(&self) -> bool;
        }
    }

    /// Acquires a write lock on the mutex.
    ///
    /// # Panics
    ///
    /// Panics when called inside a read or write section. In that case the underlying mutex
    /// would not wait for this thread's own lock, handing out `&mut T` while a `&T` or another
    /// `&mut T` is live.
    pub fn write(&self) -> Result<RecursiveLockWriteGuard<'_, T>, Box<dyn Error + '_>> {
        assert!(
            self.recursive_depth.get() == 0,
            "Cannot call write() inside an existing read or write section"
        );
        self.write_calls.set(self.write_calls.get() + 1);
        self.recursive_depth.set(1);
        Ok(RecursiveLockWriteGuard {
            mutex: self,
            guard: self.inner.write()?,
            #[cfg(loom)]
            ptr: self.inner.data_ptr(),
        })
    }

    /// Acquires a read lock on the mutex.
    ///
    /// # Panics
    ///
    /// Panics when called inside a read or write section; the raw read lock is not reentrant.
    /// Use [`RecursiveLock::read_recursive`] instead.
    pub fn read(&self) -> Result<BfSharedMutexReadGuard<'_, T>, Box<dyn Error + '_>> {
        assert!(
            self.recursive_depth.get() == 0,
            "Cannot call read() inside an existing read or write section"
        );
        self.inner.read()
    }

    /// Acquires a read lock on the mutex, allowing for recursive read locking.
    ///
    /// May also be called inside a write section: the returned guard then borrows the write
    /// lock instead of acquiring the underlying mutex. While such a guard is alive, mutating
    /// through the [`RecursiveLockWriteGuard`] panics.
    pub fn read_recursive<'a>(&'a self) -> Result<RecursiveLockReadGuard<'a, T>, Box<dyn Error + 'a>> {
        self.read_recursive_calls.set(self.read_recursive_calls.get() + 1);
        if self.recursive_depth.get() == 0 {
            // If we are not already holding a read lock, we acquire one.
            // Acquire the read guard, but forget it to prevent it from being dropped.
            self.recursive_depth.set(1);
            mem::forget(self.inner.read()?);
            Ok(RecursiveLockReadGuard {
                mutex: self,
                #[cfg(loom)]
                ptr: self.inner.data_ptr(),
            })
        } else {
            // If we are already holding a read lock, we just increment the depth.
            self.recursive_depth.set(self.recursive_depth.get() + 1);
            Ok(RecursiveLockReadGuard {
                mutex: self,
                #[cfg(loom)]
                ptr: self.inner.data_ptr(),
            })
        }
    }

    /// Returns the number of times `write()` has been called.
    pub fn write_call_count(&self) -> usize {
        self.write_calls.get()
    }

    /// Returns the number of times `read_recursive()` has been called.
    pub fn read_recursive_call_count(&self) -> usize {
        self.read_recursive_calls.get()
    }
}

#[must_use = "Dropping the guard unlocks the recursive lock immediately"]
pub struct RecursiveLockReadGuard<'a, T> {
    mutex: &'a RecursiveLock<T>,

    #[cfg(loom)]
    ptr: loom::cell::ConstPtr<T>,
}

impl<T> RecursiveLockReadGuard<'_, T> {
    /// Returns the read depth of the recursive lock.
    pub fn read_depth(&self) -> usize {
        self.mutex.recursive_depth.get()
    }
}

/// Allow dereferences the underlying object.
impl<T> Deref for RecursiveLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be shared guards, which only provide immutable access to the object.
        #[cfg(not(loom))]
        unsafe {
            self.mutex.inner.data_ptr().as_ref().unwrap_unchecked()
        }

        #[cfg(loom)]
        unsafe {
            self.ptr.deref()
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
                // Drop the guard immediately to release busy=false via its Drop impl.
                let _ = self.mutex.inner.create_read_guard_unchecked();
            }
        }
    }
}

#[must_use = "Dropping the guard unlocks the recursive lock immediately"]
pub struct RecursiveLockWriteGuard<'a, T> {
    mutex: &'a RecursiveLock<T>,

    guard: BfSharedMutexWriteGuard<'a, T>,

    #[cfg(loom)]
    ptr: loom::cell::ConstPtr<T>,
}

/// Allow dereferences the underlying object.
impl<T> Deref for RecursiveLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // We hold the write guard, so immutable access is safe.
        #[cfg(loom)]
        unsafe {
            return self.ptr.deref();
        }

        #[cfg(not(loom))]
        self.guard.deref()
    }
}

/// Allow dereferences the underlying object.
impl<T> DerefMut for RecursiveLockWriteGuard<'_, T> {
    /// # Panics
    ///
    /// Panics while a recursive read guard taken inside this write section is alive. Such a
    /// guard hands out `&T` derived from the data pointer, invisible to the borrow checker,
    /// so a `&mut T` would alias it.
    fn deref_mut(&mut self) -> &mut Self::Target {
        assert!(
            self.mutex.recursive_depth.get() == 1,
            "Cannot mutate through RecursiveLockWriteGuard while recursive read guards from its write section are alive"
        );
        // We hold the write guard exclusively and no recursive read guards exist, so mutable
        // access is safe.
        self.guard.deref_mut()
    }
}

impl<T> Drop for RecursiveLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        // Read guards taken with `read_recursive()` inside this write section borrow the write
        // lock: once this guard drops, the underlying mutex is released and their `&T` would be
        // unprotected. Panic instead of silently allowing that use-after-unlock.
        assert!(
            self.mutex.recursive_depth.get() == 1,
            "RecursiveLockWriteGuard dropped while recursive read guards from its write section are still alive"
        );
        self.mutex.recursive_depth.set(0);
    }
}

#[cfg(test)]
mod tests {
    use crate::BfSharedMutex;
    use crate::RecursiveLock;

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

    #[test]
    fn test_read_recursive_inside_write() {
        let lock = RecursiveLock::new(42);
        let mut write = lock.write().unwrap();
        *write += 1;

        // Piggybacks on the write lock instead of acquiring the underlying mutex.
        let read = lock.read_recursive().unwrap();
        assert_eq!(*read, 43);
        assert_eq!(read.read_depth(), 2);
        drop(read);

        // Mutation is allowed again once the read guard is gone.
        *write += 1;
        assert_eq!(*write, 44);
        drop(write);

        assert_eq!(*lock.read().unwrap(), 44);
    }

    #[test]
    fn test_write_call_counter() {
        let lock = RecursiveLock::new(42);

        // Initially, the counter should be 0
        assert_eq!(lock.write_call_count(), 0);

        // After one write call, counter should be 1
        {
            let _guard = lock.write().unwrap();
            assert_eq!(lock.write_call_count(), 1);
        }

        // After another write call, counter should be 2
        {
            let _guard = lock.write().unwrap();
            assert_eq!(lock.write_call_count(), 2);
        }

        // Counter should remain 2
        assert_eq!(lock.write_call_count(), 2);
    }

    #[test]
    fn test_read_recursive_call_counter() {
        let lock = RecursiveLock::new(42);

        // Initially, the counter should be 0
        assert_eq!(lock.read_recursive_call_count(), 0);

        // After one read_recursive call, counter should be 1
        {
            let _guard = lock.read_recursive().unwrap();
            assert_eq!(lock.read_recursive_call_count(), 1);
        }

        // After another read_recursive call, counter should be 2
        {
            let _guard = lock.read_recursive().unwrap();
            assert_eq!(lock.read_recursive_call_count(), 2);
        }

        // Test nested recursive reads increment the counter
        {
            let _guard1 = lock.read_recursive().unwrap();
            assert_eq!(lock.read_recursive_call_count(), 3);

            let _guard2 = lock.read_recursive().unwrap();
            assert_eq!(lock.read_recursive_call_count(), 4);
        }

        // Counter should remain 4
        assert_eq!(lock.read_recursive_call_count(), 4);
    }

    #[test]
    fn test_both_counters() {
        let lock = RecursiveLock::new(42);

        // Initially, both counters should be 0
        assert_eq!(lock.write_call_count(), 0);
        assert_eq!(lock.read_recursive_call_count(), 0);

        // Call write and check counters
        {
            let _guard = lock.write().unwrap();
            assert_eq!(lock.write_call_count(), 1);
            assert_eq!(lock.read_recursive_call_count(), 0);
        }

        // Call read_recursive and check counters
        {
            let _guard = lock.read_recursive().unwrap();
            assert_eq!(lock.write_call_count(), 1);
            assert_eq!(lock.read_recursive_call_count(), 1);
        }

        // Call write again
        {
            let _guard = lock.write().unwrap();
            assert_eq!(lock.write_call_count(), 2);
            assert_eq!(lock.read_recursive_call_count(), 1);
        }

        // Call read_recursive multiple times
        {
            let _guard1 = lock.read_recursive().unwrap();
            let _guard2 = lock.read_recursive().unwrap();
            assert_eq!(lock.write_call_count(), 2);
            assert_eq!(lock.read_recursive_call_count(), 3);
        }
    }
}
