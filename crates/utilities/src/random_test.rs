use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::test_logger;

/// Constructs a random number generator that should be used in random tests. Prints its seed to the console for reproducibility.
pub fn random_test<F>(iterations: usize, test_function: F)
where
    F: Fn(&mut StdRng),
{
    let _ = test_logger();

    let seed: u64 = rand::random();
    println!("seed: {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    for _ in 0..iterations {
        test_function(&mut rng);
    }
}
