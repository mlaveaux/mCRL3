/// Constructs a logger for tests. This logger will not print anything to the console, but will instead write to a buffer.
pub fn test_logger() {      
    if cfg!(not(feature = "mcrl3_miri")) {
        env_logger::builder().is_test(true).try_init().expect("Failed to initialise emv_logger");
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

    for thread in threads {
        let _ = thread.join();
    }
}
