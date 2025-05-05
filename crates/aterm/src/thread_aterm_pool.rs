use log::trace;
use mcrl3_utilities::MCRL3Error;
use mcrl3_utilities::ProtectionIndex;
use pest_consume::Parser;
use std::cell::Cell;
use std::cell::RefCell;
use std::sync::Arc;

use crate::AGRESSIVE_GC;
use crate::GlobalTermPool;
use crate::Markable;
use crate::Rule;
use crate::SharedTermProtection;
use crate::Symb;
use crate::Symbol;
use crate::SymbolRef;
use crate::Term;
use crate::TermParser;
use crate::aterm::ATerm;
use crate::aterm::ATermRef;
use crate::global_aterm_pool::GLOBAL_TERM_POOL;
use crate::global_aterm_pool::Mutex;
use crate::global_aterm_pool::mutex_unwrap;

thread_local! {
    /// Thread-specific term pool that manages protection sets.
    pub static THREAD_TERM_POOL: RefCell<ThreadTermPool> = RefCell::new(ThreadTermPool::new());
}

/// Per-thread term pool managing local protection sets.
pub struct ThreadTermPool {
    /// A reference to the protection set of this thread pool.
    protection_set: Arc<Mutex<SharedTermProtection>>,

    /// The number of times termms have been created before garbage collection is triggered.
    garbage_collection_counter: Cell<usize>,

    /// Copy of the default terms since thread local access is cheaper.
    int_symbol: Symbol,
    empty_list_symbol: Symbol,
    list_symbol: Symbol,
}

impl ThreadTermPool {
    /// Creates a new thread-local term pool.
    fn new() -> Self {
        // Register protection sets with global pool
        let tp = GLOBAL_TERM_POOL.lock();
        let pool = (*tp).borrow_mut();
        let protection_set = pool.register_thread_term_pool();

        let protection = protection_set.lock();
        
        let int_symbol = pool.create_symbol("Int", 0, |index| protection.symbol_protection_set.protect(&unsafe { SymbolRef::from_index(&index) }));
        let list_symbol = pool.create_symbol("List", 2, |index| unsafe { SymbolRef::from_index(&index) });
        let empty_list_symbol = pool.create_symbol("[]", 0, |index| unsafe { SymbolRef::from_index(&index) });

        // Arbitrary value to trigger garbage collection
        Self {
            protection_set,
            garbage_collection_counter: Cell::new(if AGRESSIVE_GC { 1 } else { 1000 }),
            int_symbol,
            empty_list_symbol,
            list_symbol,
        }
    }

