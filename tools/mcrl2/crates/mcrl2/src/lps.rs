
/// A linear process specification.
/// 
/// This is a wrapper around the `mcrl2::lps::specification` class, which
/// represents a linear process specification (LPS) in mCRL2. An LPS is a number
/// of summands that each specify a condition-action-effect triple, and it
/// has an initial state.
pub struct LinearProcessSpecification {

}

/// Read an LPS from a file in the binary mCRL2 format.
pub fn read_lps(filename: &str) -> Result<LinearProcessSpecification, String> {
    unimplemented!()
}

/// Read an LPS from a file in textual format.
pub fn read_lps_text(filename: &str) -> Result<LinearProcessSpecification, String> {
    unimplemented!()
}