//! Runtime values, ported from `values/StarkValue.java`'s sealed hierarchy.
//!
//! The Java reference models `StarkValue` as an interface with one class per
//! case (`StarkIntegerValue`, `StarkRealValue`, ...). Here the same case
//! analysis is one flat `Copy` enum, matching the arena IR's "small, `Copy`,
//! contiguous" philosophy (see `IR_LOWERING_PLAN.md`).
//!
//! Construction, `Debug`/`Display`, [Value::type_of] and the arithmetic
//! (`sum`/`product`/`is_less_than`/…) all land here, ported from
//! `StarkValue`'s static dispatch methods with Java's int-preserving-then-
//! widening promotion rules (`int ⊕ int -> Integer`, anything touching a
//! `real -> Real`). The operator *dispatch* (`ir::BinaryOp` / `MathUnaryFunction`
//! / `MathBinaryFunction` -> the right method here) lives in `eval::expr`,
//! not here, so this module never has to depend on `ir`.
//!
//! Every operation here returns [Value::Error] instead of panicking on a
//! type mismatch, mirroring `StarkValue.ERROR_VALUE`. See
//! `EVALUATOR_PLAN.md`'s "the one contract to preserve" for why, and its
//! "Deliberate deviations from the Java reference" for the handful of places
//! this intentionally does *not* match Java: integer division/modulo by zero
//! (and the `i64::MIN / -1` overflow) yield [Value::Error] rather than
//! throwing/panicking, and `==` is defined on [Value::Boolean] and
//! [Value::Custom] as well as the numeric cases — Java's `StarkValue.isEqualTo`
//! only dispatches on `StarkInteger`/`StarkReal` and silently errors on any
//! other pairing (including two equal booleans), which reads as an oversight
//! rather than a deliberate semantics, so it isn't preserved.

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

    /// Mirrors `StarkValue.isTrue`: a non-boolean value is simply *not* true
    /// (no error) — this is the "is this guard satisfied" reading used by
    /// the controller/environment stepper. Contrast with the `Select`/`if`
    /// expression, which errors on a non-boolean guard (see `eval::expr`).
    pub fn truthy(&self) -> bool {
        matches!(self, Value::Boolean(true))
    }

    /// `StarkValue.sum` / `StarkInteger.sum` / `StarkReal.sum`. Integer
    /// overflow wraps rather than panicking (Java's 32-bit `int` also wraps
    /// silently on `+`/`-`/`*`; this just does it at 64 bits).
    pub fn sum(self, other: Value) -> Value {
        numeric_op(self, other, i64::wrapping_add, |a, b| a + b)
    }

    /// `StarkValue.product`.
    pub fn product(self, other: Value) -> Value {
        numeric_op(self, other, i64::wrapping_mul, |a, b| a * b)
    }

    /// `StarkValue.subtraction`.
    pub fn subtraction(self, other: Value) -> Value {
        numeric_op(self, other, i64::wrapping_sub, |a, b| a - b)
    }

    /// `StarkValue.division`. **Deliberate deviation:** Java's `int / int`
    /// throws `ArithmeticException` on a zero divisor (the source even flags
    /// this as unresolved: `//TODO: Check how to handle division by zero!`).
    /// This must never panic, so `int / 0` (and the `i64::MIN / -1` overflow)
    /// yield [Value::Error] instead. Real division keeps `f64`'s `±inf`/`NaN`
    /// behaviour, since that never threw in Java either.
    pub fn division(self, other: Value) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a.checked_div(b).map(Value::Integer).unwrap_or(Value::Error),
            (Value::Integer(a), Value::Real(b)) => Value::Real(a as f64 / b),
            (Value::Real(a), Value::Integer(b)) => Value::Real(a / b as f64),
            (Value::Real(a), Value::Real(b)) => Value::Real(a / b),
            _ => Value::Error,
        }
    }

    /// `StarkValue.modulo`. Same zero/overflow guard as [Value::division].
    pub fn modulo(self, other: Value) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a.checked_rem(b).map(Value::Integer).unwrap_or(Value::Error),
            (Value::Integer(a), Value::Real(b)) => Value::Real(a as f64 % b),
            (Value::Real(a), Value::Integer(b)) => Value::Real(a % b as f64),
            (Value::Real(a), Value::Real(b)) => Value::Real(a % b),
            _ => Value::Error,
        }
    }

    /// Truncating integer division (`//`, `ir::BinaryOp::IntDiv`).
    ///
    /// **Not a port of Java behaviour — there isn't any to port.** The `.g4`
    /// grammar parses `//` (`mulDivExpression` accepts `'*'|'/'|'//'`), but
    /// `StarkExpressionEvaluator`'s `binaryOperators` map only registers
    /// `"+" "*" "-" "/" "%"` plus the math functions; `getBinaryOperator`
    /// falls back to `(x,y) -> ERROR_VALUE` for anything else, so every use
    /// of `//` in the Java reference evaluates to `ERROR_VALUE` unconditionally
    /// — the operator parses but was never implemented. Rather than replicate
    /// that gap, this implements the operator its syntax promises: truncating
    /// division that always yields an integral quotient — `Integer` for
    /// `int // int` (same zero/overflow guard as [Value::division]), and the
    /// real quotient truncated toward zero, as a `Real`, whenever either side
    /// is real.
    pub fn int_div(self, other: Value) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a.checked_div(b).map(Value::Integer).unwrap_or(Value::Error),
            (Value::Integer(a), Value::Real(b)) => Value::Real((a as f64 / b).trunc()),
            (Value::Real(a), Value::Integer(b)) => Value::Real((a / b as f64).trunc()),
            (Value::Real(a), Value::Real(b)) => Value::Real((a / b).trunc()),
            _ => Value::Error,
        }
    }

    /// `StarkValue.isLessThan`.
    pub fn is_less_than(self, other: Value) -> Value {
        comparison_op(self, other, |a, b| a < b, |a, b| a < b)
    }

    /// `StarkValue.isLessOrEqualThan`.
    pub fn is_less_or_equal_than(self, other: Value) -> Value {
        comparison_op(self, other, |a, b| a <= b, |a, b| a <= b)
    }

    /// `StarkValue.isGreaterOrEqualThan`.
    pub fn is_greater_or_equal_than(self, other: Value) -> Value {
        comparison_op(self, other, |a, b| a >= b, |a, b| a >= b)
    }

    /// `StarkValue.isGreaterThan`.
    pub fn is_greater_than(self, other: Value) -> Value {
        comparison_op(self, other, |a, b| a > b, |a, b| a > b)
    }

    /// `StarkValue.isEqualTo`, **extended**: Java's version dispatches only
    /// on `StarkInteger`/`StarkReal` and returns `ERROR_VALUE` for every other
    /// pairing — including two equal `StarkBoolean`s or two equal
    /// `StarkCustomValue`s, since neither class overrides it. That reads as
    /// an oversight (`.equals()` is defined and correct on both; `isEqualTo`
    /// just never calls it) rather than an intended "booleans/custom values
    /// aren't comparable" semantics, especially since `typecheck.rs` already
    /// accepts `==` between two booleans or two same-typed custom values. So
    /// this covers those cases too, numeric comparison still widening.
    pub fn is_equal_to(self, other: Value) -> Value {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a == b),
            (Value::Custom(a), Value::Custom(b)) => Value::Boolean(a == b),
            // Exact integer comparison when both sides are `Integer` (not
            // widened through `f64`, which loses precision above 2^53) —
            // matches `StarkInteger.isEqualTo`'s own `instanceof StarkInteger`
            // fast path.
            _ => comparison_op(self, other, |a, b| a == b, |a, b| a == b),
        }
    }

    /// `StarkValue.and` (`StarkBoolean.and`). Boolean-only, like Java: both
    /// `&&` and `&` (`ir::BinaryOp::And`/`BitAnd`) lower to this — the two
    /// spellings are one grammar rule split across two precedence levels in
    /// both the Java `.g4` and `stark_grammar.pest`, not two operations (see
    /// `visitAndExpression`, which ignores `ctx.op.getText()` entirely).
    pub fn and(self, other: Value) -> Value {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a && b),
            _ => Value::Error,
        }
    }

    /// `StarkValue.or` (`StarkBoolean.or`). See [Value::and]'s doc comment —
    /// `||` and `|` (`Or`/`BitOr`) are likewise one operation, two spellings.
    pub fn or(self, other: Value) -> Value {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a || b),
            _ => Value::Error,
        }
    }

    /// `StarkValue.apply(DoubleUnaryOperator, ..)`: always widens to `Real`,
    /// even for an integer argument — `max(1, 2)` is `Real(2.0)`, not
    /// `Integer(2)`. Used for every `MathUnaryFunction`.
    pub fn apply_unary(self, f: impl Fn(f64) -> f64) -> Value {
        match self {
            Value::Integer(v) => Value::Real(f(v as f64)),
            Value::Real(v) => Value::Real(f(v)),
            _ => Value::Error,
        }
    }

    /// `StarkValue.apply(DoubleBinaryOperator, ..)`: the binary counterpart
    /// of [Value::apply_unary], used for every `MathBinaryFunction`.
    pub fn apply_binary(self, other: Value, f: impl Fn(f64, f64) -> f64) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Value::Real(f(a as f64, b as f64)),
            (Value::Integer(a), Value::Real(b)) => Value::Real(f(a as f64, b)),
            (Value::Real(a), Value::Integer(b)) => Value::Real(f(a, b as f64)),
            (Value::Real(a), Value::Real(b)) => Value::Real(f(a, b)),
            _ => Value::Error,
        }
    }
}

