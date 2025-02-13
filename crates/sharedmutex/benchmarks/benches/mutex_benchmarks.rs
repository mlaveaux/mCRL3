use std::sync::{Arc, RwLock, Mutex};

use criterion::Criterion;

use bf_sharedmutex::BfSharedMutex;

use benchmarks::{benchmark, NUM_ITERATIONS, READ_RATIOS, THREADS};

/// Benchmark the bfsharedmutex implementation
pub fn benchmark_bfsharedmutex(c: &mut Criterion) {
    for num_threads in THREADS {
        for read_ratio in READ_RATIOS {

            // Benchmark various configurations.
            benchmark(
                c,
                "bf-sharedmutex::BfSharedMutex",
                BfSharedMutex::new(()),
                |shared| {
                    let _guard = shared.read().unwrap();
                },
                |shared| {
                    let _guard = shared.write().unwrap();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );
        }
    }
}