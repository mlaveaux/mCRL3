#![allow(dead_code)]

use std::hash::Hash;
use std::ptr::NonNull;
use std::ptr::slice_from_raw_parts_mut;

use equivalent::Equivalent;
use merc_unsafety::AllocBlock;
use merc_unsafety::StablePointer;
use merc_unsafety::StablePointerSet;
use rustc_hash::FxBuildHasher;

use crate::ATermIndex;
use crate::SymbolRef;
use crate::storage::SharedTerm;

/// Storage for ATerms with a fixed number of arguments.
///
/// Should be the same layout as [`crate::SharedTerm`] for the shared fields.
#[repr(C)]
#[derive(Hash, Eq, PartialEq)]
struct SharedTermFixed<const N: usize> {
    symbol: SymbolRef<'static>,
    args: [ATermIndex; N],
}

/// Storage for ATerms with a fixed number of arguments.
///
/// Should be the same layout as [`crate::SharedTerm`] for the shared fields.
#[repr(C)]
#[derive(Hash, Eq, PartialEq)]
pub(crate) struct SharedTermInt {
    symbol: SymbolRef<'static>,
    args: [usize; 1],
}

impl SharedTermInt {
    /// Returns the value of the integer term.
    pub fn value(&self) -> usize {
        self.args[0]
    }
}

/// The actualy storage for ATerm. Terms are stored in separated
/// `StablePointerSet`s based on their arity, and whether they have annotations
/// or not.
/// 
/// 
pub(crate) struct ATermStorage {
    /// Stores terms of any arity.
    terms: StablePointerSet<SharedTerm>,

    /// Stores the fixed size integer terms
    int_terms: StablePointerSet<SharedTermInt, FxBuildHasher, AllocBlock<SharedTermInt, 1024>>,
}

impl ATermStorage {
    /// Creates a new, empty storage.
    pub fn new() -> Self {
        Self {
            terms: StablePointerSet::new(),
            int_terms: StablePointerSet::with_capacity_in(1000, AllocBlock::new()),
        }
    }

    /// Returns the number of stored terms.
    pub fn len(&self) -> usize {
        self.int_terms.len() + self.terms.len()
    }

    /// Retains only the terms for which the given predicate returns `true`.
    pub fn retain<F>(&self, mut f: F)
    where
        F: FnMut(&StablePointer<SharedTerm>) -> bool,
    {
        // self.int_terms.retain(|term| f(term));
        self.terms.retain(|term| f(term));
    }

    /// Inserts a term into the storage, returning a pointer to the stored term
    /// and whether it was newly inserted.
    ///
    /// The given term should not be an integer term.
    pub unsafe fn insert_equiv_dst<'a, Q, C>(
        &self,
        value: &'a Q,
        length: usize,
        construct: C,
    ) -> (StablePointer<SharedTerm>, bool)
    where
        Q: Hash + Equivalent<SharedTerm>,
        C: Fn(*mut SharedTerm, &'a Q),
    {
        unsafe { self.terms.insert_equiv_dst(value, length, construct) }
    }

    /// Inserts an integer term into the storage, returning a pointer to the stored term
    /// and whether it was newly inserted.
    pub unsafe fn insert_int_term<'a>(
        &self,
        symbol: SymbolRef<'_>,
        value: usize,
    ) -> (StablePointer<SharedTerm>, bool) {
        unsafe {
            let (result, inserted) = self.int_terms.insert(SharedTermInt {
                symbol: std::mem::transmute::<SymbolRef<'_>, SymbolRef<'static>>(symbol),
                args: [value as usize],
            });

            // Convert into a StablePointer<SharedTerm> by transmuting the pointer and adjusting the layout.
            let ptr = slice_from_raw_parts_mut(result.ptr().as_ptr(), 0) as *mut SharedTerm;
            (StablePointer::from_ptr(NonNull::new_unchecked(ptr)), inserted)
        }
    }
}
