//! Authors: Maurice Laveaux, Flip van Spaendonck and Jan Friso Groote

use std::error::Error;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

#[cfg(not(loom))]
mod inner {
    pub use std::cell::UnsafeCell;
    pub use std::hint::spin_loop;
    pub use std::sync::Arc;
    pub use std::sync::Mutex;
    pub use std::sync::MutexGuard;
    pub use std::sync::atomic::AtomicBool;
    pub use std::sync::atomic::Ordering;
    pub use std::sync::atomic::fence;
}

// We replace the standard implementation by loom's implementation.
#[cfg(loom)]
mod inner {
    pub use std::mem::ManuallyDrop;

    pub use loom::cell::UnsafeCell;
    pub use loom::hint::spin_loop;
    pub use loom::sync::Arc;
    pub use loom::sync::Mutex;
    pub use loom::sync::MutexGuard;
    pub use loom::sync::atomic::AtomicBool;
    pub use loom::sync::atomic::Ordering;
    pub use loom::sync::atomic::fence;
}

use inner::*;

use crossbeam_utils::CachePadded;

/// A shared mutex (readers-writer lock) implementation based on the so-called
/// busy-forbidden protocol.
///
/// # Details
///
/// Compared to a regular [std::sync::Mutex] this struct is Send but not Sync.
/// This means that every thread must acquire a clone of the shared mutex and
/// the cloned instances of the same shared mutex guarantee shared access
/// through the `read` operation and exclusive access for the `write` operation
/// of the given object.
pub struct BfSharedMutex<T> {
    /// The local control bits of each instance.
    ///
    /// TODO: Maybe use pin to share the control bits among shared mutexes.
    control: Arc<CachePadded<SharedMutexControl>>,

    /// Index into the `other` table.
    index: usize,

    /// Information shared between all clones.
    shared: Arc<CachePadded<SharedData<T>>>,
}

// SAFETY: Sending a BfSharedMutex to another thread transfers ownership of the
// protected T.
unsafe impl<T: Send> Send for BfSharedMutex<T> {}

/// The busy and forbidden flags used to implement the protocol.
#[derive(Default)]
struct SharedMutexControl {
    busy: AtomicBool,
    forbidden: AtomicBool,
}

/// The shared data between all instances of the shared mutex.
struct SharedData<T> {
    /// The object that is being protected.
    object: UnsafeCell<T>,

    /// The list of all the shared mutex instances.
    other: Mutex<Vec<Option<Arc<CachePadded<SharedMutexControl>>>>>,
}

impl<T> BfSharedMutex<T> {
    /// Constructs a new shared mutex for protecting access to the given object.
    pub fn new(object: T) -> Self {
        let control = Arc::new(CachePadded::new(SharedMutexControl::default()));

        Self {
            control: control.clone(),
            shared: Arc::new(CachePadded::new(SharedData {
                object: UnsafeCell::new(object),
                other: Mutex::new(vec![Some(control.clone())]),
            })),
            index: 0,
        }
    }
}

impl<T> Clone for BfSharedMutex<T> {
    fn clone(&self) -> Self {
        // Register a new instance in the other list
        let control = Arc::new(CachePadded::new(SharedMutexControl::default()));

        let mut other = self.shared.other.lock().expect("Failed to lock mutex");
        let index = match other.iter().position(|slot| slot.is_none()) {
            Some(index) => {
                // Reuse an empty slot if available.
                other[index] = Some(control.clone());
                index
            }
            None => {
                other.push(Some(control.clone()));
                other.len() - 1
            }
        };

        Self {
            control,
            index,
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for BfSharedMutex<T> {
    fn drop(&mut self) {
        let mut other = self.shared.other.lock().expect("Failed to lock mutex");

        // Remove ourselves from the table.
        other[self.index] = None;

        // Trim trailing None slots to keep the vec compact.
        while other.last().is_some_and(|slot| slot.is_none()) {
            other.pop();
        }
    }
}

/// The guard object for exclusive access to the underlying object.
#[must_use = "Dropping the guard unlocks the shared mutex immediately"]
pub struct BfSharedMutexWriteGuard<'a, T> {
    #[allow(dead_code)]
    mutex: &'a BfSharedMutex<T>,

    guard: MutexGuard<'a, Vec<Option<Arc<CachePadded<SharedMutexControl>>>>>,

    /// When loom is enabled, we store a write reference tracked by Loom.
    #[cfg(loom)]
    ptr: ManuallyDrop<loom::cell::MutPtr<T>>,
}

/// Allow dereferencing the underlying object.
impl<T> Deref for BfSharedMutexWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // We are the only guard after `write()`, so we can provide immutable access to the underlying object. (No mutable references the guard can exist)
        #[cfg(not(loom))]
        unsafe {
            &*self.mutex.shared.object.get()
        }

        #[cfg(loom)]
        unsafe {
            self.ptr.deref().deref()
        }
    }
}

