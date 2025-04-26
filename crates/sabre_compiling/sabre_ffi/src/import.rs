//!
//! See the export module for the function documentation.
//!

#[link(name = "sabre-ffi")]
unsafe extern "C" {
    /// Returns the argument of a data expression.
    ///
    /// # Safety
    ///
    /// The resulting data expression ref is an argument of the provided data expression.
    fn data_expression_arg(term: DataExpressionRef<'_>, index: usize) -> DataExpressionRef<'_>;

    /// Returns the data function symbol of a data expression.
    ///
    /// # Safety
    ///
    /// The resulting data function symbol ref is an argument of the provided data expression.
    fn data_expression_symbol(term: DataExpressionRef<'_>) -> DataFunctionSymbolRef<'_>;

    /// Protects the data expression.
    fn data_expression_protect(term: DataExpressionRef<'_>) -> DataExpression;
}
