//!
//! See the export module for the function documentation.
//!

#[link(name = "sabre-ffi")]
unsafe extern "C" {
    fn initialize_thread_local_term_pool(global_term_pool: *mut c_void);

    /// Returns the argument of a data expression.
    fn data_expression_arg(term: DataExpressionRefFFI<'_>, index: usize) -> DataExpressionRefFFI<'_>;

    /// Returns the data function symbol of a data expression.
    fn data_expression_symbol(term: DataExpressionRefFFI<'_>) -> DataFunctionSymbolRefFFI<'_>;

    /// Returns the number of data arguments of a data expression.
    fn data_expression_arity(term: DataExpressionRefFFI<'_>) -> usize;

    /// Protects the data expression.
    fn data_expression_protect(term: DataExpressionRefFFI<'_>) -> DataExpressionFFI;
}
