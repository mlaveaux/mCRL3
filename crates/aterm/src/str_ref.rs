use std::fmt;
use std::ops::Deref;

use crate::GlobalTermPoolGuard;
use crate::Symb;
use crate::SymbolRef;

/// A reference to the name of a function symbol that is never invalidated.
/// Locks the global term pool so should be kept shortlived.
pub struct StrRef<'a> {
    symbol: SymbolRef<'a>,
    guard: GlobalTermPoolGuard<'a>,
}

impl<'a> StrRef<'a> {
    /// Creates a new protected reference to a string.
    pub(crate) fn new(value: &SymbolRef<'a>, guard: GlobalTermPoolGuard<'_>) -> StrRef<'a> {
        unsafe {
            StrRef {
                symbol: value.copy(),
                guard: std::mem::transmute::<GlobalTermPoolGuard<'_>, GlobalTermPoolGuard<'a>>(guard),
            }
        }
    }
}

impl fmt::Debug for StrRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.deref())
    }
}

impl fmt::Display for StrRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl Deref for StrRef<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self.guard.borrow().symbol_name(&self.symbol)) }
    }
}

impl PartialEq<&str> for StrRef<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}
