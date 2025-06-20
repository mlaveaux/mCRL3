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

/// A trait that can be used to extend `Allocator` implementations with the ability to allocate (and deallocate) dynamically sized slices that implement `SliceDst`.
pub trait AllocatorDst {
    /// Allocate an object whose type implements `SliceDst`. The resulting memory is uninitialize.
    fn allocate_slice_dst<T: SliceDst + ?Sized>(&mut self, length: usize) -> Result<NonNull<T>, AllocError>;

    /// Deallocates an allocation returned by `allocate_slice_dst`.
    fn deallocate_slice_dst<T: ?Sized + SliceDst>(&mut self, ptr: NonNull<T>);
}

/// This trait should be implemented by dynamic sized types.
pub trait SliceDst {
    /// Returns the layout of the slice containing `length` elements for this DST.
    fn layout_for(length: usize) -> Result<Layout, LayoutError>;

    /// Add the type on an untyped pointer
    fn retype(ptr: NonNull<[()]>) -> NonNull<Self>;

    /// The number of elements in this dynamic sized type. This information is necessary for deallocation.
    fn length(&self) -> usize;
}

impl<A: Allocator> AllocatorDst for A {
    fn allocate_slice_dst<T: SliceDst + ?Sized>(&mut self, length: usize) -> Result<NonNull<T>, AllocError> {
        let ptr = self.allocate(T::layout_for(length).unwrap())?;
        // Create a slice of the correct length for proper metadata
        let slice_ptr = unsafe { NonNull::new_unchecked(slice_from_raw_parts_mut(ptr.as_ptr() as *mut (), length)) };
        Ok(T::retype(slice_ptr))
    }

    fn deallocate_slice_dst<T: ?Sized + SliceDst>(&mut self, ptr: NonNull<T>) {
        unsafe {
            self.deallocate(
                NonNull::new_unchecked(ptr.as_ptr() as *mut u8),
                T::layout_for(ptr.as_ref().length()).unwrap(),
            );
        }
    }
}

/// To calculate the layout of a #[repr(C)] structure and the offsets of the fields from its fields’ layouts:
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

#[cfg(test)]
mod tests {
    use super::*;

    use allocator_api2::alloc::Global;

    #[repr(C)]
    struct WithHeader<T> {
        length: usize,
        array: [T],
    }

    impl<T> SliceDst for WithHeader<T> {
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
        let mut _test = Global
            .allocate_slice_dst::<WithHeader<usize>>(5)
            .expect("Allocation failed in test");
    }
}
