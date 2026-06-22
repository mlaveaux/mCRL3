use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::Builder;
use std::thread::JoinHandle;

use log::error;
use merc_utilities::MercError;

/// A thread that continuously runs a closure in a loop that can be paused and stopped.
pub struct PauseableThread {
    /// Handle to the underlying thread.
    handle: Option<JoinHandle<()>>,
    shared: Arc<PauseableThreadShared>,
}

/// Pause state shared between the controller and the worker thread.
struct PauseState {
    /// Whether the worker is currently parked.
    paused: bool,
    /// Incremented on every `resume()` call. The worker records the value it observed before an
    /// iteration and only parks itself afterwards if the value is unchanged, so a `resume()` that
    /// races a self-pause is never lost.
    resume_generation: u64,
}

/// Data that is shared between the main thread and the pauseable thread.
struct PauseableThreadShared {
    /// Whether the thread should keep running. Set to false to signal the thread to stop.
    running: AtomicBool,
    /// The pause state, guarded together so the resume generation cannot be lost.
    state: Mutex<PauseState>,
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
    /// The error_function is called (on the worker thread) with any error returned by init or loop, so
    /// failures are surfaced instead of silently killing the thread; the error is also stored for `poll_error`/`join`.
    pub fn new<C, I, F, E>(
        name: &str,
        init_function: I,
        loop_function: F,
        error_function: E,
    ) -> Result<PauseableThread, std::io::Error>
    where
        I: Fn() -> Result<C, MercError> + Send + 'static,
        F: Fn(&mut C) -> Result<bool, MercError> + Send + 'static,
        E: Fn(&MercError) + Send + 'static,
    {
        let shared = Arc::new(PauseableThreadShared {
            running: AtomicBool::new(true),
            state: Mutex::new(PauseState {
                paused: false,
                resume_generation: 0,
            }),
            cond_var: Condvar::new(),
            error: Mutex::new(None),
        });

        let thread_name = name.to_string();
        let thread = {
            let shared = shared.clone();
            Builder::new().name(name.to_string()).spawn(move || {
                let mut init = match init_function() {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Thread '{thread_name}' failed to initialize: {e}");
                        error_function(&e);
                        *shared.error.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                        return;
                    }
                };

                while shared.running.load(Ordering::Relaxed) {
                    // Wait while paused, then record the resume generation we are servicing.
                    let serviced_generation = {
                        let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
                        while state.paused {
                            state = shared.cond_var.wait(state).unwrap_or_else(|e| e.into_inner());
                        }
                        state.resume_generation
                    };

                    match loop_function(&mut init) {
                        Ok(false) => {
                            // Pause the thread when requested by the loop function, unless a resume
                            // request arrived during this iteration (which would otherwise be lost).
                            let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
                            if state.resume_generation == serviced_generation {
                                state.paused = true;
                            }
                        }
                        Ok(true) => {}
                        Err(e) => {
                            error!("Thread '{thread_name}' terminated with an error: {e}");
                            error_function(&e);
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
        let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        state.paused = true;
        // We notify the condvar that the value has changed.
        self.shared.cond_var.notify_one();
    }

    /// Resume the thread.
    pub fn resume(&self) {
        let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        state.paused = false;
        state.resume_generation = state.resume_generation.wrapping_add(1);
        // We notify the condvar that the value has changed.
        self.shared.cond_var.notify_one();
    }

    /// Returns the error stored by the thread if it has terminated with an error, or `None` if
    /// the thread is still running or terminated successfully.
    pub fn poll_error(&mut self) -> Option<MercError> {
        if self.handle.as_ref().is_none_or(|h| h.is_finished()) {
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

            // After the thread has finished, we can check if it stored an error.
            if let Some(error) = self.poll_error() {
                return Err(error);
            }
        }

        Ok(())
    }
}

impl Drop for PauseableThread {
    fn drop(&mut self) {
        self.stop();

        // Joining consumes the handle. A panic in the worker must not panic the Drop (which would
        // abort under panic = "abort"), so it is logged instead.
        if let Some(handle) = self.handle.take()
            && handle.join().is_err()
        {
            error!("The pauseable thread panicked while it was being dropped");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;

    use crate::PauseableThread;

    #[test]
    fn test_pausablethread() {
        let thread = PauseableThread::new(
            "test",
            || Ok(()),
            move |_| {
                // Do nothing.
                Ok(true)
            },
            |_| {},
        )
        .unwrap();

        thread.stop();
    }

    #[test]
    fn test_pauseablethread_surfaces_errors() {
        // A loop closure that returns an error should invoke the error callback and store the error
        // so it can be observed via `join`, instead of silently killing the thread.
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let mut thread = PauseableThread::new(
            "test error",
            || Ok(()),
            move |_| Err("boom".into()),
            move |_| called_clone.store(true, Ordering::Relaxed),
        )
        .unwrap();

        let result = thread.join();
        assert!(result.is_err(), "expected the stored error to be surfaced by join");
        assert!(
            called.load(Ordering::Relaxed),
            "expected the error callback to be invoked"
        );
    }
}
