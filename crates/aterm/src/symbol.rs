use std::borrow::Borrow;
use std::marker::PhantomData;

use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

use crate::GLOBAL_TERM_POOL;

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

    /// Protect the symbol from garbage collection
    pub fn protect(&self) -> Symbol {
        GLOBAL_TERM_POOL.lock().symbol_pool_mut().protect(self)
    }

    pub fn copy(&self) -> SymbolRef<'_> {
        SymbolRef::new(self.index)
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }
}

impl SymbolRef<'_> {
    /// Obtain the symbol's name
    pub fn name(&self) -> &str {
        GLOBAL_TERM_POOL.lock().symbol_pool().get(self).name()
    }

    /// Obtain the symbol's arity
    pub fn arity(&self) -> usize {
        GLOBAL_TERM_POOL.lock().symbol_pool().get(self).arity()
    }
}

impl fmt::Display for SymbolRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for SymbolRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}

#[derive(Default)]
pub struct Symbol {
    symbol: SymbolRef<'static>,
    root: usize,
}

impl Symbol {
    pub fn new(name: impl Into<String>, arity: usize) -> Symbol {
        GLOBAL_TERM_POOL
            .lock()
            .symbol_pool_mut()
            .create(name, arity)
    }

    pub(crate) fn new_internal(index: usize, root: usize) -> Symbol {
        Self {
            symbol: SymbolRef::new(index),
            root,
        }
    }

    pub fn name(&self) -> &str {
        self.symbol.name()
    }

    pub fn arity(&self) -> usize {
        self.symbol.arity()
    }

    pub(crate) fn root(&self) -> usize {
        self.root
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        GLOBAL_TERM_POOL
            .lock()
            .symbol_pool_mut()
            .unprotect(std::mem::take(self));
    }
}

impl Symbol {
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
