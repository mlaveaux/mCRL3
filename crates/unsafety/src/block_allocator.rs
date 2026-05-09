use std::alloc::Layout;
use std::array;
use std::cell::UnsafeCell;
use std::fmt;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering;

use allocator_api2::alloc::AllocError;
use allocator_api2::alloc::Allocator;
use crossbeam_utils::CachePadded;
use itertools::Itertools;

use crate::FreeList;
use crate::FreeListEntry;

/// This is a memory pool or also called fixed-size block allocator for a
/// concrete type `T`. It stores blocks of `N` to minimize the overhead of
/// individual memory allocations, which are typically in the range of one or
/// two words.
///
/// Behaves like `Allocator`, except that it only allocates for layouts of `T`.
/// Requires periodic calls to `remove_free_blocks` to prevent memory usage from
/// growing indefinitely.
///
/// This allocator is lock-free for the common allocation/deallocation paths and
/// only takes a lock when a new block needs to be allocated. This does mean
/// that external synchronisation is required to prevent concurrent allocations
/// overlapping with `remove_free_blocks`.
///
/// Two separate freelists are maintained to avoid the ABA problem:
/// - `free`: only popped from during allocation.
/// - `deallocated`: only pushed to during deallocation.
///
/// During `remove_free_blocks` (which requires `&mut self`), entries from
/// `deallocated` are merged into `free`.
///
/// # Details
///
/// Internally stores blocks of `N` elements.
pub struct BlockAllocator<T, const N: usize> {
    /// Blocks and bump pointer are protected by a mutex, for the cold path.
    blocks: Mutex<BlockList<T, N>>,

    /// Recycled entries available for allocation, only popped from during
    /// allocation. Cache-padded to avoid false sharing with `deallocated`.
    free: CachePadded<FreeList<Entry<T>>>,

    /// Only pushed to during deallocation. Moved into `free` during
    /// `remove_free_blocks`. Cache-padded to avoid false sharing with `free`.
    deallocated: CachePadded<FreeList<Entry<T>>>,
}

/// The block list and bump pointer, protected by the blocks mutex.
struct BlockList<T, const N: usize> {
    /// The block that is currently being bump-allocated from. We avoid Box here
    /// to allow multiple blocks to point to the same next block without
    /// violating Box's noalias.
    head_block: Option<NonNull<Block<T, N>>>,

    /// Current bump offset within the head block.
    bump_offset: usize,
}

