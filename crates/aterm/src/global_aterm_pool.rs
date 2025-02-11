use std::fmt::Debug;
use std::sync::LazyLock;
use std::collections::HashMap;

use parking_lot::Mutex;

use mcrl3_utilities::ProtectionSet;

use crate::ATerm;
use crate::ATermRef;
use crate::SymbolRef;

#[derive(Debug, Hash, Eq, PartialEq)]
struct SharedTerm {
    symbol: SymbolRef<'static>,
    arguments: Vec<ATermRef<'static>>,
}

/// The single global (singleton) term pool.
pub(crate) struct GlobalTermPool {
    /// Unique table of all terms
    terms: Vec<SharedTerm>,
    /// Map from term to index in unique table
    term_map: HashMap<SharedTerm, usize>,
    /// The protection set for global terms.
    protection_set: ProtectionSet<usize>,
}

impl GlobalTermPool {
    fn new() -> GlobalTermPool {
        GlobalTermPool {
            terms: Vec::new(),
            term_map: HashMap::new(),
            protection_set: ProtectionSet::new(),
        }
    }
    
    pub fn protect(&mut self, term: ATermRef<'_>) -> ATerm {
        // Create a SharedTerm from the given ATermRef
        let shared_term = SharedTerm {
            symbol: term.symbol(),
            arguments: term.arguments().to_vec(),
        };

        // Check if the term already exists in the term_map
        let index = if let Some(&index) = self.term_map.get(&shared_term) {
            index
        } else {
            // If the term does not exist, insert it into the terms vector and term_map
            let index = self.terms.len();
            self.terms.push(shared_term);
            self.term_map.insert(self.terms[index].clone(), index);
            index
        };

        // Protect the term by adding its index to the protection set
        let root = self.protection_set.protect(index);

        // Return the protected term
        ATerm::new(index, root)
    }

    /// Unprotect the term by removing its index from the protection set
    pub fn unprotect(&mut self, term: ATerm) {
        self.protection_set.unprotect(term.root);
    }

    pub fn get(&self, term: ATermRef<'_>) -> ATerm {
        // Get the index of the term from the term_map
        let index = self.term_map.get(&SharedTerm {
            symbol: term.symbol(),
            arguments: term.arguments().to_vec(),
        }).expect("Term not found in global term pool");

        // Return the term
        ATerm::new(ATermRef::new(*index), self.protection_set.protect(*index))
    }


    /// Returns the number of terms in the pool.
    pub fn len(&self) -> usize {
        self.term_map.len()
    }

    /// Returns the number of terms in the pool.
    pub fn capacity(&self) -> usize {
        self.term_map.capacity()
    }
}


/// This is the global set of protection sets that are managed by the ThreadTermPool
pub(crate) static GLOBAL_TERM_POOL: LazyLock<Mutex<GlobalTermPool>> = LazyLock::new(|| Mutex::new(GlobalTermPool::new()));