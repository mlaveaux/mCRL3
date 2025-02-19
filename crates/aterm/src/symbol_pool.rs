use mcrl3_utilities::IndexedSet;
use mcrl3_utilities::ProtectionSet;

use crate::Symbol;
use crate::SymbolRef;

/// Pool for maximal sharing of function symbols.
/// Ensures that function symbols with the same name and arity are the same object.
pub struct SymbolPool {
    /// Unique table of all function symbols
    symbols: IndexedSet<SharedSymbol>,
}

impl SymbolPool {
    /// Creates a new empty symbol pool.
    pub(crate) fn new() -> Self {
        Self {
            symbols: IndexedSet::new(),
        }
    }

    /// Creates or retrieves a function symbol with the given name and arity.
    pub fn create<P>(&mut self, name: impl Into<String>, arity: usize, protect: P) -> Symbol 
    where
        P: FnOnce(usize) -> Symbol
    {
        let name = name.into();

        // Get or create symbol index
        let index = self.symbols.insert(SharedSymbol::new(name, arity));

        // Return cloned symbol
        protect(index)
    }
    
    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn symbol_name<'a>(&'a self, symbol: &'a SymbolRef<'_>) -> &'a str {
        self.symbols.get(symbol.index()).unwrap().name()
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn symbol_arity(&self, symbol: &SymbolRef<'_>) -> usize {
        self.symbols.get(symbol.index()).unwrap().arity()
    }

    /// Returns the number of symbols in the pool.
    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    /// Returns borrow of the shared symbol from a SymbolRef
    pub fn get<'a>(&self, symbol: &'a SymbolRef<'_>) -> &'a SharedSymbol {
        unsafe { 
            std::mem::transmute(self.symbols.get(symbol.index()).unwrap())
        }
    }
}

/// Represents a function symbol with a name and arity.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SharedSymbol {
    /// Name of the function
    name: String,
    /// Number of arguments
    arity: usize,
}

impl SharedSymbol {
    /// Creates a new function symbol.
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self {
            name: name.into(),
            arity,
        }
    }

    /// Returns the name of the function symbol
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the arity of the function symbol
    pub fn arity(&self) -> usize {
        self.arity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_sharing() {        
        let _ = mcrl3_utilities::test_logger();

        let f1 = Symbol::new("f", 2);
        let f2 = Symbol::new("f", 2);

        // Should be the same object
        assert_eq!(f1, f2);
    }
}
