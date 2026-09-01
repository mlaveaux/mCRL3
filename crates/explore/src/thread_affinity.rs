use log::debug;
use log::info;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;

use merc_utilities::MercError;

use crate::CpuTopology;

/// Builds a rayon thread pool with `num_threads` workers.
///
/// When `pinned` is set, workers are pinned round-robin to the available CPU
/// cores, and the measured CPU topology (SMT/shared-cache/socket clustering
/// and inter-core latencies) is logged at info level. If no core information
/// is available on this platform, the pool is still created but worker
/// pinning is skipped.
///
/// # Errors
///
/// Returns an error if the underlying thread pool fails to build.
pub fn configure_rayon_thread_pool(num_threads: usize, pinned: bool) -> Result<ThreadPool, MercError> {
    let worker_count = num_threads.max(1);
    let cores = if pinned {
        match CpuTopology::detect() {
            Ok(topology) => info!("Detected CPU topology:\n{topology}"),
            Err(error) => debug!("Failed to detect CPU topology: {error}"),
        }

        core_affinity2::get_core_ids().unwrap_or_default()
    } else {
        Vec::new()
    };

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
                        }
                        Err(error) => {
                            debug!(
                                "Failed to pin rayon worker thread {} to core {}: {error}",
                                thread_index, core.0
                            );
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
