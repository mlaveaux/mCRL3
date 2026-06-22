use std::collections::HashSet;
use std::error::Error;

use glob::glob;

/// Discovers test files with specific extensions and prints test cases for them
pub fn discover_tests() -> Result<(), Box<dyn Error>> {
    // Discover different types of test files
    discover_files_with_extension("mcrl2", "examples/mCRL2/**/*.mcrl2")?;
    discover_files_with_extension("mcf", "examples/mCRL2/**/*.mcf")?;
    discover_files_with_extension("dataspec", "examples/REC/**/*.dataspec")?;

    Ok(())
}

/// Discovers files matching a pattern and prints a `#[test_case]` annotation for each unique
/// filename. Errors if the pattern matches no files, since that usually indicates a typo.
fn discover_files_with_extension(ext_name: &str, pattern: &str) -> Result<(), Box<dyn Error>> {
    // Track seen filenames to avoid duplicates
    let mut seen_filenames = HashSet::new();

    for path_result in glob(pattern)? {
        let path = path_result?;

        // Normalize path separators to forward slashes for cross-platform compatibility.
        let normalized_path = path.to_string_lossy().replace('\\', "/");
        let filename = path.file_name().unwrap_or_default().to_string_lossy();

        // Replace spaces with underscores and convert to lowercase for consistent test naming.
        let sanitized_filename = filename.replace(' ', "_").to_lowercase();

        // Only generate a test case if this filename hasn't been seen before.
        if seen_filenames.insert(sanitized_filename.clone()) {
            println!(
                "#[test_case(include_str!(\"../../../{normalized_path}\"), \"tests/snapshot/result_{sanitized_filename}\" ; \"{sanitized_filename}\")]"
            );
        }
    }

    if seen_filenames.is_empty() {
        return Err(format!("No {ext_name} test files were discovered for pattern '{pattern}'").into());
    }

    Ok(())
}
