use core::num;
use std::array::from_fn;
use std::collections::VecDeque;
use std::hint::black_box;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use mcrl3_aterm::ATerm;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::ATermSend;
use mcrl3_aterm::Protected;
use mcrl3_aterm::Symb;
use mcrl3_aterm::Symbol;
use mcrl3_aterm::Term;

/// Executes a function across multiple threads and measures total execution time.
/// The main thread executes with id 0, while additional threads get ids 1..number_of_threads-1.
fn benchmark_threads<F>(number_of_threads: usize, f: F)
where
    F: Fn(usize) + Send + Sync + 'static,
{
    debug_assert!(number_of_threads > 0, "Number of threads must be greater than 0");

    let f = Arc::new(f);

    // Initialize worker threads (excluding main thread)
    let mut handles = Vec::with_capacity(number_of_threads - 1);
    for id in 1..number_of_threads {
        let f_clone = f.clone();
        let handle = thread::spawn(move || {
            f_clone(id);
        });
        handles.push(handle);
    }

    // Run the benchmark on the main thread with id 0
    f(0);

    // Wait for all worker threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked during benchmark");
    }
}

/// Creates a nested function application f_depth where f_0 = c and f_i = f(f_{i-1}, ..., f_{i-1}).
/// This version uses dynamic argument creation and ATerm construction.
fn create_nested_function_dynamic(
    function_name: &str,
    leaf_name: &str,
    number_of_arguments: usize,
    depth: usize,
) -> ATerm {
    debug_assert!(number_of_arguments > 0, "Number of arguments must be greater than 0");
    debug_assert!(depth > 0, "Depth must be greater than 0");

    // Create function symbols for the nested function and the leaf constant
    let f_symbol = Symbol::new(function_name, number_of_arguments);
    let c_symbol = Symbol::new(leaf_name, 0);

    // Create the leaf term (constant with arity 0)
    let c_term = ATerm::constant(&c_symbol);

    // Initialize arguments with the constant term
    let mut arguments: Vec<ATermRef<'_>> = Vec::new();
    arguments.resize_with(number_of_arguments, || c_term.copy());

    // Create initial function application f(c, c, ..., c)
    let mut f_term = ATerm::with_args(&f_symbol, &arguments);

    // Build nested structure by repeatedly applying f to previous results
    for _ in 0..depth {
        // Update all arguments to point to the current f_term
        let mut arguments: Vec<ATermRef<'_>> = Vec::new();
        for arg in &mut arguments {
            *arg = f_term.copy();
        }

        // Create new function application f(f_term, f_term, ..., f_term)
        f_term = ATerm::with_args(&f_symbol, &arguments);
    }

    debug_assert_eq!(f_term.get_head_symbol().name(), function_name);
    debug_assert_eq!(f_term.get_head_symbol().arity(), number_of_arguments);

    f_term
}

/// Optimized version for function symbols with a constant arity.
/// Creates a nested function application where f_0 = c and f_i = f(f_{i-1}, f_{i-1}).
fn create_nested_function<const ARITY: usize>(function_name: &str, leaf_name: &str, depth: usize) -> ATerm {
    debug_assert!(depth > 0, "Depth must be greater than 0");

    // Create function symbols
    let f_symbol = Symbol::new(function_name, 2);
    let c_symbol = Symbol::new(leaf_name, 0);

    // Create the leaf term c
    let c_term = ATerm::constant(&c_symbol);

    // Initialize with f(c, ..., c)
    let mut f_term = ATerm::with_args(&f_symbol, &from_fn::<_, ARITY, _>(|_| c_term.copy()));

    // Build nested structure: f(f_term, ..., f_term) for each level
    for _ in 0..depth {
        f_term = ATerm::with_args(&f_symbol, &from_fn::<_, ARITY, _>(|_| f_term.copy()));
    }

    debug_assert_eq!(f_term.get_head_symbol().name(), function_name);
    debug_assert_eq!(f_term.get_head_symbol().arity(), 2);

    f_term
}

pub const THREADS: [usize; 6] = [1, 2, 4, 8, 16, 20];

// In these three benchmarks all threads operate on a shared term.
fn benchmark_shared_creation(c: &mut Criterion) {
    const SIZE: usize = 400000;

    for num_threads in THREADS {
        c.bench_function(&format!("shared_creation_{}", num_threads), |b| {
            b.iter(|| {
                benchmark_threads(num_threads, |_id| {
                    black_box(create_nested_function::<2>("f", "c", SIZE));
                });
            });
        });
    }
}

