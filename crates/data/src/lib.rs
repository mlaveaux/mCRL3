#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod data_expression;
mod data_terms;
mod mcrl2_data_specification;
mod sort_terms;

pub(crate) use data_terms::*;

// Public API
pub use data_expression::BinderType;
pub use data_expression::DataAbstraction;
pub use data_expression::DataApplication;
pub use data_expression::DataEquation;
pub use data_expression::DataExpression;
pub use data_expression::DataExpressionRef;
pub use data_expression::DataFunctionSymbol;
pub use data_expression::DataFunctionSymbolRef;
pub use data_expression::DataVariable;
pub use data_expression::DataVariableRef;
pub use data_expression::DataWhereClause;
pub use data_expression::DataWhrDecl;
pub use data_expression::MachineNumber;
pub use data_expression::to_untyped_data_expression;
pub use data_terms::is_container_sort;
pub use data_terms::is_data_application;
pub use data_terms::is_data_binder;
pub use data_terms::is_data_function_symbol;
pub use data_terms::is_data_machine_number;
pub use data_terms::is_data_variable;
pub use data_terms::is_data_where_clause;
pub use data_terms::is_function_sort;
pub use mcrl2_data_specification::Mcrl2DataSpecification;
pub use sort_terms::BasicSort;
pub use sort_terms::BasicSortRef;
pub use sort_terms::ContainerSortKind;
pub use sort_terms::SortAlias;
pub use sort_terms::SortArrow;
pub use sort_terms::SortCons;
pub use sort_terms::SortExpression;
pub use sort_terms::SortExpressionRef;
