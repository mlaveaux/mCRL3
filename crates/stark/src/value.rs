use std::fmt;

use thiserror::Error;

use crate::ast::DefId;
use crate::resolve::SymbolTable;
use crate::types::StarkType;

/// Error when evaluating an expression.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EvalError {
    /// Integer `/`, `//` or `%` with a zero divisor, or the `i64::MIN / -1`
    /// overflow. The only variant reachable from a well-typed program.
    #[error("division by zero")]
    DivisionByZero,
    /// A binary operator applied to operands it isn't defined for — e.g. `true + 1`.
    #[error("operator `{op}` is not defined for {left} and {right}")]
    UnsupportedBinaryOperands {
        op: &'static str,
        left: ValueKind,
        right: ValueKind,
    },
    /// A unary operator applied to an operand it isn't defined for — e.g. `!1`.
    #[error("operator `{op}` is not defined for {operand}")]
    UnsupportedUnaryOperand { op: &'static str, operand: ValueKind },
    /// A guard or condition that didn't evaluate to a boolean.
    #[error("{context} must be a boolean, but was {found}")]
    ExpectedBoolean { context: &'static str, found: ValueKind },
    /// A `step` count that didn't evaluate to an integer.
    #[error("{context} must be an integer, but was {found}")]
    ExpectedInteger { context: &'static str, found: ValueKind },
    /// A sampling bound (`R[a,b]`, `N[m,v]`) that didn't evaluate to a number.
    #[error("{context} must be a number, but was {found}")]
    ExpectedNumber { context: &'static str, found: ValueKind },
    /// A function body fell off the end without reaching a `return`. Should
    /// never occur and be handled by the typechecker.
    #[error("function body reached no `return` on this path")]
    MissingReturn,
    /// An `ir::ExprNode::Unreachable` was evaluated. A bug in the lowering.
    #[error("evaluated an expression that should be unreachable: {0}")]
    Unreachable(&'static str),
    /// A distance was computed between two sample sets whose sizes aren't a
    /// multiple of one another — `SampleSet.distance` throws
    /// `IllegalArgumentException("Incompatible size of data sets!")` here.
    /// Only reachable by asking for a perturbed sequence with a zero
    /// `scale`, since a perturbed sequence is otherwise `scale` replicas of
    /// the reference one.
    #[error("cannot compare sample sets of size {reference} and {perturbed}: the latter must be a multiple of the former")]
    IncompatibleSampleSizes { reference: usize, perturbed: usize },
    /// A robustness analysis was asked for with a zero sample size, so there
    /// is no distribution to compute a distance over.
    #[error("a robustness analysis needs at least one sample per step")]
    EmptySampleSet,
}

/// Which [Value] case a value was, without its payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Integer,
    Real,
    Boolean,
    Custom,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Integer => write!(f, "an integer"),
            ValueKind::Real => write!(f, "a real"),
            ValueKind::Boolean => write!(f, "a boolean"),
            ValueKind::Custom => write!(f, "a custom value"),
        }
    }
}

/// An instance of a user-defined `type X = A | B | C;` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustomValue {
    /// The `DefId` of the owning `type X = ...;` declaration.
    pub type_id: DefId,
    /// The index of the declared element.
    pub element: u32,
}

