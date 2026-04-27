use std::alloc::Layout;
use std::array;
use std::fmt;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

#[cfg(not(loom))]
mod inner {
    pub use std::sync::Mutex;
    pub use std::sync::MutexGuard;
}

// We replace the standard implementation by loom's implementation.
#[cfg(loom)]
mod inner {
    pub use loom::sync::Mutex;
    pub use loom::sync::MutexGuard;
}

use inner::*;

use allocator_api2::alloc::AllocError;
use allocator_api2::alloc::Allocator;
use itertools::Itertools;

use crate::FreeList;
use crate::FreeListEntry;

/// This is a memory pool or also called fixed-size block allocator for a
/// concrete type `T`. It stores blocks of `N` to minimize the overhead of
/// individual memory allocations, which are typically in the range of one or
/// two words.
///
/// Behaves like `Allocator`, except that it only allocates for layouts of `T`.
///
/// This allocator is lock-free for the common allocation/deallocation paths and
/// only takes a lock when a new block needs to be allocated.
///
/// # Details
///
/// Internally stores blocks of `N` elements.
pub struct BlockAllocator<T, const N: usize> {
    /// Blocks and bump pointer are protected by a mutex, for the cold path.
    blocks: Mutex<BlockList<T, N>>,

    /// Recycled entries managed in a lock-free Treiber stack.
    free: FreeList<Entry<T>>,
}

/// The block list and bump pointer, protected by the blocks mutex.
struct BlockList<T, const N: usize> {
    /// The block that is currently being bump-allocated from.
    head_block: Option<Box<Block<T, N>>>,

    /// Current bump offset within the head block.
    bump_offset: usize,
}

impl<T, const N: usize> Default for BlockAllocator<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> BlockAllocator<T, N> {
    pub fn new() -> Self {
        Self {
            blocks: Mutex::new(BlockList {
                head_block: None,
                bump_offset: 0,
            }),
            free: FreeList::new(),
        }
    }

    /// Allocates a slot for one object of type `T`.
    ///
    /// The fast path pops from the lock-free freelist; the slow path
    /// bump-allocates from blocks and takes a lock for now blocks.
    pub fn allocate_object(&self) -> Result<NonNull<T>, AllocError> {
        // Fast path: pop from the lock-free freelist (Treiber stack).
        if let Some(entry) = self.free.try_pop() {
            return Ok(entry.cast());
        }

        // Slow path: bump-allocate from a block.
        self.allocate_from_block()
    }

    /// Slow path: allocate from the block list, potentially allocating a new block.
    #[cold]
    fn allocate_from_block(&self) -> Result<NonNull<T>, AllocError> {
        let mut blocks = self.blocks.lock().expect("Lock poisoned");

        // Ensure we have a block with space.
        match &blocks.head_block {
            Some(_block) if blocks.bump_offset < N => {
                // Current block has room.
            }
            _ => {
                // Either no block exists, or the current one is full.
                let mut new_block = Box::new(Block::new());
                if let Some(old_block) = blocks.head_block.take() {
                    new_block.next = Some(old_block);
                }
                blocks.head_block = Some(new_block);
                blocks.bump_offset = 0;
            }
        }

        let offset = blocks.bump_offset;
        blocks.bump_offset += 1;
        let block = blocks.head_block.as_mut().expect("block was just ensured");

        unsafe {
            // Safety: offset < N, so the index is in bounds. We take a pointer to
            // ManuallDrop<T>, which has the same layout as T.
            Ok(NonNull::new_unchecked(
                &mut block.data[offset].data as *mut ManuallyDrop<T> as *mut T,
            ))
        }
    }

    /// Deallocates a previously-allocated pointer (lock-free push onto freelist).
    pub fn deallocate_object(&self, ptr: NonNull<T>) {
        self.free.push(ptr.cast());
    }

