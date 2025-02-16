use mcrl3_utilities::IndexedSet;
use mcrl3_utilities::ProtectionSet;

use crate::Symbol;
use crate::SymbolRef;

/// Pool for maximal sharing of function symbols.
/// Ensures that function symbols with the same name and arity are the same object.
pub struct SymbolPool {
    /// Unique table of all function symbols
    symbols: IndexedSet<SharedSymbol>,
    /// Protection set to prevent garbage collection of symbols
    protection_set: ProtectionSet<usize>,
}

impl Default for SymbolPool {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolPool {
    /// Creates a new empty symbol pool.
    pub fn new() -> Self {
        let mut pool = Self {
            symbols: IndexedSet::new(),
            protection_set: ProtectionSet::new(),
        };

        // Initialize built-in symbols
        // Create built-in symbols
        let symbols = [("Int", 1), ("List", 2), ("[]", 0)];

        for (name, arity) in symbols {
            pool.symbols.insert(SharedSymbol::new(name, arity));
        }

        pool
    }

    /// Creates or retrieves a function symbol with the given name and arity.
    pub fn create(&mut self, name: impl Into<String>, arity: usize) -> Symbol {
        let name = name.into();

        // Get or create symbol index
        let index = self.symbols.insert(SharedSymbol::new(name, arity));

        // Return cloned symbol
        Symbol::new_internal(index, self.protection_set.protect(index))
    }

    /// Protects a symbol from garbage collection.
    pub fn protect(&mut self, symbol: &SymbolRef<'_>) -> Symbol {
        Symbol::new_internal(symbol.index(), self.protection_set.protect(symbol.index()))
    }

    /// Unprotects a symbol, allowing it to be garbage collected.
    pub fn unprotect(&mut self, symbol: Symbol) {
        self.protection_set.unprotect(symbol.root());
    }

    /// Returns the number of symbols in the pool.
    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    /// Returns borrow of the shared symbol from a SymbolRef
    pub fn get<'a>(&self, symbol: &'a SymbolRef<'_>) -> &'a SharedSymbol {
        self.symbols.get(symbol.index()).unwrap()
    }

    /// Check if the symbol is the default "Int" symbol
    pub fn is_int(&self, symbol: &SymbolRef<'_>) -> bool {
        self.get(symbol).name() == "Int" && self.get(symbol).arity() == 1
    }

    /// Check if the symbol is the default "List" symbol
    pub fn is_list(&self, symbol: &SymbolRef<'_>) -> bool {
        self.get(symbol).name() == "List" && self.get(symbol).arity() == 2
    }

    /// Check if the symbol is the default "[]" symbol
    pub fn is_empty_list(&self, symbol: &SymbolRef<'_>) -> bool {
        self.get(symbol).name() == "[]" && self.get(symbol).arity() == 0
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
        let mut pool = SymbolPool::new();

        let f1 = pool.create("f", 2);
        let f2 = pool.create("f", 2);

        // Should be the same object
        assert_eq!(f1, f2);
    }
}
