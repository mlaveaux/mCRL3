use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use mcrl3_utilities::ProtectionSet;

use crate::{Symbol, SymbolRef};

/// This is the global set of protection sets that are managed by the ThreadTermPool
pub(crate) static GLOBAL_SYMBOL_POOL: LazyLock<parking_lot::Mutex<SymbolPool>> = 
    LazyLock::new(|| parking_lot::Mutex::new(SymbolPool::new()));

/// Pool for maximal sharing of function symbols.
/// Ensures that function symbols with the same name and arity are the same object.
#[derive(Debug)]
pub struct SymbolPool {
    /// Unique table of all function symbols
    symbols: Mutex<Vec<SharedSymbol>>,
    /// Map from (name, arity) to symbol index
    symbol_map: Mutex<HashMap<(String, usize), usize>>,
    /// Protection set to prevent garbage collection of symbols
    protection_set: Mutex<ProtectionSet<usize>>,
}

impl SymbolPool {
    /// Creates a new empty symbol pool.
    pub fn new() -> Self {
        let pool = Self {
            symbols: Mutex::new(Vec::new()),
            symbol_map: Mutex::new(HashMap::new()),
            protection_set: Mutex::new(ProtectionSet::new()),
        };

        // Initialize built-in symbols
        pool.create_builtin_symbols();
        pool
    }

    /// Creates or retrieves a function symbol with the given name and arity.
    pub fn create(&self, name: impl Into<String>, arity: usize) -> Symbol {
        let name = name.into();
        
        // Get or create symbol index
        let index = {
            let mut symbol_map = self.symbol_map.lock().unwrap();
            let mut symbols = self.symbols.lock().unwrap();
            
            *symbol_map.entry((name.clone(), arity))
                .or_insert_with(|| {
                    let symbol = SharedSymbol::new(name, arity);
                    let index = symbols.len();
                    symbols.push(symbol);
                    index
                })
        };

        // Return cloned symbol
        let mut protection_set = self.protection_set.lock().unwrap();
        Symbol::new(index, protection_set.protect(index))
    }

    /// Protects a symbol from garbage collection.
    pub fn protect(&self, symbol: &SymbolRef<'_>) -> Symbol {
        let index = self.get_symbol_index(symbol);
        let mut protection_set = self.protection_set.lock().unwrap();

        Symbol::new(index, protection_set.protect(index))
    }

    /// Unprotects a symbol, allowing it to be garbage collected.
    pub fn unprotect(&self, symbol: &SymbolRef<'_>) {
        let index = self.get_symbol_index(symbol);
        let mut protection_set = self.protection_set.lock().unwrap();
        protection_set.unprotect(index);
    }

    /// Returns the number of symbols in the pool.
    pub fn size(&self) -> usize {
        self.symbols.lock().unwrap().len()
    }

    /// Returns borrow of the shared symbol from a SymbolRef
    fn get_symbol_index(&self, symbol: &SymbolRef<'_>) -> usize {
        let (name, arity) = (symbol.name(), symbol.arity());
        let symbol_map = self.symbol_map.lock().unwrap();
        symbol_map[&(name.to_string(), arity)]
    }

    fn create_builtin_symbols(&self) {
        // Create built-in symbols
        let symbols = [
            ("Int", 1),
            ("List", 2),
            ("[]", 0),
        ];

        for (name, arity) in symbols {
            self.create(name, arity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_sharing() {
        let pool = SymbolPool::new();
        
        let f1 = pool.create("f", 2);
        let f2 = pool.create("f", 2);
        
        // Should be the same object
        assert_eq!(f1, f2);
        assert_eq!(pool.size(), 4); // 3 builtin + 1 created
    }

    #[test]
    fn test_symbol_protection() {
        let pool = SymbolPool::new();
        let symbol = pool.create("test", 1);

        // Protect and unprotect
        pool.protect(&symbol);
        pool.unprotect(&symbol);

        // Symbol should still be valid
        assert_eq!(symbol.name(), "test");
        assert_eq!(symbol.arity(), 1);
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
    /// 
    /// # Arguments
    /// * `name` - Name of the function
    /// * `arity` - Number of arguments
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