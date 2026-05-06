use std::marker::PhantomData;
use std::ptr::NonNull;
use std::ptr::slice_from_raw_parts_mut;

use merc_aterm::ATermIndex;
use merc_aterm::ATermRef;
use merc_aterm::SymbolRef;
use merc_aterm::Term;
use merc_data::DataApplication;
use merc_data::DataExpressionRef;
use merc_unsafety::ProtectionIndex;

#[cfg(feature = "import")]
mod import;
#[cfg(feature = "import")]
pub use import::*;

#[cfg(not(feature = "import"))]
mod export;
#[cfg(not(feature = "import"))]
pub use export::*;

/// Represents a FFI stable reference to a [merc_data::DataExpression].
#[repr(C)]
pub struct DataExpressionFFI {
    index: ATermIndex,
    root: ProtectionIndex,
}

impl DataExpressionFFI {
    /// Creates a new data expression from a symbol and its arguments.
    pub fn create(symbol: DataExpressionRefFFI, args: &[DataExpressionRefFFI]) -> Self {
        unsafe {
            let symbol_ref = DataExpressionRef::from(ATermRef::from_index(&symbol.index));
            let t = DataApplication::with_iter(
                &symbol_ref,
                args.len(),
                args.iter().map(|a| ATermRef::from_index(a.shared())),
            );
            let d = DataExpressionFFI::from_index(t.shared(), t.root());
            std::mem::forget(t);
            d
        }
    }

    /// Creates a new data expression from a symbol with no arguments.
    pub fn constant(symbol: DataExpressionRefFFI) -> Self {
        unsafe {
            let symbol_ref = DataExpressionRef::from(ATermRef::from_index(&symbol.index));
            let t = symbol_ref.protect();
            let d = DataExpressionFFI::from_index(t.shared(), t.root());
            std::mem::forget(t);
            d
        }
    }

    /// Creates a new data expression from an index and a root.
    ///
    /// # Safety
    ///
    /// The index must be a valid index of a data expression, that is valid for this lifetime.
    pub unsafe fn from_index(index: &ATermIndex, root: ProtectionIndex) -> Self {
        Self {
            index: index.copy(),
            root,
        }
    }

    /// # Safety
    ///
    /// The index must be a valid index of a data expression, that is valid for this lifetime.
    pub fn copy(&self) -> DataExpressionRefFFI<'_> {
        unsafe { DataExpressionRefFFI::from_index(&self.index) }
    }

    /// Returns the index of the data expression.
    pub fn index(&self) -> &ATermIndex {
        &self.index
    }

    /// Protects the data expression, preventing it from being garbage collected.
    pub fn protect(&self) -> DataExpressionFFI {
        unsafe { data_expression_protect(&self.copy()) }
    }
}

/// Represents a FFI stable reference to a [merc_data::DataExpressionRef].
#[repr(transparent)]
pub struct DataExpressionRefFFI<'a> {
    index: ATermIndex,
    _marker: PhantomData<&'a ()>,
}

impl DataExpressionRefFFI<'_> {
    /// # Safety
    /// The index must be a valid index of a data expression, that is valid for this lifetime.
    pub unsafe fn from_index(index: &ATermIndex) -> Self {
        Self {
            index: index.copy(),
            _marker: PhantomData,
        }
    }

    ///
    ///
    /// # Safety
    ///
    /// The index must be a valid index of a symbol, that is valid for this lifetime.
    pub unsafe fn from_ptr(index: usize, arity: usize) -> Self {
        // ATermIndex is a stable pointer to SharedTerm (DST), so it must include
        // metadata for the argument slice length (arity).
        let raw = slice_from_raw_parts_mut(index as *mut SymbolRef<'static>, arity) as *mut _;
        Self {
            index: unsafe { ATermIndex::from_ptr(NonNull::new(raw).unwrap()) },
            _marker: PhantomData,
        }
    }

    /// Returns the index of the data expression.
    pub fn shared(&self) -> &ATermIndex {
        &self.index
    }
}

impl<'a> DataExpressionRefFFI<'a> {
    /// Returns the data function symbol of the data expression.
    pub fn data_function_symbol(&self) -> DataFunctionSymbolRefFFI<'a> {
        unsafe { data_expression_symbol(self) }
    }

    /// Returns the argument of the data expression at the given index.
    pub fn data_arg(&self, index: usize) -> DataExpressionRefFFI<'a> {
        unsafe { data_expression_arg(self, index) }
    }

    /// Returns a copy of the data expression.
    pub fn copy(&self) -> DataExpressionRefFFI<'a> {
        unsafe { DataExpressionRefFFI::from_index(&self.index) }
    }

    /// Protects the data expression, preventing it from being garbage collected.
    pub fn protect(&self) -> DataExpressionFFI {
        unsafe { data_expression_protect(self) }
    }
}

#[repr(C)]
pub struct DataFunctionSymbolRefFFI<'a> {
    index: ATermIndex,
    _marker: PhantomData<&'a ()>,
}

impl DataFunctionSymbolRefFFI<'_> {
    /// # Safety
    /// The index must be a valid index of a data function symbol, that is valid for this lifetime.
    unsafe fn from_index(index: &ATermIndex) -> Self {
        Self {
            index: index.copy(),
            _marker: PhantomData,
        }
    }
}

impl DataFunctionSymbolRefFFI<'_> {
    /// Returns the operation id of the data function symbol.
    pub fn operation_id(&self) -> usize {
        self.index.index()
    }
}
