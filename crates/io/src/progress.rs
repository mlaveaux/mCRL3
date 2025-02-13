//!
//! A utility function to easily print progress information for procedures that
//! take a fixed number of steps. In particular, avoids writing too many
//! progress indications.
//!

/// The struct that can be initialised to keep track of progress counters.
pub struct Progress<F: Fn(usize, usize)> {
    maximum: usize,
    counter: usize,

    message: F,
}

impl<F: Fn(usize, usize)> Progress<F> {
    /// Create a new progress tracker with a given maximum.
    pub fn new(message: F, maximum: usize) -> Progress<F> {
        Progress {
            message,
            maximum,
            counter: 0,
        }
    }

    /// Increase the progress with the given amount, prints periodic progress
    /// messages.
    pub fn add(&mut self, amount: usize) {
        let increment = (self.maximum / 100usize).max(1);

        if (self.counter + amount) / increment > self.counter / increment {
            // Print a progress message when the increment increased.
            (self.message)(self.counter, increment);
        }

        self.counter += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::Progress;

    #[test]
    fn test_progress() {
        // Test with extreme values.
        let _min = Progress::new(|_, _| {}, 0);

        let _progress = Progress::new(|_, _| {}, 1000);
    }
}
