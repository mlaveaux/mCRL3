use log::trace;
use mcrl3_sharedmutex::BfSharedMutex;
use mcrl3_sharedmutex::BfSharedMutexReadGuard;
use mcrl3_sharedmutex::BfSharedMutexWriteGuard;
use parking_lot::Mutex;
use pest_consume::Parser;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::mem::ManuallyDrop;
use std::sync::Arc;

use crate::aterm::ATerm;
use crate::aterm::ATermRef;
use crate::global_aterm_pool::GLOBAL_TERM_POOL;
use crate::Markable;
use crate::Rule;
use crate::SharedTermProtection;
use crate::Symbol;
use crate::SymbolRef;
use crate::TermParser;

thread_local! {
    /// Thread-specific term pool that manages protection sets.
    pub static THREAD_TERM_POOL: RefCell<ThreadTermPool> = RefCell::new(ThreadTermPool::new());
}

/// Per-thread term pool managing local protection sets.
pub struct ThreadTermPool {
    ///
    protection_set: Arc<Mutex<SharedTermProtection>>,

    /// These default symbols should not call their drop, will be removed when the protection set goes out of scope
    int_symbol: ManuallyDrop<Symbol>,
    empty_list_symbol: ManuallyDrop<Symbol>,
    list_symbol: ManuallyDrop<Symbol>,
    /// Thread local lock
    lock: BfSharedMutex<()>,
}

impl ThreadTermPool {
    /// Creates a new thread-local term pool.
    fn new() -> Self {
        // Register protection sets with global pool
        let mut tp = GLOBAL_TERM_POOL.lock();
        let protection_set =
            tp.register_thread_term_pool();

        Self {
            lock: BfSharedMutex::new(()),
            int_symbol: ManuallyDrop::new(tp.create_symbol("Int", 0, |index| {
                Symbol::new_internal(index,protection_set.lock().symbol_protection_set.protect(index))
            })),
            list_symbol: ManuallyDrop::new(tp.create_symbol("List", 2, |index| {
                Symbol::new_internal(index,protection_set.lock().symbol_protection_set.protect(index))
            })),
            empty_list_symbol: ManuallyDrop::new(tp.create_symbol("[]", 0, |index| {
                Symbol::new_internal(index, protection_set.lock().symbol_protection_set.protect(index))
            })),
            protection_set,
        }
    }

