use std::alloc::Layout;
use std::array;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering;

use allocator_api2::alloc::AllocError;
use allocator_api2::alloc::Allocator;
use thread_local::ThreadLocal;

use crate::FreeList;
use crate::FreeListEntry;

const FREE_LIST_CHUNK_SIZE: usize = 1000;

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
/// A single thread-local freelist is maintained per thread:
/// - `free`: popped from during allocation and pushed to during deallocation.
///
/// # Details
///
/// Internally stores blocks of `N` elements.
/// All thread-local state (bump_offset, current_block, free) is stored
/// per-thread to eliminate shared atomic contention in the hot path.
pub struct BlockAllocator<T: Send, const N: usize> {
    /// Owns the block chain; only locked when a new block must be allocated.
    /// This is the only shared state accessed during allocation.
    blocks: Mutex<BlockList<T, N>>,

    /// Per-thread state for allocation: current block and bump offset. This eliminates
    alloc_state: ThreadLocal<ThreadLocalAllocState<T, N>>,
}

/// The block list and bump pointer, protected by the blocks mutex.
struct BlockList<T, const N: usize> {
    /// The block that is currently being bump-allocated from. We avoid Box here
    /// to allow multiple blocks to point to the same next block without
    /// violating Box's noalias.
    head_block: Option<NonNull<Block<T, N>>>,

    /// Shared chunks of free entries represented by list heads. Each head
    /// points to a null-terminated list of length `FREE_LIST_CHUNK_SIZE`
    /// (except possibly the final chunk).
    free_chunks: Vec<NonNull<Entry<T>>>,
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

impl<T: Send, const N: usize> Default for BlockAllocator<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send, const N: usize> BlockAllocator<T, N> {
    pub fn new() -> Self {
        Self {
            blocks: Mutex::new(BlockList {
                head_block: None,
                free_chunks: Vec::new(),
            }),
            alloc_state: ThreadLocal::new(),
        }
    }

    /// Allocates a slot for one object of type `T`.
    pub fn allocate_object(&self) -> Result<NonNull<T>, AllocError> {
        let state = self.alloc_state.get_or(ThreadLocalAllocState::new);

        // Fast path 1: try thread-local free list.
        if let Some(entry) = state.free.try_pop() {
            return Ok(entry.cast());
        }

        if self.refill_local_free_from_chunks(state)
            && let Some(entry) = state.free.try_pop()
        {
            return Ok(entry.cast());
        }

        // Fast path 2: lock-free bump allocation from current thread's block.
        let block_ptr = state.current_block.get();
        let offset = state.bump_offset.get();
        if !block_ptr.is_null() && offset < N {
            state.bump_offset.set(offset + 1);
            return unsafe {
                let data_ptr = (*block_ptr).data.get() as *mut Entry<T>;
                let entry_ptr = data_ptr.add(offset);
                Ok(NonNull::new_unchecked(
                    std::ptr::addr_of_mut!((*entry_ptr).data) as *mut T
                ))
            };
        }

        // Slow path: allocate a new block under the mutex.
        self.allocate_new_block(state)
    }

    /// Refills the calling thread's local freelist from one shared chunk.
    fn refill_local_free_from_chunks(&self, state: &ThreadLocalAllocState<T, N>) -> bool {
        let mut guard = self.blocks.lock().expect("Lock poisoned");
        let Some(current) = guard.free_chunks.pop() else {
            return false;
        };
        drop(guard);

        debug_assert!(state.free.is_empty(), "local freelist must be empty before chunk refill");
        unsafe {
            state.free.set_head(current.as_ptr());
        }
        true
    }

