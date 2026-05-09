use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering;

/// Intrusive node requirements for use in [`FreeList`].
///
/// # Safety
///
/// Implementors must guarantee that `get_next` and `set_next` read and write
/// the same link field, and that the field is valid for all nodes managed by
/// a corresponding freelist.
pub unsafe trait FreeListEntry: Sized {
    /// Returns the next pointer for `ptr` (or null).
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a node managed by the corresponding
    /// freelist.
    unsafe fn get_next(ptr: *mut Self) -> *mut Self;

    /// Sets the next pointer for `ptr`.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a node managed by the corresponding
    /// freelist, and `next` must be either null or a valid pointer to a node
    /// managed by the same freelist.
    unsafe fn set_next(ptr: *mut Self, next: *mut Self);
}

/// Lock-free intrusive freelist based on a Treiber stack.
pub struct FreeList<T: FreeListEntry> {
    /// The head of the lock-free freelist. Null means empty.
    head: AtomicPtr<T>,

    /// We implement Send and Sync manually.
    _marker: PhantomData<*mut T>,
}

// Safety: Transferring a FreeList to another thread is safe if T is Send.
unsafe impl<T: FreeListEntry + Send> Send for FreeList<T> {}

// Safety: Concurrent pop transfers ownership of T nodes across threads, which
// requires T: Send.
unsafe impl<T: FreeListEntry + Send> Sync for FreeList<T> {}

impl<T: FreeListEntry> Default for FreeList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FreeListEntry> FreeList<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
            _marker: PhantomData,
        }
    }

    /// Pops one entry from the freelist.
    pub fn try_pop(&self) -> Option<NonNull<T>> {
        let mut head = self.head.load(Ordering::Relaxed);
        loop {
            let node = NonNull::new(head)?;

            // Safety: `head` is non-null and the Acquire on CAS failure synchronizes
            // with the Release in push, guaranteeing we see the correct next pointer.
            let next = unsafe { T::get_next(node.as_ptr()) };

            match self
                .head
                .compare_exchange_weak(head, next, Ordering::Acquire, Ordering::Acquire)
            {
                Ok(_) => return Some(node),
                Err(actual) => head = actual,
            }
        }
    }

    /// Pushes an entry onto the freelist.
    pub fn push(&self, entry: NonNull<T>) {
        let ptr = entry.as_ptr();
        let mut head = self.head.load(Ordering::Relaxed);
        loop {
            // Safety: caller transfers ownership of a valid node; writing its next link is valid.
            unsafe { T::set_next(ptr, head) };

            match self
                .head
                .compare_exchange_weak(head, ptr, Ordering::Release, Ordering::Relaxed)
            {
                Ok(_) => return,
                Err(actual) => head = actual,
            }
        }
    }

    /// Returns an iterator over freelist entries.
    ///
    /// # Safety
    ///
    /// This is only safe when no concurrent push/pop operations are in progress.
    pub unsafe fn iter(&self) -> FreeListIterator<T> {
        FreeListIterator {
            current: NonNull::new(self.head.load(Ordering::Relaxed)),
        }
    }

    /// Returns a mutable iterator over freelist entries.
    ///
    /// # Safety
    ///
    /// This is only safe when no concurrent push/pop operations are in progress.
    pub unsafe fn iter_mut(&mut self) -> FreeListIteratorMut<'_, T> {
        FreeListIteratorMut {
            current: NonNull::new(self.head.load(Ordering::Relaxed)),
            marker: PhantomData,
        }
    }

    /// Clears the freelist head.
    ///
    /// This is only safe when no concurrent push/pop operations are in progress.
    pub fn clear(&mut self) {
        self.head.store(std::ptr::null_mut(), Ordering::Relaxed);
    }
}

/// Iterator over entries in a [`FreeList`].
pub struct FreeListIterator<T: FreeListEntry> {
    current: Option<NonNull<T>>,
}

impl<T: FreeListEntry> Iterator for FreeListIterator<T> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            // Safety: `current` is a freelist node; its link field is valid to read.
            unsafe {
                self.current = NonNull::new(T::get_next(current.as_ptr()));
            }
            Some(current)
        } else {
            None
        }
    }
}

/// Mutable iterator over entries in a [`FreeList`].
pub struct FreeListIteratorMut<'a, T: FreeListEntry> {
    current: Option<NonNull<T>>,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T: FreeListEntry> Iterator for FreeListIteratorMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            // Safety: `current` is a freelist node; its link field is valid to read.
            unsafe {
                let current_ptr = current.as_ptr();
                self.current = NonNull::new(T::get_next(current_ptr));
                Some(&mut *current_ptr)
            }
        } else {
            None
        }
    }
}
