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
}

impl<T: FreeListEntry> Default for FreeList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FreeListEntry> FreeList<T> {
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    /// Pops one entry from the freelist.
    pub fn try_pop(&self) -> Option<NonNull<T>> {
        let mut head = self.head.load(Ordering::Acquire);
        loop {
            if head.is_null() {
                return None;
            }

            // Safety: `head` is the current freelist head. Reading its link field is
            // valid for nodes managed by this freelist.
            let next = unsafe { T::get_next(head) };
            match self
                .head
                .compare_exchange_weak(head, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    // Safety: `head` was checked non-null above and was observed as list head.
                    return Some(unsafe { NonNull::new_unchecked(head) });
                }
                Err(actual) => {
                    head = actual;
                }
            }
        }
    }

    /// Pushes an entry onto the freelist.
    pub fn push(&self, entry: NonNull<T>) {
        let entry = entry.as_ptr();
        let mut head = self.head.load(Ordering::Relaxed);
        loop {
            // Safety: caller provides ownership of a free node; writing its next link is valid.
            unsafe {
                T::set_next(entry, head);
            }

            match self
                .head
                .compare_exchange_weak(head, entry, Ordering::AcqRel, Ordering::Relaxed)
            {
                Ok(_) => return,
                Err(actual) => {
                    head = actual;
                }
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