impl<T> DerefMut for BfSharedMutexWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        #[cfg(not(loom))]
        unsafe {
            &mut *self.mutex.shared.object.get()
        }

        #[cfg(loom)]
        unsafe {
            self.ptr.deref().deref()
        }
    }
}

impl<T> Drop for BfSharedMutexWriteGuard<'_, T> {
    fn drop(&mut self) {
        // End Loom's write tracking before releasing the protocol.
        #[cfg(loom)]
        unsafe {
            ManuallyDrop::drop(&mut self.ptr);
        }

        // Allow other threads to acquire access to the shared mutex.
        for control in self.guard.iter().flatten() {
            control.forbidden.store(false, Ordering::Release);
        }

        // The mutex guard is then dropped here.
    }
}

// SAFETY: Sharing &WriteGuard across threads only exposes &T.
unsafe impl<T: Sync> Sync for BfSharedMutexWriteGuard<'_, T> {}

#[must_use = "Dropping the guard unlocks the shared mutex immediately"]
pub struct BfSharedMutexReadGuard<'a, T> {
    mutex: &'a BfSharedMutex<T>,

    /// When loom is enabled, we store a read reference tracked by Loom.
    #[cfg(loom)]
    ptr: ManuallyDrop<loom::cell::ConstPtr<T>>,
}

// SAFETY: Sharing &ReadGuard across threads only exposes &T.
unsafe impl<T: Sync> Sync for BfSharedMutexReadGuard<'_, T> {}

/// Allows dereferencing the underlying object.
impl<T> Deref for BfSharedMutexReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be shared guards, which only provide immutable access to the object.
        #[cfg(not(loom))]
        unsafe {
            &*self.mutex.shared.object.get()
        }

        #[cfg(loom)]
        unsafe {
            self.ptr.deref().deref()
        }
    }
}

impl<T> Drop for BfSharedMutexReadGuard<'_, T> {
    fn drop(&mut self) {
        debug_assert!(
            self.mutex.control.busy.load(Ordering::Relaxed),
            "Cannot unlock shared lock that was not acquired"
        );

        // End Loom's read tracking before releasing the protocol.
        #[cfg(loom)]
        unsafe {
            ManuallyDrop::drop(&mut self.ptr);
        }

        // Release is sufficient, this synchronises the writes of this thread with writer.
        self.mutex.control.busy.store(false, Ordering::Release);
    }
}

