use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::num::NonZero;
use std::ops::Deref;

use delegate::delegate;

use crate::StrRef;
use crate::THREAD_TERM_POOL;

/// The public interface for a function symbol, can be used to write generic
/// functions that accept both `Symbol` and `SymbolRef`.
pub trait Symb<'a, 'b> {
    /// Obtain the symbol's name.
    fn name(&'b self) -> StrRef<'a>;

    /// Obtain the symbol's arity.
    fn arity(&'b self) -> usize;

    /// Create a copy of the symbol reference.
    fn copy(&'b self) -> SymbolRef<'a>;

    /// Returns the internal index of the symbol.
    fn index(&'b self) -> NonZero<usize>;
}

/// A reference to a function symbol in the term pool.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolRef<'a> {
    index: NonZero<usize>,
    marker: PhantomData<&'a ()>,
}

/// Check that the SymbolRef is the same size as a usize.
const _: () = assert!(std::mem::size_of::<SymbolRef>() == std::mem::size_of::<usize>());

/// A reference to a function symbol with a known lifetime.
impl<'a> SymbolRef<'a> {
    pub(crate) fn from_index(index: NonZero<usize>) -> SymbolRef<'a> {
        SymbolRef {
            index,
            marker: PhantomData,
        }
    }

    /// Protects the symbol from garbage collection, yielding a `Symbol`.
    pub fn protect(&self) -> Symbol {
        THREAD_TERM_POOL.with_borrow(|tp| tp.protect_symbol(self))
    }
}

impl SymbolRef<'_> {
    /// Internal constructo to convert any `Symb` to a `SymbolRef`.
    pub(crate) fn from_symbol<'a, 'b>(symbol: &'b impl Symb<'a, 'b>) -> Self {
        SymbolRef {
            index: symbol.index(),
            marker: PhantomData,
        }
    }
}

impl<'a> Symb<'a, '_> for SymbolRef<'a> {
    fn name(&self) -> StrRef<'a> {
        THREAD_TERM_POOL.with_borrow(|tp| tp.symbol_name(self))
    }

    fn arity(&self) -> usize {
        THREAD_TERM_POOL.with_borrow(|tp| tp.symbol_arity(self))
    }

    fn copy(&self) -> SymbolRef<'a> {
        SymbolRef::from_index(self.index)
    }

    fn index(&self) -> NonZero<usize> {
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

/// A protected function symbol.
pub struct Symbol {
    symbol: SymbolRef<'static>,
    root: usize,
}

impl Symbol {
    /// Create a new symbol with the given name and arity.
    pub fn new(name: impl Into<String> + AsRef<str>, arity: usize) -> Symbol {
        THREAD_TERM_POOL.with_borrow(|tp| tp.create_symbol(name, arity))
    }
}

impl Symbol {
    /// Internal constructor to create a symbol from an index and a root.
    pub(crate) fn from_index(index: NonZero<usize>, root: usize) -> Symbol {
        Self {
            symbol: SymbolRef::from_index(index),
            root,
        }
    }

    /// Returns the root index, i.e., the index in the protection set. See `SharedTermProtection`.
    pub(crate) fn root(&self) -> usize {
        self.root
    }

    /// Create a copy of the symbol reference.
    pub fn copy(&self) -> SymbolRef<'_> {
        self.symbol.copy()
    }
}

impl<'a> Symb<'a, '_> for &'a Symbol {
    delegate! {
        to self.symbol {
            fn name(&self) -> StrRef<'a>;
            fn arity(&self) -> usize;
            fn copy(&self) -> SymbolRef<'a>;
            fn index(&self) -> NonZero<usize>;
        }
    }
}

impl<'a, 'b> Symb<'a, 'b> for Symbol
where
    'b: 'a,
{
    delegate! {
        to self.symbol {
            fn name(&self) -> StrRef<'a>;
            fn arity(&self) -> usize;
            fn copy(&self) -> SymbolRef<'a>;
            fn index(&self) -> NonZero<usize>;
        }
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.drop_symbol(self);
        })
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
