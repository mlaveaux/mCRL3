//! Expression and function-body evaluation over [IrProgram]'s arena: a
//! straight post-order walk of `ExprRef`/`StmtRef` indices rather than the
//! original's tree of lambda closures, since lowering already collapsed the
//! AST into that arena.
//!
//! Every function here returns a `Result<Value, EvalError>` and never panics.
//! A malformed runtime state (which shouldn't arise against a checked +
//! lowered [IrProgram]) is an `Err` naming what went wrong, rather than the
//! absorbing error *value* the original propagates. The `Result` honours the
//! same contract more strictly, since the error cannot be silently dropped.

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
use crate::value::EvalError;
use crate::value::EvalErrorKind;
use crate::value::Value;

use super::store::Store;

/// Evaluates one expression against `store`, sampling from `rng` wherever
/// the expression does.
///
/// This is a thin wrapper over [eval_inner] that pins the offending source
/// location: on failure it attaches `id`'s [Span](merc_utilities::Span) to the error
/// unless a more specific inner one was already recorded (see
/// [EvalError::or_span]). Because every recursive sub-evaluation goes through
/// this wrapper too, the span that survives is the innermost failing
/// expression's — so `1 / 0` deep inside a larger expression is reported
/// against `1 / 0`, not the whole expression.
pub(crate) fn eval<R: Rng + ?Sized>(
    program: &IrProgram,
    store: &mut Store,
    rng: &mut R,
    id: ExprRef,
) -> Result<Value, EvalError> {
    eval_inner(program, store, rng, id).map_err(|error| error.or_span(program.expr_span(id)))
}

fn eval_inner<R: Rng + ?Sized>(
    program: &IrProgram,
    store: &mut Store,
    rng: &mut R,
    id: ExprRef,
) -> Result<Value, EvalError> {
    match *program.expr(id) {
        ExprNode::Literal(value) => Ok(value),
        ExprNode::Unreachable(what) => Err(EvalErrorKind::Unreachable(what).into()),
        ExprNode::Load(slot) => Ok(store.load(slot)),
        ExprNode::Not(inner) => Ok(Value::Boolean((!eval(program, store, rng, inner)?)?)),
        // Both always widen to `Real`, matching the original — see
        // `ExprNode::Negate` and `ExprNode::Widen`'s doc comments in `ir.rs`.
        // The `Value` operations produce a bare [EvalErrorKind] (they have no
        // [crate::ir::Span] to give); `EvalError::from` lifts it, and the
        // outer `eval` wrapper then anchors it to this node's span.
        ExprNode::Negate(inner) => eval(program, store, rng, inner)?
            .apply_unary("-", |x| -x)
            .map_err(EvalError::from),
        ExprNode::Widen(inner) => eval(program, store, rng, inner)?
            .apply_unary("+", |x| x)
            .map_err(EvalError::from),
        ExprNode::Binary(op, left, right) => {
            let left = eval(program, store, rng, left)?;
            let right = eval(program, store, rng, right)?;
            apply_binary_op(op, left, right).map_err(EvalError::from)
        }
        ExprNode::MathUnary(function, inner) => {
            let value = eval(program, store, rng, inner)?;
            let (name, f) = math_unary_fn(function);
            value.apply_unary(name, f).map_err(EvalError::from)
        }
        ExprNode::MathBinary(function, left, right) => {
            let left = eval(program, store, rng, left)?;
            let right = eval(program, store, rng, right)?;
            let (name, f) = math_binary_fn(function);
            left.apply_binary(right, name, f).map_err(EvalError::from)
        }
        ExprNode::Select {
            guard,
            then_branch,
            else_branch,
        } => {
            // Lazy, as in the original: only the taken branch is evaluated,
            // since the untaken one may sample (advancing `rng`) or divide
            // by zero.
            if eval(program, store, rng, guard)?.as_boolean("the condition of a `?:` expression")? {
                eval(program, store, rng, then_branch)
            } else {
                eval(program, store, rng, else_branch)
            }
        }
        ExprNode::Call { function, arguments } => {
            let function_ir = program.function(function);
            // Evaluate every argument against the *caller's* slots first...
            let mut values = Vec::with_capacity(function_ir.arguments.len());
            for &argument in program.expr_list(arguments) {
                values.push(eval(program, store, rng, argument)?);
            }
            // ...then write them into the callee's fixed argument slots.
            // No frame save/restore: `resolve.rs` forbids recursion, so
            // every function's argument/`let` slots are disjoint from every
            // other function's and no function is ever live twice at once.
            for (&slot, value) in function_ir.arguments.iter().zip(values) {
                store.set(slot, value);
            }
            eval_stmt(program, store, rng, function_ir.body)
        }
        ExprNode::SampleUnit => Ok(Value::Real(rng.random::<f64>())),
        ExprNode::SampleRange { min, max } => {
            let min = eval(program, store, rng, min)?;
            let max = eval(program, store, rng, max)?;
            sample_range(rng, min, max)
        }
        ExprNode::SampleNormal { mean, variance } => {
            let mean = eval(program, store, rng, mean)?;
            let variance = eval(program, store, rng, variance)?;
            sample_normal(rng, mean, variance)
        }
        ExprNode::SampleChoice(list) => {
            let elements = program.expr_list(list);
            // Lazy like `Select`: the original picks an index and evaluates
            // only that one element.
            let selected = rng.random_range(0..elements.len());
            eval(program, store, rng, elements[selected])
        }
    }
}