impl<T> BfSharedMutex<T> {
    /// Provides read access to the underlying object, allowing multiple immutable references to it.
    ///
    /// # Panics
    ///
    /// Panics when called while this instance already holds a read guard. Reentrant reads would
    /// corrupt the `busy` flag.
    pub fn read<'a>(&'a self) -> Result<BfSharedMutexReadGuard<'a, T>, Box<dyn Error + 'a>> {
        assert!(
            !self.control.busy.load(Ordering::Relaxed),
            "Cannot acquire read access again inside a reader section"
        );

        self.control.busy.store(true, Ordering::Relaxed);
        fence(Ordering::SeqCst);
        while self.control.forbidden.load(Ordering::Acquire) {
            // Signal the writer that this thread is no longer busy, allowing it to make progress.
            self.control.busy.store(false, Ordering::Relaxed);

            // For loom with spin locks we must ensure that other threads can make progress for fairness.
            #[cfg(loom)]
            loom::thread::yield_now();

            // Wait for the mutex of the writer.
            let _guard = self.shared.other.lock()?;

            // Allow another thread to acquire the lock.
            #[cfg(loom)]
            loom::thread::yield_now();

            self.control.busy.store(true, Ordering::Relaxed);

            // The busy store must become visible before the forbidden load is performed, exactly
            // as on the initial acquisition above.
            fence(Ordering::SeqCst);
        }

        // We now have immutable access to the object due to the protocol.
        Ok(BfSharedMutexReadGuard {
            mutex: self,
            #[cfg(loom)]
            ptr: ManuallyDrop::new(self.shared.object.get()),
        })
    }

    /// Creates a new `BfSharedMutexReadGuard` without checking if the lock is held.
    ///
    /// # Safety
    ///
    /// This method must only be called if the thread logically holds a read lock.
    ///
    /// This function does not increment the read count of the lock. Calling this function when a
    /// guard has already been produced is undefined behaviour unless the guard was forgotten
    /// with `mem::forget`.
    pub unsafe fn create_read_guard_unchecked(&self) -> BfSharedMutexReadGuard<'_, T> {
        BfSharedMutexReadGuard {
            mutex: self,
            #[cfg(loom)]
            ptr: ManuallyDrop::new(self.shared.object.get()),
        }
    }

    /// Returns a raw pointer to the underlying data.
    ///
    /// This is useful when combined with `mem::forget` to hold a lock without
    /// the need to maintain a [`BfSharedMutexReadGuard`] or [`BfSharedMutexWriteGuard`] object
    /// alive, for example when dealing with FFI.
    ///
    /// # Safety
    ///
    /// You must ensure that there are no data races when dereferencing the
    /// returned pointer, for example if the current thread logically owns a
    /// [`BfSharedMutexReadGuard`] or [`BfSharedMutexWriteGuard`] but that guard has been discarded
    /// using `mem::forget`.
    #[cfg(not(loom))]
    pub fn data_ptr(&self) -> *mut T {
        self.shared.object.get()
    }

    #[cfg(loom)]
    pub fn data_ptr(&self) -> loom::cell::ConstPtr<T> {
        self.shared.object.get()
    }

    /// Provide write access to the underlying object, only a single mutable reference to the object exists.
    pub fn write<'a>(&'a self) -> Result<BfSharedMutexWriteGuard<'a, T>, Box<dyn Error + 'a>> {
        let other = self.shared.other.lock()?;

        debug_assert!(
            !self.control.busy.load(Ordering::Relaxed),
            "Can only exclusive lock outside of a shared lock, no upgrading!"
        );
        debug_assert!(
            !self.control.forbidden.load(Ordering::Relaxed),
            "Can not acquire exclusive lock inside of exclusive section"
        );

        // Make all instances wait due to forbidden access.
        for control in other.iter().flatten() {
            debug_assert!(
                !control.forbidden.load(Ordering::Relaxed),
                "Other instance is already forbidden, this cannot happen"
            );

            control.forbidden.store(true, Ordering::Relaxed);
        }

        fence(Ordering::SeqCst);

        // Wait for the instances to exit their busy status.
        for (index, option) in other.iter().enumerate() {
            if index != self.index
                && let Some(object) = option
            {
                // We just synchronize with the busy store of the other instances.
                while object.busy.load(Ordering::Acquire) {
                    spin_loop();
                }
            }
        }

        // We now have exclusive access to the object according to the protocol
        Ok(BfSharedMutexWriteGuard {
            mutex: self,
            guard: other,
            #[cfg(loom)]
            ptr: ManuallyDrop::new(self.shared.object.get_mut()),
        })
    }

    /// Check if this instance's read lock is currently held (i.e., this instance's `busy` flag is set).
    ///
    /// Note: this only reflects the state of **this** clone of the shared mutex. Other clones
    /// may independently hold their own read locks.
    pub fn is_locked(&self) -> bool {
        self.control.busy.load(Ordering::Relaxed)
    }

    /// Check if this instance has been forbidden from acquiring a read lock, which indicates
    /// that another clone is holding or acquiring a write lock.
    pub fn is_locked_exclusive(&self) -> bool {
        self.control.forbidden.load(Ordering::Relaxed)
    }

    /// Obtain mutable access to the object without locking.
    ///
    /// Returns `None` when other clones of this shared mutex exist, since those clones can
    /// concurrently hold read or write guards on the shared object.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        {
            let other = self.shared.other.lock().expect("Failed to lock mutex");
            if other.iter().flatten().count() != 1 {
                return None;
            }
        }

        // SAFETY: The registration table contains only this instance, so no other handle exists
        // (guards borrow their handle, and new clones can only be created from a handle). Holding
        // `&mut self` therefore guarantees no guard is alive and none can be created.
        #[cfg(not(loom))]
        unsafe {
            Some(&mut *self.shared.object.get())
        }
        #[cfg(loom)]
        unsafe {
            Some(self.shared.object.get_mut().with(|ptr| &mut *ptr))
        }
    }
}