    /// Slow path: allocate a new block and update thread-local state.
    #[cold]
    fn allocate_new_block(&self, state: &ThreadLocalAllocState<T, N>) -> Result<NonNull<T>, AllocError> {
        let mut guard = self.blocks.lock().expect("Lock poisoned");

        // Allocate a new block and link it to the existing chain.
        let mut new_block = Block::new();
        new_block.next = guard.head_block;
        let new_block_ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(new_block))) };
        guard.head_block = Some(new_block_ptr);

        drop(guard);

        // Update thread-local state with new block.
        state.current_block.set(new_block_ptr.as_ptr());
        state.bump_offset.set(1);

        // Return slot 0 of the new block.
        unsafe {
            let data_ptr = (*new_block_ptr.as_ptr()).data.get() as *mut Entry<T>;
            Ok(NonNull::new_unchecked(
                std::ptr::addr_of_mut!((*data_ptr).data) as *mut T
            ))
        }
    }

    /// Deallocates a previously-allocated pointer.
    pub fn deallocate_object(&self, ptr: NonNull<T>) {
        let state = self.alloc_state.get_or(ThreadLocalAllocState::new);
        state.free.push(ptr.cast());
    }

    /// Removes empty blocks from the block list.
    ///
    /// Should be called periodically to prevent memory usage from growing
    /// indefinitely.
    ///
    /// **Important**: This method must not be called concurrently with any allocations
    /// or deallocations. It should be called at a synchronization point where no other
    /// threads are active on this allocator.
    ///
    /// This method scans every thread-local `free` list,
    /// marks those entries with a sentinel value, then removes blocks where all
    /// entries are marked as free.
    ///
    /// Returns `(removed_blocks, free_size, deallocated_size)`: the number of
    /// blocks removed, and the size of the merged `free` list before cleanup.
    /// `deallocated_size` is always 0 and kept for API compatibility.
    pub fn remove_free_blocks(&mut self) -> (usize, usize)
    where
        T: BlockAllocatorSafe,
    {
        // Mark all elements in all thread-local freelists with a special
        // value that none of the live entries can have.
        let nonexisting_value = NONEXISTING_VALUE as *mut Entry<T>;

        // In debug mode, collect all freelist entry pointers.
        #[cfg(debug_assertions)]
        let mut freelist_ptrs: std::collections::HashSet<*mut Entry<T>> = std::collections::HashSet::new();

        // Helper: walk a freelist and mark every entry with the sentinel.
        // We only update previous entries to ensure that iter keeps working.
        // Returns the number of entries in the freelist.
        let mark_freelist =
            |list: &FreeList<Entry<T>>,
             #[cfg(debug_assertions)] ptrs: &mut std::collections::HashSet<*mut Entry<T>>| unsafe {
                let mut previous: Option<NonNull<Entry<T>>> = None;
                let mut count: usize = 0;
                for current in list.iter() {
                    #[cfg(debug_assertions)]
                    ptrs.insert(current.as_ptr());

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

        let mut free_size = 0;
        for state in self.alloc_state.iter_mut() {
            free_size += mark_freelist(
                &state.free,
                #[cfg(debug_assertions)]
                &mut freelist_ptrs,
            );
            state.free.clear();
            state.current_block.set(std::ptr::null_mut());
            state.bump_offset.set(N);
        }

        let mut guard = self.blocks.lock().expect("Lock poisoned");

        // Mark entries currently staged in shared free chunks.
        for &chunk_head in &guard.free_chunks {
            let mut current = Some(chunk_head);
            while let Some(entry) = current {
                #[cfg(debug_assertions)]
                freelist_ptrs.insert(entry.as_ptr());

                let next = unsafe { Entry::get_next(entry.as_ptr()) };
                unsafe {
                    (*entry.as_ptr())
                        .next
                        .store(nonexisting_value, Ordering::Relaxed);
                }
                free_size += 1;
                current = NonNull::new(next);
            }
        }
        guard.free_chunks.clear();

        // Debug check: verify that no live entry has the sentinel value.
        #[cfg(debug_assertions)]
        {
            for block_ptr in Self::iter_blocks(&guard) {
                let data = unsafe { &*(*block_ptr.as_ptr()).data.get() };
                for entry in data {
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

        // Recreate shared free chunks from remaining blocks.
        let mut rebuilt_chunk_heads: Vec<NonNull<Entry<T>>> = Vec::new();
        let mut chunk_head: Option<NonNull<Entry<T>>> = None;
        let mut chunk_tail: Option<NonNull<Entry<T>>> = None;
        let mut chunk_len = 0usize;

        for block_ptr in Self::iter_blocks(&guard) {
            let data = unsafe { &*(*block_ptr.as_ptr()).data.get() };
            for entry in data {
                unsafe {
                    if entry.next.load(Ordering::Relaxed) == nonexisting_value {
                        let entry_ptr = NonNull::new_unchecked(entry as *const Entry<T> as *mut Entry<T>);
                        (*entry_ptr.as_ptr())
                            .next
                            .store(std::ptr::null_mut(), Ordering::Relaxed);

                        if let Some(tail) = chunk_tail {
                            (*tail.as_ptr()).next.store(entry_ptr.as_ptr(), Ordering::Relaxed);
                            chunk_tail = Some(entry_ptr);
                            chunk_len += 1;
                        } else {
                            chunk_head = Some(entry_ptr);
                            chunk_tail = Some(entry_ptr);
                            chunk_len = 1;
                        }

                        if chunk_len == FREE_LIST_CHUNK_SIZE {
                            rebuilt_chunk_heads.push(chunk_head.expect("chunk head set"));
                            chunk_head = None;
                            chunk_tail = None;
                            chunk_len = 0;
                        }
                    }
                }
            }
        }

        if let Some(head) = chunk_head {
            rebuilt_chunk_heads.push(head);
        }

        guard.free_chunks.extend(rebuilt_chunk_heads);

        drop(guard);
        (removed, free_size)
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
}

/// Per-thread state for allocation: current block and bump offset.
/// This eliminates shared atomic contention in the hot path.
struct ThreadLocalAllocState<T, const N: usize> {
    /// The block currently being bump-allocated from.
    current_block: Cell<*mut Block<T, N>>,

    /// Bump offset within `current_block`. N or greater means the block is full.
    bump_offset: Cell<usize>,

    /// Thread-local free list (only popped from during allocation).
    free: FreeList<Entry<T>>,
}

impl<T, const N: usize> ThreadLocalAllocState<T, N> {
    fn new() -> Self {
        Self {
            current_block: Cell::new(std::ptr::null_mut()),
            bump_offset: Cell::new(N),
            free: FreeList::new(),
        }
    }
}

unsafe impl<T: Send, const N: usize> Send for ThreadLocalAllocState<T, N> {}

/// Marker trait asserting that `std::ptr::null_mut::<Entry<T>>()` can never
/// appear as the first `size_of::<*mut _>()` bytes of any valid value of `T`.
///
/// # Safety
///
/// Implementing this trait for a type `T` asserts that the special sentinel
/// value can be used.
pub unsafe trait BlockAllocatorSafe {}

/// Sentinel value used to identify entries that are not on the freelist.
const NONEXISTING_VALUE: usize = usize::MAX;

/// The [BlockAllocator] is thread-safe.
unsafe impl<T: Send, const N: usize> Send for BlockAllocator<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for BlockAllocator<T, N> {}

/// `AllocBlock` implements the [`Allocator`] trait using the underlying [`BlockAllocator`].
pub struct AllocBlock<T: Send, const N: usize> {
    block_allocator: BlockAllocator<T, N>,
}

impl<T: Send, const N: usize> Default for AllocBlock<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send, const N: usize> AllocBlock<T, N> {
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
        unsafe { (*ptr).next.load(Ordering::Acquire) }
    }

    unsafe fn set_next(ptr: *mut Self, next: *mut Self) {
        // Safety: caller ensures `ptr` is a valid freelist node.
        unsafe {
            (*ptr).next.store(next, Ordering::Release);
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

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;
    use std::sync::Arc;

    use rand::RngExt;

    use merc_utilities::random_test;

    use super::BlockAllocator;
    use super::BlockAllocatorSafe;

    // In practice u64 is used only in tests; real clients must audit their types.
    unsafe impl BlockAllocatorSafe for usize {}

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_block_allocator() {
        random_test(100, |rng| {
            let mut allocator: BlockAllocator<usize, 32> = BlockAllocator::new();

            // Allocate 1000 elements and keep track of ptr to value mapping.
            let mut allocated: Vec<(NonNull<usize>, usize)> = Vec::new();
            for _ in 0..1000 {
                let ptr = allocator.allocate_object().unwrap();
                let value: usize = rng.random_range(0..=usize::MAX - 1);
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

            let (removed, free_size) = allocator.remove_free_blocks();
            println!("{removed} removed, {free_size} free");

            for _ in 0..500 {
                let ptr = allocator.allocate_object().unwrap();
                let value: usize = rng.random_range(0..=usize::MAX - 1);
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
