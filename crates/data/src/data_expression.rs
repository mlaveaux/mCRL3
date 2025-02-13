use core::fmt;

use crate::aterm::ATerm;
use crate::aterm::ATermArgs;
use crate::aterm::ATermRef;
use crate::aterm::THREAD_TERM_POOL;
use mcrl2_macros::mcrl2_derive_terms;
use mcrl2_sys::data::ffi;
pub struct DataExpressionSymbols;

impl DataExpressionSymbols {
    pub fn is_data_variable(term: &ATermRef<'_>) -> bool {
        term.require_valid();
        unsafe { ffi::is_data_variable(term.get()) }
    }

    pub fn is_data_expression(term: &ATermRef<'_>) -> bool {
        term.require_valid();
        Self::is_data_variable(term)
            || Self::is_data_function_symbol(term)
            || Self::is_data_machine_number(term)
            || Self::is_data_application(term)
            || Self::is_data_abstraction(term)
            || Self::is_data_where_clause(term)
    }

    pub fn is_data_function_symbol(term: &ATermRef<'_>) -> bool {
        term.require_valid();
        unsafe { ffi::is_data_function_symbol(term.get()) }
    }

    pub fn is_data_machine_number(term: &ATermRef<'_>) -> bool {
        term.require_valid();
        unsafe { ffi::is_data_machine_number(term.get()) }
    }

    pub fn is_data_where_clause(term: &ATermRef<'_>) -> bool {
        term.require_valid();
        unsafe { ffi::is_data_where_clause(term.get()) }
    }

    pub fn is_data_abstraction(term: &ATermRef<'_>) -> bool {
        term.require_valid();
        unsafe { ffi::is_data_abstraction(term.get()) }
    }

    pub fn is_data_untyped_identifier(term: &ATermRef<'_>) -> bool {
        term.require_valid();
        unsafe { ffi::is_data_untyped_identifier(term.get()) }
    }

    pub fn is_data_application(term: &ATermRef<'_>) -> bool {
        term.require_valid();

        THREAD_TERM_POOL.with_borrow_mut(|tp| tp.is_data_application(term))
    }
}

// This module is only used internally to run the proc macro.
#[mcrl2_derive_terms]
mod inner {
    use super::*;

    use std::borrow::Borrow;
    use std::ops::Deref;

    use crate::aterm::Markable;
    use crate::aterm::TermPool;
    use crate::aterm::Todo;
    use crate::data::SortExpression;
    use crate::data::SortExpressionRef;
    use mcrl2_macros::mcrl2_ignore;
    use mcrl2_macros::mcrl2_term;

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
    #[mcrl2_term(is_data_expression)]
    pub struct DataExpression {
        term: ATerm,
    }

    impl DataExpression {
        /// Returns the head symbol a data expression
        ///     - function symbol                  f -> f
        ///     - application       f(t_0, ..., t_n) -> f
        pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'_> {
            if is_data_application(&self.term) {
                self.term.arg(0).upgrade(&self.term).into()
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
                Default::default()
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

    #[mcrl2_term(is_data_function_symbol)]
    pub struct DataFunctionSymbol {
        term: ATerm,
    }

    impl DataFunctionSymbol {
        #[mcrl2_ignore]
        pub fn new(tp: &mut TermPool, name: &str) -> DataFunctionSymbol {
            DataFunctionSymbol {
                term: tp.create_with(|| mcrl2_sys::data::ffi::create_data_function_symbol(name.to_string())),
            }
        }

        /// Returns the sort of the function symbol.
        pub fn sort(&self) -> SortExpressionRef<'_> {
            self.term.arg(1).into()
        }

        /// Returns the name of the function symbol
        pub fn name(&self) -> &str {
            // We only change the lifetime, but that is fine since it is derived from the current term.
            unsafe { std::mem::transmute(self.term.arg(0).get_head_symbol().name()) }
        }

        /// Returns the internal operation id (a unique number) for the data::function_symbol.
        pub fn operation_id(&self) -> usize {
            debug_assert!(
                is_data_function_symbol(&self.term),
                "term {} is not a data function symbol",
                self.term
            );
            unsafe { ffi::get_data_function_symbol_index(self.term.get()) }
        }
    }

