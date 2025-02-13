use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use log::debug;

#[derive(Default)]
pub struct Timing {
    results: Rc<RefCell<Vec<(String, f32)>>>,
}

pub struct Timer {
    name: String,
    start: Instant,
    results: Rc<RefCell<Vec<(String, f32)>>>,
    registered: bool,
}

impl Timer {
    pub fn finish(&mut self) {
        let time = self.start.elapsed().as_secs_f64();
        debug!("Time {}: {:.3}s", self.name, time);

        // Register the result.
        self.results.borrow_mut().push((self.name.clone(), time as f32));
        self.registered = true
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if !self.registered {
            debug!("Timer {} was dropped before 'finish()'", self.name);
        }
    }
}

impl Timing {
    /// Creates a new timing object to track timers.
    pub fn new() -> Self {
        Self {
            results: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Starts a new timer with the given name.
    pub fn start(&mut self, name: &str) -> Timer {
        Timer {
            name: name.to_string(),
            start: Instant::now(),
            results: self.results.clone(),
            registered: false,
        }
    }

    /// Prints all the finished timers.
    pub fn print(&self) {
        for (name, time) in self.results.borrow().iter() {
            eprintln!("Time {}: {:.3}s", name, time);
        }
    }
}
