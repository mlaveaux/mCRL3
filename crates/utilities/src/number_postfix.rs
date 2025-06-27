use std::collections::HashMap;

/// Generates unique identifiers by appending numbers to a prefix.
/// For each prefix, maintains an index that is incremented after each generation.
#[derive(Debug, Clone)]
pub struct NumberPostfixGenerator {
    /// Maps prefixes to their highest used index
    index: HashMap<String, usize>,
    /// Default prefix for generated identifiers
    default: String,
}

impl NumberPostfixGenerator {
    /// Creates a new generator with the specified default hint.
    pub fn new(default: impl Into<String>) -> Self {
        Self {
            index: HashMap::new(),
            default: default.into(),
        }
    }

    /// Creates a new generator with context from an iterator of strings.
    pub fn with_context<I, S>(iter: I, default: impl Into<String>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut generator = Self::new(default);
        generator.add_identifiers(iter);
        generator
    }

    /// Adds an identifier to the context.
    pub fn add_identifier(&mut self, id: impl AsRef<str>) {
        let id = id.as_ref();

        // Find last non-digit character
        let (name, new_index) = match id.rfind(|c: char| !c.is_ascii_digit()) {
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

    /// Generates a fresh identifier using the given hint without numbers as prefix.
    /// If add_to_context is true, the new identifier is added to the context.
    pub fn generate(&mut self, hint: impl AsRef<str>, add_to_context: bool) -> String {
        let mut hint = hint.as_ref().to_string();

        // Remove trailing digits
        if let Some(last) = hint.chars().last() {
            if last.is_ascii_digit() {
                if let Some(i) = hint.rfind(|c: char| !c.is_ascii_digit()) {
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
                format!("{hint}{new_index}")
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
        let hint = self.default.clone();
        self.generate(hint, true)
    }

    /// Returns the default hint.
    pub fn hint(&self) -> &str {
        &self.default
    }

    /// Sets the default hint.
    pub fn set_hint(&mut self, hint: impl Into<String>) {
        self.default = hint.into();
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
        let mut generator = NumberPostfixGenerator::default();
        assert_eq!(generator.generate_default(), "FRESH_VAR");
        assert_eq!(generator.generate_default(), "FRESH_VAR1");
        assert_eq!(generator.generate_default(), "FRESH_VAR2");
    }

    #[test]
    fn test_with_context() {
        let mut generator = NumberPostfixGenerator::with_context(["x1", "x2", "y", "z3"], "x");
        assert_eq!(generator.generate("x", true), "x3");
        assert_eq!(generator.generate("y", true), "y1");
        assert_eq!(generator.generate("z", true), "z4");
    }

    #[test]
    fn test_without_adding_to_context() {
        let mut generator = NumberPostfixGenerator::default();
        assert_eq!(generator.generate("test", false), "test");
        assert_eq!(generator.generate("test", false), "test");

        generator.add_identifier("test1");
        assert_eq!(generator.generate("test", false), "test2");
    }

    #[test]
    fn test_with_suffix() {
        let mut generator = NumberPostfixGenerator::default();
        assert_eq!(generator.generate("test1", true), "test");
        assert_eq!(generator.generate("test1", true), "test1");
        assert_eq!(generator.generate("test1", true), "test2");
    }

    #[test]
    fn test_boost_case() {
        let mut generator = NumberPostfixGenerator::default();
        generator.add_identifier("c6");
        let mut s;

        s = generator.generate("c", true);
        assert_eq!(s, "c7");

        let v = vec!["a1", "c012"];
        generator.add_identifiers(v);

        s = generator.generate("c", true);
        assert_eq!(s, "c13");

        s = generator.generate("a", true);
        assert_eq!(s, "a2");

        s = generator.generate("a", true);
        assert_eq!(s, "a3");

        s = generator.generate("a2", true);
        assert_eq!(s, "a4");

        s = generator.generate_default();
        assert_eq!(s, "FRESH_VAR");

        s = generator.generate_default();
        assert_eq!(s, "FRESH_VAR1");

        s = generator.generate("b", false);
        assert_eq!(s, "b");

        s = generator.generate("b", false);
        assert_eq!(s, "b");

        s = generator.generate("b", true);
        assert_eq!(s, "b");

        s = generator.generate("b", false);
        assert_eq!(s, "b1");

        generator.add_identifier("a0");
        s = generator.generate("a2", true);
        assert_eq!(s, "a5");
    }
}
