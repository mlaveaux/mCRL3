use std::borrow::Borrow;
use std::marker::PhantomData;
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

use delegate::delegate;

use crate::THREAD_TERM_POOL;

/// The public interface for a function symbol, can be used to write generic
/// functions that accept both `Symbol` and `SymbolRef`.
pub trait Symb<'a> {
    /// Obtain the symbol's name.
    fn name(&self) -> &'a str;

    /// Obtain the symbol's arity.
    fn arity(&self) -> usize;

    /// Create a copy of the symbol reference.
    fn copy(&self) -> SymbolRef<'a>;

    /// Returns the internal index of the symbol.
    fn index(&self) -> usize;
}

/// A reference to a function symbol in the term pool.
#[derive(Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolRef<'a> {
    index: usize,
    marker: PhantomData<&'a ()>,
}

/// A Symbol references to an aterm function symbol, which has a name and an arity.
impl<'a> SymbolRef<'a> {
    /// Protect the symbol from garbage collection.
    pub fn protect(&self) -> Symbol {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.protect_symbol(self)
        })
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn from_index(index: usize) -> SymbolRef<'a> {
        SymbolRef {
            index,
            marker: PhantomData,
        }
    }
}

impl SymbolRef<'_> {
    pub(crate) fn from_symbol<'a>(symbol: &impl Symb<'a>) -> SymbolRef<'a> {
        SymbolRef {
            index: symbol.index(),
            marker: PhantomData,
        }
    }
}

impl<'a> Symb<'a> for SymbolRef<'a> {
    fn name(&self) -> &'a str {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.symbol_name(self)
        })
    }

    fn arity(&self) -> usize {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.symbol_arity(self)
        })
    }

    fn copy(&self) -> SymbolRef<'a> {
        SymbolRef::from_index(self.index)
    }

    fn index(&self) -> usize {
        self.index
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
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.create_symbol(name, arity)
        })
    }
}

impl Symbol {
    pub(crate) fn new_internal(index: usize, root: usize) -> Symbol {
        Self {
            symbol: SymbolRef::from_index(index),
            root,
        }
    }

    pub(crate) fn root(&self) -> usize {
        self.root
    }

    /// Create a copy of the symbol reference.
    pub fn copy(&self) -> SymbolRef<'_> {
        self.symbol.copy()
    }
}

impl<'a> Symb<'a> for Symbol {
    delegate! {
        to self.symbol {
            fn name(&self) -> &'a str;
            fn arity(&self) -> usize;
            fn copy(&self) -> SymbolRef<'a>;
            fn index(&self) -> usize;
        }
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        if self.index != 0 {
            THREAD_TERM_POOL.with_borrow(|tp| {
                if self.index != 0 {
                    tp.drop_symbol(self);
                }
            })
        }
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
