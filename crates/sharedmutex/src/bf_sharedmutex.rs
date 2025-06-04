use std::cell::UnsafeCell;
use std::error::Error;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use crossbeam_utils::CachePadded;

/// A shared mutex (readers-writer lock) implementation based on the so-called
/// busy-forbidden protocol. Instead of a regular Mutex this class is Send and
/// not Sync, every thread must acquire a clone of the shared mutex and the
/// cloned instances of the same shared mutex guarantee shared access through
/// the `read` operation and exclusive access for the `write` operation of the
/// given object.
pub struct BfSharedMutex<T> {
    /// The local control bits of each instance. TODO: Maybe use pin to share the control bits among shared mutexes.
    control: Arc<CachePadded<SharedMutexControl>>,

    /// Index into the `other` table.
    index: usize,

    /// Information shared between all clones.
    shared: Arc<CachePadded<SharedData<T>>>,
}

// Can only be send, but is not sync
unsafe impl<T> Send for BfSharedMutex<T> {}

/// The busy and forbidden flags used to implement the protocol.
#[derive(Default)]
struct SharedMutexControl {
    busy: AtomicBool,
    forbidden: AtomicBool,
}

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
        // Register a new instance in the other list.
        let control = Arc::new(CachePadded::new(SharedMutexControl::default()));

        let mut other = self.shared.other.lock().expect("Failed to lock mutex");
        other.push(Some(control.clone()));

        Self {
            control,
            index: other.len() - 1,
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for BfSharedMutex<T> {
    fn drop(&mut self) {
        let mut other = self.shared.other.lock().expect("Failed to lock mutex");

        // Remove ourselves from the table.
        other[self.index] = None;
    }
}

/// The guard object for exclusive access to the underlying object.
#[must_use = "Dropping the guard unlocks the shared mutex immediately"]
pub struct BfSharedMutexWriteGuard<'a, T> {
    mutex: &'a BfSharedMutex<T>,
    guard: MutexGuard<'a, Vec<Option<Arc<CachePadded<SharedMutexControl>>>>>,
}

/// Allow dereferencing the underlying object.
impl<T> Deref for BfSharedMutexWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // We are the only guard after `write()`, so we can provide immutable access to the underlying object. (No mutable references the guard can exist)
        unsafe { &*self.mutex.shared.object.get() }
    }
}

impl<T> DerefMut for BfSharedMutexWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.shared.object.get() }
    }
}

impl<T> Drop for BfSharedMutexWriteGuard<'_, T> {
    fn drop(&mut self) {
        // Allow other threads to acquire access to the shared mutex.
        for control in self.guard.iter().flatten() {
            control.forbidden.store(false, std::sync::atomic::Ordering::SeqCst);
        }

        // The mutex guard is then dropped here.
    }
}

pub struct BfSharedMutexReadGuard<'a, T> {
    mutex: &'a BfSharedMutex<T>,
}

/// Allow dereferences the underlying object.
impl<T> Deref for BfSharedMutexReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be shared guards, which only provide immutable access to the object.
        unsafe { &*self.mutex.shared.object.get() }
    }
}

impl<T> Drop for BfSharedMutexReadGuard<'_, T> {
    fn drop(&mut self) {
        debug_assert!(
            self.mutex.control.busy.load(Ordering::SeqCst),
            "Cannot unlock shared lock that was not acquired"
        );

        self.mutex.control.busy.store(false, Ordering::SeqCst);
    }
}

