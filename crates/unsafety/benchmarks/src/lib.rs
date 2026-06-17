use std::hint::black_box;
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::{self};

use criterion::Criterion;
use dashmap::DashSet;
use rand::RngExt;
use rand::distr::Bernoulli;
use rand::distr::Distribution;

use merc_unsafety::ShardedHashMap;

/// The number of operations each thread performs per measured iteration.
pub const NUM_ITERATIONS: usize = 50000;
/// The number of threads to use for the benchmarks.
pub const THREADS: [usize; 6] = [1, 2, 4, 8, 16, 20];
/// The (average) number of `get` operations per `insert` operation.
pub const READ_RATIOS: [u32; 4] = [1, 10, 100, 1000];
/// The percentage of the value space that is pre-populated, controlling how
/// many lookups hit and how many insertions find an existing value.
pub const EXISTING_PERCENTAGES: [u64; 5] = [0, 50, 95, 99, 100];
/// The range `[0, VALUE_SPACE)` of `usize` values operations sample from. Kept
/// bounded so the table size stays bounded across criterion's repeated runs.
pub const VALUE_SPACE: u64 = 1 << 16;

/// A concurrent set of `usize` values exercised by the benchmark harness.
/// `clone` must share the underlying storage so all threads hit one instance.
pub trait ConcurrentSet: Clone + Send + 'static {
    /// Returns true if `value` is present.
    fn get(&self, value: usize) -> bool;
    /// Inserts `value`, returning true if it was freshly inserted.
    fn insert(&self, value: usize) -> bool;
}

/// [`ShardedHashMap`] storing the values directly via its convenience API.
#[derive(Clone)]
pub struct ShardedSet(Arc<ShardedHashMap<usize>>);

impl ShardedSet {
    pub fn new() -> ShardedSet {
        ShardedSet(Arc::new(ShardedHashMap::new()))
    }
}

impl Default for ShardedSet {
    fn default() -> ShardedSet {
        ShardedSet::new()
    }
}

impl ConcurrentSet for ShardedSet {
    fn get(&self, value: usize) -> bool {
        self.0.contains(&value)
    }

    fn insert(&self, value: usize) -> bool {
        self.0.insert(value).1
    }
}

/// [`DashSet`] baseline, the concurrent set the rest of the crate uses.
#[derive(Clone)]
pub struct DashSetWrapper(Arc<DashSet<usize>>);

impl DashSetWrapper {
    pub fn new() -> DashSetWrapper {
        DashSetWrapper(Arc::new(DashSet::new()))
    }
}

impl Default for DashSetWrapper {
    fn default() -> DashSetWrapper {
        DashSetWrapper::new()
    }
}

impl ConcurrentSet for DashSetWrapper {
    fn get(&self, value: usize) -> bool {
        self.0.contains(&value)
    }

    fn insert(&self, value: usize) -> bool {
        self.0.insert(value)
    }
}

/// Pre-populates `set` so that roughly `existing_percent` of [`VALUE_SPACE`] is
/// resident, then drives `num_threads` threads that each perform
/// `num_iterations` operations per measured iteration, choosing a `get` over an
/// `insert` with odds `read_ratio` to one. Values are sampled uniformly from
/// [`VALUE_SPACE`], so `existing_percent` sets the lookup hit rate and the share
/// of insertions that find an existing value.
pub fn benchmark<S: ConcurrentSet>(
    c: &mut Criterion,
    name: &str,
    set: S,
    num_threads: usize,
    num_iterations: usize,
    read_ratio: u32,
    existing_percent: u64,
) {
    // Pre-populate the lower part of the value space.
    let existing = VALUE_SPACE * existing_percent / 100;
    for value in 0..existing {
        set.insert(value as usize);
    }

    // Share threads to avoid spawn overhead between samples.
    let mut threads = vec![];

    #[derive(Clone)]
    struct ThreadInfo<S> {
        busy: Arc<AtomicBool>,
        begin_barrier: Arc<Barrier>,
        end_barrier: Arc<Barrier>,
        dist: Bernoulli,
        set: S,
    }

    let info = ThreadInfo {
        busy: Arc::new(AtomicBool::new(true)),
        begin_barrier: Arc::new(Barrier::new(num_threads + 1)),
        end_barrier: Arc::new(Barrier::new(num_threads + 1)),
        // Sampling true means an insertion, occurring once per `read_ratio` ops.
        dist: Bernoulli::from_ratio(1, read_ratio + 1).unwrap(),
        set,
    };

    for _ in 0..num_threads {
        let info = info.clone();
        threads.push(thread::spawn(move || {
            let mut rng = rand::rng();

            loop {
                info.begin_barrier.wait();

                if !info.busy.load(Ordering::SeqCst) {
                    // Quit the thread.
                    break;
                }

                for _ in 0..num_iterations {
                    let value = rng.random_range(0..VALUE_SPACE) as usize;
                    if info.dist.sample(&mut rng) {
                        black_box(info.set.insert(value));
                    } else {
                        black_box(info.set.get(value));
                    }
                }

                info.end_barrier.wait();
            }
        }));
    }

    c.bench_function(
        format!("{name} threads {num_threads} iterations {num_iterations} read-ratio {read_ratio} existing {existing_percent}%").as_str(),
        |bencher| {
            bencher.iter(|| {
                info.begin_barrier.wait();

                info.end_barrier.wait();
            });
        },
    );

    // Tell the threads to quit and wait for them to join.
    info.busy.store(false, Ordering::SeqCst);
    info.begin_barrier.wait();

    for thread in threads {
        thread.join().unwrap();
    }
}
