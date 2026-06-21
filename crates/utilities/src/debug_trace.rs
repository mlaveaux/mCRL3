//! Debug tracing functionality for the Merc toolset
//! This file provides debug tracing that is only active when the merc_debug-trace feature is enabled

/// Macro that prints debug trace information only when the merc_debug-trace feature is enabled.
/// When enabled, this macro delegates to the standard trace! macro from the log crate.
///
/// # Examples
///
/// ```
/// # use merc_utilities::debug_trace;
/// debug_trace!("Processing item {}", 42);
/// debug_trace!("Complex calculation result: {:#?}", [1, 2, 3]);
/// ```
#[macro_export]
#[cfg(feature = "merc_debug-trace")]
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        {
            log::trace!($($arg)*);
        }
    };
}

#[macro_export]
#[cfg(not(feature = "merc_debug-trace"))]
macro_rules! debug_trace {
    ($($arg:tt)*) => {{
        // No-op when merc_debug-trace is not enabled
    }};
}

/// Macro that conditionally uses items only when the merc_debug-trace feature is enabled.
/// This is useful for importing items that are only needed for debug tracing.
///
/// # Examples
///
/// ```
/// # use merc_utilities::debug_use;
/// debug_use!(std::collections::HashMap);
/// debug_use!(std::fmt::Debug);
/// ```
#[macro_export]
#[cfg(feature = "merc_debug-trace")]
macro_rules! debug_use {
    ($($item:tt)*) => {
        use $($item)*;
    };
}

#[macro_export]
#[cfg(not(feature = "merc_debug-trace"))]
macro_rules! debug_use {
    ($($item:tt)*) => {
        // No-op when merc_debug-trace is not enabled
    };
}
