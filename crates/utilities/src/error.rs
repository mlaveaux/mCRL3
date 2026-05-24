use core::error::Error;
use core::fmt::Debug;
use core::fmt::Display;

/// The Merc error type. This has a blanket [`From`] impl for any type that implements Rust's [`Error`],
/// meaning it can be used as a "catch all" error. Captures a backtrace that can be printed from this object.
pub struct MercError {
    inner: Box<InnerMercError>,
}

impl MercError {
    /// Attempts to downcast the internal error to the given type.
    ///
    /// This is useful when you need to inspect or handle a specific error type
    /// that may have been wrapped inside a `MercError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use merc_utilities::MercError;
    /// use std::io;
    ///
    /// fn handle_error(err: MercError) {
    ///     // Check if the underlying error is an IO error
    ///     if let Some(io_err) = err.downcast_ref::<io::Error>() {
    ///         println!("IO error occurred: {}", io_err.kind());
    ///     } else {
    ///         println!("Some other error occurred");
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn downcast_ref<E: Error + 'static>(&self) -> Option<&E> {
        self.inner.error.downcast_ref::<E>()
    }
}

/// This type exists to make [`MercError`] use a "thin pointer" instead of a
/// "fat pointer", which reduces the size of our Result by a usize. This does
/// introduce an extra indirection, but error handling is a "cold path". We
/// don't need to optimize it to that degree.
struct InnerMercError {
    /// The underlying error
    error: Box<dyn Error + Send + Sync + 'static>,
    /// A backtrace captured at creation
    backtrace: std::backtrace::Backtrace,
}

// NOTE: writing the impl this way gives us From<&str>
impl<E> From<E> for MercError
where
    Box<dyn Error + Send + Sync + 'static>: From<E>,
{
    #[cold]
    fn from(error: E) -> Self {
        MercError {
            inner: Box::new(InnerMercError {
                error: error.into(),
                backtrace: std::backtrace::Backtrace::capture(),
            }),
        }
    }
}

impl Display for MercError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.inner.error)?;
        {
            let backtrace = &self.inner.backtrace;
            if let std::backtrace::BacktraceStatus::Captured = backtrace.status() {
                writeln!(f, "{backtrace}")?;
            }
        }
        Ok(())
    }
}

impl Debug for MercError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{:?}", self.inner.error)?;
        {
            let backtrace = &self.inner.backtrace;
            if let std::backtrace::BacktraceStatus::Captured = backtrace.status() {
                writeln!(f, "{backtrace}")?;
            }
        }

        Ok(())
    }
}