/// A runtime value flowing through the evaluator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Custom(CustomValue),
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
        }
    }

    /// This value's case, for error reporting — see [ValueKind].
    pub fn kind(self) -> ValueKind {
        match self {
            Value::Integer(_) => ValueKind::Integer,
            Value::Real(_) => ValueKind::Real,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Custom(_) => ValueKind::Custom,
        }
    }

    /// Reads this value as a boolean.
    pub fn as_boolean(self, context: &'static str) -> Result<bool, EvalError> {
        match self {
            Value::Boolean(value) => Ok(value),
            other => Err(EvalError::ExpectedBoolean {
                context,
                found: other.kind(),
            }),
        }
    }

    /// Reads this value as an integer.
    pub fn as_integer(self, context: &'static str) -> Result<i64, EvalError> {
        match self {
            Value::Integer(value) => Ok(value),
            other => Err(EvalError::ExpectedInteger {
                context,
                found: other.kind(),
            }),
        }
    }

    /// Widens either numeric case to `f64`. Errors on a non-numeric values..
    pub fn as_number(self, context: &'static str) -> Result<f64, EvalError> {
        match self {
            Value::Integer(value) => Ok(value as f64),
            Value::Real(value) => Ok(value),
            other => Err(EvalError::ExpectedNumber {
                context,
                found: other.kind(),
            }),
        }
    }

    /// Integer overflow wraps rather than panicking.
    pub fn sum(self, other: Value) -> Result<Value, EvalError> {
        numeric_op("+", self, other, i64::wrapping_add, |a, b| a + b)
    }

    pub fn product(self, other: Value) -> Result<Value, EvalError> {
        numeric_op("*", self, other, i64::wrapping_mul, |a, b| a * b)
    }

    pub fn subtraction(self, other: Value) -> Result<Value, EvalError> {
        numeric_op("-", self, other, i64::wrapping_sub, |a, b| a - b)
    }

    /// `StarkValue.division`. Integer division by zero (and the
    /// `i64::MIN / -1` overflow) is [EvalError::DivisionByZero]; real division
    /// keeps `f64`'s `±inf`/`NaN` behaviour — see the module doc comment.
    pub fn division(self, other: Value) -> Result<Value, EvalError> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                a.checked_div(b).map(Value::Integer).ok_or(EvalError::DivisionByZero)
            }
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(a as f64 / b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a / b as f64)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a / b)),
            (left, right) => Err(unsupported("/", left, right)),
        }
    }

    /// `StarkValue.modulo`. Same zero/overflow guard as [Value::division].
    pub fn modulo(self, other: Value) -> Result<Value, EvalError> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                a.checked_rem(b).map(Value::Integer).ok_or(EvalError::DivisionByZero)
            }
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(a as f64 % b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a % b as f64)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a % b)),
            (left, right) => Err(unsupported("%", left, right)),
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
    pub fn int_div(self, other: Value) -> Result<Value, EvalError> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                a.checked_div(b).map(Value::Integer).ok_or(EvalError::DivisionByZero)
            }
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real((a as f64 / b).trunc())),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real((a / b as f64).trunc())),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real((a / b).trunc())),
            (left, right) => Err(unsupported("//", left, right)),
        }
    }

    /// `StarkValue.isLessThan`. Returns a bare `bool` rather than a
    /// [Value::Boolean]: now that the failure case is an `Err`, the success
    /// case is known to be a boolean, and saying so in the type keeps a caller
    /// from having to re-inspect it. `eval::expr` wraps it back into a [Value].
    pub fn is_less_than(self, other: Value) -> Result<bool, EvalError> {
        comparison_op("<", self, other, |a, b| a < b, |a, b| a < b)
    }

    /// `StarkValue.isLessOrEqualThan`.
    pub fn is_less_or_equal_than(self, other: Value) -> Result<bool, EvalError> {
        comparison_op("<=", self, other, |a, b| a <= b, |a, b| a <= b)
    }

    /// `StarkValue.isGreaterOrEqualThan`.
    pub fn is_greater_or_equal_than(self, other: Value) -> Result<bool, EvalError> {
        comparison_op(">=", self, other, |a, b| a >= b, |a, b| a >= b)
    }

    /// `StarkValue.isGreaterThan`.
    pub fn is_greater_than(self, other: Value) -> Result<bool, EvalError> {
        comparison_op(">", self, other, |a, b| a > b, |a, b| a > b)
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
    pub fn is_equal_to(self, other: Value) -> Result<bool, EvalError> {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(a == b),
            (Value::Custom(a), Value::Custom(b)) => Ok(a == b),
            // Exact integer comparison when both sides are `Integer` (not
            // widened through `f64`, which loses precision above 2^53) —
            // matches `StarkInteger.isEqualTo`'s own `instanceof StarkInteger`
            // fast path.
            _ => comparison_op("==", self, other, |a, b| a == b, |a, b| a == b),
        }
    }

    /// `StarkValue.and` (`StarkBoolean.and`). Boolean-only, like Java: both
    /// `&&` and `&` (`ir::BinaryOp::And`/`BitAnd`) lower to this — the two
    /// spellings are one grammar rule split across two precedence levels in
    /// both the Java `.g4` and `stark_grammar.pest`, not two operations (see
    /// `visitAndExpression`, which ignores `ctx.op.getText()` entirely).
    pub fn and(self, other: Value) -> Result<bool, EvalError> {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(a && b),
            (left, right) => Err(unsupported("&&", left, right)),
        }
    }

    /// `StarkValue.or` (`StarkBoolean.or`). See [Value::and]'s doc comment —
    /// `||` and `|` (`Or`/`BitOr`) are likewise one operation, two spellings.
    pub fn or(self, other: Value) -> Result<bool, EvalError> {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(a || b),
            (left, right) => Err(unsupported("||", left, right)),
        }
    }

    /// `StarkValue.apply(DoubleUnaryOperator, ..)`: always widens to `Real`,
    /// even for an integer argument — `max(1, 2)` is `Real(2.0)`, not
    /// `Integer(2)`. Used for every `MathUnaryFunction`. `op` names the
    /// operation for the error message.
    pub fn apply_unary(self, op: &'static str, f: impl Fn(f64) -> f64) -> Result<Value, EvalError> {
        match self {
            Value::Integer(v) => Ok(Value::Real(f(v as f64))),
            Value::Real(v) => Ok(Value::Real(f(v))),
            operand => Err(EvalError::UnsupportedUnaryOperand {
                op,
                operand: operand.kind(),
            }),
        }
    }

    /// `StarkValue.apply(DoubleBinaryOperator, ..)`: the binary counterpart
    /// of [Value::apply_unary], used for every `MathBinaryFunction`.
    pub fn apply_binary(self, other: Value, op: &'static str, f: impl Fn(f64, f64) -> f64) -> Result<Value, EvalError> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Real(f(a as f64, b as f64))),
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(f(a as f64, b))),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(f(a, b as f64))),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(f(a, b))),
            (left, right) => Err(unsupported(op, left, right)),
        }
    }
}

