use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::Builder;
use std::thread::JoinHandle;

use merc_utilities::MercError;

/// A thread that continuously runs a closure in a loop that can be paused and stopped.
pub struct PauseableThread {
    /// Handle to the underlying thread.
    handle: Option<JoinHandle<()>>,
    shared: Arc<PauseableThreadShared>,
}

/// Data that is shared between the main thread and the pauseable thread.
struct PauseableThreadShared {
    /// Whether the thread should keep running. Set to false to signal the thread to stop.
    running: AtomicBool,
    /// Whether the thread is paused. Set to true to pause the thread, and false to resume it.
    paused: Mutex<bool>,
    /// Condition variable to notify the thread when it should resume.
    cond_var: Condvar,
    /// Error stored by the thread if it terminates due to a failure.
    error: Mutex<Option<MercError>>,
}

impl PauseableThread {
    /// Spawns a new thread that runs `loop_function` continuously while enabled.
    ///
    /// The init_function is called once when the thread starts, and it can return a value of type `C`.
    /// The loop_function can return false to pause the thread explicitly, or the loop pauses whenever `stop` is called.
    pub fn new<C, I, F>(name: &str, init_function: I, loop_function: F) -> Result<PauseableThread, std::io::Error>
    where
        I: Fn() -> Result<C, MercError> + Send + 'static,
        F: Fn(&mut C) -> Result<bool, MercError> + Send + 'static,
    {
        let shared = Arc::new(PauseableThreadShared {
            running: AtomicBool::new(true),
            paused: Mutex::new(false),
            cond_var: Condvar::new(),
            error: Mutex::new(None),
        });

        let thread = {
            let shared = shared.clone();
            Builder::new().name(name.to_string()).spawn(move || {
                let mut init = match init_function() {
                    Ok(v) => v,
                    Err(e) => {
                        *shared.error.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                        return;
                    }
                };

                while shared.running.load(std::sync::atomic::Ordering::Relaxed) {
                    // Check if paused is true and wait for it.
                    {
                        let mut paused = shared.paused.lock().unwrap_or_else(|e| e.into_inner());
                        while *paused {
                            paused = shared.cond_var.wait(paused).unwrap_or_else(|e| e.into_inner());
                        }
                    }

                    match loop_function(&mut init) {
                        Ok(false) => {
                            // Pause the thread when requested by the loop function.
                            *shared.paused.lock().unwrap_or_else(|e| e.into_inner()) = true;
                        }
                        Ok(true) => {}
                        Err(e) => {
                            *shared.error.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                            return;
                        }
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
        *self.shared.paused.lock().unwrap_or_else(|e| e.into_inner()) = true;
        // We notify the condvar that the value has changed.
        self.shared.cond_var.notify_one();
    }

    /// Resume the thread.
    pub fn resume(&self) {
        *self.shared.paused.lock().unwrap_or_else(|e| e.into_inner()) = false;
        // We notify the condvar that the value has changed.
        self.shared.cond_var.notify_one();
    }

    /// Returns the error stored by the thread if it has terminated with an error, or `None` if
    /// the thread is still running or terminated successfully.
    pub fn poll_error(&mut self) -> Option<MercError> {
        if self.handle.as_ref().map_or(true, |h| h.is_finished()) {
            self.shared.error.lock().ok()?.take()
        } else {
            None
        }
    }

    /// Joins the thread and returns its result
    pub fn join(&mut self) -> Result<(), MercError> {
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|e| {
                if let Some(s) = e.downcast_ref::<&'static str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Thread panicked with unknown error".to_string()
                }
            })?;
        }

        Ok(())
    }
}

impl Drop for PauseableThread {
    fn drop(&mut self) {
        self.stop();

        // Joining consumes the handle
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .expect("The thread terminated with an error when the handle was dropped!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pausablethread() {
        let thread = PauseableThread::new(
            "test",
            || Ok(()),
            move |_| {
                // Do nothing.
                Ok(true)
            },
        )
        .unwrap();

        thread.stop();
    }
}
