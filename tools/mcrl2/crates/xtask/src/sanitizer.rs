use std::error::Error;

pub use duct::cmd;

fn add_target_flag(arguments: &mut Vec<String>) {
    // Derive the architecture from the host so the sanitizer build targets the
    // machine it runs on (e.g. aarch64 on Apple Silicon, not hardcoded x86_64).
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };

    #[cfg(target_os = "linux")]
    {
        arguments.push("--target".to_string());
        arguments.push(format!("{arch}-unknown-linux-gnu"));
    }

    #[cfg(target_os = "macos")]
    {
        arguments.push("--target".to_string());
        arguments.push(format!("{arch}-apple-darwin"));
    }

    // On other hosts no explicit target is added; silence the unused warning.
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let _ = (arch, &arguments);
}

///
/// Run the tests with the address sanitizer enabled to detect memory issues in unsafe code.
///
/// This only works under Linux and MacOS currently and requires the nightly toolchain.
///
pub fn address_sanitizer(mut arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    arguments.push("-Zbuild-std".to_string());

    add_target_flag(&mut arguments);

    cmd("cargo", arguments)
        .env("RUSTFLAGS", "-Zsanitizer=address,leak")
        .env("RUSTDOCFLAGS", "-Zsanitizer=address,leak")
        .env("CFLAGS", "-fsanitize=address,leak")
        .env("CXXFLAGS", "-fsanitize=address,leak")
        .run()?;
    println!("ok.");

    Ok(())
}

///
/// Run the tests with the thread sanitizer enabled to detect data race conditions.
///
/// This only works under Linux and MacOS currently and requires the nightly toolchain.
///
pub fn thread_sanitizer(mut arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    arguments.push("-Zbuild-std".to_string());

    add_target_flag(&mut arguments);

    cmd("cargo", arguments)
        .env("RUSTFLAGS", "-Zsanitizer=thread")
        .env("RUSTDOCFLAGS", "-Zsanitizer=thread")
        .env("CFLAGS", "-fsanitize=thread")
        .env("CXXFLAGS", "-fsanitize=thread")
        .run()?;
    println!("ok.");

    Ok(())
}