impl<T, const N: usize> Drop for BlockList<T, N> {
    fn drop(&mut self) {
        // Drop the head block; Block's Drop impl recursively drops the chain.
        if let Some(block_ptr) = self.head_block.take() {
            // Safety: we own all blocks in the list.
            unsafe { drop(Box::from_raw(block_ptr.as_ptr())) };
        }
    }
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
            free: CachePadded::new(FreeList::new()),
            deallocated: CachePadded::new(FreeList::new()),
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
                let mut new_block = Block::new();
                new_block.next = blocks.head_block.take();
                // Use Box::into_raw to get a raw pointer without Box's noalias.
                let new_block_ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(new_block))) };
                blocks.head_block = Some(new_block_ptr);
                blocks.bump_offset = 0;
            }
        }

        let offset = blocks.bump_offset;
        blocks.bump_offset += 1;
        let block_ptr = blocks.head_block.expect("block was just ensured");

        unsafe {
            // Safety: offset < N, so the index is in bounds.
            // Use raw pointer arithmetic to avoid creating &mut to the array,
            // which would cause a Unique retag invalidating existing pointers.
            let data_ptr = (*block_ptr.as_ptr()).data.get() as *mut Entry<T>;
            let entry_ptr = data_ptr.add(offset);
            Ok(NonNull::new_unchecked(
                std::ptr::addr_of_mut!((*entry_ptr).data) as *mut T
            ))
        }
    }

    /// Deallocates a previously-allocated pointer.
    pub fn deallocate_object(&self, ptr: NonNull<T>) {
        self.deallocated.push(ptr.cast());
    }

    /// Removes empty blocks from the block list, and merges the free lists.
    ///
    /// Should be called periodically to prevent memory usage from growing
    /// indefinitely.
    ///
    /// Returns `(removed_blocks, free_size, deallocated_size)`: the number of
    /// blocks removed, and the sizes of the `free` and `deallocated` freelists
    /// before merging.
    pub fn remove_free_blocks(&mut self) -> (usize, usize, usize)
    where
        T: BlockAllocatorSafe,
    {
        // Mark all elements in both freelists with a special value that none of the live entries can have.
        let nonexisting_value = std::ptr::null_mut();

        // In debug mode, collect all freelist entry pointers.
        #[cfg(debug_assertions)]
        let mut freelist_ptrs: Vec<*mut Entry<T>> = Vec::new();

        // Helper: walk a freelist and mark every entry with the sentinel.
        // We only update previous entries to ensure that iter keeps working.
        // Returns the number of entries in the freelist.
        let mark_freelist = |list: &FreeList<Entry<T>>, #[cfg(debug_assertions)] ptrs: &mut Vec<*mut Entry<T>>| unsafe {
            let mut previous: Option<NonNull<Entry<T>>> = None;
            let mut count: usize = 0;
            for current in list.iter() {
                #[cfg(debug_assertions)]
                ptrs.push(current.as_ptr());

                if let Some(previous) = previous {
                    (*previous.as_ptr()).next.store(nonexisting_value, Ordering::Relaxed);
                }
                previous = Some(current);
                count += 1;
            }

            if let Some(previous) = previous {
                (*previous.as_ptr()).next.store(nonexisting_value, Ordering::Relaxed);
            }
            count
        };

        let free_size = mark_freelist(
            &self.free,
            #[cfg(debug_assertions)]
            &mut freelist_ptrs,
        );
        let deallocated_size = mark_freelist(
            &self.deallocated,
            #[cfg(debug_assertions)]
            &mut freelist_ptrs,
        );

        // We will rebuild the freelists from the remaining blocks below.
        self.free.clear();
        self.deallocated.clear();

        let mut guard = self.blocks.lock().expect("Lock poisoned");

        // Debug check: verify that no live entry has the sentinel value.
        #[cfg(debug_assertions)]
        {
            let mut is_head = true;
            for block_ptr in Self::iter_blocks(&guard) {
                let data = unsafe { &*(*block_ptr.as_ptr()).data.get() };
                // Only check entries that have been bump-allocated; entries
                // beyond bump_offset in the head block were never handed out.
                let limit = if is_head { guard.bump_offset } else { N };
                for entry in &data[..limit] {
                    let entry_ptr = entry as *const Entry<T> as *mut Entry<T>;
                    if !freelist_ptrs.contains(&entry_ptr) {
                        // This entry is live — it must not look like the sentinel.
                        unsafe {
                            debug_assert!(
                                entry.next.load(Ordering::Relaxed) != nonexisting_value,
                                "Live entry at {entry_ptr:?} has the sentinel value (null in first word). \
                                 This violates the BlockAllocatorSafe contract."
                            );
                        }
                    }
                }
                is_head = false;
            }
        }
        let removed = if guard.head_block.is_some() {
            // Remove blocks that are now empty, i.e., all their entries have nonexisting_value.
            let mut prev_next_field: *mut Option<NonNull<Block<T, N>>> = &mut guard.head_block;
            let mut removed_blocks = 0;

            loop {
                let current_ptr = match unsafe { &*prev_next_field } {
                    Some(ptr) => *ptr,
                    None => break,
                };

                let all_free = unsafe {
                    let data = &*(*current_ptr.as_ptr()).data.get();
                    data.iter()
                        .all(|entry| entry.next.load(Ordering::Relaxed) == nonexisting_value)
                };

                if all_free {
                    // Unlink and drop the current block.
                    let next = unsafe { (*current_ptr.as_ptr()).next.take() };
                    unsafe { *prev_next_field = next };
                    // Drop the block (deallocates memory).
                    unsafe { drop(Box::from_raw(current_ptr.as_ptr())) };
                    removed_blocks += 1;
                    // prev_next_field stays the same — it now points to the next block.
                } else {
                    // Keep this block; advance to next.
                    prev_next_field = unsafe { &mut (*current_ptr.as_ptr()).next };
                }
            }

            removed_blocks
        } else {
            // No blocks, nothing to remove.
            0
        };

        // Recreate the free list from remaining blocks. We add all free entries
        // to the freelist and set bump_offset = N so future allocations go
        // through the freelist.
        for block_ptr in Self::iter_blocks(&guard) {
            // Safety: exclusive access via &mut self; no concurrent readers.
            let data = unsafe { &*(*block_ptr.as_ptr()).data.get() };
            for entry in &data[..] {
                // Safety: we only push entries that were marked with the special value, which means they are not live.
                unsafe {
                    if entry.next.load(Ordering::Relaxed) == nonexisting_value {
                        self.free
                            .push(NonNull::new_unchecked(entry as *const Entry<T> as *mut Entry<T>));
                    }
                }
            }
        }

        guard.bump_offset = if guard.head_block.is_some() { N } else { 0 };
        drop(guard);
        (removed, free_size, deallocated_size)
    }

    /// Returns an iterator over the blocks.
    ///
    /// The caller must pass the already-acquired guard to avoid a deadlock.
    fn iter_blocks<'a>(guard: &'a MutexGuard<'_, BlockList<T, N>>) -> BlockIter<'a, T, N> {
        BlockIter {
            current: guard.head_block,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over the free list entries.
    ///
    /// # Safety
    ///
    /// This is only safe when no concurrent allocations or deallocations are in progress
    /// (e.g., in single-threaded tests or with `&mut self`).
    unsafe fn iter_free(&self) -> impl Iterator<Item = NonNull<T>> + '_ {
        unsafe {
            self.free
                .iter()
                .chain(self.deallocated.iter())
                .map(|entry| entry.cast())
        }
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
    next: ManuallyDrop<AtomicPtr<Entry<T>>>,
}

// Safety: `Entry<T>` stores a single intrusive next-pointer in `next` used only
// while the slot is on the freelist.
unsafe impl<T> FreeListEntry for Entry<T> {
    unsafe fn get_next(ptr: *mut Self) -> *mut Self {
        // Safety: caller ensures `ptr` is a valid freelist node.
        unsafe { (*ptr).next.load(Ordering::SeqCst) }
    }

    unsafe fn set_next(ptr: *mut Self, next: *mut Self) {
        // Safety: caller ensures `ptr` is a valid freelist node.
        unsafe {
            (*ptr).next.store(next, Ordering::SeqCst);
        }
    }
}

/// An iterator over the blocks in the block allocator.
struct BlockIter<'a, T, const N: usize> {
    current: Option<NonNull<Block<T, N>>>,
    _marker: PhantomData<&'a BlockList<T, N>>,
}

