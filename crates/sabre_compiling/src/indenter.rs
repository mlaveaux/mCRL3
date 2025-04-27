use std::fmt::Write;

use indenter::indented;

/// An indentation manager that maintains the current indentation level and provides
/// methods for formatting text with proper indentation.
///
/// The indentation level can be increased with `indent()`, which returns an `Indent`
/// guard that automatically decreases the indentation when dropped.
#[derive(Debug, Clone)]
pub struct Indenter {
    /// The current indentation level (number of tabs)
    level: usize,
    /// The string used for each level of indentation
    tab: String,
}

/// A guard object that decreases indentation when dropped.
/// Created by calling `Indenter::indent()`.
#[derive(Debug)]
pub struct Indent<'a> {
    /// Reference to the indenter which created this guard
    indenter: &'a mut Indenter,
}

impl Indenter {
    /// Creates a new Indenter with zero indentation.
    ///
    /// The default tab is two spaces.
    pub fn new() -> Self {
        Self {
            level: 0,
            tab: "  ".to_string(),
        }
    }

    /// Creates a new Indenter with zero indentation and a custom tab string.
    pub fn with_tab(tab: impl Into<String>) -> Self {
        Self {
            level: 0,
            tab: tab.into(),
        }
    }

    /// Increases the indentation level and returns a guard that will
    /// decrease it when dropped.
    pub fn indent(&mut self) -> Indent {
        self.level += 1;
        Indent { indenter: self }
    }

    /// Returns the current indentation level.
    pub fn level(&self) -> usize {
        self.level
    }

    /// Returns a new `Indenter` that writes everything with the chosen indentation level.
    fn writer(&self, writer: &mut impl Write) -> impl Write {
        indented(writer).ind(self.level())
    }
}

/// Formats text with the current indentation level.
///
/// Takes an indenter and format arguments, returns a formatted String.
#[macro_export]
macro_rules! format_indent {
    ($indenter:expr, $($arg:tt)*) => {{
        let mut result = String::new();
        let _ = write_indent!($indenter, &mut result, $($arg)*);
        result
    }};
}

/// Writes formatted text with the current indentation level to the specified writer.
///
/// Returns the result of the underlying `write!` operation.
#[macro_export]
macro_rules! write_indent {
    ($indenter:expr, $writer:expr, $($arg:tt)*) => {{
        write!($indenter.writer($writer), $($arg)*)
    }};
}

/// Writes formatted text with the current indentation level to the specified writer,
/// followed by a newline.
///
/// Returns the result of the underlying `writeln!` operation.
#[macro_export]
macro_rules! writeln_indent {
    ($indenter:expr, $writer:expr, $($arg:tt)*) => {{
        writeln!($indenter.writer($writer), $($arg)*)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_indent() {
        let mut indenter = Indenter::new();
        assert_eq!(format_indent!(indenter, "Hello"), "Hello");
        
        {
            let _guard = indenter.indent();
            assert_eq!(format_indent!(indenter, "World"), "  World");
            
            {
                let _guard = indenter.indent();
                assert_eq!(format_indent!(indenter, "Nested"), "    Nested");
            }
            
            // Verify indentation decreases when guard is dropped
            assert_eq!(format_indent!(indenter, "Still indented"), "  Still indented");
        }
        
        // Verify indentation returns to zero when all guards are dropped
        assert_eq!(format_indent!(indenter, "Back to zero"), "Back to zero");
    }
}

impl<'a> Drop for Indent<'a> {
    /// Decreases the indentation level when this guard is dropped.
    fn drop(&mut self) {
        debug_assert!(self.indenter.level > 0, "Indentation level cannot go below zero");
        self.indenter.level -= 1;
    }
}