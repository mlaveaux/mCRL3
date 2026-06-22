use glob::glob;
use std::env;
use std::error::Error;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::remove_file;
use std::path::PathBuf;

use duct::cmd;

///
/// Remove a set of files given a glob
///
/// # Errors
/// Fails if listing or removal fails
///
fn clean_files(pattern: &str) -> Result<(), Box<dyn Error>> {
    let files: Result<Vec<PathBuf>, _> = glob(pattern)?.collect();
    files?.iter().try_for_each(|path| {
        remove_file(path)?;
        Ok(())
    })
}

///
/// Run coverage, pass the given arguments to cargo.
///
pub(crate) fn coverage(arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    // Ignore errors about missing directory.
    let _ = remove_dir_all("target/coverage");
    create_dir_all("target/coverage")?;

    println!("=== running coverage ===");

    // The path from which cargo is called.
    let mut base_directory = env::current_dir()?;
    base_directory.push("target");
    base_directory.push("coverage");

    let mut prof_directory = base_directory.clone();
    prof_directory.push("cargo-test-%p-%m.profraw");

    cmd("cargo", arguments)
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-C instrument-coverage -Z coverage-options=condition")
        .env("LLVM_PROFILE_FILE", prof_directory)
        .run()?;
    println!("ok.");

    println!("=== generating report ===");
    let report_dir = "target/coverage/html";
    cmd!(
        "grcov",
        base_directory,
        "--binary-path",
        "./target/debug/deps",
        "-s",
        ".",
        "-t",
        "html",
        "--branch",
        "--ignore-not-existing",
        "--ignore",
        "**/target/*",
        "-o",
        report_dir,
    )
    .run()?;
    println!("ok.");

    println!("=== cleaning up ===");
    // The `.profraw` files are written under target/coverage; only clean those.
    clean_files("target/coverage/**/*.profraw")?;
    println!("ok.");

    println!("report location: {report_dir}");

    Ok(())
}
