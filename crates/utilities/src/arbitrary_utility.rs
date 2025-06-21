//! Utility functions for the Arbitrary trait and various types from that crate.
//!

use arbitrary::Unstructured;
use rand::Rng;

/// Generate a readable string by creating a sequence of ASCII characters
/// This ensures the generated string is valid UTF-8 and human-readable
pub fn readable_string(u: &mut Unstructured<'_>, length: usize) -> arbitrary::Result<String> {
    let len = u.int_in_range(1..=length)?;
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        let ch = u.int_in_range(b'a'..=b'z')? as char;
        s.push(ch);
    }

    Ok(s)
}

/// Returns a vector of the given length with random u64 values (from 0..max_value).
pub fn random_unstructured(length: usize) -> Vec<u8> {
    let mut rng = rand::rng();

    let mut vector: Vec<u8> = Vec::new();
    for _ in 0..length {
        vector.push(rng.random());
    }

    vector
}
