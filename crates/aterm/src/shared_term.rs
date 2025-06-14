use std::fmt;
use std::hash::Hash;

use hashbrown::Equivalent;

use crate::ATermRef;
use crate::Symb;
use crate::SymbolRef;
use crate::Term;

/// The underlying type of terms that are actually shared.
#[derive(Eq, PartialEq)]
pub struct SharedTerm {
    symbol: SymbolRef<'static>,
    arguments: Vec<ATermRef<'static>>,
    annotation: Option<usize>,
}

// #[cfg(not(debug_assertions))]
// const _: () = assert!(std::mem::size_of::<SharedTerm>() == std::mem::size_of::<usize>() * 3);

impl fmt::Debug for SharedTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SharedTerm {{ symbol: {:?}, arguments: {:?}, annotation: {:?} }}",
            self.symbol, self.arguments, self.annotation
        )
    }
}

impl Clone for SharedTerm {
    fn clone(&self) -> Self {
        SharedTerm {
            symbol: unsafe { SymbolRef::from_index(self.symbol.shared()) },
            arguments: self
                .arguments
                .iter()
                .map(|x| unsafe { ATermRef::from_index(x.shared()) })
                .collect(),
            annotation: self.annotation,
        }
    }
}

impl SharedTerm {
    pub fn symbol(&self) -> &SymbolRef<'_> {
        &self.symbol
    }

    pub fn arguments(&self) -> &[ATermRef<'static>] {
        &self.arguments
    }

    pub fn annotation(&self) -> Option<usize> {
        self.annotation
    }
}

/// A cheap reference to the elements of a shared term that can be used for
/// lookup of terms without allocating.
pub(crate) struct SharedTermLookup<'a> {
    pub(crate) symbol: SymbolRef<'a>,
    pub(crate) arguments: &'a [ATermRef<'a>],
    pub(crate) annotation: Option<usize>,
}

impl From<&SharedTermLookup<'_>> for SharedTerm {
    fn from(lookup: &SharedTermLookup<'_>) -> Self {
        SharedTerm {
            symbol: unsafe { SymbolRef::from_index(lookup.symbol.shared()) },
            arguments: lookup
                .arguments
                .iter()
                .map(|x| unsafe { ATermRef::from_index(x.shared()) })
                .collect(),
            annotation: lookup.annotation,
        }
    }
}

impl Equivalent<SharedTerm> for SharedTermLookup<'_> {
    fn equivalent(&self, other: &SharedTerm) -> bool {
        self.symbol == other.symbol && self.arguments == other.arguments && self.annotation == other.annotation
    }
}

/// This Hash implement must be the same as for [SharedTerm]
impl Hash for SharedTermLookup<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.arguments.hash(state);
        self.annotation.hash(state);
    }
}

impl Hash for SharedTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.arguments.hash(state);
        self.annotation.hash(state);
    }
}
