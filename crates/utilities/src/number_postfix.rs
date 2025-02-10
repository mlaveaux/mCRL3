//! 
//! # Examples
//! 
//! Using the number postfix generator:
//! ```
//! use utilities::NumberPostfixGenerator;
//! 
//! // Create a generator with default hint "FRESH_VAR"
//! let mut gen = NumberPostfixGenerator::default();
//! 
//! // Generate unique identifiers
//! assert_eq!(gen.generate_default(), "FRESH_VAR");
//! assert_eq!(gen.generate_default(), "FRESH_VAR1");
//! 
//! // Create with existing context
//! let mut gen = NumberPostfixGenerator::with_context(
//!     ["x1", "x2", "y"], 
//!     "x"
//! );
//! 
//! // Generate from context
//! assert_eq!(gen.generate("x", true), "x3");  // Next after x2
//! assert_eq!(gen.generate("y", true), "y1");  // Next after y
//! ```
//! 
//! See individual module documentation for more examples.

use std::collections::HashMap;

/// Generates unique identifiers by appending numbers to a prefix.
/// For each prefix, maintains an index that is incremented after each generation.
#[derive(Debug, Clone)]
pub struct NumberPostfixGenerator {
    /// Maps prefixes to their highest used index
    index: HashMap<String, usize>,
    /// Default prefix for generated identifiers
    hint: String,
}

impl NumberPostfixGenerator {
    /// Creates a new generator with the specified default hint.
    pub fn new(hint: impl Into<String>) -> Self {
        Self {
            index: HashMap::new(),
            hint: hint.into(),
        }
    }

    /// Creates a new generator with context from an iterator of strings.
    pub fn with_context<I, S>(iter: I, hint: impl Into<String>) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut generator = Self::new(hint);
        generator.add_identifiers(iter);
        generator
    }

    /// Adds an identifier to the context.
    pub fn add_identifier(&mut self, id: impl AsRef<str>) {
        let id = id.as_ref();
        
        // Find last non-digit character
        let (name, new_index) = match id.chars().rposition(|c| !c.is_ascii_digit()) {
            Some(i) if i + 1 < id.len() => {
                let (name, num) = id.split_at(i + 1);
                let index = num.parse().unwrap_or(0);
                (name.to_string(), index)
            }
            _ => (id.to_string(), 0),
        };

        let entry = self.index.entry(name).or_insert(0);
        *entry = (*entry).max(new_index);
    }

    /// Adds multiple identifiers to the context.
    pub fn add_identifiers<I, S>(&mut self, iter: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for id in iter {
            self.add_identifier(id);
        }
    }

    /// Generates a fresh identifier using the given hint.
    /// If add_to_context is true, the new identifier is added to the context.
    pub fn generate(&mut self, hint: impl AsRef<str>, add_to_context: bool) -> String {
        let mut hint = hint.as_ref().to_string();
        
        // Remove trailing digits
        if let Some(last) = hint.chars().last() {
            if last.is_ascii_digit() {
                if let Some(i) = hint.chars().rposition(|c| !c.is_ascii_digit()) {
                    hint.truncate(i + 1);
                }
            }
        }

        match self.index.get_mut(&hint) {
            Some(index) => {
                let new_index = if add_to_context {
                    *index += 1;
                    *index
                } else {
                    *index + 1
                };
                format!("{}{}", hint, new_index)
            }
            None => {
                if add_to_context {
                    self.index.insert(hint.clone(), 0);
                }
                hint
            }
        }
    }

    /// Generates a fresh identifier using the default hint.
    pub fn generate_default(&mut self) -> String {
        self.generate(&self.hint, true)
    }

    /// Returns the default hint.
    pub fn hint(&self) -> &str {
        &self.hint
    }

    /// Sets the default hint.
    pub fn set_hint(&mut self, hint: impl Into<String>) {
        self.hint = hint.into();
    }

    /// Clears all context.
    pub fn clear(&mut self) {
        self.index.clear();
    }
}

impl Default for NumberPostfixGenerator {
    fn default() -> Self {
        Self::new("FRESH_VAR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_generation() {
        let mut gen = NumberPostfixGenerator::default();
        assert_eq!(gen.generate_default(), "FRESH_VAR");
        assert_eq!(gen.generate_default(), "FRESH_VAR1");
        assert_eq!(gen.generate_default(), "FRESH_VAR2");
    }

    #[test]
    fn test_with_context() {
        let mut gen = NumberPostfixGenerator::with_context(
            ["x1", "x2", "y", "z3"], 
            "x"
        );
        assert_eq!(gen.generate("x", true), "x3");
        assert_eq!(gen.generate("y", true), "y1");
        assert_eq!(gen.generate("z", true), "z4");
    }

    #[test]
    fn test_without_adding_to_context() {
        let mut gen = NumberPostfixGenerator::default();
        assert_eq!(gen.generate("test", false), "test");
        assert_eq!(gen.generate("test", false), "test");
        
        gen.add_identifier("test1");
        assert_eq!(gen.generate("test", false), "test2");
    }
}