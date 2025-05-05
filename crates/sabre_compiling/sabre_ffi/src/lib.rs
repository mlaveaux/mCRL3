use std::marker::PhantomData;

#[cfg(feature = "import")]
mod import;
#[cfg(feature = "import")]
pub use import::*;

#[cfg(not(feature = "import"))]
mod export;
#[cfg(not(feature = "import"))]
pub use export::*;
use mcrl3_aterm::ATermIndex;
use mcrl3_utilities::ProtectionIndex;

#[repr(C)]
pub struct DataExpression {
    index: ATermIndex,
    root: ProtectionIndex,
}

impl DataExpression {
    pub unsafe fn from_index(index: &ATermIndex, root: ProtectionIndex) -> Self {
        Self {
            index: index.copy(),
            root,
        }
    }

    /// # Safety
    ///
    /// The index must be a valid index of a data expression, that is valid for this lifetime.
    pub fn copy(&self) -> DataExpressionRef<'_> {
        unsafe { DataExpressionRef::from_index(&self.index) }
    }

    /// Returns the index of the data expression.
    pub fn index(&self) -> &ATermIndex {
        &self.index
    }
}

#[repr(C)]
pub struct DataExpressionRef<'a> {
    index: ATermIndex,
    _marker: PhantomData<&'a ()>,
}

impl DataExpressionRef<'_> {
    /// # Safety
    /// The index must be a valid index of a data expression, that is valid for this lifetime.
    pub unsafe fn from_index(index: &ATermIndex) -> Self {
        Self {
            index: index.copy(),
            _marker: PhantomData,
        }
    }

    /// Returns the index of the data expression.
    pub fn shared(&self) -> &ATermIndex {
        &self.index
    }
}

impl<'a> DataExpressionRef<'a> {
    /// Returns the data function symbol of the data expression.
    pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'a> {
        unsafe { data_expression_symbol(self) }
    }

    /// Returns the argument of the data expression at the given index.
    pub fn arg(&self, index: usize) -> DataExpressionRef<'a> {
        unsafe { data_expression_arg(self, index) }
    }

    /// Returns a copy of the data expression.
    pub fn copy(&self) -> DataExpressionRef<'a> {
        unsafe { DataExpressionRef::from_index(&self.index) }
    }

    /// Protects the data expression, preventing it from being garbage collected.
    pub fn protect(&self) -> DataExpression {
        unsafe { data_expression_protect(self) }
    }
}

#[repr(C)]
pub struct DataFunctionSymbolRef<'a> {
    index: ATermIndex,
    _marker: PhantomData<&'a ()>,
}

impl DataFunctionSymbolRef<'_> {
    /// # Safety
    /// The index must be a valid index of a data function symbol, that is valid for this lifetime.
    unsafe fn from_index(index: &ATermIndex) -> Self {
        Self {
            index: index.copy(),
            _marker: PhantomData,
        }
    }
}

impl DataFunctionSymbolRef<'_> {
    /// Returns the operation id of the data function symbol.
    pub fn operation_id(&self) -> usize {
        self.index.address()
    }
}