/// The [EvalError::UnsupportedBinaryOperands] for a binary `op` — the
/// fallthrough every operation below shares.
fn unsupported(op: &'static str, left: Value, right: Value) -> EvalError {
    EvalError::UnsupportedBinaryOperands {
        op,
        left: left.kind(),
        right: right.kind(),
    }
}

/// The shared "int-preserving-then-widening" promotion used by `+`, `*`, `-`:
/// `int op int -> Integer`; anything touching a `Real` -> `Real`; anything with
/// a non-numeric operand is an error.
fn numeric_op(
    op: &'static str,
    lhs: Value,
    rhs: Value,
    int_op: impl Fn(i64, i64) -> i64,
    real_op: impl Fn(f64, f64) -> f64,
) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(int_op(a, b))),
        (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(real_op(a as f64, b))),
        (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(real_op(a, b as f64))),
        (Value::Real(a), Value::Real(b)) => Ok(Value::Real(real_op(a, b))),
        (left, right) => Err(unsupported(op, left, right)),
    }
}

/// The comparison shared by `<`, `<=`, `>=`, `>`, `==`: `int_op` compares two
/// `Integer`s exactly (matching `StarkInteger`'s own `instanceof StarkInteger`
/// fast path — not widened through `f64`, which loses precision above 2^53);
/// any pairing touching a `Real` widens through `real_op` instead. Anything
/// else (including a mismatched non-numeric pairing) is an error.
fn comparison_op(
    op: &'static str,
    lhs: Value,
    rhs: Value,
    int_op: impl Fn(i64, i64) -> bool,
    real_op: impl Fn(f64, f64) -> bool,
) -> Result<bool, EvalError> {
    match (lhs, rhs) {
        (Value::Integer(a), Value::Integer(b)) => Ok(int_op(a, b)),
        (Value::Integer(a), Value::Real(b)) => Ok(real_op(a as f64, b)),
        (Value::Real(a), Value::Integer(b)) => Ok(real_op(a, b as f64)),
        (Value::Real(a), Value::Real(b)) => Ok(real_op(a, b)),
        (left, right) => Err(unsupported(op, left, right)),
    }
}

/// Boolean negation (`!x`), `StarkValue.negate`. Still an `impl` of the
/// standard trait rather than an inherent method so `!value` reads naturally
/// at call sites; the `Output` is a `Result<bool, _>` like every other
/// operation here, so a call site spells it `(!value)?`.
impl std::ops::Not for Value {
    type Output = Result<bool, EvalError>;