impl<T: Debug> Debug for BfSharedMutex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let other = self.shared.other.lock().unwrap();

        f.debug_map()
            .entry(&"busy", &self.control.busy.load(Ordering::Relaxed))
            .entry(&"forbidden", &self.control.forbidden.load(Ordering::Relaxed))
            .entry(&"index", &self.index)
            .entry(&"len(other)", &other.len())
            .finish()?;

        writeln!(f)?;
        writeln!(f, "other values: [")?;
        for control in other.iter().flatten() {
            f.debug_map()
                .entry(&"busy", &control.busy.load(Ordering::Relaxed))
                .entry(&"forbidden", &control.forbidden.load(Ordering::Relaxed))
                .finish()?;
            writeln!(f)?;
        }

        writeln!(f, "]")
    }
}

/// A global shared mutex that can be used to protect global data.
///
/// # Details
///
/// This is a wrapper around `BfSharedMutex` that provides a global instance
/// that can be used to protect global data. Must be cloned to obtain mutable
/// access.
pub struct GlobalBfSharedMutex<T> {
    /// The shared mutex that is used to protect the global data.
    shared_mutex: BfSharedMutex<T>,
}

impl<T> GlobalBfSharedMutex<T> {
    /// Constructs a new global shared mutex for protecting access to the given object.
    pub fn new(object: T) -> Self {
        Self {
            shared_mutex: BfSharedMutex::new(object),
        }
    }

    /// Returns a clone of the global shared mutex, which allows writing and reading.
    pub fn share(&self) -> BfSharedMutex<T> {
        self.shared_mutex.clone()
    }
}

// SAFETY: Sending a GlobalBfSharedMutex<T> to another thread requires T: Send, same as RwLock<T>.
unsafe impl<T: Send> Send for GlobalBfSharedMutex<T> {}

// SAFETY: Multiple threads holding &GlobalBfSharedMutex<T> can call share() concurrently
unsafe impl<T: Send + Sync> Sync for GlobalBfSharedMutex<T> {}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use rand::RngExt;

    use merc_utilities::random_test_threads;
    use merc_utilities::test_threads;

    use super::BfSharedMutex;

    // These are just simple tests.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_random_bf_shared_mutex_exclusive() {
        let shared_number = BfSharedMutex::new(5);
        let num_iterations = 500;
        let num_threads = 3;

        test_threads(
            num_threads,
            || shared_number.clone(),
            move |number| {
                for _ in 0..num_iterations {
                    *number.write().unwrap() += 5;
                }
            },
        );

        assert_eq!(*shared_number.write().unwrap(), num_threads * num_iterations * 5 + 5);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_random_bf_shared_mutex() {
        let shared_vector = BfSharedMutex::new(vec![]);

        let num_threads = 20;
        let num_iterations = 5000;

        random_test_threads(
            num_iterations,
            num_threads,
            || shared_vector.clone(),
            |rng, shared_vector| {
                if rng.random_bool(0.95) {
                    // Read a random index.
                    let read = shared_vector.read().unwrap();
                    if !read.is_empty() {
                        let index = rng.random_range(0..read.len());
                        assert_eq!(*black_box(&read[index]), 5);
                    }
                } else {
                    // Add a new vector element.
                    shared_vector.write().unwrap().push(5);
                }
            },
        );
    }

    #[test]
    #[cfg(loom)]
    fn test_loom_bf_shared_mutex() {
        let mut builder = loom::model::Builder::new();
        // A bound of at least 2 is needed to find the missing fence.
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let shared_mutex = BfSharedMutex::new(false);

            let threads: Vec<_> = (0..3)
                .map(|_| {
                    let shared_mutex = shared_mutex.clone();
                    loom::thread::spawn(move || {
                        // Just perform some operations on the shared mutex.
                        let result = *shared_mutex.read().unwrap();

                        *shared_mutex.write().unwrap() = !result;
                    })
                })
                .collect();

            for th in threads {
                th.join().unwrap();
            }
        });
    }
}
