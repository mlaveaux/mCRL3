use criterion::Criterion;

use benchmarks_unsafety::DashSetWrapper;
use benchmarks_unsafety::EXISTING_PERCENTAGES;
use benchmarks_unsafety::NUM_ITERATIONS;
use benchmarks_unsafety::READ_RATIOS;
use benchmarks_unsafety::ShardedSet;
use benchmarks_unsafety::THREADS;
use benchmarks_unsafety::benchmark;

/// Benchmark the scalability of the sharded hash map across the number of
/// threads, the read/write (get/insert) ratio and the percentage of existing
/// versus newly inserted values.
pub fn benchmark_sharded_hashmap(c: &mut Criterion) {
    for num_threads in THREADS {
        for read_ratio in READ_RATIOS {
            for existing_percent in EXISTING_PERCENTAGES {
                benchmark(
                    c,
                    "unsafety::ShardedHashMap",
                    ShardedSet::new(),
                    num_threads,
                    NUM_ITERATIONS,
                    read_ratio,
                    existing_percent,
                );
            }
        }
    }
}

/// Benchmark the same workloads against [`dashmap::DashSet`] as a baseline to
/// put the sharded hash map's scalability in context.
pub(crate) fn benchmark_dashset(c: &mut Criterion) {
    for num_threads in THREADS {
        for read_ratio in READ_RATIOS {
            for existing_percent in EXISTING_PERCENTAGES {
                benchmark(
                    c,
                    "dashmap::DashSet",
                    DashSetWrapper::new(),
                    num_threads,
                    NUM_ITERATIONS,
                    read_ratio,
                    existing_percent,
                );
            }
        }
    }
}
