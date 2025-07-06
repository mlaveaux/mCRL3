use std::hash::Hash;
use std::hash::Hasher;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use dashmap::DashMap;
use equivalent::Equivalent;
use rustc_hash::FxBuildHasher;

use mcrl3_unsafety::StablePointerSet;

use crate::Symb;
use crate::SymbolIndex;
use crate::SymbolRef;

/// Pool for maximal sharing of function symbols, see [SymbolRef]. Ensures that function symbols
/// with the same name and arity point to the same [SharedSymbol] object.
/// Returns [Symbol] that can be used to refer to the shared symbol, avoiding
/// garbage collection of the underlying shared symbol.
pub struct SymbolPool {
    /// Unique table of all function symbols
    symbols: StablePointerSet<SharedSymbol, FxBuildHasher>,

    /// A map from prefixes to counters that track the next available index for function symbols
    prefix_to_register_function_map: DashMap<String, Arc<AtomicUsize>, FxBuildHasher>,
}

impl SymbolPool {
    /// Creates a new empty symbol pool.
    pub(crate) fn new() -> Self {
        Self {
            symbols: StablePointerSet::with_hasher(FxBuildHasher),
            prefix_to_register_function_map: DashMap::with_hasher(FxBuildHasher::default()),
        }
    }

    /// Creates or retrieves a function symbol with the given name and arity.
    pub fn create<N, P, R>(&self, name: N, arity: usize, protect: P) -> R
    where
        N: Into<String> + AsRef<str>,
        P: FnOnce(SymbolIndex) -> R,
    {
        // Get or create symbol index
        let (shared_symbol, inserted) = self.symbols.insert_equiv(&SharedSymbolLookup { name, arity });

        if inserted {
            // If the symbol was newly created, register its prefix.
            self.update_prefix(shared_symbol.name());
        }

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
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Returns true if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Returns the capacity of the pool.
    pub fn capacity(&self) -> usize {
        self.symbols.capacity()
    }

    /// Retain only symbols satisfying the given predicate.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&SymbolIndex) -> bool,
    {
        self.symbols.retain(|element| f(element));
    }

    /// Creates a new prefix counter for the given prefix.
    pub fn create_prefix(&self, prefix: &str) -> Arc<AtomicUsize> {
        // Create a new counter for the prefix if it does not exist
        self.prefix_to_register_function_map
            .entry(prefix.to_string())
            .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
            .clone()
    }

    /// Removes a prefix counter from the pool.
    pub fn remove_prefix(&self, prefix: &str) {
        // Remove the prefix counter if it exists
        self.prefix_to_register_function_map.remove(prefix);
    }

    /// Updates the counter for a registered prefix for the newly created symbol.
    fn update_prefix(&self, name: &str) {
        // Check whether there is a registered prefix p such that name equal pn where n is a number.
        // In that case prevent that pn will be generated as a fresh function name.
        let start_of_index = name.rfind(|c: char| !c.is_ascii_digit())
            .map(|pos| pos + 1)
            .unwrap_or(0);

        if start_of_index < name.len() {
            let potential_number = &name[start_of_index..];
            let prefix = &name[..start_of_index];
            
            if let Some(counter) = self.prefix_to_register_function_map.get(prefix) {
                if let Ok(number) = potential_number.parse::<usize>() {
                    counter.fetch_max(number + 1, Ordering::Relaxed);
                }
            }
        }
    }
}

/// Represents a function symbol with a name and arity.
#[derive(Debug, Clone, Eq, PartialEq)]
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

    /// Returns a unique index for this shared symbol
    pub fn index(&self) -> usize {
        self as *const Self as *const u8 as usize
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
        self.name.as_ref() == other.name && self.arity == other.arity
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
