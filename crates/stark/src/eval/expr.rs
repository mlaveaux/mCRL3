//! Expression and function-body evaluation over [IrProgram]'s arena, ported
//! from `StarkExpressionEvaluator.java`'s case analysis — but as a straight
//! post-order walk of `ExprRef`/`StmtRef` indices instead of a tree of
//! `Supplier`/lambda closures, since lowering already collapsed the AST into
//! that arena (see `IR_LOWERING_PLAN.md`).
//!
//! Every function here returns a [Value] and never panics: a malformed
//! runtime state (which shouldn't arise against a checked + lowered
//! [IrProgram]) yields [Value::Error], mirroring `StarkValue.ERROR_VALUE` —
//! see `EVALUATOR_PLAN.md`'s "the one contract to preserve".

use rand::Rng;
use rand::RngExt;

use crate::ir::BinaryOp;
use crate::ir::ExprNode;
use crate::ir::ExprRef;
use crate::ir::IrProgram;
use crate::ir::MathBinaryFunction;
use crate::ir::MathUnaryFunction;
use crate::ir::StmtNode;
use crate::ir::StmtRef;
use crate::value::Value;

use super::store::Store;

/// Evaluates one expression against `store`, sampling from `rng` wherever
/// the expression does.
pub(crate) fn eval<R: Rng + ?Sized>(program: &IrProgram, store: &mut Store, rng: &mut R, id: ExprRef) -> Value {
    match *program.expr(id) {
        ExprNode::Literal(value) => value,
        ExprNode::Load(slot) => store.load(slot),
        ExprNode::Not(inner) => !eval(program, store, rng, inner),
        // Both always widen to `Real`, matching Java — see `ExprNode::Negate`
        // and `ExprNode::Widen`'s doc comments in `ir.rs`.
        ExprNode::Negate(inner) => eval(program, store, rng, inner).apply_unary(|x| -x),
        ExprNode::Widen(inner) => eval(program, store, rng, inner).apply_unary(|x| x),
        ExprNode::Binary(op, left, right) => {
            let left = eval(program, store, rng, left);
            let right = eval(program, store, rng, right);
            apply_binary_op(op, left, right)
        }
        ExprNode::MathUnary(function, inner) => {
            let value = eval(program, store, rng, inner);
            value.apply_unary(math_unary_fn(function))
        }
        ExprNode::MathBinary(function, left, right) => {
            let left = eval(program, store, rng, left);
            let right = eval(program, store, rng, right);
            left.apply_binary(right, math_binary_fn(function))
        }
        ExprNode::Select {
            guard,
            then_branch,
            else_branch,
        } => {
            // Lazy, matching `StarkValue.ifThenElse`'s `Supplier`-based
            // laziness in the Java reference: only the taken branch is
            // evaluated, since the untaken one may sample (advancing `rng`)
            // or divide by zero.
            match eval(program, store, rng, guard) {
                Value::Boolean(true) => eval(program, store, rng, then_branch),
                Value::Boolean(false) => eval(program, store, rng, else_branch),
                _ => Value::Error,
            }
        }
        ExprNode::Call { function, arguments } => {
            let function_ir = program.function(function);
            // Evaluate every argument against the *caller's* slots first...
            let mut values = Vec::with_capacity(function_ir.arguments.len());
            for &argument in program.expr_list(arguments) {
                values.push(eval(program, store, rng, argument));
            }
            // ...then write them into the callee's fixed argument slots.
            // No frame save/restore: `resolve.rs` forbids recursion, so
            // every function's argument/`let` slots are disjoint from every
            // other function's and no function is ever live twice at once
            // (see `IR_LOWERING_PLAN.md`, "Why one flat slot space works").
            for (&slot, value) in function_ir.arguments.iter().zip(values) {
                store.set(slot, value);
            }
            eval_stmt(program, store, rng, function_ir.body)
        }
        ExprNode::SampleUnit => Value::Real(rng.random::<f64>()),
        ExprNode::SampleRange { min, max } => {
            let min = eval(program, store, rng, min);
            let max = eval(program, store, rng, max);
            sample_range(rng, min, max)
        }
        ExprNode::SampleNormal { mean, variance } => {
            let mean = eval(program, store, rng, mean);
            let variance = eval(program, store, rng, variance);
            sample_normal(rng, mean, variance)
        }
        ExprNode::SampleChoice(list) => {
            let elements = program.expr_list(list);
            // Lazy like `Select`: `visitUniformExpression` indexes
            // `elements[selected]` and evaluates only that one element.
            let selected = rng.random_range(0..elements.len());
            eval(program, store, rng, elements[selected])
        }
    }
}

