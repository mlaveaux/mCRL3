//! Debug tracing functionality for the mCRL3 toolset
//! This file provides debug tracing that is only active when the mcrl3_debug feature is enabled

/// Macro that prints debug trace information only when the mcrl3_debug feature is enabled.
/// When enabled, this macro delegates to the standard trace! macro from the log crate.
/// 
/// # Examples
/// 
/// ```
/// debug_trace!("Processing item {}", item_id);
/// debug_trace!("Complex calculation result: {:#?}", result);
/// ```
#[macro_export]
#[cfg(feature = "mcrl3_debug")]
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        {
            log::trace!($($arg)*);
        }
    };
}

#[macro_export]
#[cfg(not(feature = "mcrl3_debug"))]
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        { 
            // No-op when mcrl3_debug is not enabled
        }
    };
}