use std::hint::black_box;

use bf_vec::BfVec;
use criterion::Criterion;

use benchmarks::{benchmark, NUM_ITERATIONS, READ_RATIOS, THREADS};

pub fn benchmark_vector(c: &mut Criterion) {
    for num_threads in THREADS {
        for read_ratio in READ_RATIOS {

            benchmark(c,
                "bf-sharedmutex::BfVec",
                VecClone::<usize>::new(),
                |x| {
                    black_box(x.vector.len());
                },
                |x| {
                    black_box(x.vector.push(1));
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio);
        }
    }
}

/// A hack where clone does actually call share
struct VecClone<T> {
    vector: BfVec<T>,
}

impl<T> VecClone<T> {
    fn new() -> Self {
        Self {
            vector: BfVec::new()
        }
    }
}

impl<T> Clone for VecClone<T> {
    fn clone(&self) -> Self {
        Self {
            vector: self.vector.share()
        }
    }
}