/// Evaluates a function body statement, returning the value of whichever
/// `Return` is reached.
pub(crate) fn eval_stmt<R: Rng + ?Sized>(
    program: &IrProgram,
    store: &mut Store,
    rng: &mut R,
    id: StmtRef,
) -> Result<Value, EvalError> {
    match *program.stmt(id) {
        StmtNode::Return(value) => eval(program, store, rng, value),
        StmtNode::IfThenElse {
            guard,
            then_branch,
            else_branch,
        } => {
            if eval(program, store, rng, guard)?.as_boolean("the condition of an `if` statement")? {
                eval_stmt(program, store, rng, then_branch)
            } else {
                match else_branch {
                    Some(else_branch) => eval_stmt(program, store, rng, else_branch),
                    // `typecheck.rs` requires a function to return on every
                    // path, so a false guard with no `else` is unreachable
                    // against a checked program.
                    None => Err(EvalErrorKind::MissingReturn.into()),
                }
            }
        }
        StmtNode::Let { slot, value, body } => {
            let value = eval(program, store, rng, value)?;
            store.set(slot, value);
            eval_stmt(program, store, rng, body)
        }
    }
}

fn apply_binary_op(op: BinaryOp, left: Value, right: Value) -> Result<Value, EvalErrorKind> {
    // The comparisons and the boolean connectives return a bare `bool` (see
    // `Value::is_less_than`); an `ExprNode::Binary` is an expression, so they
    // are wrapped back into a `Value` here.
    match op {
        BinaryOp::Add => left.sum(right),
        BinaryOp::Subtract => left.subtraction(right),
        BinaryOp::Mult => left.product(right),
        BinaryOp::Div => left.division(right),
        BinaryOp::IntDiv => left.int_div(right),
        BinaryOp::Mod => left.modulo(right),
        BinaryOp::Less => left.is_less_than(right).map(Value::Boolean),
        BinaryOp::Leq => left.is_less_or_equal_than(right).map(Value::Boolean),
        BinaryOp::Eq => left.is_equal_to(right).map(Value::Boolean),
        BinaryOp::Geq => left.is_greater_or_equal_than(right).map(Value::Boolean),
        BinaryOp::Greater => left.is_greater_than(right).map(Value::Boolean),
        // `&&`/`&` and `||`/`|` are one operation each, two spellings — see
        // `Value::and`/`Value::or`'s doc comments.
        BinaryOp::And | BinaryOp::BitAnd => left.and(right).map(Value::Boolean),
        BinaryOp::Or | BinaryOp::BitOr => left.or(right).map(Value::Boolean),
    }
}

