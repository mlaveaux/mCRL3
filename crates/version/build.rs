use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let output = String::from_utf8(output.stdout).expect("Invalid UTF-8 in git output");
    let build_hash = output.trim();

    println!("cargo:rustc-env=BUILD_HASH={}", build_hash);
}
