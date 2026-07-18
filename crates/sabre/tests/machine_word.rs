//! End-to-end tests that the [InnermostRewriter] natively evaluates the
//! machine-word (`@word`) operations, which carry no rewrite rules and must be
//! computed directly from their concrete arguments.

use merc_data::BasicSort;
use merc_data::DataApplication;
use merc_data::DataExpression;
use merc_data::DataFunctionSymbol;
use merc_data::MachineNumber;
use merc_data::SortExpression;
use merc_sabre::InnermostRewriter;
use merc_sabre::RewriteEngine;
use merc_sabre::RewriteSpecification;

/// A machine-number data expression.
fn word(value: u64) -> DataExpression {
    MachineNumber::new(value).into()
}

/// A `@word` operation `name` applied to `args`.
fn op(name: &str, args: &[DataExpression]) -> DataExpression {
    DataApplication::with_args(&DataFunctionSymbol::new(name), args).into()
}

/// A `Bool` literal, built exactly as the IR lowering does.
fn boolean(value: bool) -> DataExpression {
    DataFunctionSymbol::with_sort(
        if value { "true" } else { "false" },
        SortExpression::from(BasicSort::new("Bool")).copy(),
    )
    .into()
}

/// Machine-word operations are evaluated even with no rewrite rules present.
fn rewriter() -> InnermostRewriter {
    InnermostRewriter::new(&RewriteSpecification::new(vec![]))
}

#[test]
fn test_word_valued_operations() {
    let mut rewriter = rewriter();

    assert_eq!(rewriter.rewrite(&op("@add_word", &[word(3), word(5)])), word(8));
    assert_eq!(rewriter.rewrite(&op("@succ_word", &[word(41)])), word(42));
    assert_eq!(rewriter.rewrite(&op("@div_word", &[word(17), word(5)])), word(3));
    // Wrapping semantics.
    assert_eq!(rewriter.rewrite(&op("@add_word", &[word(u64::MAX), word(1)])), word(0));
    // (2^64 * 1 + 0) div 2 == 2^63.
    assert_eq!(
        rewriter.rewrite(&op("@div_doubleword", &[word(1), word(0), word(2)])),
        word(1u64 << 63)
    );
}

#[test]
fn test_nested_operations_reduce_innermost_first() {
    let mut rewriter = rewriter();

    // (1 + 2) + (17 mod 5) == 3 + 2 == 5. Exercises the machine-number sub-term
    // short-circuit in the innermost rewriter as well as the native dispatch.
    let nested = op(
        "@add_word",
        &[
            op("@add_word", &[word(1), word(2)]),
            op("@mod_word", &[word(17), word(5)]),
        ],
    );
    assert_eq!(rewriter.rewrite(&nested), word(5));
}

#[test]
fn test_shift_right_with_bool_argument() {
    let mut rewriter = rewriter();

    // @shift_right(false, 0b100) == 0b10.
    assert_eq!(
        rewriter.rewrite(&op("@shift_right", &[boolean(false), word(0b100)])),
        word(0b10)
    );
    // @shift_right(true, 0) inserts a new most-significant bit.
    assert_eq!(
        rewriter.rewrite(&op("@shift_right", &[boolean(true), word(0)])),
        word(1u64 << 63)
    );
}

#[test]
fn test_bool_valued_operations() {
    let mut rewriter = rewriter();

    assert_eq!(rewriter.rewrite(&op("@less", &[word(2), word(5)])), boolean(true));
    assert_eq!(rewriter.rewrite(&op("@less", &[word(5), word(2)])), boolean(false));
    assert_eq!(rewriter.rewrite(&op("@equal", &[word(7), word(7)])), boolean(true));
    assert_eq!(rewriter.rewrite(&op("@equals_zero_word", &[word(0)])), boolean(true));
}

/// A lone machine number is already in normal form and rewrites to itself.
#[test]
fn test_machine_number_is_normal_form() {
    let mut rewriter = rewriter();
    assert_eq!(rewriter.rewrite(&word(42)), word(42));
}
