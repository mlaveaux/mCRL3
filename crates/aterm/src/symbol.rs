use std::borrow::Borrow;
use std::marker::PhantomData;
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

use crate::GLOBAL_TERM_POOL;
use crate::THREAD_TERM_POOL;

/// A reference to a function symbol in the term pool.
#[derive(Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolRef<'a> {
    index: usize,
    marker: PhantomData<&'a ()>,
}

/// A Symbol references to an aterm function symbol, which has a name and an arity.
impl<'a> SymbolRef<'a> {
    pub(crate) fn new(symbol: usize) -> SymbolRef<'a> {
        SymbolRef {
            index: symbol,
            marker: PhantomData,
        }
    }

    /// Protect the symbol from garbage collection.
    pub fn protect(&self) -> Symbol {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.protect_symbol(self)
        })
    }

    /// Create a copy of the symbol reference.
    pub fn copy(&self) -> SymbolRef<'_> {
        SymbolRef::new(self.index)
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }
}

impl SymbolRef<'_> {
    /// Obtain the symbol's name.
    pub fn name(&self) -> &str {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.symbol_name(self)
        })
    }

    /// Obtain the symbol's arity.
    pub fn arity(&self) -> usize {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.symbol_arity(self)
        })
    }
}

impl fmt::Display for SymbolRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for SymbolRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A struct representing a function symbol with a name and arity.
#[derive(Default)]
pub struct Symbol {
    symbol: SymbolRef<'static>,
    root: usize,
}

impl Symbol {
    /// Create a new symbol with the given name and arity.
    pub fn new(name: impl Into<String>, arity: usize) -> Symbol {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.create_symbol(name, arity)
        })
    }

    pub(crate) fn new_internal(index: usize, root: usize) -> Symbol {
        Self {
            symbol: SymbolRef::new(index),
            root,
        }
    }

    /// Get the name of the symbol.
    pub fn name(&self) -> &str {
        self.symbol.name()
    }

    /// Get the arity of the symbol.
    pub fn arity(&self) -> usize {
        self.symbol.arity()
    }

    pub(crate) fn root(&self) -> usize {
        self.root
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        if self.index != 0 {
            THREAD_TERM_POOL.with_borrow_mut(|tp| {
                if self.index != 0 {
                    tp.drop_symbol(self);
                }
            })
        }
    }
}

impl Symbol {
    /// Create a copy of the symbol reference.
    pub fn copy(&self) -> SymbolRef<'_> {
        self.symbol.copy()
    }
}

impl From<&SymbolRef<'_>> for Symbol {
    fn from(value: &SymbolRef) -> Self {
        value.protect()
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        self.copy().protect()
    }
}

impl Deref for Symbol {
    type Target = SymbolRef<'static>;

    fn deref(&self) -> &Self::Target {
        &self.symbol
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.copy().hash(state)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.copy().eq(&other.copy())
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.copy().cmp(&other.copy()))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        self.copy().cmp(&other.copy())
    }
}

impl Borrow<SymbolRef<'static>> for Symbol {
    fn borrow(&self) -> &SymbolRef<'static> {
        &self.symbol
    }
}

impl Eq for Symbol {}
