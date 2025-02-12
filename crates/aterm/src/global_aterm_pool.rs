use std::fmt::Debug;
use std::sync::LazyLock;

use parking_lot::Mutex;

use mcrl3_utilities::{ProtectionSet, IndexedSet};

use crate::{ATerm, ATermRef, SymbolPool, SymbolRef};

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct SharedTerm {
    symbol: SymbolRef<'static>,
    arguments: Vec<ATermRef<'static>>,
}

impl Clone for SharedTerm {
    fn clone(&self) -> Self {
        SharedTerm {
            symbol: SymbolRef::new(self.symbol.index()),
            arguments: self.arguments.iter().map(|x| ATermRef::new(x.index())).collect(),
        }
    }
}

impl SharedTerm {
    pub fn symbol(&self) -> SymbolRef<'_> {
        self.symbol.copy()
    }

    pub fn arguments(&self) -> &[ATermRef<'static>] {
        &self.arguments
    }
}

/// The single global (singleton) term pool.
pub(crate) struct GlobalTermPool {
    /// Unique table of all terms
    terms: IndexedSet<SharedTerm>,
    /// The protection set for global terms.
    protection_set: ProtectionSet<usize>,
    /// The symbol pool for managing function symbols.
    symbol_pool: SymbolPool,
}

impl GlobalTermPool {
    fn new() -> GlobalTermPool {
        GlobalTermPool {
            terms: IndexedSet::new(),
            protection_set: ProtectionSet::new(),
            symbol_pool: SymbolPool::new(),
        }
    }
    
    /// Protect the term by adding its index to the protection set
    pub fn protect(&mut self, term: ATermRef<'_>) -> ATerm {
        // Create a SharedTerm from the given ATermRef
        let shared_term = self.terms.get(term.index()).unwrap().clone();

        // Get or insert the term
        let (index, _) = self.terms.get_or_insert(shared_term);

        // Protect the term by adding its index to the protection set
        let root = self.protection_set.protect(index);

        // Return the protected term
        ATerm::new(index, root)
    }

    /// Unprotect the term by removing its index from the protection set
    pub fn unprotect(&mut self, term: ATerm) {
        self.protection_set.unprotect(term.root);
    }
    
    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn get_head_symbol(&self, term: &ATermRef<'_>) -> SymbolRef<'_> {
        self.terms.get(term.index()).unwrap().symbol()
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn get_argument(&self, term: &ATermRef<'_>, i: usize) -> ATermRef<'_> {
        self.terms.get(term.index()).unwrap().arguments().get(i).unwrap().copy()
    }

    /// Returns the number of terms in the pool.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Returns the number of terms in the pool.
    pub fn capacity(&self) -> usize {
        self.terms.len()
    }

    /// Expose the symbol pool
    pub fn symbol_pool(&self) -> &SymbolPool {
        &self.symbol_pool
    }

    /// Expose the symbol pool as mutable
    pub fn symbol_pool_mut(&mut self) -> &mut SymbolPool {
        &mut self.symbol_pool
    }
}

/// This is the global set of protection sets that are managed by the ThreadTermPool
pub(crate) static GLOBAL_TERM_POOL: LazyLock<Mutex<GlobalTermPool>> = LazyLock::new(|| Mutex::new(GlobalTermPool::new()));