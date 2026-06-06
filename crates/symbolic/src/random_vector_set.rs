//! Functions in this module are only relevant for testing purposes.

use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
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

/// Returns a sorted vector of the given length with unique u64 values (from 0..max_value).
pub fn random_sorted_vector<R: Rng>(rng: &mut R, length: usize, max_value: Value) -> Vec<Value> {
    use rand::prelude::IteratorRandom;

    let mut result = (0..max_value).sample(rng, length);
    result.sort();
    result
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

/// Returns an LDD containing all elements of the given iterator over vectors.
pub fn from_iter<'a, I>(manager: &LDDManagerRef, iter: I) -> LDDFunction
where
    I: Iterator<Item = &'a Vec<Value>>,
{
    let mut result = LDDFunction::empty_set(manager).expect("Failed to create the empty set");

    for vector in iter {
        let single = LDDFunction::singleton(manager, vector).expect("Failed to create a singleton");
        result = result.union(&single).expect("Failed to compute the union");
    }

    result
}


/// Returns project(vector, proj), see [project]. Requires proj to be sorted.
pub fn project_vector(vector: &[Value], proj: &[Value]) -> Vec<Value> {
    let mut result = Vec::<Value>::new();
    for i in proj {
        result.push(vector[*i as usize]);
    }
    result
}
