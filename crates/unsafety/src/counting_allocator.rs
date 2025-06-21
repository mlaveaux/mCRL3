use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;
use std::fmt;
use std::ptr::NonNull;

use allocator_api2::alloc::AllocError;
use allocator_api2::alloc::Allocator;

/// An allocator that can be used to count performance metrics
/// on the allocations performed.
pub struct AllocCounter {
    number_of_allocations: Cell<usize>,
    size_of_allocations: Cell<usize>,

    total_number_of_allocations: Cell<usize>,
    total_size_of_allocations: Cell<usize>,

    max_number_of_allocations: Cell<usize>,
    max_size_of_allocations: Cell<usize>,
}

pub struct AllocMetrics {     
    number_of_allocations: usize,
    size_of_allocations: usize,

    total_number_of_allocations: usize,
    total_size_of_allocations: usize,

    max_number_of_allocations: usize,
    max_size_of_allocations: usize,
}

impl fmt::Display for AllocMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Current allocations: {} (size: {} bytes)\n", 
            self.number_of_allocations, self.size_of_allocations)?;
        write!(f, "Total allocations: {} (size: {} bytes)\n", 
            self.total_number_of_allocations, self.total_size_of_allocations)?;
        write!(f, "Peak allocations: {} (size: {} bytes)", 
            self.max_number_of_allocations, self.max_size_of_allocations)
    }
}

impl AllocCounter {
    /// Returns the performance metrics of the
    pub fn get_metrics(&self) -> AllocMetrics {
        AllocMetrics {
            number_of_allocations: self.number_of_allocations.get(),
            size_of_allocations: self.size_of_allocations.get(),

            total_number_of_allocations: self.total_number_of_allocations.get(),
            total_size_of_allocations: self.total_size_of_allocations.get(),

            max_number_of_allocations: self.max_number_of_allocations.get(),
            max_size_of_allocations: self.max_size_of_allocations.get(),
        }
    }

    fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe {
            System.alloc(layout)
        };

        if !ret.is_null() {
            self.number_of_allocations.set(self.number_of_allocations.get() + 1);
            self.size_of_allocations.set(self.size_of_allocations.get() + layout.size());

            self.total_number_of_allocations.set(self.total_number_of_allocations.get() + 1);
            self.total_size_of_allocations.set(self.total_size_of_allocations.get() + layout.size());

            self.max_number_of_allocations.set(self.max_number_of_allocations.get().max(self.number_of_allocations.get()));
            self.max_size_of_allocations.set(self.max_size_of_allocations.get().max(self.size_of_allocations.get()));
        }

        ret
    }
    
    fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            System.dealloc(ptr, layout);
        }

        self.number_of_allocations.set(self.number_of_allocations.get() - 1);
        self.size_of_allocations.set(self.size_of_allocations.get() - layout.size());
    }
}

unsafe impl GlobalAlloc for AllocCounter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.dealloc(ptr, layout)
    }
}

unsafe impl Allocator for AllocCounter {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.alloc(layout);

        if ptr.is_null() {
            return Err(AllocError);
        }

        let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr, layout.size());
        Ok(NonNull::new(slice_ptr).expect("The resulting ptr will never be null"))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.dealloc(ptr.as_ptr(), layout)
    }
}