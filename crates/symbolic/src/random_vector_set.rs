use oxidd::ldd::Value;
use rand::Rng;
use rand::RngExt;
use std::collections::HashSet;

/// Returns a vector of the given length with random u64 values (from 0..max_value).
pub fn random_vector<R: Rng>(rng: &mut R, length: usize, max_value: Value) -> Vec<Value> {
    let mut vector: Vec<Value> = Vec::new();
    for _ in 0..length {
        vector.push(rng.random_range(0..max_value));
    }

    vector
}

/// Returns a set of 'amount' vectors where every vector has the given length.
pub fn random_vector_set<R: Rng>(rng: &mut R, amount: usize, length: usize, max_value: Value) -> HashSet<Vec<Value>> {
    let mut result: HashSet<Vec<Value>> = HashSet::new();

    // Insert 'amount' number of vectors into the result.
    for _ in 0..amount {
        result.insert(random_vector(rng, length, max_value));
    }

    result
}