    /// Removes empty blocks from the block list. Should be called periodically
    /// to prevent memory usage from growing indefinitely.
    pub fn remove_free_blocks(&mut self) -> usize
    where
        T: BlockAllocatorSafe,
    {
        // A special value that must not occur in the values of `T`.
        let nonexisting_value = std::ptr::null_mut();

        for block in unsafe { self.iter() } {
            // Check that none of the entries contain the special value.
            debug_assert!(
                block
                    .data
                    .iter()
                    .all(|entry| { unsafe { entry.next != nonexisting_value } }),
                "The special value used to mark free entries must not be present in any live entry"
            );
        }

        let mut guard = self.blocks.lock().expect("Lock poisoned");
        let removed = if let Some(head_block) = guard.head_block.as_mut() {
            // Mark all elements in the free list with a special value that none of the live entries can have (e.g., a non-canonical pointer).
            unsafe {
                let mut previous: Option<NonNull<Entry<T>>> = None;
                for current in self.free.iter() {
                    if let Some(previous) = previous {
                        (*previous.as_ptr()).next = nonexisting_value;
                    }
                    previous = Some(current);
                }

                if let Some(previous) = previous {
                    (*previous.as_ptr()).next = nonexisting_value;
                }
            }

            // We will rebuild the freelist from the remaining blocks below.
            self.free.clear();

            // Remove blocks that are now empty, i.e., all their entries have nonexisting_value.
            let mut previous_block: Option<*mut Box<Block<T, N>>> = None;
            let mut block = head_block as *mut Box<Block<T, N>>;
            let mut removed_blocks = 0;

            loop {
                let all_free = unsafe { (*block).data.iter().all(|entry| entry.next == nonexisting_value) };

                if all_free {
                    // Extract next before the current block is dropped.
                    let next = unsafe { (*block).next.take() };

                    // Unlink current block; the old Box is dropped here.
                    if let Some(previous_block) = previous_block {
                        unsafe {
                            (*previous_block).next = next;
                        }
                    } else {
                        guard.head_block = next;
                    }

                    removed_blocks += 1;

                    // Advance to the next block, which now lives at previous's
                    // next slot (or the head).
                    let next_ref = if let Some(prev) = previous_block {
                        unsafe { &mut (*prev).next }
                    } else {
                        &mut guard.head_block
                    };
                    block = match next_ref {
                        Some(next_block) => next_block as *mut Box<Block<T, N>>,
                        None => break,
                    };
                } else {
                    // Keep this block; advance normally.
                    let next = unsafe { &mut (*block).next };
                    previous_block = Some(block);
                    block = match next {
                        Some(next_block) => next_block as *mut Box<Block<T, N>>,
                        None => break,
                    };
                }
            }

            removed_blocks
        } else {
            // No blocks, nothing to remove.
            0
        };

        drop(guard);

        // Recreate the free list by pushing all entries of the remaining blocks back onto the free list.
        for block in unsafe { self.iter() } {
            for entry in block.data.iter() {
                // Safety: we only push entries that were marked with the special value, which means they are not live.
                unsafe {
                    if entry.next == nonexisting_value {
                        self.free
                            .push(NonNull::new_unchecked(entry as *const Entry<T> as *mut Entry<T>));
                    }
                }
            }
        }

        removed
    }

    /// Returns an iterator over the blocks
    unsafe fn iter<'a>(&'a self) -> BlockIter<'a, T, N> {
        let guard = self.blocks.lock().expect("Lock poisoned");
        BlockIter {
            current: guard.head_block.as_ref().map(|b| b as *const _),
            _guard: guard,
        }
    }

    /// Returns an iterator over the free list entries.
    ///
    /// # Safety
    ///
    /// This is only safe when no concurrent allocations or deallocations are in progress
    /// (e.g., in single-threaded tests or with `&mut self`).
    unsafe fn iter_free(&self) -> impl Iterator<Item = NonNull<T>> + '_ {
        unsafe { self.free.iter().map(|entry| entry.cast()) }
    }
}

/// Marker trait asserting that `std::ptr::null_mut::<Entry<T>>()` can never
/// appear as the first `size_of::<*mut _>()` bytes of any valid value of `T`.
///
/// # Safety
///
/// Implementing this trait for a type `T` asserts that the special sentinel
/// value can be used.
pub unsafe trait BlockAllocatorSafe {}

/// The [BlockAllocator] is thread-safe.
unsafe impl<T: Send, const N: usize> Send for BlockAllocator<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for BlockAllocator<T, N> {}

/// `AllocBlock` implements the [`Allocator`] trait using the underlying [`BlockAllocator`].
pub struct AllocBlock<T, const N: usize> {
    block_allocator: BlockAllocator<T, N>,
}

impl<T, const N: usize> Default for AllocBlock<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> AllocBlock<T, N> {
    /// Creates a new `AllocBlock`.
    pub fn new() -> Self {
        Self {
            block_allocator: BlockAllocator::new(),
        }
    }

    /// Removes free blocks from the underlying block allocator, see [`BlockAllocator::remove_free_blocks`].
    pub fn remove_free_blocks(&mut self)
    where
        T: BlockAllocatorSafe,
    {
        self.block_allocator.remove_free_blocks();
    }
}

unsafe impl<T: Send, const N: usize> Allocator for AllocBlock<T, N> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert_eq!(
            layout,
            Layout::new::<T>(),
            "The requested layout should match the type T"
        );

        let ptr = self.block_allocator.allocate_object()?;

        // Convert NonNull<T> to NonNull<[u8]> with the correct size
        let byte_ptr = ptr.cast::<u8>();
        let slice_ptr = NonNull::slice_from_raw_parts(byte_ptr, std::mem::size_of::<T>());

        Ok(slice_ptr)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        debug_assert_eq!(
            layout,
            Layout::new::<T>(),
            "The requested layout should match the type T"
        );
        self.block_allocator.deallocate_object(ptr.cast::<T>());
    }
}

union Entry<T> {
    /// Stores the actual element.
    data: ManuallyDrop<T>,

    /// If the element is free, this points to the next entry in the freelist, or null if this is the last entry.
    next: *mut Entry<T>,
}

// Safety: `Entry<T>` stores a single intrusive next-pointer in `next` used only
// while the slot is on the freelist.
unsafe impl<T> FreeListEntry for Entry<T> {
    unsafe fn get_next(ptr: *mut Self) -> *mut Self {
        // Safety: caller ensures `ptr` is a valid freelist node.
        unsafe { (*ptr).next }
    }

