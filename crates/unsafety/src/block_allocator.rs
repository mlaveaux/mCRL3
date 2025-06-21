use std::{alloc::Layout, mem::ManuallyDrop, ptr::NonNull};

use allocator_api2::alloc::{AllocError, Allocator};


/// This is a slab allocator or also called block allocator for a concrete type `T`. It stores blocks of `Size` to minimize the overhead of individual memory allocations (which are typically in the range of one or two words).
/// 
/// Behaves like `Allocator`, except that it only allocates for layouts of `T`.
/// 
/// # Details
/// 
/// Internally stores blocks of `N` elements 
struct BlockAllocator<T, const N: usize> {
    /// This is the block that contains unoccupied entries.
    first: Option<Box<Block<T, N>>>,
}

impl<T, const N: usize> BlockAllocator<T, N> {
    pub fn new() -> Self {
        Self {
            first: None
        }
    }

    /// Similar to the [Allocator] trait, but instead of passing a layout we allocate just an object of type `T`.
    pub fn allocate_object(&mut self) -> Result<NonNull<T>, AllocError> {
        // if self.first.is_none() {
        //     self.first = Some(Block::new());
        // }
        unreachable!()
    }

    /// Deallocate the given pointer.
    pub fn deallocate_object(&mut self, ptr: NonNull<T>) {

    }
}

// unsafe impl<T, const N: usize> Allocator for BlockAllocator<T, N> {
//     fn allocate(&self, layout: std::alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
//         debug_assert_eq!(layout, Layout::new::<T>(), "The requested layout should match the type T");

//         let ptr = self.allocate_object()?;

//         // Convert NonNull<T> to NonNull<[u8]> with the correct size
//         let byte_ptr = ptr.cast::<u8>();
//         let slice_ptr = NonNull::slice_from_raw_parts(byte_ptr, std::mem::size_of::<T>());
        
//         Ok(slice_ptr)        
//     }

//     unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
//         debug_assert_eq!(layout, Layout::new::<T>(), "The requested layout should match the type T");

//         // TODO: Implement dealloc.
//     }
// }

union Entry<T> {
    data: ManuallyDrop<T>,

    next: *const Entry<T>, 
}

///
struct Block<T, const N: usize> {
    /// 
    data: [T; N],

    length: usize,

    next: Option<Box<Block<T, N>>>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_allocator() {
        let mut allocator: BlockAllocator::<usize, 256> = BlockAllocator::new();

        let object = allocator.allocate_object();

    }
}