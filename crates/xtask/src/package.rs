//!
//! Package command for creating release distributions.
//!

use duct::cmd;
use std::env;
use std::error::Error;
use std::fs::copy;
use std::fs::create_dir_all;

/// Returns the platform-specific executable file name for a binary (adds `.exe` on Windows).
fn exe_name(binary_name: &str) -> String {
    if cfg!(windows) {
        format!("{binary_name}.exe")
    } else {
        binary_name.to_string()
    }
}

/// Builds the project in release mode and packages specified binaries into a
/// newly created 'package' directory.
pub fn package() -> Result<(), Box<dyn Error>> {
    // Get the workspace root directory
    let workspace_root = env::current_dir()?;

    // Precondition: Ensure we're in a valid Rust workspace
    debug_assert!(
        workspace_root.join("Cargo.toml").exists(),
        "Must be run from workspace root containing Cargo.toml"
    );

    println!("=== Creating package directory ===");

    // Create package directory for distribution artifacts
    let package_dir = workspace_root.join("package");
    create_dir_all(&package_dir)?;

    println!("=== Building and copying release binaries ===");

    // Mapping from workspace paths to their binaries
    let workspace_binaries = [
        (
            workspace_root.clone(),
            vec!["merc-lts", "merc-rewrite", "merc-vpg", "merc-sym"],
        ),
        (workspace_root.join("tools/gui"), vec!["merc-ltsgraph"]),
        (workspace_root.join("tools/mcrl2"), vec!["merc-pbes", "merc-lps"]),
    ];

    // All workspaces share the root `target/` directory: the `tools/gui` and `tools/mcrl2`
    // workspaces set `target-dir = "../../target"` in their `.cargo/config.toml`, so every
    // release binary ends up under `<workspace_root>/target/release` regardless of which
    // workspace built it.
    let target_release_dir = workspace_root.join("target").join("release");

    // Build all workspaces in release mode
    for (workspace_path, binaries) in &workspace_binaries {
        cmd!("cargo", "build", "--release").dir(workspace_path).run()?;

        for binary_name in binaries {
            let source_path = target_release_dir.join(exe_name(binary_name));
            let dest_path = package_dir.join(exe_name(binary_name));

            // Precondition: Binary must exist after successful build
            assert!(
                source_path.exists(),
                "Binary {binary_name} should exist after cargo build --release"
            );

            copy(&source_path, &dest_path)?;
            println!("Copied {binary_name} to package directory");
        }
    }

    println!("=== Package creation completed ===");
    println!("Package directory: {}", package_dir.display());

    // Add the LICENSE to the package
    let license_src = workspace_root.join("LICENSE");
    let license_dest = package_dir.join("LICENSE");
    copy(&license_src, &license_dest)?;

    // Add KaHyPar configuration used by the symbolic crate
    let kahypar_ini_src = workspace_root.join("crates/symbolic/data/kahypar.ini");
    let kahypar_ini_dest = package_dir.join("kahypar.ini");
    copy(&kahypar_ini_src, &kahypar_ini_dest)?;

    Ok(())
}