    fn not(self) -> Result<bool, EvalError> {
        match self {
            Value::Boolean(v) => Ok(!v),
            operand => Err(EvalError::UnsupportedUnaryOperand {
                op: "!",
                operand: operand.kind(),
            }),
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
    }

    #[test]
    fn display_formats_plain_values() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Real(1.5).to_string(), "1.5");
        assert_eq!(Value::Boolean(false).to_string(), "false");
    }

    #[test]
    fn sum_preserves_integer_then_widens() {
        assert_eq!(Value::Integer(1).sum(Value::Integer(2)), Ok(Value::Integer(3)));
        assert_eq!(Value::Integer(1).sum(Value::Real(2.0)), Ok(Value::Real(3.0)));
        assert_eq!(Value::Real(1.0).sum(Value::Integer(2)), Ok(Value::Real(3.0)));
        assert_eq!(Value::Real(1.0).sum(Value::Real(2.0)), Ok(Value::Real(3.0)));
    }

    #[test]
    fn arithmetic_on_a_non_numeric_operand_names_both_sides() {
        assert_eq!(
            Value::Boolean(true).sum(Value::Integer(1)),
            Err(EvalError::UnsupportedBinaryOperands {
                op: "+",
                left: ValueKind::Boolean,
                right: ValueKind::Integer,
            })
        );
        assert_eq!(
            Value::Integer(1).sum(Value::Boolean(true)).unwrap_err().to_string(),
            "operator `+` is not defined for an integer and a boolean"
        );
    }

    #[test]
    fn product_and_subtraction_promote_the_same_way() {
        assert_eq!(Value::Integer(3).product(Value::Integer(4)), Ok(Value::Integer(12)));
        assert_eq!(Value::Integer(3).product(Value::Real(4.0)), Ok(Value::Real(12.0)));
        assert_eq!(Value::Integer(5).subtraction(Value::Integer(2)), Ok(Value::Integer(3)));
        assert_eq!(Value::Real(5.0).subtraction(Value::Integer(2)), Ok(Value::Real(3.0)));
    }

    #[test]
    fn integer_division_and_modulo_by_zero_error_instead_of_panicking() {
        assert_eq!(
            Value::Integer(1).division(Value::Integer(0)),
            Err(EvalError::DivisionByZero)
        );
        assert_eq!(
            Value::Integer(1).modulo(Value::Integer(0)),
            Err(EvalError::DivisionByZero)
        );
        assert_eq!(
            Value::Integer(1).int_div(Value::Integer(0)),
            Err(EvalError::DivisionByZero)
        );
        // The i64::MIN / -1 overflow is likewise caught, not a panic.
        assert_eq!(
            Value::Integer(i64::MIN).division(Value::Integer(-1)),
            Err(EvalError::DivisionByZero)
        );
    }

    #[test]
    fn integer_division_truncates_like_java_int_division() {
        assert_eq!(Value::Integer(7).division(Value::Integer(2)), Ok(Value::Integer(3)));
        assert_eq!(Value::Integer(-7).division(Value::Integer(2)), Ok(Value::Integer(-3)));
    }

    #[test]
    fn real_division_by_zero_keeps_f64_infinities() {
        assert_eq!(
            Value::Real(1.0).division(Value::Real(0.0)),
            Ok(Value::Real(f64::INFINITY))
        );
        assert!(matches!(
            Value::Real(0.0).division(Value::Real(0.0)),
            Ok(Value::Real(v)) if v.is_nan()
        ));
    }

    #[test]
    fn int_div_always_truncates_towards_zero() {
        assert_eq!(Value::Integer(7).int_div(Value::Integer(2)), Ok(Value::Integer(3)));
        assert_eq!(Value::Real(7.5).int_div(Value::Integer(2)), Ok(Value::Real(3.0)));
        assert_eq!(Value::Real(-7.5).int_div(Value::Integer(2)), Ok(Value::Real(-3.0)));
    }

    #[test]
    fn comparisons_are_numeric_only_and_widen() {
        assert_eq!(Value::Integer(1).is_less_than(Value::Integer(2)), Ok(true));
        assert_eq!(Value::Integer(2).is_less_than(Value::Real(2.5)), Ok(true));
        assert_eq!(
            Value::Boolean(true).is_less_than(Value::Integer(1)),
            Err(unsupported("<", Value::Boolean(true), Value::Integer(1)))
        );
    }

    #[test]
    fn equality_covers_numeric_boolean_and_custom() {
        assert_eq!(Value::Integer(2).is_equal_to(Value::Real(2.0)), Ok(true));
        assert_eq!(Value::Boolean(true).is_equal_to(Value::Boolean(true)), Ok(true));
        assert_eq!(Value::Boolean(true).is_equal_to(Value::Boolean(false)), Ok(false));
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
        assert_eq!(a.is_equal_to(b), Ok(true));
        assert_eq!(a.is_equal_to(c), Ok(false));
        assert!(Value::Integer(1).is_equal_to(Value::Boolean(true)).is_err());
    }

    #[test]
    fn equality_compares_large_integers_exactly() {
        // 2^53 + 1 and 2^53 + 2 collapse to the same f64, so this would
        // wrongly compare equal if `is_equal_to` widened through `f64`.
        let a = (1i64 << 53) + 1;
        let b = (1i64 << 53) + 2;
        assert_eq!(Value::Integer(a).is_equal_to(Value::Integer(b)), Ok(false));
    }

    #[test]
    fn and_or_are_boolean_only() {
        assert_eq!(Value::Boolean(true).and(Value::Boolean(false)), Ok(false));
        assert_eq!(Value::Boolean(true).or(Value::Boolean(false)), Ok(true));
        assert!(Value::Integer(1).and(Value::Boolean(true)).is_err());
    }

    #[test]
    fn boolean_not_is_distinct_from_arithmetic_negation() {
        // `!` (boolean) is `std::ops::Not`; arithmetic `-x`/`+x`
        // (`ExprNode::Negate`/`Widen`) go through `apply_unary` instead —
        // see those two variants' doc comments in `ir.rs`.
        assert_eq!(!Value::Boolean(true), Ok(false));
        assert!((!Value::Integer(1)).is_err());
    }

    #[test]
    fn arithmetic_negate_and_widen_always_widen_to_real() {
        // `-x`/`+x` are *not* integer-preserving, matching Java's
        // `unaryOperators` map, which routes both through the same
        // always-widening `DoubleUnaryOperator` mechanism as the math
        // functions — see `ExprNode::Negate`'s doc comment.
        assert_eq!(Value::Integer(3).apply_unary("-", |x| -x), Ok(Value::Real(-3.0)));
        assert_eq!(Value::Real(3.0).apply_unary("-", |x| -x), Ok(Value::Real(-3.0)));
        assert!(Value::Boolean(true).apply_unary("-", |x| -x).is_err());
        assert_eq!(Value::Integer(3).apply_unary("+", |x| x), Ok(Value::Real(3.0)));
    }

    #[test]
    fn math_functions_always_widen_to_real() {
        // The pinning test from `EVALUATOR_PLAN.md`: `max(1, 2)` is `Real(2.0)`,
        // not `Integer(2)`, since `StarkInteger.apply(DoubleBinaryOperator)`
        // always returns a `StarkReal`.
        assert_eq!(
            Value::Integer(1).apply_binary(Value::Integer(2), "max", f64::max),
            Ok(Value::Real(2.0))
        );
        assert_eq!(Value::Integer(4).apply_unary("sqrt", f64::sqrt), Ok(Value::Real(2.0)));
    }

    #[test]
    fn as_boolean_errors_on_a_non_boolean_guard() {
        // The behaviour change from `StarkValue.isTrue`, which silently
        // answered `false` here — see the module doc comment.
        assert_eq!(Value::Boolean(true).as_boolean("a guard"), Ok(true));
        assert_eq!(Value::Boolean(false).as_boolean("a guard"), Ok(false));
        assert_eq!(
            Value::Integer(1).as_boolean("a guard"),
            Err(EvalError::ExpectedBoolean {
                context: "a guard",
                found: ValueKind::Integer,
            })
        );
    }

    #[test]
    fn as_number_widens_either_numeric_case() {
        assert_eq!(Value::Integer(3).as_number("a bound"), Ok(3.0));
        assert_eq!(Value::Real(3.5).as_number("a bound"), Ok(3.5));
        assert!(Value::Boolean(true).as_number("a bound").is_err());
    }
}
