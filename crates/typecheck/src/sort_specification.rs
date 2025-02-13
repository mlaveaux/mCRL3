use std::collections::HashSet;
use std::fmt;

use mcrl3_syntax::SortExpression;

/// A specification of sorts, containing user-defined sorts with their definitions.
#[derive(Debug, Clone)]
pub struct SortSpecification {
    /// Mapping of sort names to their definitions
    sorts: HashSet<SortDeclaration>,
    /// Context sorts that are used but not defined
    context_sorts: HashSet<SortExpression>,
}

/// Declaration of a user-defined sort.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SortDeclaration {
    /// Name of the sort
    name: String,
    /// Parameters for parameterized sorts
    parameters: Vec<String>,
    /// Optional sort expression defining the sort
    definition: Option<SortExpression>,
}

impl SortSpecification {
    /// Creates a new empty sort specification.
    pub fn new() -> Self {
        Self {
            sorts: HashSet::new(),
            context_sorts: HashSet::new(),
        }
    }

    /// Creates a new sort specification with initial sorts and context sorts.
    pub fn with_sorts_and_contexts(sorts: HashSet<SortDeclaration>, context_sorts: HashSet<SortExpression>) -> Self {
        Self { sorts, context_sorts }
    }

    /// Returns all declared sorts.
    pub fn sorts(&self) -> &HashSet<SortDeclaration> {
        &self.sorts
    }

    /// Returns all context sorts.
    pub fn context_sorts(&self) -> &HashSet<SortExpression> {
        &self.context_sorts
    }

    /// Finds all sort expressions used in this specification.
    pub fn find_sort_expressions(&self) -> HashSet<SortExpression> {
        let mut result = HashSet::new();

        // Add context sorts
        result.extend(self.context_sorts.clone());

        // Add sorts from declarations
        for decl in &self.sorts {
            if let Some(def) = &decl.definition {
                result.insert(def.clone());
            }
        }

        result
    }
}

impl SortDeclaration {
    /// Creates a new sort declaration.
    pub fn new(name: impl Into<String>, parameters: Vec<String>, definition: Option<SortExpression>) -> Self {
        Self {
            name: name.into(),
            parameters,
            definition,
        }
    }

    /// Returns the name of the sort.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parameters of the sort.
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    /// Returns the definition of the sort, if any.
    pub fn definition(&self) -> Option<&SortExpression> {
        self.definition.as_ref()
    }
}

impl Default for SortSpecification {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SortDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.parameters.is_empty() {
            write!(f, "(")?;
            for (i, param) in self.parameters.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            write!(f, ")")?;
        }
        if let Some(def) = &self.definition {
            write!(f, " = {}", def)?;
        }
        Ok(())
    }
}
