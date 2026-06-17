use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use log::debug;

use merc_utilities::MercError;

/// Builds a rayon thread pool with `num_threads` workers pinned round-robin to
/// the available CPU cores.
///
/// If no core information is available on this platform, the pool is still
/// created but worker pinning is skipped.
pub fn configure_rayon_thread_pool(num_threads: usize) -> Result<ThreadPool, MercError> {
    let worker_count = num_threads.max(1);
    let cores = core_affinity2::get_core_ids().unwrap_or_default();

    ThreadPoolBuilder::new()
        .num_threads(worker_count)
        .spawn_handler(move |thread| {
            let thread_index = thread.index();
            let core = if cores.is_empty() {
                None
            } else {
                Some(cores[thread_index % cores.len()])
            };

            let mut builder = std::thread::Builder::new();
            if let Some(name) = thread.name() {
                builder = builder.name(name.to_owned());
            }
            if let Some(stack_size) = thread.stack_size() {
                builder = builder.stack_size(stack_size);
            }

            builder.spawn(move || {
                if let Some(core) = core {
                    match core.set_affinity() {
                        Ok(()) => {
                            debug!("Pinned rayon worker thread {} to core {}", thread_index, core.0);
                        },
                        Err(error) => {
                            debug!("Failed to pin rayon worker thread {} to core {}: {error}", thread_index, core.0);
                        }
                    }
                }

                thread.run();
            })?;

            Ok(())
        })
        .build()
        .map_err(|error| MercError::from(format!("Failed to build thread pool: {error}")))
}