    unsafe fn set_next(ptr: *mut Self, next: *mut Self) {
        // Safety: caller ensures `ptr` is a valid freelist node.
        unsafe {
            (*ptr).next = next;
        }
    }
}

/// An iterator over the blocks in the block allocator.
struct BlockIter<'a, T, const N: usize> {
    current: Option<*const Box<Block<T, N>>>,
    _guard: MutexGuard<'a, BlockList<T, N>>,
}

impl<'a, T, const N: usize> Iterator for BlockIter<'a, T, N> {
    type Item = &'a Box<Block<T, N>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_block = self.current?;

        // Move to the next block for the next iteration.
        self.current = unsafe { (*current_block).next.as_ref() }.map(|b| b as *const _);

        Some(unsafe { &*current_block })
    }
}

/// We maintain a list of blocks that store N elements each.
struct Block<T, const N: usize> {
    data: [Entry<T>; N],

    /// Pointer to the next block.
    next: Option<Box<Block<T, N>>>,
}

impl<T, const N: usize> Block<T, N> {
    fn new() -> Self {
        Self {
            data: array::from_fn(|_i| Entry {
                next: std::ptr::null_mut(),
            }),
            next: None,
        }
    }
}

impl<T, const N: usize> fmt::Debug for BlockAllocator<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Safety: Debug is only meaningful in single-threaded / non-concurrent contexts.
        write!(f, "freelist = {:?}", unsafe { self.iter_free() }.format(", "))
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use rand::RngExt;

    use merc_utilities::random_test;

    use super::BlockAllocator;
    use super::BlockAllocatorSafe;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestValue {
        marker: NonZeroUsize,
        value: u64,
    }

    // Safety: the first word is a NonZeroUsize, so the null sentinel can never
    // coincide with a live TestValue.
    unsafe impl BlockAllocatorSafe for TestValue {}

    #[test]
    // #[cfg_attr(miri, ignore)]
    fn test_block_allocator() {
        random_test(100, |rng| {
            let mut allocator: BlockAllocator<TestValue, 32> = BlockAllocator::new();

            // Allocate 1000 elements, recording each pointer alongside its written value.
            let mut allocated: Vec<(NonNull<TestValue>, TestValue)> = Vec::new();
            for _ in 0..1000 {
                let ptr = allocator.allocate_object().unwrap();
                let value = TestValue {
                    marker: NonZeroUsize::new(1).unwrap(),
                    value: rng.random(),
                };
                unsafe {
                    ptr.as_ptr().write(value);
                }
                allocated.push((ptr, value));
            }

            // Deallocate a random subset and keep track of which entries remain live.
            let mut remaining = Vec::new();
            for (ptr, value) in allocated {
                if rng.random_bool(0.5) {
                    allocator.deallocate_object(ptr);
                } else {
                    remaining.push((ptr, value));
                }
            }

            // All surviving elements must still hold their original values.
            for (ptr, expected) in &remaining {
                unsafe {
                    assert_eq!(*ptr.as_ref(), *expected);
                }
            }

            println!("{} removed", allocator.remove_free_blocks());

            // Reallocate 500 elements to exercise the freelist and verify no aliasing.
            for _ in 0..500 {
                let ptr = allocator.allocate_object().unwrap();
                let value = TestValue {
                    marker: NonZeroUsize::new(1).unwrap(),
                    value: rng.random(),
                };
                unsafe {
                    ptr.as_ptr().write(value);
                }
                remaining.push((ptr, value));
            }

            // All elements (old survivors and newly allocated) must hold their correct values.
            for (ptr, expected) in &remaining {
                unsafe {
                    assert_eq!(*ptr.as_ref(), *expected);
                }
            }
        })
    }

    #[test]
    #[cfg(loom)]
    fn test_loom_block_allocator() {
        loom::model(|| {
            let block_allocator = loom::sync::Arc::new(BlockAllocator::<loom::cell::UnsafeCell<u32>, 4>::new());

            let threads: Vec<_> = (0..3)
                .map(|_| {
                    let block_allocator = block_allocator.clone();

                    loom::thread::spawn(move || {
                        let mut ptrs = Vec::new();
                        for _ in 0..10 {
                            let ptr = block_allocator.allocate_object().unwrap();
                            unsafe {
                                // Construct a loom UnsafeCell in the allocated slot so
                                // loom can instrument all subsequent accesses.
                                ptr.as_ptr().write(loom::cell::UnsafeCell::new(0));
                                (*ptr.as_ptr()).with_mut(|p| *p = 42);
                            }
                            ptrs.push(ptr);
                        }

                        loom::thread::yield_now();

                        for ptr in ptrs {
                            unsafe {
                                (*ptr.as_ptr()).with(|p| assert_eq!(*p, 42));
                                // Drop the UnsafeCell before returning the slot to the freelist.
                                std::ptr::drop_in_place(ptr.as_ptr());
                            }
                            block_allocator.deallocate_object(ptr);
                        }
                    })
                })
                .collect();

            for th in threads {
                th.join().unwrap();
            }
        });
    }
}
