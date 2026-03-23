use std::alloc::Layout;
use std::array;
use std::fmt;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering;

use allocator_api2::alloc::AllocError;
use allocator_api2::alloc::Allocator;
use itertools::Itertools;

/// This is a slab allocator or also called block allocator for a concrete type
/// `T`. It stores blocks of `Size` to minimize the overhead of individual
/// memory allocations (which are typically in the range of one or two words).
///
/// Behaves like `Allocator`, except that it only allocates for layouts of `T`.
///
/// This allocator is lock-free for the common allocation/deallocation paths
/// (freelist pop/push) and only takes a lock when a new block needs to be
/// allocated.
///
/// # Details
///
/// Internally stores blocks of `N` elements
pub struct BlockAllocator<T, const N: usize> {
    /// Blocks and bump pointer are protected by a mutex (cold path only — new block allocation).
    blocks: Mutex<BlockList<T, N>>,

    /// The head of the lock-free freelist (Treiber stack). Null means empty.
    free: AtomicPtr<Entry<T>>,
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
            free: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    /// Allocates a slot for one object of type `T`.
    ///
    /// The fast path pops from the lock-free freelist; the slow path
    /// bump-allocates from blocks (taking a lock only when a new block
    /// must be allocated).
    pub fn allocate_object(&self) -> Result<NonNull<T>, AllocError> {
        // Fast path: pop from the lock-free freelist (Treiber stack).
        let mut head = self.free.load(Ordering::Acquire);
        loop {
            if head.is_null() {
                break;
            }

            // Safety: `head` is a valid freed entry in our freelist. Reading `next` is safe
            // because no other thread can reclaim this node until our CAS succeeds (the
            // entry is logically owned by whoever read `head` before the CAS).
            let next = unsafe { (*head).next };

            match self.free.compare_exchange_weak(head, next, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => {
                    // Safety: `head` was a valid entry pointer; cast back to T.
                    return Ok(unsafe { NonNull::new_unchecked(head as *mut T) });
                }
                Err(actual) => {
                    head = actual;
                }
            }
        }

        // Slow path: bump-allocate from a block.
        self.allocate_from_block()
    }

    /// Slow path: allocate from the block list, potentially allocating a new block.
    #[cold]
    fn allocate_from_block(&self) -> Result<NonNull<T>, AllocError> {
        let mut blocks = self.blocks.lock().map_err(|_| AllocError)?;

        // Ensure we have a block with space.
        match &blocks.head_block {
            Some(block) if blocks.bump_offset < N => {
                // Current block has room.
                let _ = block;
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
        let entry = ptr.as_ptr() as *mut Entry<T>;

        let mut head = self.free.load(Ordering::Relaxed);
        loop {
            // Safety: `entry` is a valid freed allocation; writing `next` is safe because
            // we own this slot (the caller guarantees it is no longer in use).
            unsafe {
                (*entry).next = head;
            }

            match self.free.compare_exchange_weak(head, entry, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_) => return,
                Err(actual) => {
                    head = actual;
                }
            }
        }
    }

    /// Returns an iterator over the free list entries.
    ///
    /// # Safety
    ///
    /// This is only safe when no concurrent allocations or deallocations are in progress
    /// (e.g., in single-threaded tests or with `&mut self`).
    unsafe fn iter_free(&self) -> FreeListIterator<T> {
        FreeListIterator {
            current: NonNull::new(self.free.load(Ordering::Relaxed)),
        }
    }
}

// SAFETY: BlockAllocator uses atomic operations for the freelist and a Mutex for
// block allocation. No unsynchronised mutable state is exposed through &self.
unsafe impl<T: Send, const N: usize> Send for BlockAllocator<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for BlockAllocator<T, N> {}

/// `AllocBlock` implements the [`Allocator`] trait using the underlying [`BlockAllocator`].
///
/// Because [`BlockAllocator`] is lock-free and `Sync`, no `RefCell` is needed.
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

/// Iterator over the free list entries in a BlockAllocator.
struct FreeListIterator<T> {
    current: Option<NonNull<Entry<T>>>,
}

impl<T> Iterator for FreeListIterator<T> {
    type Item = NonNull<Entry<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            // Safety: current is a valid entry in the freelist; next is either a valid pointer or null.
            unsafe {
                self.current = NonNull::new(current.as_ref().next);
            }
            Some(current)
        } else {
            None
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

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_block_allocator() {
        random_test(100, |rng| {
            let allocator: BlockAllocator<u64, 256> = BlockAllocator::new();

            // Allocate 1000 elements, recording each pointer alongside its written value.
            let mut allocated: Vec<(NonNull<u64>, u64)> = Vec::new();
            for _ in 0..1000 {
                let ptr = allocator.allocate_object().unwrap();
                let value: u64 = rng.random();
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

            // Reallocate 500 elements to exercise the freelist and verify no aliasing.
            for _ in 0..500 {
                let ptr = allocator.allocate_object().unwrap();
                let value: u64 = rng.random();
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
}
