#[cfg(feature = "mcrl3_metrics")]
#[global_allocator]
static GLOBAL_ALLOCATOR: counting_allocator::CountingAllocator = counting_allocator::CountingAllocator;

#[cfg(not(target_env = "msvc"))]
#[cfg(not(feature = "mcrl3_metrics"))]
#[cfg(feature = "mcrl3_jemalloc")]
#[global_allocator]
static GLOBAL_ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(feature = "mcrl3_metrics"))]
#[cfg(feature = "mcrl3_mimalloc")]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Prints information from the [AllocCounter].
#[cfg(feature = "mcrl3_metrics")]
pub fn print_allocator_metrics() -> counting_allocator::AllocMetrics {
    GLOBAL_ALLOCATOR.get_metrics()
}

#[cfg(not(feature = "mcrl3_metrics"))]
pub fn print_allocator_metrics() -> () {
    ()
}
