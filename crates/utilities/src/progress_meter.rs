use std::sync::Arc;

/// Displays progress messages for a task that performs a fixed number of steps.
/// Progress is reported through the logger's status messages.
#[derive(Debug)]
pub struct ProgressMeter {
    /// Current step number
    current: usize,
    /// Total number of steps
    total: usize,
    /// Logger instance for status updates
    logger: Arc<Logger>,
}

impl ProgressMeter {
    /// Creates a new progress meter with the specified number of total steps.
    /// 
    /// # Arguments
    /// * `total` - Total number of steps (0 means unknown)
    /// * `logger` - Logger instance for status updates
    pub fn new(total: usize, logger: Arc<Logger>) -> Self {
        Self {
            current: 0,
            total,
            logger,
        }
    }

    /// Sets the total number of steps for the task.
    pub fn set_size(&mut self, total: usize) {
        self.total = total;
    }

    /// Should be called after every step. Regularly prints a status message.
    /// Messages are shown for each 0.1% progress increment, or more frequently
    /// for tasks with fewer than 1000 steps.
    pub fn step(&mut self) {
        self.current += 1;
        
        // Show progress if:
        // - Task has less than 1000 steps, or
        // - We hit a 0.1% increment, or
        // - We completed the task
        if self.total < 1000 || (self.current % (self.total / 1000) == 0) || self.current == self.total {
            let percentage = 1000 * self.current / self.total;
            self.logger.log(
                LogLevel::Status,
                &format!("{}.{} percent completed", percentage / 10, percentage % 10)
            );
        }
    }

    /// Returns the current step number.
    pub fn current(&self) -> usize {
        self.current
    }

    /// Returns the total number of steps.
    pub fn total(&self) -> usize {
        self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::io::Write;

    // Helper test writer that captures output
    struct TestWriter(Mutex<Vec<u8>>);
    
    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_progress_reporting() {
        let writer = TestWriter(Mutex::new(Vec::new()));
        let logger = Arc::new(Logger::with_output(Box::new(writer)));
        let mut meter = ProgressMeter::new(100, logger.clone());

        // Step through all iterations
        for _ in 0..100 {
            meter.step();
        }

        let output = String::from_utf8(
            logger.output.borrow_mut()
                .downcast_ref::<TestWriter>()
                .unwrap()
                .0.lock()
                .unwrap()
                .clone()
        ).unwrap();

        // Verify some expected output
        assert!(output.contains("0.0 percent"));
        assert!(output.contains("50.0 percent"));
        assert!(output.contains("100.0 percent"));
    }
}