//! This is adapted from the `erasable` crate, but actually allows one to pass an `?Sized` type that stores its length inline. For example types implementing the `SliceDst` trait.

use std::{marker::PhantomData, ptr::NonNull};

pub struct Thin<T: ?Sized + Erasable> {
    ptr: NonNull<ErasedPtr>,
    marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized + Erasable> Copy for Thin<T> {}

impl<T: ?Sized + Erasable> Clone for Thin<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr.clone(), marker: self.marker.clone() }
    }
}

impl<T: ?Sized + Erasable> Thin<T> {
    pub fn new(ptr: NonNull<T>) -> Self {
        unreachable!();
    }
}

unsafe impl<T: Sized> Erasable for T {
    fn erase(this: NonNull<Self>) -> ErasedPtr {
        todo!()
    }

    unsafe fn unerase(this: ErasedPtr) -> NonNull<Self> {
        todo!()
    }
}

const _: () = assert!(std::mem::size_of::<ErasedPtr>() == std::mem::size_of::<usize>());

impl<T: ?Sized + Erasable> Thin<T> {
    pub fn as_ptr(&self) -> *mut T {
        unreachable!();
    }

    pub fn as_nonnull(&self) -> NonNull<T> {
        unreachable!();
    }

    pub unsafe fn as_ref(&self) -> &T {
        unreachable!();
    }
}


pub unsafe trait Erasable {
    /// Turn this erasable pointer into an erased pointer.
    ///
    /// To retrieve the original pointer, use `unerase`.
    fn erase(this: NonNull<Self>) -> ErasedPtr;

    /// Unerase this erased pointer.
    ///
    /// # Safety
    ///
    /// The erased pointer must have been created by `erase`.
    unsafe fn unerase(this: ErasedPtr) -> NonNull<Self>;
}


#[doc(hidden)]
pub mod priv_in_pub {
    /// This is simply a u8, but with a concrete type to avoid confusion. Must be a type that has size one and alignment one.
    pub struct Erased(u8);
}

/// A thin, type-erased pointer.
///
/// The `Erased` type is private, and should be treated as an opaque type.
/// When `extern type` is stabilized, `Erased` will be defined as one.
///
/// The current implementation uses a `struct Erased` with size 0 and align 1.
/// If you want to offset the pointer, make sure to cast to a `u8` or other known type pointer first.
/// When `Erased` becomes an extern type, it will properly have unknown size and align.
pub type ErasedPtr = NonNull<priv_in_pub::Erased>;