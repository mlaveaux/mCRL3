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
    /// The default tab is four spaces.
    pub fn new() -> Self {
        Self {
            level: 0,
            tab: "    ".to_string(),
        }
    }

    /// Creates a new Indenter with zero indentation and a custom tab string.
    ///
    /// # Arguments
    /// * `tab` - The string to use for each level of indentation
    pub fn with_tab(tab: impl Into<String>) -> Self {
        Self {
            level: 0,
            tab: tab.into(),
        }
    }

    /// Increases the indentation level and returns a guard that will
    /// decrease it when dropped.
    ///
    /// # Returns
    /// An `Indent` guard that will decrease the indentation when dropped
    pub fn indent(&mut self) -> Indent {
        self.level += 1;
        Indent { indenter: self }
    }

    /// Returns the current indentation as a string.
    ///
    /// # Returns
    /// A string containing the appropriate number of tabs for the current level
    pub fn get_indent(&self) -> String {
        self.tab.repeat(self.level)
    }

    /// Formats text with the current indentation level.
    ///
    /// # Arguments
    /// * `text` - The text to format with indentation
    ///
    /// # Returns
    /// The indented text
    pub fn format(&self, text: impl AsRef<str>) -> String {
        format!("{}{}", self.get_indent(), text.as_ref())
    }

    /// Formats text with the current indentation level and appends a newline.
    ///
    /// # Arguments
    /// * `text` - The text to format with indentation
    ///
    /// # Returns
    /// The indented text with a trailing newline
    pub fn format_line(&self, text: impl AsRef<str>) -> String {
        format!("{}{}\n", self.get_indent(), text.as_ref())
    }

    /// Returns the current indentation level.
    ///
    /// # Returns
    /// The current indentation level
    pub fn level(&self) -> usize {
        self.level
    }
}

impl<'a> Drop for Indent<'a> {
    /// Decreases the indentation level when this guard is dropped.
    fn drop(&mut self) {
        debug_assert!(self.indenter.level > 0, "Indentation level cannot go below zero");
        self.indenter.level -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indenter_basic() {
        let mut indenter = Indenter::new();
        
        // Initial indentation level should be zero
        assert_eq!(indenter.level(), 0);
        assert_eq!(indenter.get_indent(), "");
        assert_eq!(indenter.format("Hello"), "Hello");
        
        // Adding indentation
        {
            let _indent = indenter.indent();
            assert_eq!(indenter.level(), 1);
            assert_eq!(indenter.get_indent(), "    ");
            assert_eq!(indenter.format("World"), "    World");
            
            // Nested indentation
            {
                let _indent2 = indenter.indent();
                assert_eq!(indenter.level(), 2);
                assert_eq!(indenter.get_indent(), "        ");
                assert_eq!(indenter.format("Nested"), "        Nested");
            }
            
            // After inner scope, indentation should return to previous level
            assert_eq!(indenter.level(), 1);
        }
        
        // After outer scope, indentation should return to initial level
        assert_eq!(indenter.level(), 0);
    }

    #[test]
    fn test_custom_tab() {
        let mut indenter = Indenter::with_tab("\t");
        
        assert_eq!(indenter.format("Test"), "Test");
        
        let _indent = indenter.indent();
        assert_eq!(indenter.format("Tabbed"), "\tTabbed");
    }

    #[test]
    fn test_format_line() {
        let mut indenter = Indenter::new();
        
        assert_eq!(indenter.format_line("Line 1"), "Line 1\n");
        
        let _indent = indenter.indent();
        assert_eq!(indenter.format_line("Line 2"), "    Line 2\n");
    }
}