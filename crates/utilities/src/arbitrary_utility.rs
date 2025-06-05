//! Utility functions for the Arbitrary trait and various types from that crate.
//! 

use rand::Rng;

/// Returns a vector of the given length with random u64 values (from 0..max_value).
pub fn random_unstructured(length: usize) -> Vec<u8> {
    let mut rng = rand::rng();

    let mut vector: Vec<u8> = Vec::new();
    for _ in 0..length {
        vector.push(rng.random());
    }

    vector
}
