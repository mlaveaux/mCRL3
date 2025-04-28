use std::hash::Hash;
use std::hash::Hasher;
use std::num::NonZero;
use std::sync::Arc;

use hashbrown::Equivalent;

use mcrl3_unsafety::StablePointerSet;
use mcrl3_utilities::IndexedSet;

use crate::Symb;
use crate::SymbolIndex;
use crate::SymbolRef;

/// Pool for maximal sharing of function symbols. Ensures that function symbols
/// with the same name and arity point to the same `SharedSymbol` object.
/// Returns `Symbol` that can be used to refer to the shared symbol, avoiding
/// garbage collection of the underlying shared symbol.
pub struct SymbolPool {
    /// Unique table of all function symbols
    symbols: StablePointerSet<SharedSymbol>,
}

impl SymbolPool {
    /// Creates a new empty symbol pool.
    pub(crate) fn new() -> Self {
        let mut symbols = StablePointerSet::new();
        symbols.insert(SharedSymbol::new("<default>", 0));

        Self { symbols }
    }

    /// Creates or retrieves a function symbol with the given name and arity.
    pub fn create<N, P, R>(&mut self, name: N, arity: usize, protect: P) -> R
    where
        N: Into<String> + AsRef<str>,
        P: FnOnce(SymbolIndex) -> R,
    {
        // Get or create symbol index
        let (shared_symbol, _inserted) = self.symbols.insert_equiv(&SharedSymbolLookup { name, arity });

        // Return cloned symbol
        protect(shared_symbol)
    }

    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn symbol_name<'a>(&self, symbol: &'a SymbolRef<'a>) -> &'a str {
        symbol.shared().name()
    }

    /// Returns the arity of the function symbol
    pub fn symbol_arity<'a, 'b>(&self, symbol: &'b impl Symb<'a, 'b>) -> usize {
        symbol.shared().arity()
    }

    /// Returns the number of symbols in the pool.
    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    /// Returns the capacity of the pool.
    pub fn capacity(&self) -> usize {
        self.symbols.capacity()
    }

    /// Retain only symbols satisfying the given predicate.
    pub unsafe fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&SymbolIndex) -> bool,
    {
        unsafe {
            self.symbols.retain(|element| f(element));
        }
    }
}

/// Represents a function symbol with a name and arity.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SharedSymbol {
    /// Name of the function
    name: Arc<String>,
    /// Number of arguments
    arity: usize,
}

impl SharedSymbol {
    /// Creates a new function symbol.
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self {
            name: Arc::new(name.into()),
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

/// A cheap way to look up SharedSymbol
struct SharedSymbolLookup<T: Into<String> + AsRef<str>> {
    name: T,
    arity: usize,
}

impl<T: Into<String> + AsRef<str>> From<&SharedSymbolLookup<T>> for SharedSymbol {
    fn from(lookup: &SharedSymbolLookup<T>) -> Self {
        // TODO: Not optimal
        let string = lookup.name.as_ref().to_string();
        Self::new(string, lookup.arity)
    }
}

impl<T: Into<String> + AsRef<str>> Equivalent<SharedSymbol> for SharedSymbolLookup<T> {
    fn equivalent(&self, other: &SharedSymbol) -> bool {
        self.name.as_ref() == *other.name && self.arity == other.arity
    }
}

/// These hash implementations should be the same as `SharedSymbol`.
impl<T: Into<String> + AsRef<str>> Hash for SharedSymbolLookup<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.as_ref().hash(state);
        self.arity.hash(state);
    }
}

impl Hash for SharedSymbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.arity.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::Symbol;

    #[test]
    fn test_symbol_sharing() {
        let _ = mcrl3_utilities::test_logger();

        let f1 = Symbol::new("f", 2);
        let f2 = Symbol::new("f", 2);

        // Should be the same object
        assert_eq!(f1, f2);
    }
}