impl<T> BfSharedMutex<T> {
    /// Provides read access to the underlying object, allowing multiple immutable references to it.
    #[inline]
    pub fn read<'a>(&'a self) -> Result<BfSharedMutexReadGuard<'a, T>, Box<dyn Error + 'a>> {
        debug_assert!(
            !self.control.busy.load(Ordering::SeqCst),
            "Cannot acquire read access again inside a reader section"
        );

        self.control.busy.store(true, Ordering::SeqCst);
        while self.control.forbidden.load(Ordering::SeqCst) {
            self.control.busy.store(false, Ordering::SeqCst);

            // Wait for the mutex of the writer.
            let mut _guard = self.shared.other.lock()?;

            self.control.busy.store(true, Ordering::SeqCst);
        }

        // We now have immutable access to the object due to the protocol.
        Ok(BfSharedMutexReadGuard { mutex: self })
    }

    /// Provide write access to the underlying object, only a single mutable reference to the object exists.
    #[inline]
    pub fn write<'a>(&'a self) -> Result<BfSharedMutexWriteGuard<'a, T>, Box<dyn Error + 'a>> {
        let other = self.shared.other.lock()?;

        debug_assert!(
            !self.control.busy.load(std::sync::atomic::Ordering::SeqCst),
            "Can only exclusive lock outside of a shared lock, no upgrading!"
        );
        debug_assert!(
            !self.control.forbidden.load(std::sync::atomic::Ordering::SeqCst),
            "Can not acquire exclusive lock inside of exclusive section"
        );

        // Make all instances wait due to forbidden access.
        for control in other.iter().flatten() {
            debug_assert!(
                !control.forbidden.load(std::sync::atomic::Ordering::SeqCst),
                "Other instance is already forbidden, this cannot happen"
            );

            control.forbidden.store(true, std::sync::atomic::Ordering::SeqCst);
        }

        // Wait for the instances to exit their busy status.
        for (index, option) in other.iter().enumerate() {
            if index != self.index {
                if let Some(object) = option {
                    while object.busy.load(std::sync::atomic::Ordering::SeqCst) {
                        std::hint::spin_loop();
                    }
                }
            }
        }

        // We now have exclusive access to the object according to the protocol
        Ok(BfSharedMutexWriteGuard {
            mutex: self,
            guard: other,
        })
    }

    /// Obtain mutable access to the object without locking, is safe because we have mutable access.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.shared.object.get() }
    }
}

impl<T: Debug> Debug for BfSharedMutex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entry(&"busy", &self.control.busy.load(Ordering::SeqCst))
            .entry(&"forbidden", &self.control.forbidden.load(Ordering::SeqCst))
            .entry(&"index", &self.index)
            .entry(&"len(other)", &self.shared.other.lock().unwrap().len())
            .finish()?;

        writeln!(f)?;
        writeln!(f, "other values: [")?;
        for control in self.shared.other.lock().unwrap().iter().flatten() {
            f.debug_map()
                .entry(&"busy", &control.busy.load(Ordering::SeqCst))
                .entry(&"forbidden", &control.forbidden.load(Ordering::SeqCst))
                .finish()?;
            writeln!(f)?;
        }

        writeln!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use std::hint::black_box;
    use crate::bf_sharedmutex::BfSharedMutex;

    use mcrl3_utilities::random_test_threads;
    use mcrl3_utilities::test_threads;

    // These are just simple tests.
    #[test]
    fn test_random_shared_mutex_exclusive() {
        let shared_number = BfSharedMutex::new(5);
        let num_iterations = 500;
        let num_threads = 20;

        test_threads(num_threads, || shared_number.clone(), move |number| {
            for _ in 0..num_iterations {
                *number.write().unwrap() += 5;
            }
        });

        assert_eq!(*shared_number.write().unwrap(), num_threads * num_iterations * 5 + 5);
    }

    #[test]
    fn test_random_shared() {
        let shared_vector = BfSharedMutex::new(vec![]);

        let num_threads = 20;
        let num_iterations = 5000;

        random_test_threads(num_iterations, num_threads, || shared_vector.clone(), |rng, shared_vector| {
            if rng.random_bool(0.95) {
                // Read a random index.
                let read = shared_vector.read().unwrap();
                if read.len() > 0 {
                    let index = rng.random_range(0..read.len());
                    black_box(assert_eq!(read[index], 5));
                }
            } else {
                // Add a new vector element.
                shared_vector.write().unwrap().push(5);
            }
        });
    }
}