/// Evaluates a function body statement, returning the value of whichever
/// `Return` is reached.
pub(crate) fn eval_stmt<R: Rng + ?Sized>(program: &IrProgram, store: &mut Store, rng: &mut R, id: StmtRef) -> Value {
    match *program.stmt(id) {
        StmtNode::Return(value) => eval(program, store, rng, value),
        StmtNode::IfThenElse {
            guard,
            then_branch,
            else_branch,
        } => match eval(program, store, rng, guard) {
            Value::Boolean(true) => eval_stmt(program, store, rng, then_branch),
            Value::Boolean(false) => match else_branch {
                Some(else_branch) => eval_stmt(program, store, rng, else_branch),
                // `typecheck.rs` requires a function to return on every
                // path, so a false guard with no `else` is unreachable
                // against a checked program.
                None => {
                    debug_assert!(false, "function body has no return on this path");
                    Value::Error
                }
            },
            _ => Value::Error,
        },
        StmtNode::Let { slot, value, body } => {
            let value = eval(program, store, rng, value);
            store.set(slot, value);
            eval_stmt(program, store, rng, body)
        }
    }
}

fn apply_binary_op(op: BinaryOp, left: Value, right: Value) -> Value {
    match op {
        BinaryOp::Add => left.sum(right),
        BinaryOp::Subtract => left.subtraction(right),
        BinaryOp::Mult => left.product(right),
        BinaryOp::Div => left.division(right),
        BinaryOp::IntDiv => left.int_div(right),
        BinaryOp::Mod => left.modulo(right),
        BinaryOp::Less => left.is_less_than(right),
        BinaryOp::Leq => left.is_less_or_equal_than(right),
        BinaryOp::Eq => left.is_equal_to(right),
        BinaryOp::Geq => left.is_greater_or_equal_than(right),
        BinaryOp::Greater => left.is_greater_than(right),
        // `&&`/`&` and `||`/`|` are one operation each, two spellings — see
        // `Value::and`/`Value::or`'s doc comments.
        BinaryOp::And | BinaryOp::BitAnd => left.and(right),
        BinaryOp::Or | BinaryOp::BitOr => left.or(right),
    }
}

fn math_unary_fn(function: MathUnaryFunction) -> fn(f64) -> f64 {
    match function {
        MathUnaryFunction::Abs => f64::abs,
        MathUnaryFunction::Acos => f64::acos,
        MathUnaryFunction::Asin => f64::asin,
        MathUnaryFunction::Atan => f64::atan,
        MathUnaryFunction::Cbrt => f64::cbrt,
        MathUnaryFunction::Ceil => f64::ceil,
        MathUnaryFunction::Cos => f64::cos,
        MathUnaryFunction::Cosh => f64::cosh,
        MathUnaryFunction::Exp => f64::exp,
        MathUnaryFunction::Expm1 => f64::exp_m1,
        MathUnaryFunction::Floor => f64::floor,
        MathUnaryFunction::Log => f64::ln,
        MathUnaryFunction::Log10 => f64::log10,
        MathUnaryFunction::Log1p => f64::ln_1p,
        MathUnaryFunction::Signum => java_signum,
        MathUnaryFunction::Sin => f64::sin,
        MathUnaryFunction::Sinh => f64::sinh,
        MathUnaryFunction::Sqrt => f64::sqrt,
        MathUnaryFunction::Tan => f64::tan,
    }
}

fn math_binary_fn(function: MathBinaryFunction) -> fn(f64, f64) -> f64 {
    match function {
        MathBinaryFunction::Atan2 => f64::atan2,
        MathBinaryFunction::Hypot => f64::hypot,
        MathBinaryFunction::Max => java_max,
        MathBinaryFunction::Min => java_min,
        MathBinaryFunction::Pow => f64::powf,
    }
}

/// `Math.signum`: unlike [f64::signum] (which returns `±1.0` for `±0.0` and
/// never `0.0`), Java's version returns the zero itself (`0.0` or `-0.0`)
/// unchanged, and propagates `NaN`.
fn java_signum(x: f64) -> f64 {
    if x == 0.0 || x.is_nan() { x } else { x.signum() }
}

/// `Math.max`: propagates `NaN` if *either* argument is `NaN`. [f64::max]
/// instead returns the non-`NaN` argument, so it can't be used directly.
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.max(b) }
}

/// `Math.min`, see [java_max].
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.min(b) }
}

/// `StarkValue.sample`: `from + rng.nextDouble() * (to - from)`.
fn sample_range<R: Rng + ?Sized>(rng: &mut R, min: Value, max: Value) -> Value {
    match (double_of(min), double_of(max)) {
        (Some(from), Some(to)) => Value::Real(from + rng.random::<f64>() * (to - from)),
        _ => Value::Error,
    }
}

/// `StarkValue.sampleNormal`. **Not actually Gaussian** — despite the name
/// and the `N[mean, variance]` syntax, the Java reference computes
/// `rng.nextDouble() * mean + variance` (a scaled-and-shifted uniform
/// sample), not a normal distribution. This is ported *exactly*, not
/// "fixed", so behaviour matches the reference tool; it reads as a bug in
/// `StarkValue.sampleNormal`, but is not this port's place to silently
/// correct.
fn sample_normal<R: Rng + ?Sized>(rng: &mut R, mean: Value, variance: Value) -> Value {
    match (double_of(mean), double_of(variance)) {
        (Some(mean), Some(variance)) => Value::Real(rng.random::<f64>() * mean + variance),
        _ => Value::Error,
    }
}