fn benchmark_shared_inspect(c: &mut Criterion) {
    const SIZE: usize = 20;
    const ITERATIONS: usize = 1000;

    let shared_term = Arc::new(ATermSend::from(create_nested_function::<2>("f", "c", SIZE)));

    for num_threads in THREADS {
        c.bench_function(&format!("shared_inspect_{}", num_threads), |b| {
            b.iter(|| {
                let term = shared_term.clone();

                benchmark_threads(num_threads, move |_id| {
                    let mut queue: Protected<VecDeque<ATermRef<'static>>> = Protected::new(VecDeque::new());

                    for _ in 0..ITERATIONS / num_threads {
                        // Simple breadth-first search to count elements
                        let mut write = queue.write();
                        let t = write.protect(&term.deref());
                        write.push(t);

                        while let Some(current_term) = write.pop() {
                            // Iterate through all arguments of the current term
                            for arg in current_term.arguments() {
                                write.push(arg);
                            }
                        }

                        write.clear(); // Reuse the queue for next iteration
                    }
                });
            });
        });
    }
}

fn benchmark_shared_lookup(c: &mut Criterion) {
    env_logger::init();

    const SIZE: usize = 400000;
    const ITERATIONS: usize = 1000;

    // Keep one protected instance
    let _term = create_nested_function::<2>("f", "c", SIZE);

    for num_threads in THREADS {
        c.bench_function(&format!("shared_lookup_{}", num_threads), |b| {
            b.iter(|| {
                benchmark_threads(num_threads, move |_id| {
                    for _ in 0..ITERATIONS / 1 {
                        black_box(create_nested_function::<2>("f", "c", SIZE));
                    }
                });
            })
        });
    }
}

// In these three benchmarks all threads operate on their own separate term.
fn benchmark_unique_creation(c: &mut Criterion) {
    const SIZE: usize = 400000;

    for num_threads in THREADS {
        c.bench_function(&format!("shared_creation_{}", num_threads), |b| {
            b.iter(|| {
                benchmark_threads(num_threads, |id| {
                    black_box(create_nested_function::<2>("f", &format!("c{}", id), SIZE));
                });
            });
        });
    }
}

fn benchmark_unique_inspect(c: &mut Criterion) {
    const SIZE: usize = 20;
    const ITERATIONS: usize = 1000;

    for num_threads in THREADS {
        let terms: Arc<Vec<ATermSend>> = Arc::new(
            (0..num_threads)
                .map(|id| ATermSend::from(create_nested_function::<2>("f", &format!("c{}", id), SIZE)))
                .collect(),
        );

        c.bench_function(&format!("shared_inspect_{}", num_threads), |b| {
            b.iter(|| {
                let terms = terms.clone();

                benchmark_threads(num_threads, move |id| {
                    let mut queue: Protected<VecDeque<ATermRef<'static>>> = Protected::new(VecDeque::new());

                    for _ in 0..ITERATIONS / num_threads {
                        // Simple breadth-first search to count elements
                        let mut write = queue.write();
                        let t = write.protect(&terms[id]);
                        write.push(t);

                        while let Some(current_term) = write.pop() {
                            // Iterate through all arguments of the current term
                            for arg in current_term.arguments() {
                                write.push(arg);
                            }
                        }

                        write.clear(); // Reuse the queue for next iteration
                    }
                });
            });
        });
    }
}

fn benchmark_unique_lookup(c: &mut Criterion) {
    env_logger::init();

    const SIZE: usize = 400000;
    const ITERATIONS: usize = 1000;

    // Keep one protected instance

    for num_threads in THREADS {
        c.bench_function(&format!("shared_lookup_{}", num_threads), |b| {
            b.iter(|| {
                benchmark_threads(num_threads, move |_id| {
                    for _ in 0..ITERATIONS / 1 {
                        black_box(create_nested_function::<2>("f", "c", SIZE));
                    }
                });
            })
        });
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark_shared_creation,
        benchmark_unique_creation,
        benchmark_shared_inspect,
        benchmark_unique_inspect,
        benchmark_shared_lookup,
        benchmark_unique_lookup,
);
criterion_main!(benches);
