use std::{marker::PhantomData, ptr::NonNull};



/// This is adapted from the `erasable` crate, but actually allows one to pass an `?Sized` type that stores its length inline. For example types implementing the `SliceDst` trait.
pub struct Thin<T: ?Sized + ErasablePtr> {
    ptr: NonNull<ErasedPtr>,
    marker: PhantomData<fn() -> T>,
}

const _: () = assert!(std::mem::size_of::<ErasedPtr>() == std::mem::size_of::<usize>());

impl<T: ?Sized + ErasablePtr> Thin<T> {
    pub fn as_ptr(&self) -> *mut T {
        unsafe { 
            let result = self.ptr.as_mut().unerase();
        }
    }
}



pub unsafe trait ErasablePtr {
    /// Turn this erasable pointer into an erased pointer.
    ///
    /// To retrieve the original pointer, use `unerase`.
    fn erase(this: Self) -> ErasedPtr;


    /// Unerase this erased pointer.
    ///
    /// # Safety
    ///
    /// The erased pointer must have been created by `erase`.
    unsafe fn unerase(this: ErasedPtr) -> Self;
}


#[doc(hidden)]
/// This module is a trick to avoid warnings about private types in public interfaces.
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