/// The shared "int-preserving-then-widening" promotion used by `+`, `*`, `-`:
/// `int op int -> Integer`; anything touching a `Real` (or a non-numeric
/// operand) -> `Real`, or [Value::Error] if either side isn't numeric at all.
fn numeric_op(lhs: Value, rhs: Value, int_op: impl Fn(i64, i64) -> i64, real_op: impl Fn(f64, f64) -> f64) -> Value {
    match (lhs, rhs) {
        (Value::Integer(a), Value::Integer(b)) => Value::Integer(int_op(a, b)),
        (Value::Integer(a), Value::Real(b)) => Value::Real(real_op(a as f64, b)),
        (Value::Real(a), Value::Integer(b)) => Value::Real(real_op(a, b as f64)),
        (Value::Real(a), Value::Real(b)) => Value::Real(real_op(a, b)),
        _ => Value::Error,
    }
}

/// The comparison shared by `<`, `<=`, `>=`, `>`, `==`: `int_op` compares two
/// `Integer`s exactly (matching `StarkInteger`'s own `instanceof StarkInteger`
/// fast path — not widened through `f64`, which loses precision above 2^53);
/// any pairing touching a `Real` widens through `real_op` instead. Anything
/// else (including a mismatched non-numeric pairing) is [Value::Error].
fn comparison_op(
    lhs: Value,
    rhs: Value,
    int_op: impl Fn(i64, i64) -> bool,
    real_op: impl Fn(f64, f64) -> bool,
) -> Value {
    match (lhs, rhs) {
        (Value::Integer(a), Value::Integer(b)) => Value::Boolean(int_op(a, b)),
        (Value::Integer(a), Value::Real(b)) => Value::Boolean(real_op(a as f64, b)),
        (Value::Real(a), Value::Integer(b)) => Value::Boolean(real_op(a, b as f64)),
        (Value::Real(a), Value::Real(b)) => Value::Boolean(real_op(a, b)),
        _ => Value::Error,
    }
}