impl<'a, T, const N: usize> Iterator for BlockIter<'a, T, N> {
    type Item = NonNull<Block<T, N>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_ptr = self.current?;

        // Move to the next block for the next iteration.
        self.current = unsafe { (*current_ptr.as_ptr()).next };

        Some(current_ptr)
    }
}

/// We maintain a list of blocks that store N elements each.
struct Block<T, const N: usize> {
    /// Wrapped in UnsafeCell to avoid Stacked Borrows invalidation of
    /// outstanding pointers during subsequent allocations from the same block.
    data: UnsafeCell<[Entry<T>; N]>,

    /// Pointer to the next block (raw to avoid Box noalias).
    next: Option<NonNull<Block<T, N>>>,
}

impl<T, const N: usize> Block<T, N> {
    fn new() -> Self {
        Self {
            data: UnsafeCell::new(array::from_fn(|_i| Entry {
                next: ManuallyDrop::new(AtomicPtr::new(std::ptr::null_mut())),
            })),
            next: None,
        }
    }
}

impl<T, const N: usize> Drop for Block<T, N> {
    fn drop(&mut self) {
        // Iteratively drop the chain to avoid stack overflow on long lists.
        let mut current = self.next.take();
        while let Some(block_ptr) = current {
            let mut block = unsafe { Box::from_raw(block_ptr.as_ptr()) };
            current = block.next.take();
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
    use std::num::NonZeroUsize;
    use std::ptr::NonNull;
    use std::sync::Arc;

    use rand::RngExt;

    use merc_utilities::random_test;

    use super::BlockAllocator;
    use super::BlockAllocatorSafe;

    // In practice u64 is used only in tests; real clients must audit their types.
    unsafe impl BlockAllocatorSafe for NonZeroUsize {}

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_block_allocator() {
        random_test(100, |rng| {
            let mut allocator: BlockAllocator<NonZeroUsize, 32> = BlockAllocator::new();

            // Allocate 1000 elements and keep track of ptr to value mapping.
            let mut allocated: Vec<(NonNull<NonZeroUsize>, NonZeroUsize)> = Vec::new();
            for _ in 0..1000 {
                let ptr = allocator.allocate_object().unwrap();
                let value: NonZeroUsize = NonZeroUsize::new(rng.random_range(1..=usize::MAX)).unwrap();
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

            // All remaining elements must still hold their original values.
            for (ptr, expected) in &remaining {
                unsafe {
                    assert_eq!(*ptr.as_ref(), *expected);
                }
            }

            let (removed, free_size, deallocated_size) = allocator.remove_free_blocks();
            println!("{removed} removed, {free_size} free, {deallocated_size} deallocated");
            
            for _ in 0..500 {
                let ptr = allocator.allocate_object().unwrap();
                let value: NonZeroUsize = NonZeroUsize::new(rng.random_range(1..=usize::MAX)).unwrap();
                unsafe {
                    ptr.as_ptr().write(value);
                }
                remaining.push((ptr, value));
            }

            // All remaining elements must have the correct values.
            for (ptr, expected) in &remaining {
                unsafe {
                    assert_eq!(*ptr.as_ref(), *expected);
                }
            }
        })
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_block_allocator_parallel_freelist() {
        let block_allocator = Arc::new(BlockAllocator::<u32, 32>::new());

        let threads: Vec<_> = (0..=2)
            .map(|_| {
                let block_allocator = block_allocator.clone();

                std::thread::spawn(move || {
                    // Not sure if this could actually trigger the ABA problem,
                    // but this is only used to detect data races.
                    let mut ptrs = Vec::new();
                    for _ in 0..100 {
                        let ptr = block_allocator.allocate_object().unwrap();
                        unsafe {
                            ptr.as_ptr().write(42);
                        }
                        ptrs.push(ptr);
                    }

                    for ptr in ptrs {
                        unsafe {
                            assert_eq!(*ptr.as_ref(), 42);
                        }
                        block_allocator.deallocate_object(ptr);
                    }
                })
            })
            .collect();

        for thread in threads {
            thread.join().unwrap();
        }
    }
}
