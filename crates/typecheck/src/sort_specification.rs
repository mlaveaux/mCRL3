use std::collections::{HashMap, HashSet};
use mcrl3_utilities::ast::{SortExpression, Sort};
use mcrl3_utilities::NumberPostfixGenerator;

/// Represents a sort specification containing declarations of basic sorts and aliases.
#[derive(Debug, Clone)]
pub struct SortSpecification {
    /// Basic sorts declared by the user
    user_defined_sorts: Vec<String>,
    
    /// Sort aliases mapping names to expressions 
    user_defined_aliases: HashMap<String, SortExpression>,

    /// Sorts required by the context but not explicitly defined
    sorts_in_context: HashSet<SortExpression>,

    /// Cache for normalized sorts (cleared when specification changes)
    normalized_sorts: HashSet<SortExpression>,

    /// Maps sorts to their normalized form (cleared when specification changes)  
    normalized_aliases: HashMap<SortExpression, SortExpression>,

    /// Generator for fresh sort names
    fresh_name_generator: NumberPostfixGenerator,
}

impl SortSpecification {
    /// Creates a new empty sort specification
    pub fn new() -> Self {
        let mut spec = Self {
            user_defined_sorts: Vec::new(),
            user_defined_aliases: HashMap::new(), 
            sorts_in_context: HashSet::new(),
            normalized_sorts: HashSet::new(),
            normalized_aliases: HashMap::new(),
            fresh_name_generator: NumberPostfixGenerator::new("S"),
        };

        // Add predefined sorts like Bool
        spec.add_predefined_sorts();
        spec
    }

    /// Adds a basic sort declaration
    pub fn add_sort(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !self.user_defined_sorts.contains(&name) {
            self.user_defined_sorts.push(name);
            self.clear_normalized_cache();
        }
    }

    /// Adds a sort alias declaration
    pub fn add_alias(&mut self, name: impl Into<String>, expr: SortExpression) {
        let name = name.into();
        self.user_defined_aliases.insert(name, expr);
        self.clear_normalized_cache();
    }

    /// Adds a sort required by the context
    pub fn add_context_sort(&mut self, sort: SortExpression) {
        self.sorts_in_context.insert(sort);
        self.clear_normalized_cache();
    }

    /// Returns the set of all normalized sorts
    pub fn sorts(&self) -> &HashSet<SortExpression> {
        self.normalize_if_needed();
        &self.normalized_sorts
    }

    /// Returns all user-defined sort names
    pub fn user_defined_sorts(&self) -> &[String] {
        &self.user_defined_sorts
    }

    /// Returns all user-defined aliases
    pub fn user_defined_aliases(&self) -> &HashMap<String, SortExpression> {
        &self.user_defined_aliases
    }

    /// Returns the normalized form of a sort expression
    pub fn normalize(&self, sort: &SortExpression) -> SortExpression {
        self.normalize_if_needed();
        // Look up in cache or return original
        self.normalized_aliases.get(sort)
            .cloned()
            .unwrap_or_else(|| sort.clone())
    }

    /// Checks if a sort is certainly finite (has finite domain)
    pub fn is_certainly_finite(&self, sort: &SortExpression) -> bool {
        match sort {
            SortExpression::Simple(Sort::Bool) => true,
            SortExpression::Reference(name) => {
                // Check if alias exists and recursively check referenced sort
                if let Some(referenced) = self.user_defined_aliases.get(name) {
                    self.is_certainly_finite(referenced)
                } else {
                    false
                }
            }
            _ => false
        }
    }

    // Private helper methods
    
    fn add_predefined_sorts(&mut self) {
        // Add Bool as predefined sort
        self.add_context_sort(SortExpression::Simple(Sort::Bool));
    }

    fn clear_normalized_cache(&mut self) {
        self.normalized_sorts.clear();
        self.normalized_aliases.clear();
    }

    fn normalize_if_needed(&self) {
        if self.normalized_sorts.is_empty() {
            // Normalize all sorts...
            // Actual implementation would rebuild normalized forms
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sort_management() {
        let mut spec = SortSpecification::new();
        
        spec.add_sort("A");
        spec.add_sort("B");
        
        assert!(spec.user_defined_sorts().contains(&"A".to_string()));
        assert!(spec.user_defined_sorts().contains(&"B".to_string()));
    }

    #[test]
    fn test_sort_aliases() {
        let mut spec = SortSpecification::new();
        
        spec.add_sort("A");
        spec.add_alias("B", SortExpression::Reference("A".to_string()));
        
        assert_eq!(spec.user_defined_aliases().get("B").unwrap(),
                  &SortExpression::Reference("A".to_string()));
    }

    #[test]
    fn test_predefined_sorts() {
        let spec = SortSpecification::new();
        assert!(spec.sorts().contains(&SortExpression::Simple(Sort::Bool)));
    }

    #[test]
    fn test_certainly_finite() {
        let spec = SortSpecification::new();
        
        assert!(spec.is_certainly_finite(&SortExpression::Simple(Sort::Bool)));
        assert!(!spec.is_certainly_finite(&SortExpression::Simple(Sort::Real)));
    }
}
