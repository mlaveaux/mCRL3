use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

mod async_benchmarks;
mod mutex_benchmarks;
mod vec_benchmarks;

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::new(10, 0)).sample_size(100);
    targets = mutex_benchmarks::benchmark_bfsharedmutex,
        mutex_benchmarks::benchmark_othermutexes,
        async_benchmarks::benchmark_async,
        vec_benchmarks::benchmark_vector,
);
criterion_main!(benches);