    /// Creates a term without arguments.
    pub fn create_constant<'a>(&self, symbol: &SymbolRef<'_>) -> ATerm {
        let t: [ATermRef<'a>; 0] = [];
        GLOBAL_TERM_POOL.lock().create_term(symbol, &t, |index| {
            self.protect(&ATermRef::new(index))
        })
    }

    /// Create a term with the given arguments
    pub fn create<'a>(&self, symbol: &SymbolRef<'_>, arguments: &[impl Borrow<ATermRef<'a>>]) -> ATerm {
        GLOBAL_TERM_POOL.lock().create_term(symbol, arguments, |index| {
            self.protect(&ATermRef::new(index))
        })
    }

    /// Create a term with the given index.
    pub fn create_int(&self, value: usize) -> ATerm {
        GLOBAL_TERM_POOL.lock().create_int(value, |index| {
            self.protect(&ATermRef::new(index))
        })
    }
    
    /// Create a term with the given arguments given by the iterator.
    pub fn create_term_iter<'a, I, T>(&self, symbol: &SymbolRef<'_>, iter: I) -> ATerm
    where
        I: IntoIterator<Item = T>,
        T: Borrow<ATermRef<'a>>,
    {
        GLOBAL_TERM_POOL.lock().create_term_iter(symbol, iter, |index| {
            self.protect(&ATermRef::new(index))
        })
    }
    
    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn get_head_symbol<'a, 'b>(&'a self, term: &'a ATermRef<'b>) -> &'a SymbolRef<'b> 
        where 'a: 'b
    {
        unsafe {
            std::mem::transmute(GLOBAL_TERM_POOL.lock().get_head_symbol(term))
        }
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn get_argument<'a>(&'a self, term: &'a ATermRef<'a>, i: usize) -> &'a ATermRef<'a> {        
        unsafe {
            std::mem::transmute(GLOBAL_TERM_POOL.lock().get_argument(term, i))
        }
    }
    
    /// Return the symbol of the SharedTerm for the given ATermRef
    pub fn symbol_name<'a>(&self, symbol: &'a SymbolRef<'_>) -> &'a str {
        unsafe {
            std::mem::transmute(GLOBAL_TERM_POOL.lock().symbol_name(symbol))
        }
    }

    /// Return the i-th argument of the SharedTerm for the given ATermRef
    pub fn symbol_arity(&self, symbol: &SymbolRef<'_>) -> usize {
        GLOBAL_TERM_POOL.lock().symbol_arity(symbol)
    }

    /// Create a function symbol
    pub fn create_symbol(&self, name: impl Into<String>, arity: usize) -> Symbol {
        GLOBAL_TERM_POOL.lock().create_symbol(name, arity, |index| {
            let root = self.protection_set.lock().symbol_protection_set.protect(index);
    
            // Return the protected term
            Symbol::new_internal(index, root)            
        })
    }

    /// Protect the term by adding its index to the protection set
    pub fn protect(&self, term: &ATermRef<'_>) -> ATerm {
        // Protect the term by adding its index to the protection set
        let root = self.protection_set.lock().protection_set.protect(term.index());

        // Return the protected term
        ATerm::new_internal(term.index(), root)
    }

    /// Unprotects a term from this thread's protection set.
    pub fn drop(&self, term: &ATerm) {        
        trace!(
            "Unprotected term {:?}, index {}, protection set {}",
            term,
            term.root(),
            self.index()
        );

        self.protection_set.lock().protection_set.unprotect(term.root());
    }

    /// Protects a container in this thread's container protection set.
    pub(crate) fn protect_container(&self, container: Arc<dyn Markable + Send + Sync>) -> usize {
        let root = self.protection_set.lock().container_protection_set.protect(container);
        trace!("Protected container index {}, protection set {}", root, self.index());
        root
    }

    /// Unprotects a container from this thread's container protection set.
    pub(crate) fn drop_container(&self, root: usize) {
        trace!("Unprotected container index {}, protection set {}", root, self.index());
        self.protection_set.lock().container_protection_set.unprotect(root);
    }

    /// Parse the given string and returns the Term representation.
    pub fn from_string(&self, term: &str) -> Result<ATerm, Box<dyn Error>> {
        let mut result = TermParser::parse(Rule::TermSpec, term)?;
        let root = result.next().unwrap();

        Ok(TermParser::TermSpec(root).unwrap())
    }

    /// Locks the thread local lock in shared mode.
    pub fn lock_shared(&self) -> BfSharedMutexReadGuard<()> {
        self.lock.read().unwrap()
    }

    /// Locks the thread local lock in exclusive mode.
    pub fn lock_exclusive(&self) -> BfSharedMutexWriteGuard<()> {
        self.lock.write().unwrap()
    }
    
    /// Protects a symbol from garbage collection.
    pub fn protect_symbol(&self, symbol: &SymbolRef<'_>) -> Symbol {
        Symbol::new_internal(symbol.index(), self.protection_set.lock().symbol_protection_set.protect(symbol.index()))
    }

    /// Unprotects a symbol, allowing it to be garbage collected.
    pub fn drop_symbol(&self, symbol: &mut Symbol) {
        self.protection_set.lock().symbol_protection_set.unprotect(symbol.root());
    }

    /// Check if the symbol is the default "Int" symbol
    pub fn is_int(&self, symbol: &SymbolRef<'_>) -> bool {
        **self.int_symbol == *symbol
    }

    /// Check if the symbol is the default "List" symbol
    pub fn is_list(&self, symbol: &SymbolRef<'_>) -> bool {
        **self.list_symbol == *symbol
    }

    /// Check if the symbol is the default "[]" symbol
    pub fn is_empty_list(&self, symbol: &SymbolRef<'_>) -> bool {
        **self.empty_list_symbol == *symbol
    }

    /// Returns the index of the protection set.
    fn index(&self) -> usize {
        self.protection_set.lock().index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_thread_local_protection() {
        let _ = mcrl3_utilities::test_logger();

        thread::scope(|scope| {
            for _ in 0..3 {
                scope.spawn(|| {
                    // Create and protect some terms
                    let symbol = Symbol::new("test", 0);
                    let term = ATerm::constant(&symbol);
                    let protected = term.protect();

                    // Verify protection
                    THREAD_TERM_POOL.with_borrow(|tp| {
                        assert!(tp.protection_set.lock().protection_set.contains_root(protected.root()));
                    });

                    // Unprotect
                    let root = protected.root();
                    drop(protected);

                    THREAD_TERM_POOL.with_borrow(|tp| {
                        assert!(!tp.protection_set.lock().protection_set.contains_root(root));
                    });
                });
            }
        });
    }

    #[test]
    fn test_parsing() {
        let _ = mcrl3_utilities::test_logger();

        let t = ATerm::from_string("f(g(a),b)").unwrap();
    }
}
