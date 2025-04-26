//!
//! xtask building block operations such as copy, remove, confirm, and more
//!
//!

#![forbid(unsafe_code)]

use std::env;
use std::error::Error;
use std::process::ExitCode;
use std::str::FromStr;

use benchmark::Rewriter;

mod coverage;
mod sanitizer;
mod benchmark;

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let mut args = env::args();

    // Ignore the first argument (which should be xtask)
    args.next();

    // The name of the task
    let task = args.next();

    match task.as_deref() {
        Some("benchmark") => {
            if let Some(rewriter) = args.next() {
                // Use the upstream mcrl2
                if let Some(output_path) = args.next() {
                    benchmark::benchmark(output_path, Rewriter::from_str(&rewriter)?)?
                } else {
                    println!("Missing argument for output file");
                    return Ok(ExitCode::FAILURE);
                }
            } else {
                println!("Missing argument for rewriter");
                return Ok(ExitCode::FAILURE);
            }
        }
        Some("create-table") => {
            if let Some(input_path) = args.next() {
                benchmark::create_table(input_path)?;
            } else {
                println!("Missing argument for input file");
                return Ok(ExitCode::FAILURE);
            }
        }
        Some("coverage") => {
            // Take the other parameters for cargo.
            let other_arguments: Vec<String> = args.collect();
            coverage::coverage(other_arguments)?
        }
        Some("address-sanitizer") => {
            // Take the other parameters for cargo.
            let other_arguments: Vec<String> = args.collect();
            sanitizer::address_sanitizer(other_arguments)?
        }
        Some("memory-sanitizer") => {
            // Take the other parameters for cargo.
            let other_arguments: Vec<String> = args.collect();
            sanitizer::memory_sanitizer(other_arguments)?
        }
        Some("thread-sanitizer") => {
            // Take the other parameters for cargo.
            let other_arguments: Vec<String> = args.collect();
            sanitizer::thread_sanitizer(other_arguments)?
        }
        Some(x) => {
            println!("Unknown task {x}");
            println!();
            print_help();
            return Ok(ExitCode::FAILURE);
        }
        _ => print_help(),
    }

    Ok(ExitCode::SUCCESS)
}

fn print_help() {
    println!("Available tasks: benchmark, coverage, address-sanitizer, thread-sanitizer");
}