/// Boolean negation (`!x`), `StarkValue.negate`. An `impl` of the standard
/// trait rather than an inherent `not` method, so `!value` reads naturally
/// at call sites and doesn't collide with `std::ops::Not::not`.
impl std::ops::Not for Value {
    type Output = Value;

    fn not(self) -> Value {
        match self {
            Value::Boolean(v) => Value::Boolean(!v),
            _ => Value::Error,
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

    #[test]
    fn sum_preserves_integer_then_widens() {
        assert_eq!(Value::Integer(1).sum(Value::Integer(2)), Value::Integer(3));
        assert_eq!(Value::Integer(1).sum(Value::Real(2.0)), Value::Real(3.0));
        assert_eq!(Value::Real(1.0).sum(Value::Integer(2)), Value::Real(3.0));
        assert_eq!(Value::Real(1.0).sum(Value::Real(2.0)), Value::Real(3.0));
        assert_eq!(Value::Boolean(true).sum(Value::Integer(1)), Value::Error);
        assert_eq!(Value::Integer(1).sum(Value::Boolean(true)), Value::Error);
    }

    #[test]
    fn product_and_subtraction_promote_the_same_way() {
        assert_eq!(Value::Integer(3).product(Value::Integer(4)), Value::Integer(12));
        assert_eq!(Value::Integer(3).product(Value::Real(4.0)), Value::Real(12.0));
        assert_eq!(Value::Integer(5).subtraction(Value::Integer(2)), Value::Integer(3));
        assert_eq!(Value::Real(5.0).subtraction(Value::Integer(2)), Value::Real(3.0));
    }

    #[test]
    fn integer_division_and_modulo_by_zero_error_instead_of_panicking() {
        assert_eq!(Value::Integer(1).division(Value::Integer(0)), Value::Error);
        assert_eq!(Value::Integer(1).modulo(Value::Integer(0)), Value::Error);
        assert_eq!(Value::Integer(1).int_div(Value::Integer(0)), Value::Error);
        // The i64::MIN / -1 overflow is likewise caught, not a panic.
        assert_eq!(Value::Integer(i64::MIN).division(Value::Integer(-1)), Value::Error);
    }

    #[test]
    fn integer_division_truncates_like_java_int_division() {
        assert_eq!(Value::Integer(7).division(Value::Integer(2)), Value::Integer(3));
        assert_eq!(Value::Integer(-7).division(Value::Integer(2)), Value::Integer(-3));
    }

    #[test]
    fn real_division_by_zero_keeps_f64_infinities() {
        assert_eq!(Value::Real(1.0).division(Value::Real(0.0)), Value::Real(f64::INFINITY));
        assert!(matches!(
            Value::Real(0.0).division(Value::Real(0.0)),
            Value::Real(v) if v.is_nan()
        ));
    }

    #[test]
    fn int_div_always_truncates_towards_zero() {
        assert_eq!(Value::Integer(7).int_div(Value::Integer(2)), Value::Integer(3));
        assert_eq!(Value::Real(7.5).int_div(Value::Integer(2)), Value::Real(3.0));
        assert_eq!(Value::Real(-7.5).int_div(Value::Integer(2)), Value::Real(-3.0));
    }

    #[test]
    fn comparisons_are_numeric_only_and_widen() {
        assert_eq!(Value::Integer(1).is_less_than(Value::Integer(2)), Value::Boolean(true));
        assert_eq!(Value::Integer(2).is_less_than(Value::Real(2.5)), Value::Boolean(true));
        assert_eq!(Value::Boolean(true).is_less_than(Value::Integer(1)), Value::Error);
    }

    #[test]
    fn equality_covers_numeric_boolean_and_custom() {
        assert_eq!(Value::Integer(2).is_equal_to(Value::Real(2.0)), Value::Boolean(true));
        assert_eq!(
            Value::Boolean(true).is_equal_to(Value::Boolean(true)),
            Value::Boolean(true)
        );
        assert_eq!(
            Value::Boolean(true).is_equal_to(Value::Boolean(false)),
            Value::Boolean(false)
        );
        let a = Value::Custom(CustomValue {
            type_id: DefId::new(0),
            element: 1,
        });
        let b = Value::Custom(CustomValue {
            type_id: DefId::new(0),
            element: 1,
        });
        let c = Value::Custom(CustomValue {
            type_id: DefId::new(0),
            element: 2,
        });
        assert_eq!(a.is_equal_to(b), Value::Boolean(true));
        assert_eq!(a.is_equal_to(c), Value::Boolean(false));
        assert_eq!(Value::Integer(1).is_equal_to(Value::Boolean(true)), Value::Error);
    }

    #[test]
    fn equality_compares_large_integers_exactly() {
        // 2^53 + 1 and 2^53 + 2 collapse to the same f64, so this would
        // wrongly compare equal if `is_equal_to` widened through `f64`.
        let a = (1i64 << 53) + 1;
        let b = (1i64 << 53) + 2;
        assert_eq!(Value::Integer(a).is_equal_to(Value::Integer(b)), Value::Boolean(false));
    }

    #[test]
    fn and_or_are_boolean_only() {
        assert_eq!(Value::Boolean(true).and(Value::Boolean(false)), Value::Boolean(false));
        assert_eq!(Value::Boolean(true).or(Value::Boolean(false)), Value::Boolean(true));
        assert_eq!(Value::Integer(1).and(Value::Boolean(true)), Value::Error);
    }

    #[test]
    fn boolean_not_is_distinct_from_arithmetic_negation() {
        // `!` (boolean) is `std::ops::Not`; arithmetic `-x`/`+x`
        // (`ExprNode::Negate`/`Widen`) go through `apply_unary` instead —
        // see those two variants' doc comments in `ir.rs`.
        assert_eq!(!Value::Boolean(true), Value::Boolean(false));
        assert_eq!(!Value::Integer(1), Value::Error);
    }

    #[test]
    fn arithmetic_negate_and_widen_always_widen_to_real() {
        // `-x`/`+x` are *not* integer-preserving, matching Java's
        // `unaryOperators` map, which routes both through the same
        // always-widening `DoubleUnaryOperator` mechanism as the math
        // functions — see `ExprNode::Negate`'s doc comment.
        assert_eq!(Value::Integer(3).apply_unary(|x| -x), Value::Real(-3.0));
        assert_eq!(Value::Real(3.0).apply_unary(|x| -x), Value::Real(-3.0));
        assert_eq!(Value::Boolean(true).apply_unary(|x| -x), Value::Error);
        assert_eq!(Value::Integer(3).apply_unary(|x| x), Value::Real(3.0));
    }

    #[test]
    fn math_functions_always_widen_to_real() {
        // The pinning test from `EVALUATOR_PLAN.md`: `max(1, 2)` is `Real(2.0)`,
        // not `Integer(2)`, since `StarkInteger.apply(DoubleBinaryOperator)`
        // always returns a `StarkReal`.
        assert_eq!(
            Value::Integer(1).apply_binary(Value::Integer(2), f64::max),
            Value::Real(2.0)
        );
        assert_eq!(Value::Integer(4).apply_unary(f64::sqrt), Value::Real(2.0));
    }

    #[test]
    fn truthy_is_false_not_error_for_non_boolean() {
        assert!(Value::Boolean(true).truthy());
        assert!(!Value::Boolean(false).truthy());
        assert!(!Value::Integer(1).truthy());
        assert!(!Value::Error.truthy());
    }
}
