//! Helper functions and structs to deal with dynamic sized types. In particular to deal with the `TermShared`.
//!
//! This code is adapted from the `slice-dst` crate, but supports the `Allocator` unstable api through the `allocator-api2` crate. Furthermore, removed all code that
//! we are not using anyway.

use std::alloc::Layout;
use std::alloc::LayoutError;
use std::ptr::NonNull;
use std::ptr::slice_from_raw_parts_mut;

use allocator_api2::alloc::AllocError;
use allocator_api2::alloc::Allocator;

/// This trait should be implemented by dynamic sized types.
///
/// # Safety
///
/// Implementing this trait requires various unsafe memory manipulations, and the layout/length must be correct. Otherwise it results in undefined behaviour.
pub unsafe trait SliceDst {
    /// Returns the layout of the slice containing `length` elements for this DST.
    fn layout_for(length: usize) -> Result<Layout, LayoutError>;

    /// Add the type on an untyped pointer
    fn retype(ptr: NonNull<[()]>) -> NonNull<Self>;

    /// The number of elements in this dynamic sized type. This information is necessary for deallocation.
    fn length(&self) -> usize;
}

/// Blanket implemented for Sized T.
unsafe impl<T> SliceDst for T {
    fn layout_for(_length: usize) -> Result<Layout, LayoutError> {
        Ok(Layout::new::<T>())
    }

    fn retype(ptr: NonNull<[()]>) -> NonNull<Self> {
        unsafe {
            let raw_ptr = ptr.as_ptr() as *mut Self;
            NonNull::new_unchecked(raw_ptr)
        }
    }

    fn length(&self) -> usize {
        0
    }
}

/// To calculate the layout of a [repr(C)] structure and the offsets of the fields from its fields’ layouts:
///
/// Copied from the `Layout` documentation.
pub fn repr_c<const N: usize>(fields: &[Layout; N]) -> Result<Layout, LayoutError> {
    let mut layout = Layout::from_size_align(0, 1)?;
    for &field in fields {
        let (new_layout, _offset) = layout.extend(field)?;
        layout = new_layout;
    }

    // Remember to finalize with `pad_to_align`!
    Ok(layout.pad_to_align())
}

/// A trait that can be used to extend `Allocator` implementations with the
/// ability to allocate (and deallocate) dynamically sized slices that implement
/// `SliceDst`.
///
/// # Safety
///
/// This trait is unsafe because it relies on the correct implementation of
/// `SliceDst` for proper memory layout and deallocation.
pub unsafe trait AllocatorDst {
    /// Allocate an object whose type implements `SliceDst`. The resulting memory is uninitialize.
    fn allocate_slice_dst<T: SliceDst + ?Sized>(&self, length: usize) -> Result<NonNull<T>, AllocError>;

    /// Deallocates an allocation returned by `allocate_slice_dst`.
    fn deallocate_slice_dst<T: ?Sized + SliceDst>(&self, ptr: NonNull<T>, length: usize);
}

unsafe impl<A: Allocator> AllocatorDst for A {
    fn allocate_slice_dst<T: SliceDst + ?Sized>(&self, length: usize) -> Result<NonNull<T>, AllocError> {
        let ptr = self.allocate(T::layout_for(length).expect("Invalid layout for SliceDst"))?;
        // Create a slice of the correct length for proper metadata
        let slice_ptr = unsafe { NonNull::new_unchecked(slice_from_raw_parts_mut(ptr.as_ptr() as *mut (), length)) };
        Ok(T::retype(slice_ptr))
    }

    fn deallocate_slice_dst<T: ?Sized + SliceDst>(&self, ptr: NonNull<T>, length: usize) {
        unsafe {
            self.deallocate(
                NonNull::new_unchecked(ptr.as_ptr() as *mut u8),
                T::layout_for(length).expect("Invalid layout for SliceDst"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use allocator_api2::alloc::Global;

    #[repr(C)]
    struct WithHeader<T> {
        length: usize,
        array: [T],
    }

    unsafe impl<T> SliceDst for WithHeader<T> {
        fn layout_for(length: usize) -> Result<Layout, LayoutError> {
            let header_layout = Layout::new::<usize>();
            let array_layout = Layout::array::<T>(length)?;

            repr_c(&[header_layout, array_layout])
        }

        fn length(&self) -> usize {
            self.length
        }

        fn retype(ptr: NonNull<[()]>) -> NonNull<Self> {
            unsafe {
                let raw_ptr = ptr.as_ptr() as *mut WithHeader<T>;
                NonNull::new_unchecked(raw_ptr)
            }
        }
    }

    #[test]
    fn test_variable_sized_array() {
        let ptr = Global
            .allocate_slice_dst::<WithHeader<usize>>(5)
            .expect("Allocation failed in test");

        Global.deallocate_slice_dst(ptr, 5);
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn sized_type_layout_matches_layout_new() {
        let layout_zero = <u32 as SliceDst>::layout_for(0).expect("layout for sized must succeed");
        assert_eq!(layout_zero, Layout::new::<u32>());

        let n: usize = kani::any();
        let layout_n = <u32 as SliceDst>::layout_for(n).expect("layout for sized ignores length");
        assert_eq!(layout_n, Layout::new::<u32>());
    }

    #[kani::proof]
    fn sized_type_length_is_zero() {
        let value: u32 = kani::any();
        assert_eq!(<u32 as SliceDst>::length(&value), 0);
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn repr_c_is_at_least_as_large_as_any_field() {
        let a: Layout = Layout::new::<u32>();
        let b: Layout = Layout::new::<u8>();
        let composite = repr_c(&[a, b]).expect("layout composes for fixed inputs");

        // pad_to_align preserves the maximum field alignment.
        assert!(composite.align() >= a.align());
        assert!(composite.align() >= b.align());
        // The composite must be large enough to hold both fields back-to-back.
        assert!(composite.size() >= a.size() + b.size());
    }
}