    impl fmt::Display for DataFunctionSymbol {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if !self.is_default() {
                write!(f, "{}", self.name())
            } else {
                write!(f, "<default>")
            }
        }
    }

    #[mcrl2_term(is_data_variable)]
    pub struct DataVariable {
        term: ATerm,
    }

    impl DataVariable {
        /// Create a new untyped variable with the given name.
        #[mcrl2_ignore]
        pub fn new(tp: &mut TermPool, name: &str) -> DataVariable {
            DataVariable {
                term: tp.create_with(|| mcrl2_sys::data::ffi::create_data_variable(name.to_string())),
            }
        }

        /// Create a variable with the given sort and name.
        pub fn with_sort(tp: &mut TermPool, name: &str, sort: &SortExpressionRef<'_>) -> DataVariable {
            DataVariable {
                term: tp.create_with(|| unsafe {
                    mcrl2_sys::data::ffi::create_sorted_data_variable(name.to_string(), sort.term.get())
                }),
            }
        }

        /// Returns the name of the variable.
        pub fn name(&self) -> &str {
            // We only change the lifetime, but that is fine since it is derived from the current term.
            unsafe { std::mem::transmute(self.term.arg(0).get_head_symbol().name()) }
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

    #[mcrl2_term(is_data_application)]
    pub struct DataApplication {
        term: ATerm,
    }

    impl DataApplication {
        #[mcrl2_ignore]
        pub fn new<'a, 'b>(
            tp: &mut TermPool,
            head: &impl Borrow<ATermRef<'a>>,
            arguments: &[impl Borrow<ATermRef<'b>>],
        ) -> DataApplication {
            DataApplication {
                term: tp.create_data_application(head, arguments),
            }
        }

        /// Returns the head symbol a data application
        pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'_> {
            self.term.arg(0).upgrade(&self.term).into()
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
            unsafe { std::mem::transmute(SortExpressionRef::from(self.term.arg(0))) }
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

    #[mcrl2_term(is_data_machine_number)]
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

    #[mcrl2_ignore]
    impl From<DataFunctionSymbol> for DataExpression {
        fn from(value: DataFunctionSymbol) -> Self {
            value.term.into()
        }
    }

    #[mcrl2_ignore]
    impl From<DataApplication> for DataExpression {
        fn from(value: DataApplication) -> Self {
            value.term.into()
        }
    }

    #[mcrl2_ignore]
    impl From<DataVariable> for DataExpression {
        fn from(value: DataVariable) -> Self {
            value.term.into()
        }
    }
}

pub use inner::*;

#[cfg(test)]
mod tests {
    use crate::aterm::ATerm;
    use crate::aterm::TermPool;
    use crate::data::is_data_application;
    use crate::data::DataApplication;
    use crate::data::DataFunctionSymbol;

    #[test]
    fn test_print() {
        let mut tp = TermPool::new();

        let a = DataFunctionSymbol::new(&mut tp, "a");
        assert_eq!("a", format!("{}", a));

        // Check printing of data applications.
        let f = DataFunctionSymbol::new(&mut tp, "f");
        let a_term: ATerm = a.clone().into();
        let appl = DataApplication::new(&mut tp, &f, &[a_term]);
        assert_eq!("f(a)", format!("{}", appl));
    }

    #[test]
    fn test_recognizers() {
        let mut tp = TermPool::new();

        let a = DataFunctionSymbol::new(&mut tp, "a");
        let f = DataFunctionSymbol::new(&mut tp, "f");
        let a_term: ATerm = a.clone().into();
        let appl = DataApplication::new(&mut tp, &f, &[a_term]);

        let term: ATerm = appl.into();
        assert!(is_data_application(&term));
    }
}
