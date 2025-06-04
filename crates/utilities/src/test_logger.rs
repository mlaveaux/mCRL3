/// Constructs a logger for tests. This logger will not print anything to the console, but will instead write to a buffer.
pub fn test_logger() -> Result<(), log::SetLoggerError> {
    env_logger::builder().is_test(true).try_init()
}

pub fn test_threads<F>(num_threads: usize, test_function: F)
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    let _ = test_logger();

    let mut threads = vec![];

    for _ in 0..num_threads {
        threads.push(std::thread::spawn(move || {
            test_function();
        }));
    }

    for thread in threads {
        let _ = thread.join();
    }
}