/// The `f64` implementation of each unary math function, paired with its
/// source-level name so a non-numeric argument can be reported against the
/// name the user actually wrote.
fn math_unary_fn(function: MathUnaryFunction) -> (&'static str, fn(f64) -> f64) {
    match function {
        MathUnaryFunction::Abs => ("abs", f64::abs),
        MathUnaryFunction::Acos => ("acos", f64::acos),
        MathUnaryFunction::Asin => ("asin", f64::asin),
        MathUnaryFunction::Atan => ("atan", f64::atan),
        MathUnaryFunction::Cbrt => ("cbrt", f64::cbrt),
        MathUnaryFunction::Ceil => ("ceil", f64::ceil),
        MathUnaryFunction::Cos => ("cos", f64::cos),
        MathUnaryFunction::Cosh => ("cosh", f64::cosh),
        MathUnaryFunction::Exp => ("exp", f64::exp),
        MathUnaryFunction::Expm1 => ("expm1", f64::exp_m1),
        MathUnaryFunction::Floor => ("floor", f64::floor),
        MathUnaryFunction::Log => ("log", f64::ln),
        MathUnaryFunction::Log10 => ("log10", f64::log10),
        MathUnaryFunction::Log1p => ("log1p", f64::ln_1p),
        MathUnaryFunction::Signum => ("signum", signum),
        MathUnaryFunction::Sin => ("sin", f64::sin),
        MathUnaryFunction::Sinh => ("sinh", f64::sinh),
        MathUnaryFunction::Sqrt => ("sqrt", f64::sqrt),
        MathUnaryFunction::Tan => ("tan", f64::tan),
    }
}

/// The binary counterpart of [math_unary_fn].
fn math_binary_fn(function: MathBinaryFunction) -> (&'static str, fn(f64, f64) -> f64) {
    match function {
        MathBinaryFunction::Atan2 => ("atan2", f64::atan2),
        MathBinaryFunction::Hypot => ("hypot", f64::hypot),
        MathBinaryFunction::Max => ("max", nan_max),
        MathBinaryFunction::Min => ("min", nan_min),
        MathBinaryFunction::Pow => ("pow", f64::powf),
    }
}

/// `signum` as the language defines it: unlike [f64::signum] (which returns
/// `±1.0` for `±0.0` and never `0.0`), this returns the zero itself (`0.0` or
/// `-0.0`) unchanged, and propagates `NaN`.
fn signum(x: f64) -> f64 {
    if x == 0.0 || x.is_nan() { x } else { x.signum() }
}

/// `max` as the language defines it: propagates `NaN` if *either* argument
/// is `NaN`. [f64::max] instead returns the non-`NaN` argument, so it can't
/// be used directly.
fn nan_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.max(b) }
}

/// `min`, see [nan_max].
fn nan_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.min(b) }
}

/// `R[a,b]`: a uniform sample, `from + u * (to - from)` for `u` in `[0, 1)`.
fn sample_range<R: Rng + ?Sized>(rng: &mut R, min: Value, max: Value) -> Result<Value, EvalError> {
    let from = min.as_f64("the lower bound of an `R[a,b]` sample")?;
    let to = max.as_f64("the upper bound of an `R[a,b]` sample")?;
    Ok(Value::Real(from + rng.random::<f64>() * (to - from)))
}

