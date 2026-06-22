use mcrl2_sys::cxx::UniquePtr;
use mcrl2_sys::data::ffi::RewriterJitty;
use mcrl2_sys::data::ffi::data_specification;
use mcrl2_sys::data::ffi::mcrl2_create_rewriter_jitty;

#[cfg(feature = "jittyc")]
use mcrl2_sys::data::ffi::RewriterCompilingJitty;
#[cfg(feature = "jittyc")]
use mcrl2_sys::data::ffi::mcrl2_create_rewriter_jittyc;

pub struct DataSpecification {
    spec: UniquePtr<data_specification>,
}

impl DataSpecification {
    /// Creates a new data specification from the given UniquePtr.
    pub(crate) fn new(spec: UniquePtr<data_specification>) -> Self {
        DataSpecification { spec }
    }

    /// Returns a reference to the underlying UniquePtr.
    pub(crate) fn get(&self) -> &UniquePtr<data_specification> {
        &self.spec
    }
}

/// Represents a mcrl2::data::detail::RewriterJitty from the mCRL2 toolset.
///
/// TODO: currently only constructs and owns the underlying rewriter; it exposes
/// no rewrite operation yet, so it is inert beyond holding the C++ object alive.
pub struct Mcrl2RewriterJitty {
    _rewriter: UniquePtr<RewriterJitty>,
}

impl Mcrl2RewriterJitty {
    /// Creates a new Jitty rewriter from the given data specification.
    pub fn new(data_spec: &DataSpecification) -> Self {
        let rewriter = mcrl2_create_rewriter_jitty(data_spec.get());
        Self { _rewriter: rewriter }
    }
}

#[cfg(feature = "jittyc")]
/// Represents a mcrl2::data::detail::RewriterJittyCompiling from the mCRL2 toolset.
///
/// TODO: currently only constructs and owns the underlying rewriter; it exposes
/// no rewrite operation yet, so it is inert beyond holding the C++ object alive.
pub struct Mcrl2RewriterJittyCompiling {
    _rewriter: UniquePtr<RewriterCompilingJitty>,
}

#[cfg(feature = "jittyc")]
impl Mcrl2RewriterJittyCompiling {
    /// Creates a new compiling Jitty rewriter from the given data specification.
    pub fn new(data_spec: &DataSpecification) -> Self {
        let rewriter = mcrl2_create_rewriter_jittyc(data_spec.get());
        Self { _rewriter: rewriter }
    }
}
