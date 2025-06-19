#[cfg(feature = "mcrl3_measure-allocs")]
#[global_allocator]
static GLOBAL_ALLOCATOR: counting_allocator::CountingAllocator = counting_allocator::CountingAllocator;

#[cfg(not(target_env = "msvc"))]
#[cfg(not(feature = "mcrl3_measure-allocs"))]
#[cfg(feature = "mcrl3_jemalloc")]
#[global_allocator]
static GLOBAL_ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(feature = "mcrl3_measure-allocs"))]
#[cfg(feature = "mcrl3_mimalloc")]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub fn print_stats() -> usize {
    #[cfg(feature = "mcrl3_measure-allocs")]
    {
        counting_allocator::number_of_allocations()
    }
    #[cfg(not(feature = "mcrl3_measure-allocs"))]
    {
        0
    }
}