/// `N[mean, variance]`. **Not actually Gaussian** — despite the name and the
/// syntax, the original computes `u * mean + variance` for `u` in `[0, 1)`,
/// a scaled-and-shifted uniform sample rather than a normal distribution.
/// This is ported *exactly*, not "fixed", so behaviour matches the original
/// tool: it reads as a bug there, but silently correcting it is not this
/// port's place.
fn sample_normal<R: Rng + ?Sized>(rng: &mut R, mean: Value, variance: Value) -> Result<Value, EvalError> {
    // Both bounds are already guaranteed numeric by `typecheck.rs` (`R[a,b]`'s
    // bounds and `N[m,v]`'s mean/variance are all checked against `real`), so
    // these errors only fire against an otherwise-unreachable malformed IR.
    let mean = mean.as_f64("the mean of an `N[m,v]` sample")?;
    let variance = variance.as_f64("the variance of an `N[m,v]` sample")?;
    Ok(Value::Real(rng.random::<f64>() * mean + variance))
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use test_case::test_case;
    use test_log::test;

    use super::*;
    use crate::StarkSpecification;
    use crate::UntypedStarkSpecification;

    fn eval_expression(source: &str) -> Value {
        let full_source = format!("const result = {source};");
        let untyped = UntypedStarkSpecification::parse(&full_source).expect("should parse");
        let spec = StarkSpecification::from_untyped(untyped).expect("should check");
        let program = IrProgram::from_spec(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);
        let store = Store::new(&program, &mut rng).expect("should initialise");
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
    fn a_runtime_error_is_anchored_to_the_innermost_offending_expression() {
        // The `1 / 0` is nested inside `4 + …`; the reported span must
        // underline the division that actually failed, not the whole
        // initializer — this is what `EvalError::or_span`'s innermost-wins
        // rule buys, and what makes the message read as "division by zero at
        // <the division>".
        let source = "const result = 4 + 1 / 0;";
        let untyped = UntypedStarkSpecification::parse(source).expect("should parse");
        let spec = StarkSpecification::from_untyped(untyped).expect("should check");
        let program = IrProgram::from_spec(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);

        let error = Store::new(&program, &mut rng).expect_err("initialisation divides by zero");
        assert_eq!(error.kind, EvalErrorKind::DivisionByZero);

        let span = error.span.clone().expect("the failure carries a source span");
        let underlined = &source[span.start..span.end];
        assert!(
            underlined.contains('/') && !underlined.contains('4'),
            "expected the span to point at the inner division, got {underlined:?}"
        );
        // And it renders in the shared `-->`/`^^^` diagnostic style.
        let rendered = error.render(source);
        assert!(rendered.starts_with("error: division by zero"), "got: {rendered}");
        assert!(rendered.contains("--> 1:"), "got: {rendered}");
    }

    #[test]
    fn select_only_evaluates_the_taken_branch() {
        // The untaken branch divides by zero; if `Select` weren't lazy this
        // would fail with `EvalError::DivisionByZero` (and, before errors
        // became a `Result`, would have silently yielded `Value::Error`)
        // instead of `Value::Integer(1)`.
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
        let untyped = UntypedStarkSpecification::parse(source).expect("should parse");
        let spec = StarkSpecification::from_untyped(untyped).expect("should check");
        let program = IrProgram::from_spec(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);
        let store = Store::new(&program, &mut rng).expect("should initialise");
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
        let untyped = UntypedStarkSpecification::parse(source).expect("should parse");
        let spec = StarkSpecification::from_untyped(untyped).expect("should check");
        let program = IrProgram::from_spec(&spec).expect("should lower");
        let mut rng = StdRng::seed_from_u64(0);
        let store = Store::new(&program, &mut rng).expect("should initialise");
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
                Ok(Value::Real(v)) => assert!((2.0..5.0).contains(&v)),
                other => panic!("expected a Real, got {other:?}"),
            }
        }
    }

    #[test]
    fn sample_normal_matches_the_non_gaussian_quirk() {
        // Pin the `u * mean + variance` quirk exactly.
        let mut rng = StdRng::seed_from_u64(3);
        let uniform = rng.random::<f64>();
        let mut rng = StdRng::seed_from_u64(3);
        let sampled = sample_normal(&mut rng, Value::Real(10.0), Value::Real(1.0));
        assert_eq!(sampled, Ok(Value::Real(uniform * 10.0 + 1.0)));
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
