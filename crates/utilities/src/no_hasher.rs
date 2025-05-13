use std::hash::BuildHasher;
use std::hash::Hasher;

/// A hasher that directly uses the value provided to write_u64 as the hash
pub struct NoHasher(u64);

impl Hasher for NoHasher {
    /// Returns the current value as the hash
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        // This implementation only supports write_u64
        debug_assert!(false, "NoHasher only supports write_u64");
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

/// A builder for NoHasher. Starts with a hash of 0 and returns whatever value is passed to write_u64
#[derive(Default)]
pub struct NoHasherBuilder;

impl BuildHasher for NoHasherBuilder {
    type Hasher = NoHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoHasher(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_hasher() {
        let mut hasher = NoHasher(0);
        hasher.write_u64(42);
        assert_eq!(
            hasher.finish(),
            42,
            "NoHasher should return the value passed to write_u64"
        );

        let builder = NoHasherBuilder;
        let hasher = builder.build_hasher();
        assert_eq!(
            hasher.finish(),
            0,
            "NoHasherBuilder should create a hasher with initial value 0"
        );
    }
}
