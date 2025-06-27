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

/// Discovers files matching a pattern and generates test cases for them
/// Returns the number of unique files found
fn discover_files_with_extension(ext_name: &str, pattern: &str) -> Result<(), Box<dyn Error>> {
    // Track seen filenames to avoid duplicates
    let mut seen_filenames = HashSet::new();

    // Glob returns a Result<impl Iterator<Item=Result<PathBuf>>>
    match glob(pattern) {
        Ok(paths) => {
            for path_result in paths {
                match path_result {
                    Ok(path) => {
                        // Get the relative path and filename
                        let path_str = path.to_string_lossy();
                        // Normalize path separators to forward slashes for cross-platform compatibility
                        let normalized_path = path_str.replace('\\', "/");
                        let filename = path.file_name().unwrap_or_default().to_string_lossy();

                        // Replace spaces with underscores and convert to lowercase for consistent test naming
                        let sanitized_filename = filename.replace(' ', "_").to_lowercase();

                        // Only generate test case if this filename hasn't been seen before
                        if !seen_filenames.contains(&sanitized_filename) {
                            // Generate the test case string with normalized path
                            println!(
                                "#[test_case(include_str!(\"../../../{normalized_path}\"), \"tests/snapshot/result_{sanitized_filename}\" ; \"{sanitized_filename}\")]"
                            );

                            // Add sanitized filename to set of seen filenames
                            seen_filenames.insert(sanitized_filename);
                        }
                    }
                    Err(e) => {
                        return Err(Box::new(e));
                    }
                }
            }
        }
        Err(e) => return Err(Box::new(e)),
    }

    // Additional debug assertion for specific file type
    debug_assert!(!seen_filenames.is_empty(), "No {ext_name} test files were discovered");

    Ok(())
}
