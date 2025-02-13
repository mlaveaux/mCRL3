use mcrl3_aterm::{ATerm, TermPool};
use mcrl3_sort::SortExpression;
use std::fmt;

/// A function symbol with a name and sort.
#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    term: ATerm,
}

impl FunctionSymbol {
    /// Creates a new function symbol.
    pub fn new(tp: &mut TermPool, name: &str, sort: SortExpression) -> Self {
        Self {
            term: tp.create_function_symbol(name, sort),
        }
    }

    /// Returns the name of the function symbol.
    pub fn name(&self) -> &str {
        self.term.get_name()
    }

    /// Returns the sort of the function symbol.
    pub fn sort(&self) -> SortExpression {
        self.term.get_sort()
    }
}

impl fmt::Display for FunctionSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}