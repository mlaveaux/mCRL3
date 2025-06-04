use rand::{RngCore, SeedableRng};
use rand::rngs::StdRng;

use crate::test_logger;

/// Constructs a random number generator that should be used in random tests. Prints its seed to the console for reproducibility.
pub fn random_test<F>(iterations: usize, mut test_function: F)
where
    F: FnMut(&mut StdRng),
{
    let _ = test_logger();

    let seed: u64 = rand::random();
    println!("seed: {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    for _ in 0..iterations {
        test_function(&mut rng);
    }
}

pub fn random_test_threads<C, F, G>(iterations: usize, num_threads: usize, init_function: G, test_function: F)
where
    C: Send + 'static,
    F: Fn(&mut StdRng, &mut C) + Copy + Send + Sync + 'static,
    G: Fn() -> C,
{
    let _ = test_logger();

    let mut threads = vec![];

    let seed: u64 = rand::random();
    println!("seed: {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    for _ in 0..num_threads {
        let mut rng = StdRng::seed_from_u64(rng.next_u64());
        let mut init = init_function();
        threads.push(std::thread::spawn(move || {
            for _ in 0..iterations {
                test_function(&mut rng, &mut init);
            }
        }));
    }

    for thread in threads {
        let _ = thread.join();
    }
}

