/// Initializes the standard logger for tests. Output is routed through the test
/// harness so it is captured rather than printed unconditionally.
pub fn test_logger() {
    if !cfg!(miri) {
        // Ignore double initialisations in tests since tests are ran in parallel.
        let _ = env_logger::builder().is_test(true).try_init();
    }
}

pub fn test_threads<C, F, G>(num_threads: usize, init_function: G, test_function: F)
where
    C: Send + 'static,
    F: Fn(&mut C) + Copy + Send + Sync + 'static,
    G: Fn() -> C,
{
    test_logger();

    let mut threads = vec![];

    for _ in 0..num_threads {
        let mut init = init_function();
        threads.push(std::thread::spawn(move || {
            test_function(&mut init);
        }));
    }

    // Propagate a worker panic so a failing assertion fails the test instead of
    // being silently swallowed.
    for thread in threads {
        thread.join().unwrap();
    }
}
