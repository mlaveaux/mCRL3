use core::fmt;
use std::borrow::Borrow;
use std::ops::Deref;

use delegate::delegate;

use mcrl3_aterm::ATerm;
use mcrl3_aterm::ATermArgs;
use mcrl3_aterm::ATermInt;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::ATermString;
use mcrl3_aterm::Markable;
use mcrl3_aterm::Marker;
use mcrl3_aterm::StrRef;
use mcrl3_aterm::Symb;
use mcrl3_aterm::SymbolRef;
use mcrl3_aterm::Term;
use mcrl3_aterm::TermIterator;
use mcrl3_macros::mcrl3_derive_terms;
use mcrl3_macros::mcrl3_ignore;
use mcrl3_macros::mcrl3_term;

use crate::DATA_SYMBOLS;
use crate::SortExpression;
use crate::SortExpressionRef;
use crate::is_data_application;
use crate::is_data_expression;
use crate::is_data_function_symbol;
use crate::is_data_machine_number;
use crate::is_data_variable;

// This module is only used internally to run the proc macro.
#[mcrl3_derive_terms]
mod inner {

    use std::{iter, num::NonZero};

    use mcrl3_aterm::ATermStringRef;

    use super::*;

    /// A data expression can be any of:
    ///     - a variable
    ///     - a function symbol, i.e. f without arguments.
    ///     - a term applied to a number of arguments, i.e., t_0(t1, ..., tn).
    ///     - an abstraction lambda x: Sort . e, or forall and exists.
    ///     - machine number, a value [0, ..., 2^64-1].
    ///
    /// Not supported:
    ///     - a where clause "e where [x := f, ...]"
    ///     - set enumeration
    ///     - bag enumeration
    ///
    #[mcrl3_term(is_data_expression)]
    pub struct DataExpression {
        term: ATerm,
    }

    impl DataExpression {
        /// Returns the head symbol a data expression
        ///     - function symbol                  f -> f
        ///     - application       f(t_0, ..., t_n) -> f
        pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'_> {
            if is_data_application(&self.term) {
                self.term.arg(0).into()
            } else if is_data_function_symbol(&self.term) {
                self.term.copy().into()
            } else {
                panic!("data_function_symbol not implemented for {}", self);
            }
        }

