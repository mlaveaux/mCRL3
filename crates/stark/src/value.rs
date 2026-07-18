//! Runtime values, ported from `values/StarkValue.java`'s sealed hierarchy.
//!
//! The Java reference models `StarkValue` as an interface with one class per
//! case (`StarkIntegerValue`, `StarkRealValue`, ...). Here the same case
//! analysis is one flat `Copy` enum, matching the arena IR's "small, `Copy`,
//! contiguous" philosophy (see `IR_LOWERING_PLAN.md`).
//!
//! Only construction, `Debug`/`Display` and [Value::type_of] land here. The
//! arithmetic (`sum`/`product`/`isLessThan`/…, with Java's int-preserving-
//! then-widening promotion rules) belongs to the (deferred) evaluator.

use std::fmt;

use crate::ast::DefId;
use crate::resolve::SymbolTable;
use crate::types::StarkType;

/// An instance of a user-defined `type X = A | B | C;` value.
///
/// `element` is the declared element's position within `type_id`'s own
/// `elements` list (`0` for the first alternative, and so on), not a name —
/// mirroring `StarkCustomValue`, but index-keyed rather than string-keyed, so
/// comparing two custom values of the same type is an integer compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustomValue {
    /// The `DefId` of the owning `type X = ...;` declaration (not the
    /// element itself).
    pub type_id: DefId,
    pub element: u32,
}

/// A runtime value flowing through the (future) evaluator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Custom(CustomValue),
    /// The result of a runtime error (e.g. division by zero). The evaluator
    /// is meant to propagate this rather than panic, mirroring
    /// `StarkValue.ERROR_VALUE` — worth preserving even though the evaluator
    /// itself is out of scope here.
    Error,
}

impl Value {
    /// This value's [StarkType]. `symbols` resolves a [CustomValue]'s
    /// `type_id` back to the type's declared name.
    pub fn type_of(&self, symbols: &SymbolTable) -> StarkType {
        match self {
            Value::Integer(_) => StarkType::Integer,
            Value::Real(_) => StarkType::Real,
            Value::Boolean(_) => StarkType::Boolean,
            Value::Custom(custom) => StarkType::Custom(symbols.def(custom.type_id).name.clone()),
            Value::Error => StarkType::Error,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(value) => write!(f, "{value}"),
            Value::Real(value) => write!(f, "{value}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Custom(custom) => write!(f, "{:?}#{}", custom.type_id, custom.element),
            Value::Error => write!(f, "<error>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_of_plain_values() {
        let symbols = SymbolTable::default();
        assert_eq!(Value::Integer(1).type_of(&symbols), StarkType::Integer);
        assert_eq!(Value::Real(1.0).type_of(&symbols), StarkType::Real);
        assert_eq!(Value::Boolean(true).type_of(&symbols), StarkType::Boolean);
        assert_eq!(Value::Error.type_of(&symbols), StarkType::Error);
    }

    #[test]
    fn display_formats_plain_values() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Real(1.5).to_string(), "1.5");
        assert_eq!(Value::Boolean(false).to_string(), "false");
        assert_eq!(Value::Error.to_string(), "<error>");
    }
}
