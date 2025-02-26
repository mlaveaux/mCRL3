use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

use mcrl3_sharedmutex::BfSharedMutexReadGuard;
use mcrl3_sharedmutex::BfSharedMutexWriteGuard;

use crate::THREAD_TERM_POOL;

/// Provides access to the mCRL2 busy forbidden protocol, where there
/// are thread-local busy flags and one central storage for the forbidden
/// flags.
pub struct BfTermPool<T: ?Sized> {
    object: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for BfTermPool<T> {}
unsafe impl<T: Send> Sync for BfTermPool<T> {}

impl<T> BfTermPool<T> {
    pub fn new(object: T) -> BfTermPool<T> {
        BfTermPool {
            object: UnsafeCell::new(object),
        }
    }
}

impl<'a, T: ?Sized> BfTermPool<T> {
    /// Provides read access to the underlying object.
    pub fn read(&'a self) -> BfTermPoolRead<'a, T> {
        unimplemented!();
    }

    /// Provides write access to the underlying object.
    pub fn write(&'a self) -> BfTermPoolWrite<'a, T> {
        unimplemented!();
    }

    /// Provides read access to the underlying object.
    ///
    /// # Safety
    ///
    /// Assumes that we are in an exclusive section.
    pub unsafe fn get(&'a self) -> &'a T {
        unsafe { &*self.object.get() }
    }

    /// Provides write access to the underlying object
    ///
    /// # Safety
    ///
    /// Provides mutable access given that other threads use [write] and [read]
    /// exclusively. If we are already in an exclusive context then lock can be
    /// set to false.
    pub unsafe fn write_exclusive(&'a self) -> BfTermPoolThreadWrite<'a, T> {
        BfTermPoolThreadWrite {
            mutex: self,
            locked: true,
            _marker: Default::default(),
        }
    }
}

pub struct BfTermPoolRead<'a, T: ?Sized> {
    mutex: &'a BfTermPool<T>,
    _guard: BfSharedMutexReadGuard<'a, ()>,
    _marker: PhantomData<&'a ()>,
}

impl<T: ?Sized> Deref for BfTermPoolRead<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be read guards.
        unsafe { &*self.mutex.object.get() }
    }
}

pub struct BfTermPoolWrite<'a, T: ?Sized> {
    mutex: &'a BfTermPool<T>,
    _guard: BfSharedMutexWriteGuard<'a, ()>,
    _marker: PhantomData<&'a ()>,
}

impl<T: ?Sized> Deref for BfTermPoolWrite<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be read guards.
        unsafe { &*self.mutex.object.get() }
    }
}

impl<T: ?Sized> DerefMut for BfTermPoolWrite<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.object.get() }
    }
}
pub struct BfTermPoolThreadWrite<'a, T: ?Sized> {
    mutex: &'a BfTermPool<T>,
    locked: bool,
    _marker: PhantomData<&'a ()>,
}

impl<T: ?Sized> BfTermPoolThreadWrite<'_, T> {
    /// Unlocks the guard prematurely, but returns whether the shared section was actually left.
    pub fn unlock(&mut self) -> bool {
        if self.locked {
            self.locked = false;
            true
        } else {
            false
        }
    }
}

impl<T: ?Sized> Deref for BfTermPoolThreadWrite<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be read guards.
        unsafe { &*self.mutex.object.get() }
    }
}

impl<T: ?Sized> DerefMut for BfTermPoolThreadWrite<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.object.get() }
    }
}

impl<T: ?Sized> Drop for BfTermPoolThreadWrite<'_, T> {
    fn drop(&mut self) {
        if self.locked {
        }
    }
}