        /// Returns the arguments of a data expression
        ///     - function symbol                  f -> []
        ///     - application       f(t_0, ..., t_n) -> [t_0, ..., t_n]
        pub fn data_arguments(&self) -> ATermArgs<'_> {
            if is_data_application(&self.term) {
                let mut result = self.term.arguments();
                result.next();
                result
            } else if is_data_function_symbol(&self.term) {
                ATermArgs::empty()
            } else {
                panic!("data_arguments not implemented for {}", self);
            }
        }

        /// Returns the arguments of a data expression
        ///     - function symbol                  f -> []
        ///     - application       f(t_0, ..., t_n) -> [t_0, ..., t_n]
        pub fn data_sort(&self) -> SortExpression {
            if is_data_function_symbol(&self.term) {
                DataFunctionSymbolRef::from(self.term.copy()).sort().protect()
            } else if is_data_variable(&self.term) {
                DataVariableRef::from(self.term.copy()).sort().protect()
            } else {
                panic!("data_sort not implemented for {}", self);
            }
        }
    }

    impl fmt::Display for DataExpression {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if is_data_function_symbol(&self.term) {
                write!(f, "{}", DataFunctionSymbolRef::from(self.term.copy()))
            } else if is_data_application(&self.term) {
                write!(f, "{}", DataApplicationRef::from(self.term.copy()))
            } else if is_data_variable(&self.term) {
                write!(f, "{}", DataVariableRef::from(self.term.copy()))
            } else if is_data_machine_number(&self.term) {
                write!(f, "{}", MachineNumberRef::from(self.term.copy()))
            } else {
                write!(f, "{}", self.term)
            }
        }
    }

    #[mcrl3_term(is_data_function_symbol)]
    pub struct DataFunctionSymbol {
        term: ATerm,
    }

    impl DataFunctionSymbol {
        #[mcrl3_ignore]
        pub fn new(name: impl Into<String> + AsRef<str>) -> DataFunctionSymbol {
            DATA_SYMBOLS.with_borrow(|ds| DataFunctionSymbol {
                term: ATerm::with_args(
                    ds.data_function_symbol.deref(),
                    &[
                        Into::<ATerm>::into(ATermString::new(name)),
                        SortExpression::unknown_sort().into(),
                    ],
                ),
            })
        }

        /// Returns the name of the function symbol
        pub fn name(&self) -> ATermStringRef<'_> {
            ATermStringRef::from(self.term.arg(0))
        }

        /// Returns the sort of the function symbol.
        pub fn sort(&self) -> SortExpressionRef<'_> {
            self.term.arg(1).into()
        }

        /// Returns the internal operation id (a unique number) for the data::function_symbol.
        pub fn operation_id(&self) -> usize {
            self.term.index().into()
        }
    }

    impl fmt::Display for DataFunctionSymbol {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }

    #[mcrl3_term(is_data_variable)]
    pub struct DataVariable {
        term: ATerm,
    }

    impl DataVariable {
        /// Create a new untyped variable with the given name.
        #[mcrl3_ignore]
        pub fn new(name: impl Into<ATermString>) -> DataVariable {
            DATA_SYMBOLS.with_borrow(|ds| {
                // TODO: Storing terms temporarily is not optimal.
                let t = name.into();
                let args: &[ATerm] = &[t.into(), ATermInt::new(0).into()];

                DataVariable {
                    term: ATerm::with_args(ds.data_variable.deref(), args),
                }
            })
        }

        /// Create a variable with the given sort and name.
        pub fn with_sort(name: impl Into<ATermString>, sort: usize) -> DataVariable {
            DATA_SYMBOLS.with_borrow(|ds| {
                // TODO: Storing terms temporarily is not optimal.
                let t = name.into();
                let args: &[ATerm] = &[t.into(), ATermInt::new(sort).into()];

                DataVariable {
                    term: ATerm::with_args(ds.data_variable.deref(), args),
                }
            })
        }

        /// Returns the name of the variable.
        pub fn name(&self) -> StrRef<'_> {
            // We only change the lifetime, but that is fine since it is derived from the current term.
            self.term.arg(0).get_head_symbol().name()
        }

        /// Returns the sort of the variable.
        pub fn sort(&self) -> SortExpressionRef<'_> {
            self.term.arg(1).into()
        }
    }

    impl fmt::Display for DataVariable {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }

    #[mcrl3_term(is_data_application)]
    pub struct DataApplication {
        term: ATerm,
    }

    impl DataApplication {

        /// Create a new data application with the given head and arguments.
        #[mcrl3_ignore]
        pub fn with_args<'a>(head: &impl Term<'a>, arguments: &[impl Term<'a>]) -> DataApplication 
        {            
            DATA_SYMBOLS.with_borrow_mut(|ds| {
                // TODO: Perhaps not the must optimal.
                let args = iter::once(head.copy()).chain(arguments.iter().map(|t| t.copy()));
                let term = ATerm::with_iter(
                    ds.get_data_application_symbol(arguments.len() + 1),
                    args,
                );

                DataApplication { term }
            })
        }

        /// Create a new data application with the given head and arguments.
        #[mcrl3_ignore]
        pub fn with_iter<'a, T, I>(head: &impl Term<'a>, arguments: I) -> DataApplication 
            where
                I: Iterator<Item = T>,
                T: Term<'a>,
        {            
            DATA_SYMBOLS.with_borrow_mut(|ds| {
                // TODO: Perhaps not the must optimal.
                let args = iter::once(head.copy()).chain(arguments.map(|t| t.copy()));
                let term = ATerm::with_iter(
                    ds.get_data_application_symbol(1),
                    args,
                );

                DataApplication { term }
            })
        }

        /// Returns the head symbol a data application
        pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'_> {
            self.term.arg(0).into()
        }

        /// Returns the arguments of a data application
        pub fn data_arguments(&self) -> ATermArgs<'_> {
            let mut result = self.term.arguments();
            result.next();
            result
        }

        /// Returns the sort of a data application.
        pub fn sort(&self) -> SortExpressionRef<'_> {
            // We only change the lifetime, but that is fine since it is derived from the current term.
            SortExpressionRef::from(self.term.arg(0))
        }
    }

    impl fmt::Display for DataApplication {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.data_function_symbol())?;

            let mut first = true;
            for arg in self.data_arguments() {
                if !first {
                    write!(f, ", ")?;
                } else {
                    write!(f, "(")?;
                }

                write!(f, "{}", DataExpressionRef::from(arg.copy()))?;
                first = false;
            }

            if !first {
                write!(f, ")")?;
            }

            Ok(())
        }
    }

    #[mcrl3_term(is_data_machine_number)]
    struct MachineNumber {
        pub term: ATerm,
    }

    impl MachineNumber {
        /// Obtain the underlying value of a machine number.
        pub fn value(&self) -> u64 {
            0
        }
    }

    impl fmt::Display for MachineNumber {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    /// Conversions to `DataExpression`
    #[mcrl3_ignore]
    impl From<DataFunctionSymbol> for DataExpression {
        fn from(value: DataFunctionSymbol) -> Self {
            value.term.into()
        }
    }

    #[mcrl3_ignore]
    impl From<DataApplication> for DataExpression {
        fn from(value: DataApplication) -> Self {
            value.term.into()
        }
    }

    #[mcrl3_ignore]
    impl From<DataVariable> for DataExpression {
        fn from(value: DataVariable) -> Self {
            value.term.into()
        }
    }

    #[mcrl3_ignore]
    impl From<DataExpression> for DataFunctionSymbol {
        fn from(value: DataExpression) -> Self {
            value.term.into()
        }
    }
}

pub use inner::*;

#[cfg(test)]
mod tests {
    use mcrl3_aterm::ATerm;

    use crate::DataApplication;
    use crate::DataFunctionSymbol;
    use crate::is_data_application;

    #[test]
    fn test_print() {
        let _ = mcrl3_utilities::test_logger();

        let a = DataFunctionSymbol::new("a");
        assert_eq!("a", format!("{}", a));

        // Check printing of data applications.
        let f = DataFunctionSymbol::new("f");
        let appl = DataApplication::with_args(&f, &[a]);
        assert_eq!("f(a)", format!("{}", appl));
    }

    #[test]
    fn test_recognizers() {
        let a = DataFunctionSymbol::new("a");
        let f = DataFunctionSymbol::new("f");
        let appl = DataApplication::with_args(&f, &[a]);

        let term: ATerm = appl.into();
        assert!(is_data_application(&term));
    }
}
