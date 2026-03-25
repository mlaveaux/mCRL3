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
use crate::Symb;
use crate::SymbolRef;
use crate::storage::SharedTerm;

/// The actual storage for [crate::ATerm]. Terms are stored in separated
/// `StablePointerSet`s based on their arity, and whether they have annotations
/// or not.
pub(crate) struct ATermStorage {
    /// Stores terms of any arity.
    terms: StablePointerSet<SharedTerm>,

    /// Stores the fixed size [SharedTermInt] integer terms.
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
    /// The given term should not be a [SharedTermInt] integer term.
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
                symbol: SymbolRef::from_index(symbol.shared()),
                annotation: value,
            });

            // Convert into a StablePointer<SharedTerm> by transmuting the pointer and adjusting the layout.
            let ptr = slice_from_raw_parts_mut(result.ptr().as_ptr(), 0) as *mut SharedTerm;
            (StablePointer::from_ptr(NonNull::new_unchecked(ptr)), inserted)
        }
    }
}

/// Storage for ATerms with a fixed number of arguments.
///
/// Should be the same layout as [`crate::SharedTerm`] for the shared fields.
#[repr(C)]
#[derive(Hash, Eq, PartialEq)]
struct SharedTermFixed<const N: usize> {
    symbol: SymbolRef<'static>,
    args: [ATermIndex; N],
}

/// Storage for integer ATerms.
///
/// Should be the same layout as [`crate::SharedTerm`] for the shared fields.
#[repr(C)]
#[derive(Hash, Eq, PartialEq)]
pub(crate) struct SharedTermInt {
    symbol: SymbolRef<'static>,

    /// The only important aspect is that `symbol` remains in the same position,
    /// and has arity 0.
    annotation: usize,
}

impl SharedTermInt {
    /// Returns the value of the integer term.
    pub fn value(&self) -> usize {
        self.annotation
    }
}

#[cfg(test)]
mod tests {
    use std::mem::align_of;
    use std::mem::offset_of;
    use std::mem::size_of;

    use crate::ATermIndex;
    use crate::ATermRef;

    use super::SharedTermFixed;
    use super::SharedTermInt;

    // `symbol` must be at offset 0 in all term representations so that any pointer to a term
    // can safely be cast to `*const SymbolRef` to read the header.
    const _: () = assert!(offset_of!(SharedTermFixed<1>, symbol) == 0);
    const _: () = assert!(offset_of!(SharedTermInt, symbol) == 0);

    // The args (SharedTermFixed) and annotation (SharedTermInt) fields must start at the same
    // byte offset.
    const _: () = assert!(offset_of!(SharedTermFixed<1>, args) == offset_of!(SharedTermInt, annotation));

    // Both element types must have identical size and alignment so that indexing into the
    // argument array produces the same byte offsets in both representations.
    const _: () = assert!(size_of::<ATermIndex>() == size_of::<ATermRef<'static>>());
    const _: () = assert!(align_of::<ATermIndex>() == align_of::<ATermRef<'static>>());
}
