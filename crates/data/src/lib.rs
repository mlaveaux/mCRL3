#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod closed;
mod data_expression;
mod data_terms;
mod machine_word_evaluation;
mod mcrl2_data_specification;
mod sort_terms;
mod visitor;

// Explicit pub(crate) re-exports.
pub(crate) use data_terms::DATA_SYMBOLS;
pub(crate) use data_terms::is_basic_sort;
pub(crate) use data_terms::is_data_expression;
pub(crate) use data_terms::is_sort_expression;

// Public API
pub use closed::is_closed;
pub use data_expression::DataApplication;
pub use data_expression::DataApplicationRef;
pub use data_expression::DataExpression;
pub use data_expression::DataExpressionRef;
pub use data_expression::DataFunctionSymbol;
pub use data_expression::DataFunctionSymbolRef;
pub use data_expression::DataVariable;
pub use data_expression::DataVariableRef;
pub use data_expression::MachineNumberRef;
pub use data_expression::to_untyped_data_expression;
pub use data_specification::DataSpecification;
pub use data_terms::is_data_application;
pub use data_terms::is_data_binder;
pub use data_terms::is_data_function_symbol;
pub use data_terms::is_data_machine_number;
pub use data_terms::is_data_variable;
pub use data_terms::is_data_where_clause;
pub use sort_terms::BasicSort;
pub use sort_terms::BasicSortRef;
pub use sort_terms::SortExpression;
pub use sort_terms::SortExpressionRef;
pub use visitor::ClosureVisitor;
pub use visitor::DataExpressionVisitor;
pub use visitor::try_visit_data_expr;
pub use visitor::visit_data_expr;
