use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::alloc::{alloc, dealloc, Layout};

/// Atomic reference counter - a thread-safe reference counting smart pointer
/// Design: Uses atomic operations for thread safety and manual memory management
/// for performance. The reference count and data are stored together in a single allocation.
#[repr(C)]
pub struct AtomicRefCounter<T> {
    ptr: NonNull<RefCountInner<T>>,
}

struct RefCountInner<T> {
    ref_count: AtomicUsize,
    data: T,
}

impl<T> AtomicRefCounter<T> {
    /// Creates a new Arc containing the given value
    /// Uses a single allocation for both counter and data for cache efficiency
    pub fn new(data: T) -> Self {
        let layout = Layout::new::<RefCountInner<T>>();
        
        // Safety: Layout is valid for ArcInner<T>
        let ptr = unsafe { alloc(layout) as *mut RefCountInner<T> };
        
        debug_assert!(!ptr.is_null(), "Allocation failed");
        
        let inner = RefCountInner {
            ref_count: AtomicUsize::new(1),
            data,
        };
        
        // Safety: ptr is valid and aligned, we just allocated it
        unsafe {
            ptr.write(inner);
        }
        
        let arc = AtomicRefCounter {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        };
        
        debug_assert_eq!(arc.strong_count(), 1, "Initial reference count should be 1");
        arc
    }
    
    /// Returns the current reference count
    /// Used primarily for debugging and testing
    pub fn strong_count(&self) -> usize {
        // Safety: ptr is always valid while Arc exists
        unsafe { self.ptr.as_ref().ref_count.load(Ordering::SeqCst) }
    }
    
    /// Gets a reference to the inner data structure
    /// Safety: ptr is guaranteed to be valid while Arc exists
    fn inner(&self) -> &RefCountInner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Clone for AtomicRefCounter<T> {
    /// Creates a new Arc pointing to the same data
    /// Atomically increments the reference count for thread safety
    fn clone(&self) -> Self {
        let old_count = self.inner().ref_count.fetch_add(1, Ordering::Relaxed);
        
        debug_assert!(old_count >= 1, "Reference count should be at least 1 before cloning");
        debug_assert!(old_count < usize::MAX, "Reference count overflow");
        
        let cloned = AtomicRefCounter { ptr: self.ptr };
        
        debug_assert_eq!(self.strong_count(), old_count + 1, "Reference count should be incremented");
        cloned
    }
}

impl<T> Drop for AtomicRefCounter<T> {
    /// Decrements reference count and deallocates if this was the last reference
    /// Uses acquire-release ordering to ensure proper synchronization
    fn drop(&mut self) {
        let old_count = self.inner().ref_count.fetch_sub(1, Ordering::Release);
        
        debug_assert!(old_count >= 1, "Reference count underflow");
        
        if old_count == 1 {
            // Acquire fence ensures we see all writes from other threads
            std::sync::atomic::fence(Ordering::Acquire);
            
            // Safety: We're the last reference, safe to deallocate
            unsafe {
                let layout = Layout::new::<RefCountInner<T>>();
                std::ptr::drop_in_place(self.ptr.as_ptr());
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T> Deref for AtomicRefCounter<T> {
    type Target = T;
    
    /// Provides transparent access to the contained data
    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}

// Safety: Arc<T> is Send if T is Send + Sync
// The atomic reference counting ensures thread safety
unsafe impl<T: Send + Sync> Send for AtomicRefCounter<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicRefCounter<T> {}

#[cfg(test)]
mod tests {
    use super::*;
            
    #[test]
    fn test_multiple_clones_and_drops() {
        let arc1 = AtomicRefCounter::new(Vec::from([1, 2, 3]));
        let arc2 = arc1.clone();
        let arc3 = arc1.clone();
        let arc4 = arc2.clone();
        
        assert_eq!(arc1.strong_count(), 4);
        
        drop(arc2);
        assert_eq!(arc1.strong_count(), 3);
        
        drop(arc3);
        drop(arc4);
        assert_eq!(arc1.strong_count(), 1);
        
        assert_eq!(*arc1, vec![1, 2, 3]);
    }
}
