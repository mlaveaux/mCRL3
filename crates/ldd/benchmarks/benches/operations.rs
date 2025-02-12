use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ldd::*;
use rand::Rng;
use std::collections::HashSet;

/// Returns a vector of the given length with random u64 values (from 0..max_value).
pub fn random_vector(length: usize, max_value: Value) -> Vec<Value> 
{
    let mut rng = rand::thread_rng();    
    let mut vector: Vec<Value> = Vec::new();
    for _ in 0..length
    {
        vector.push(rng.gen_range(0..max_value));
    }

    vector
}

/// Returns a sorted vector of the given length with unique u64 values (from 0..max_value).
pub fn random_sorted_vector(length: usize, max_value: Value) -> Vec<Value> 
{
    use rand::prelude::IteratorRandom;

    let mut rng = rand::thread_rng(); 
    let mut result = (0..max_value).choose_multiple(&mut rng, length);
    result.sort();
    result
}

/// Returns a set of 'amount' vectors where every vector has the given length.
pub fn random_vector_set(amount: usize, length: usize, max_value: Value) ->  HashSet<Vec<Value>>
{
    let mut result: HashSet<Vec<Value>> = HashSet::new();

    // Insert 'amount' number of vectors into the result.
    for _ in 0..amount
    {
        result.insert(random_vector(length, max_value));
    }

    result
}

/// Returns an LDD containing all elements of the given iterator over vectors.
pub fn from_iter<'a, I>(storage: &mut Storage, iter: I) -> Ldd
    where I: Iterator<Item = &'a Vec<Value>>
{
    let mut result = storage.empty_set().clone();

    for vector in iter
    {
        let single = singleton(storage, vector);
        result = union(storage, &result, &single);
    }

    result
}

pub fn criterion_benchmark(c: &mut Criterion) 
{      
    c.bench_function("union 1000", 
    |bencher| 
        {
            let mut storage = Storage::new();

            bencher.iter(
            || {
                let set_a = random_vector_set(1000, 10, 10);
                let set_b = random_vector_set(1000, 10, 10);
            
                let a = from_iter(&mut storage, set_a.iter());
                let b = from_iter(&mut storage, set_b.iter());
            
                black_box(union(&mut storage, &a, &b));
            })
        });

        
    c.bench_function("minus 1000", 
    |bencher| 
        {
            let mut storage = Storage::new();
            
            bencher.iter(
            || {
                let set_a = random_vector_set(1000, 10, 10);
                let set_b = random_vector_set(1000, 10, 10);
            
                let a = from_iter(&mut storage, set_a.iter());
                let b = from_iter(&mut storage, set_b.iter());
            
                black_box(minus(&mut storage, &a, &b));
            })
        });
        

    c.bench_function("relational_product 1000", 
    |bencher| 
        {
            let mut storage = Storage::new();

            bencher.iter(
            || {
                let set = random_vector_set(1000, 10, 10);        
                let relation = random_vector_set(32, 4, 10);

                // Pick arbitrary read and write parameters in order.
                let read_proj = random_sorted_vector(2,9);
                let write_proj = random_sorted_vector(2,9);

                // Compute LDD result.
                let ldd = from_iter(&mut storage, set.iter());
                let rel = from_iter(&mut storage, relation.iter());

                let meta = compute_meta(&mut storage, &read_proj, &write_proj);
                black_box(relational_product(&mut storage, &ldd, &rel, &meta));
            })
        });
}
 	
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);