/// `StarkValue.doubleOf`, except a non-numeric value maps to `None` (evaluated
/// as [Value::Error] at the call site) rather than `Double.NaN` — every call
/// site here is already guaranteed numeric by `typecheck.rs` (`R[a,b]`'s
/// bounds and `N[m,v]`'s mean/variance are both checked against `real`), so
/// this only matters for an otherwise-unreachable malformed IR.
fn double_of(value: Value) -> Option<f64> {
    match value {
        Value::Integer(v) => Some(v as f64),
        Value::Real(v) => Some(v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use test_case::test_case;
    use test_log::test;

    use super::*;
    use crate::UntypedStarkSpecification;
    use crate::lower;

    fn eval_expression(source: &str) -> Value {
        let full_source = format!("const result = {source};");
        let spec = UntypedStarkSpecification::parse(&full_source)
            .expect("should parse")
            .check()
            .expect("should check");
        let program = lower(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);
        let store = Store::new(&program, &mut rng);
        store.load(program.globals()[0].slot)
    }

    #[test_case("1 + 2", Value::Integer(3) ; "integer addition stays integer")]
    #[test_case("1 + 2.0", Value::Real(3.0) ; "integer plus real widens")]
    #[test_case("7 / 2", Value::Integer(3) ; "integer division truncates")]
    #[test_case("7 % 2", Value::Integer(1) ; "integer modulo")]
    #[test_case("max(1, 2)", Value::Real(2.0) ; "math functions always widen to real")]
    #[test_case("true && false", Value::Boolean(false) ; "double ampersand and")]
    #[test_case("true & false", Value::Boolean(false) ; "single ampersand and")]
    #[test_case("!true", Value::Boolean(false) ; "boolean not")]
    #[test_case("-3", Value::Real(-3.0) ; "arithmetic negate widens to real")]
    #[test_case("+3", Value::Real(3.0) ; "unary plus widens to real")]
    #[test_case("2 < 3", Value::Boolean(true) ; "less than")]
    #[test_case("2 == 2.0", Value::Boolean(true) ; "equality widens")]
    #[test_case("2 < 3 ? 10 : 20", Value::Integer(10) ; "select ternary")]
    fn evaluates_literal_expressions(source: &str, expected: Value) {
        assert_eq!(eval_expression(source), expected);
    }

    #[test]
    fn select_only_evaluates_the_taken_branch() {
        // The untaken branch divides by zero; if `Select` weren't lazy this
        // would produce `Value::Error` instead of `Value::Integer(1)`.
        assert_eq!(eval_expression("true ? 1 : 1/0"), Value::Integer(1));
        assert_eq!(eval_expression("false ? 1/0 : 1"), Value::Integer(1));
    }

    #[test]
    fn function_calls_write_into_callee_slots_and_return() {
        // Constants are resolved *before* functions (`resolve.rs`), so a
        // function call can't appear in a `const` initializer — a variable's
        // initial value is resolved after functions, so it can.
        let source = r"
            function add(int a, int b) {
              return a + b;
            }
            global variables {
              int result = add(3, 4);
            }
        ";
        let spec = UntypedStarkSpecification::parse(source)
            .expect("should parse")
            .check()
            .expect("should check");
        let program = lower(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);
        let store = Store::new(&program, &mut rng);
        let result_slot = program.variables()[0].slot;
        assert_eq!(store.load(result_slot), Value::Integer(7));
    }

    #[test]
    fn let_binding_shadows_within_its_body() {
        let source = r"
            function with_let(int x) {
              let y = x + 1 in
              return y + 1;
            }
            global variables {
              int result = with_let(1);
            }
        ";
        let spec = UntypedStarkSpecification::parse(source)
            .expect("should parse")
            .check()
            .expect("should check");
        let program = lower(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);
        let store = Store::new(&program, &mut rng);
        assert_eq!(store.load(program.variables()[0].slot), Value::Integer(3));
    }

    #[test]
    fn sample_unit_is_seeded_and_reproducible() {
        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);
        let value_a = rng_a.random::<f64>();
        let value_b = rng_b.random::<f64>();
        assert_eq!(value_a, value_b);
        assert!((0.0..1.0).contains(&value_a));
    }

    #[test]
    fn sample_range_stays_in_bounds() {
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..100 {
            match sample_range(&mut rng, Value::Real(2.0), Value::Real(5.0)) {
                Value::Real(v) => assert!((2.0..5.0).contains(&v)),
                other => panic!("expected a Real, got {other:?}"),
            }
        }
    }

    #[test]
    fn sample_normal_matches_the_non_gaussian_java_quirk() {
        // Pin the `rng.nextDouble() * mean + variance` quirk exactly.
        let mut rng = StdRng::seed_from_u64(3);
        let uniform = rng.random::<f64>();
        let mut rng = StdRng::seed_from_u64(3);
        let sampled = sample_normal(&mut rng, Value::Real(10.0), Value::Real(1.0));
        assert_eq!(sampled, Value::Real(uniform * 10.0 + 1.0));
    }

    #[test]
    fn sample_choice_selects_each_element() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..200 {
            seen.insert(rng.random_range(0..3usize));
        }
        assert_eq!(seen, std::collections::HashSet::from([0, 1, 2]));
    }
}
