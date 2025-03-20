use mcrl3_utilities::IndexedSet;

use crate::Symb;
use crate::Symbol;
use crate::SymbolRef;

/// Pool for maximal sharing of function symbols. Ensures that function symbols
/// with the same name and arity point to the same `SharedSymbol` object.
/// Returns `Symbol` that can be used to refer to the shared symbol, avoiding
/// garbage collection of the underlying shared symbol.
pub struct SymbolPool {
    /// Unique table of all function symbols
    symbols: IndexedSet<SharedSymbol>,
}

impl SymbolPool {
    /// Creates a new empty symbol pool.
    pub(crate) fn new() -> Self {
        let mut symbols = IndexedSet::new();
        symbols.insert(SharedSymbol::new("<default>>", 0));

        Self { symbols }
    }

    /// Creates or retrieves a function symbol with the given name and arity.
    pub fn create<P>(&mut self, name: impl Into<String>, arity: usize, protect: P) -> Symbol
    where
        P: FnOnce(usize) -> Symbol,
    {
        let name = name.into();

        // Get or create symbol index
        let index = self.symbols.insert(SharedSymbol::new(name, arity));

        // Return cloned symbol
        protect(index)
    }

    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn symbol_name<'a, 'b: 'a>(&'b self, symbol: &SymbolRef<'a>) -> &'a str {
        self.symbols.get(symbol.index()).unwrap().name()
    }

    /// Returns the arity of the function symbol
    pub fn symbol_arity<'a>(&self, symbol: &impl Symb<'a>) -> usize {
        self.symbols.get(symbol.index()).unwrap().arity()
    }

    /// Returns the number of symbols in the pool.
    pub fn size(&self) -> usize {
        self.symbols.len()
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
