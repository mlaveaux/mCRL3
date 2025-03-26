use std::mem::ManuallyDrop;

use crate::Symbol;
use crate::SymbolRef;

thread_local! {
    pub(crate) static DEFAULT_SYMBOLS: DefaultSymbol = DefaultSymbol::new();
}

pub(crate) struct DefaultSymbol {
    /// These default symbols should not call their drop, will be removed when the protection set goes out of scope
    pub(crate) int_symbol: ManuallyDrop<Symbol>,
    pub(crate) empty_list_symbol: ManuallyDrop<Symbol>,
    pub(crate) list_symbol: ManuallyDrop<Symbol>,
}

impl DefaultSymbol {
    fn new() -> Self {
        Self {
            int_symbol: ManuallyDrop::new(Symbol::new("Int", 0)),
            list_symbol: ManuallyDrop::new(Symbol::new("List", 2)),
            empty_list_symbol: ManuallyDrop::new(Symbol::new("[]", 0)),
        }
    }
}

/// Check if the symbol is the default "Int" symbol
pub fn is_int(symbol: &SymbolRef<'_>) -> bool {
    DEFAULT_SYMBOLS.with(|ds| **ds.int_symbol == *symbol)
}

/// Check if the symbol is the default "List" symbol
pub fn is_list(symbol: &SymbolRef<'_>) -> bool {
    DEFAULT_SYMBOLS.with(|ds| **ds.list_symbol == *symbol)
}

/// Check if the symbol is the default "[]" symbol
pub fn is_empty_list(symbol: &SymbolRef<'_>) -> bool {
    DEFAULT_SYMBOLS.with(|ds| **ds.empty_list_symbol == *symbol)
}
