use rand::Rng;
use rand::SeedableRng;

/// Constructs a logger for tests. This logger will not print anything to the console, but will instead write to a buffer.
pub fn test_logger() -> Result<(), log::SetLoggerError> {
    env_logger::builder().is_test(true).try_init()
}

/// Constructs a random number generator that should be used in random tests. Prints its seed to the console for reproducibility.
pub fn test_rng() -> impl Rng {
    use rand::rngs::StdRng;

    let seed: u64 = rand::random();
    println!("seed: {}", seed);
    StdRng::seed_from_u64(seed)
}