    /// Creates a term without arguments.
    pub fn create_constant(&self, symbol: &SymbolRef<'_>) -> ATerm {
        assert!(symbol.arity() == 0, "A constant should not have arity > 0");

        let tp = GLOBAL_TERM_POOL.lock();
        let empty_args: [ATermRef<'_>; 0] = [];
        tp.borrow_mut().create_term(symbol, &empty_args, |tp, index, inserted| {
            self.protect_inserted(tp, &unsafe { ATermRef::from_index(index) }, inserted)
        })
    }

    /// Create a term with the given arguments
    pub fn create_term<'a, 'b>(&self, symbol: &'b impl Symb<'a, 'b>, arguments: &'b [impl Term<'a, 'b>]) -> ATerm {
        let tp = GLOBAL_TERM_POOL.lock();
        (*tp)
            .borrow_mut()
            .create_term(symbol, arguments, |tp, index, inserted| {
                self.protect_inserted(tp, &unsafe { ATermRef::from_index(index) }, inserted)
            })
    }

    /// Create a term with the given index.
    pub fn create_int(&self, value: usize) -> ATerm {
        let tp = GLOBAL_TERM_POOL.lock();
        (*tp).borrow_mut().create_int(value, |tp, index, inserted| {
            self.protect_inserted(tp, &unsafe { ATermRef::from_index(index) }, inserted)
        })
    }

    /// Create a term with the given arguments given by the iterator.
    pub fn create_term_iter<'a, 'b, 'c, 'd, I, T>(&self, symbol: &'b impl Symb<'a, 'b>, iter: I) -> ATerm
    where
        I: IntoIterator<Item = T>,
        T: Term<'c, 'd>,
    {
        let tp = GLOBAL_TERM_POOL.lock();
        (*tp)
            .borrow_mut()
            .create_term_iter(symbol, iter, |tp, index, inserted| {
                self.protect_inserted(tp, &unsafe { ATermRef::from_index(index) }, inserted)
            })
    }

    /// Create a function symbol
    pub fn create_symbol(&self, name: impl Into<String> + AsRef<str>, arity: usize) -> Symbol {
        let tp = GLOBAL_TERM_POOL.lock();

        tp.borrow_mut().create_symbol(name, arity, |index| unsafe {
            self.protect_symbol(&SymbolRef::from_index(&index))
        })
    }

    /// Protect the term by adding its index to the protection set
    pub fn protect(&self, term: &ATermRef<'_>) -> ATerm {
        // Protect the term by adding its index to the protection set
        let root = mutex_unwrap(self.protection_set.lock())
            .protection_set
            .protect(term.shared().copy());

        // Return the protected term
        ATerm::from_index(term.shared(), root)
    }

    /// Protects a term from garbage collection after it was potentially inserted
    fn protect_inserted(&self, tp: &mut GlobalTermPool, term: &ATermRef<'_>, inserted: bool) -> ATerm {
        // Protect the term by adding its index to the protection set
        let root = mutex_unwrap(self.protection_set.lock())
            .protection_set
            .protect(term.shared().copy());

        // If the term was newly inserted, decrease the garbage collection counter and trigger garbage collection if necessary
        if inserted {
            let mut value = self.garbage_collection_counter.get();
            value -= 1;

            if value == 0 {
                // Trigger garbage collection and acquire a new counter value.
                value = tp.trigger_garbage_collection();
            }

            self.garbage_collection_counter.set(value);
        }

        // Return the protected terms
        ATerm::from_index(term.shared(), root)
    }

    /// Unprotects a term from this thread's protection set.
    pub fn drop(&self, term: &ATerm) {
        mutex_unwrap(self.protection_set.lock())
            .protection_set
            .unprotect(term.root());

        trace!(
            "Unprotected term {:?}, root {}, protection set {}",
            term,
            term.root(),
            self.index()
        );
    }

    /// Protects a container in this thread's container protection set.
    pub(crate) fn protect_container(&self, container: Arc<dyn Markable + Send + Sync>) -> ProtectionIndex {
        let root = mutex_unwrap(self.protection_set.lock())
            .container_protection_set
            .protect(container);
        trace!("Protected container index {}, protection set {}", root, self.index());
        root
    }

    /// Unprotects a container from this thread's container protection set.
    pub(crate) fn drop_container(&self, root: ProtectionIndex) {
        mutex_unwrap(self.protection_set.lock())
            .container_protection_set
            .unprotect(root);
        trace!("Unprotected container index {}, protection set {}", root, self.index());
    }

    /// Parse the given string and returns the Term representation.
    pub fn from_string(&self, term: &str) -> Result<ATerm, MCRL3Error> {
        let mut result = TermParser::parse(Rule::TermSpec, term)?;
        let root = result.next().unwrap();

        Ok(TermParser::TermSpec(root).unwrap())
    }

    /// Protects a symbol from garbage collection.
    pub fn protect_symbol(&self, symbol: &SymbolRef<'_>) -> Symbol {
        let mut lock = mutex_unwrap(self.protection_set.lock());
        let result = unsafe {
            Symbol::from_index(
                symbol.shared(),
                lock.symbol_protection_set.protect(symbol.shared().copy()),
            )
        };

        // Drop to avoid double borrowing.
        trace!(
            "Protected symbol {}, root {}, protection set {}",
            symbol,
            result.root(),
            lock.index,
        );

        result
    }

    /// Unprotects a symbol, allowing it to be garbage collected.
    pub fn drop_symbol(&self, symbol: &mut Symbol) {
        mutex_unwrap(self.protection_set.lock())
            .symbol_protection_set
            .unprotect(symbol.root());
    }
    
    /// Check if the symbol is the default ATermInt symbol
    pub fn is_int_symbol<'a, 'b>(&self, symbol: &'b impl Symb<'a, 'b>) -> bool {
        *self.int_symbol == symbol.copy()
    }

    /// Check if the symbol is the default ATermList symbol
    pub fn is_list<'a, 'b>(&self, symbol: &'b impl Symb<'a, 'b>) -> bool {
        *self.list_symbol == symbol.copy()
    }

    /// Check if the symbol is the default "empty ATermList symbol
    pub fn is_empty_list<'a, 'b>(&self, symbol: &'b impl Symb<'a, 'b>) -> bool {
        *self.empty_list_symbol == symbol.copy()
    }


    /// Returns the index of the protection set.
    fn index(&self) -> usize {
        mutex_unwrap(self.protection_set.lock()).index
    }
}

impl Drop for ThreadTermPool {
    fn drop(&mut self) {
        GLOBAL_TERM_POOL
            .lock()
            .borrow_mut()
            .deregister_thread_pool(self.index());
    }
}

#[cfg(test)]
mod tests {
    use crate::Term;

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
                        assert!(
                            mutex_unwrap(tp.protection_set.lock())
                                .protection_set
                                .contains_root(protected.root())
                        );
                    });

                    // Unprotect
                    let root = protected.root();
                    drop(protected);

                    THREAD_TERM_POOL.with_borrow(|tp| {
                        assert!(
                            !mutex_unwrap(tp.protection_set.lock())
                                .protection_set
                                .contains_root(root)
                        );
                    });
                });
            }
        });
    }

    #[test]
    fn test_parsing() {
        let _ = mcrl3_utilities::test_logger();

        let t = ATerm::from_string("f(g(a),b)").unwrap();

        assert!(t.get_head_symbol().name() == "f");
        assert!(t.arg(0).get_head_symbol().name() == "g");
        assert!(t.arg(1).get_head_symbol().name() == "b");
    }

    #[test]
    fn test_create_term() {
        let _ = mcrl3_utilities::test_logger();

        let f = Symbol::new("f", 2);
        let g = Symbol::new("g", 1);

        let t = THREAD_TERM_POOL.with_borrow(|tp| {
            tp.create_term(
                &f,
                &[
                    tp.create_term(&g, &[tp.create_constant(&Symbol::new("a", 0))]),
                    tp.create_constant(&Symbol::new("b", 0)),
                ],
            )
        });

        assert!(t.get_head_symbol().name() == "f");
        assert!(t.arg(0).get_head_symbol().name() == "g");
        assert!(t.arg(1).get_head_symbol().name() == "b");
    }
}
