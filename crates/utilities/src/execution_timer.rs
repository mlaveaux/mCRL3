//! # Examples
//!
//! Using the execution timer:
//!
//! ```
//! use utilities::ExecutionTimer;
//! use std::thread::sleep;
//! use std::time::Duration;
//!
//! let mut timer = ExecutionTimer::new("example_tool", "");
//!
//! // Time multiple operations
//! timer.start("operation1");
//! sleep(Duration::from_millis(10));
//! timer.finish("operation1");
//!
//! timer.start("operation2");
//! sleep(Duration::from_millis(20));
//! timer.finish("operation2");
//!
//! // Print timings to stderr (since filename is empty)
//! timer.report().expect("Failed to write timing report");
//! ```

use crate::timer::SimpleTimer;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{self};

/// Records CPU time used by different parts of the code with a tool name and optional output file.
/// Output is written in YAML format to a file or stderr.
#[derive(Debug)]
pub struct ExecutionTimer {
    /// Name of the tool being timed
    tool_name: String,
    /// Output file path (empty for stderr)
    filename: String,
    /// Collection of timings by name
    timings: BTreeMap<String, SimpleTimer>,
}

impl ExecutionTimer {
    /// Creates a new execution timer with specified tool name and output file.
    pub fn new(tool_name: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            filename: filename.into(),
            timings: BTreeMap::new(),
        }
    }

    /// Starts timing measurement with the given name.
    ///
    /// # Panics
    /// Panics if timing with this name already exists
    pub fn start(&mut self, timing_name: impl Into<String>) {
        let name = timing_name.into();
        if self.timings.contains_key(&name) {
            panic!(
                "Starting already known timing '{}'. This causes unreliable results.",
                name
            );
        }

        let mut timer = SimpleTimer::new();
        timer.start();
        self.timings.insert(name, timer);
    }

    /// Finishes timing measurement with the given name.
    ///
    /// # Panics
    /// Panics if timing wasn't started or was already finished
    pub fn finish(&mut self, timing_name: &str) {
        if let Some(timer) = self.timings.get_mut(timing_name) {
            if !timer.running() {
                panic!("Finishing timing '{}' for the second time.", timing_name);
            }
            timer.stop();
        } else {
            panic!("Finishing timing '{}' that was not started.", timing_name);
        }
    }

    /// Writes timing information in YAML format to the configured output.
    pub fn report(&self) -> io::Result<()> {
        if self.filename.is_empty() {
            self.write_report(&mut io::stderr())
        } else {
            let mut file = OpenOptions::new().create(true).append(true).open(&self.filename)?;
            self.write_report(&mut file)
        }
    }

    fn write_report(&self, writer: &mut impl Write) -> io::Result<()> {
        writeln!(writer, "- tool: {}", self.tool_name)?;
        writeln!(writer, "  timing:")?;

        for (name, timer) in &self.timings {
            if !timer.running() {
                writeln!(writer, "    {}: {:.3}s", name, timer.elapsed().as_secs_f64())?;
            } else {
                writeln!(writer, "    {}: did not finish.", name)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_basic_timing() {
        let mut timer = ExecutionTimer::new("test_tool", "");

        timer.start("operation");
        sleep(Duration::from_millis(10));
        timer.finish("operation");

        let mut output = Vec::new();
        timer.write_report(&mut output).unwrap();

        let report = String::from_utf8(output).unwrap();
        assert!(report.contains("test_tool"));
        assert!(report.contains("operation:"));
    }

    #[test]
    #[should_panic(expected = "Starting already known timing")]
    fn test_double_start() {
        let mut timer = ExecutionTimer::new("test", "");
        timer.start("op");
        timer.start("op");
    }

    #[test]
    #[should_panic(expected = "Finishing timing")]
    fn test_finish_nonexistent() {
        let mut timer = ExecutionTimer::new("test", "");
        timer.finish("op");
    }

    #[test]
    #[should_panic(expected = "for the second time")]
    fn test_double_finish() {
        let mut timer = ExecutionTimer::new("test", "");
        timer.start("op");
        timer.finish("op");
        timer.finish("op");
    }
}
