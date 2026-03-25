use std::alloc::Layout;
use std::alloc::LayoutError;
use std::fmt;
use std::hash::Hash;
use std::mem::offset_of;
use std::ptr;
use std::ptr::NonNull;
use std::ptr::slice_from_raw_parts_mut;

use equivalent::Equivalent;
use merc_unsafety::Erasable;
use merc_unsafety::ErasedPtr;
use merc_unsafety::SliceDst;
use merc_unsafety::repr_c;

use crate::ATermRef;
use crate::Symb;
use crate::SymbolRef;
use crate::Term;

/// The underlying type of terms that are maximally shared.
///
/// # Details
///
/// Uses a C representation and is a dynamically sized type for compact memory
/// usage, implementing [SliceDst] and [Erasable]. This allows us to avoid
/// storing the length and capacity of an underlying vector. As such this is
/// even more compact than `smallvec`. Arguments are stored as [ATermRef] slices.
#[repr(C)]
pub struct SharedTerm {
    symbol: SymbolRef<'static>,
    arguments: [ATermRef<'static>],
}

impl PartialEq for SharedTerm {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol && self.arguments() == other.arguments()
    }
}

impl Eq for SharedTerm {}

/// Note that the length is stored in the symbol's arity
unsafe impl SliceDst for SharedTerm {
    fn layout_for(len: usize) -> Result<Layout, LayoutError> {
        let header_layout = Layout::new::<SymbolRef<'static>>();
        let slice_layout = Layout::array::<ATermRef<'static>>(len)?;

        repr_c(&[header_layout, slice_layout])
    }

    fn retype(ptr: std::ptr::NonNull<[()]>) -> NonNull<Self> {
        unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut _) }
    }

    fn length(&self) -> usize {
        self.symbol().arity()
    }
}

unsafe impl Erasable for SharedTerm {
    fn erase(this: NonNull<Self>) -> ErasedPtr {
        this.cast()
    }

    unsafe fn unerase(this: ErasedPtr) -> NonNull<Self> {
        unsafe {
            let symbol: SymbolRef = ptr::read(this.as_ptr().cast());
            let len = symbol.arity();

            let raw = NonNull::new_unchecked(slice_from_raw_parts_mut(this.as_ptr().cast(), len));
            Self::retype(raw)
        }
    }
}

impl fmt::Debug for SharedTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SharedTerm {{ symbol: {:?}, arguments: {:?} }}",
            self.symbol,
            self.arguments()
        )
    }
}

impl SharedTerm {
    /// Returns the symbol of the term.
    pub fn symbol(&self) -> &SymbolRef<'_> {
        &self.symbol
    }

    /// Returns the arguments of the term.
    pub fn arguments(&self) -> &[ATermRef<'static>] {
        &self.arguments
    }

    /// Returns a unique index for this shared term.
    pub fn index(&self) -> usize {
        self as *const Self as *const u8 as usize
    }

    /// Returns the length for a [SharedTermLookup]
    pub(crate) fn length_for(object: &SharedTermLookup) -> usize {
        object.arguments.len()
    }

    /// Constructs an uninitialised ptr from a [SharedTermLookup]
    pub(crate) unsafe fn construct(ptr: *mut SharedTerm, object: &SharedTermLookup) {
        let header_layout = Layout::new::<SymbolRef<'static>>();
        let slice_layout =
            Layout::array::<ATermRef<'static>>(object.arguments.len()).expect("Layout should not exceed isize");

        let (_, slice_offset) = header_layout
            .extend(slice_layout)
            .expect("Layout should not exceed isize");
        unsafe {
            ptr.cast::<SymbolRef<'static>>()
                .write(SymbolRef::from_index(object.symbol.shared()));

            for (index, argument) in object.arguments.iter().enumerate() {
                ptr.byte_offset(slice_offset as isize)
                    .cast::<ATermRef<'static>>()
                    .add(index)
                    .write(ATermRef::from_index(argument.shared()));
            }
        }
    }
}

impl Hash for SharedTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.arguments().hash(state);
    }
}

/// A cheap reference to the elements of a [SharedTerm] that can be used for
/// lookup of terms without allocating.
pub(crate) struct SharedTermLookup<'a> {
    pub(crate) symbol: SymbolRef<'a>,
    pub(crate) arguments: &'a [ATermRef<'a>],
}

impl Equivalent<SharedTerm> for SharedTermLookup<'_> {
    fn equivalent(&self, other: &SharedTerm) -> bool {
        self.symbol == other.symbol && self.arguments == other.arguments()
    }
}

/// This Hash implement must be the same as for [SharedTerm]
impl Hash for SharedTermLookup<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.arguments.hash(state);
    }
}

// `symbol` must be at offset 0 in all term representations so that any pointer to a term.
const _: () = assert!(offset_of!(SharedTerm, symbol) == 0);

#[cfg(test)]
mod tests {
    use allocator_api2::alloc::Global;
    use merc_unsafety::AllocatorDst;

    use crate::ATerm;
    use crate::Symbol;
    use crate::Term;

    use super::*;

    #[test]
    #[cfg(not(debug_assertions))]
    fn test_shared_symbol_size() {
        // Cannot be a const assertion since the size depends on the length.
        assert_eq!(
            SharedTerm::layout_for(0)
                .expect("The layout should not overflow")
                .size(),
            1 * std::mem::size_of::<usize>(),
            "A SharedTerm without arguments should be the same size as the Symbol"
        );

        assert_eq!(
            SharedTerm::layout_for(2)
                .expect("The layout should not overflow")
                .size(),
            3 * std::mem::size_of::<usize>(),
            "A SharedTerm with arity two should be the same size as the Symbol and two ATermRef arguments"
        );
    }

    #[test]
    fn test_shared_term_lookup() {
        let symbol = Symbol::new("a", 2);

        let term = ATerm::constant(&Symbol::new("b", 0));

        let lookup = SharedTermLookup {
            symbol: symbol.copy(),
            arguments: &[term.copy(), term.copy()],
        };

        let ptr = Global.allocate_slice_dst(2).expect("Could not allocate slice dst");

        unsafe {
            SharedTerm::construct(ptr.as_ptr(), &lookup);
            assert_eq!(
                *ptr.as_ref().symbol(),
                symbol.copy(),
                "The symbol should match the lookup symbol"
            );
            assert_eq!(
                ptr.as_ref().arguments()[0],
                term.copy(),
                "The arguments should match the lookup arguments"
            );
            assert_eq!(
                ptr.as_ref().arguments()[1],
                term.copy(),
                "The arguments should match the lookup arguments"
            );
        }

        Global.deallocate_slice_dst(ptr, 2);
    }
}
