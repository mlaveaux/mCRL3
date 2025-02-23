use std::error::Error;

pub use duct::cmd;

#[allow(clippy::ptr_arg)]
fn add_target_flag(_arguments: &mut Vec<String>) {
    #[cfg(target_os = "linux")]
    {
        _arguments.push("--target".to_string());
        _arguments.push("x86_64-unknown-linux-gnu".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        _arguments.push("--target".to_string());
        _arguments.push("x86_64-apple-darwin".to_string());
    }
}

///
/// Run the tests with the address sanitizer enabled to detect memory issues in unsafe and C++ code.
///
/// This only works under Linux and MacOS currently and requires the nightly toolchain.
///
pub fn address_sanitizer(cargo_arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut arguments: Vec<String> = vec![
        "nextest".to_string(),
        "run".to_string(),
        "-Zbuild-std".to_string(),
        "--no-fail-fast".to_string(),
        "--features".to_string(),
        "unstable".to_string(),
    ];

    add_target_flag(&mut arguments);
    arguments.extend(cargo_arguments);

    cmd("cargo", arguments)
        .env("RUSTFLAGS", "-Zsanitizer=address,undefined,leak")
        .env("CFLAGS", "-fsanitize=address,undefined,leak")
        .env("CXXFLAGS", "-fsanitize=address,undefined,leak")
        .run()?;
    println!("ok.");

    Ok(())
}

///
/// Run the tests with the address sanitizer enabled to detect memory issues in unsafe and C++ code.
///
/// This only works under Linux and MacOS currently and requires the nightly toolchain.
///
pub fn memory_sanitizer(cargo_arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut arguments: Vec<String> = vec![
        "nextest".to_string(),
        "run".to_string(),
        "-Zbuild-std".to_string(),
        "--no-fail-fast".to_string(),
        "--features".to_string(),
        "unstable".to_string(),
    ];

    add_target_flag(&mut arguments);
    arguments.extend(cargo_arguments);

    cmd("cargo", arguments)
        .env("RUSTFLAGS", "-Zsanitizer=memory")
        .env("CFLAGS", "-fsanitize=memory")
        .env("CXXFLAGS", "-fsanitize=memory")
        .run()?;
    println!("ok.");

    Ok(())
}

///
/// Run the tests with the thread sanitizer enabled to detect data race conditions.
///
/// This only works under Linux and MacOS currently and requires the nightly toolchain.
///
pub fn thread_sanitizer(cargo_arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut arguments: Vec<String> = vec![
        "nextest".to_string(),
        "run".to_string(),
        "-Zbuild-std".to_string(),
        "--no-fail-fast".to_string(),
        "--features".to_string(),
        "unstable".to_string(),
    ];

    add_target_flag(&mut arguments);
    arguments.extend(cargo_arguments);

    cmd("cargo", arguments)
        .env("RUSTFLAGS", "-Zsanitizer=thread")
        .env("CFLAGS", "-fsanitize=thread")
        .env("CXXFLAGS", "-fsanitize=thread")
        .run()?;
    println!("ok.");

    Ok(())
}
