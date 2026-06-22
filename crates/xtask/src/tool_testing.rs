use std::env;
use std::error::Error;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::path::Path;

use duct::cmd;
use which::which_in;

/// Runs a smoke test of the packaged tool binaries to ensure that basic functionality works.
///
/// `directory` is expected to be a release package directory laid out as produced by
/// `xtask package`, with an `examples/` directory as a sibling (i.e. `<directory>/../examples`).
pub fn test_tools(directory: &Path) -> Result<(), Box<dyn Error>> {
    // Create a unique temporary directory to perform the tests in, scoped to this process so
    // concurrent or repeated runs do not clash, and clean it up afterwards.
    let tmp_path = env::temp_dir().join(format!("merc-test-tools-{}", std::process::id()));
    create_dir_all(&tmp_path)?;

    let result = run_tool_tests(directory, &tmp_path);

    // Best-effort cleanup; do not mask the test result with a cleanup error.
    let _ = remove_dir_all(&tmp_path);

    result
}

fn run_tool_tests(directory: &Path, tmp_path: &Path) -> Result<(), Box<dyn Error>> {
    // Find the binaries
    let merc_lts = which_in("merc-lts", Some(directory), env::current_dir()?)?;

    // Copy some test files to the temporary directory.
    std::fs::copy(directory.join("../examples/lts/abp.aut"), tmp_path.join("abp.aut"))?;

    for algorithm in ["strong-bisim", "branching-bisim", "weak-bisim"] {
        cmd!(
            &merc_lts,
            "reduce",
            algorithm,
            "abp.aut",
            "--output",
            format!("abp.{algorithm}.aut")
        )
        .dir(tmp_path)
        .run()?;
    }

    Ok(())
}
