use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::Builder;
use std::thread::JoinHandle;

use mcrl3_utilities::MCRL3Error;

/// A thread that can be paused and stopped.
pub struct PauseableThread {
    handle: Option<JoinHandle<()>>,
    shared: Arc<PauseableThreadShared>,
}

struct PauseableThreadShared {
    running: AtomicBool,
    paused: Mutex<bool>,
    cond_var: Condvar,
}

impl PauseableThread {
    /// Spawns a new thread that runs `loop_function` continuously while enabled.
    ///
    /// The init_function is called once when the thread starts, and it can return a value of type `C`.
    /// The loop_function can return false to pause the thread explicitly, or the loop pauses whenever `stop` is called.
    pub fn new<C, I, F>(name: &str, init_function: I, loop_function: F) -> Result<PauseableThread, std::io::Error>
    where
        I: Fn() -> Result<C, MCRL3Error> + Send + 'static,
        F: Fn(&mut C) -> Result<bool, MCRL3Error> + Send + 'static,
    {
        let shared = Arc::new(PauseableThreadShared {
            running: AtomicBool::new(true),
            paused: Mutex::new(false),
            cond_var: Condvar::new(),
        });

        let thread = {
            let shared = shared.clone();
            Builder::new().name(name.to_string()).spawn(move || {
                let mut init = init_function().unwrap();

                while shared.running.load(std::sync::atomic::Ordering::Relaxed) {
                    // Check if paused is true and wait for it.
                    {
                        let mut paused = shared.paused.lock().unwrap();
                        while *paused {
                            paused = shared.cond_var.wait(paused).unwrap();
                        }
                    }

                    if !loop_function(&mut init).unwrap() {
                        // Pause the thread when requested by the loop function.
                        *shared.paused.lock().unwrap() = true;
                    }
                }
            })
        }?;

        Ok(PauseableThread {
            handle: Some(thread),
            shared,
        })
    }

    /// Signal the thread to quit, will be joined when it is dropped.
    pub fn stop(&self) {
        self.shared.running.store(false, Ordering::Relaxed);
        self.resume();
    }

    /// Pause the thread on the next iteration.
    pub fn pause(&self) {
        *self.shared.paused.lock().unwrap() = true;
        // We notify the condvar that the value has changed.
        self.shared.cond_var.notify_one();
    }

    /// Resume the thread.
    pub fn resume(&self) {
        *self.shared.paused.lock().unwrap() = false;
        // We notify the condvar that the value has changed.
        self.shared.cond_var.notify_one();
    }
}

impl Drop for PauseableThread {
    fn drop(&mut self) {
        self.stop();

        // Joining consumes the handle
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pausablethread() {
        let thread = PauseableThread::new("test", || Ok(()), move |_| {
            // Do nothing.
            Ok(true)
        })
        .unwrap();

        thread.stop();
    }
}
