use std::{cell::RefCell, mem::ManuallyDrop};

use mcrl3_aterm::{ATerm, ATermRef, Symbol, SymbolRef};

thread_local! {
    pub static DATA_SYMBOLS: RefCell<DataSymbols> = RefCell::new(DataSymbols::new());
}

/// Defines default symbols and terms for data elements.
pub struct DataSymbols {
    sort_expression: ManuallyDrop<Symbol>,
    basic_sort: ManuallyDrop<Symbol>,
    function_sort: ManuallyDrop<Symbol>,

    pub data_function_symbol: ManuallyDrop<Symbol>,
    data_variable: ManuallyDrop<Symbol>,
    data_machine_number: ManuallyDrop<Symbol>,
    data_where_clause: ManuallyDrop<Symbol>,
    data_abstraction: ManuallyDrop<Symbol>,
    data_untyped_identifier: ManuallyDrop<Symbol>,
    
    data_appl: Vec<Symbol>,
}

impl DataSymbols {
    fn new() -> Self {
        Self {
            sort_expression: ManuallyDrop::new(Symbol::new("SortExpression", 2)),
            basic_sort: ManuallyDrop::new(Symbol::new("SortExpression", 2)),
            function_sort: ManuallyDrop::new(Symbol::new("SortExpression", 2)),

            data_function_symbol: ManuallyDrop::new(Symbol::new("OpId", 3)),
            data_variable: ManuallyDrop::new(Symbol::new("SortExpression", 2)),
            data_machine_number: ManuallyDrop::new(Symbol::new("SortExpression", 2)),
            data_where_clause: ManuallyDrop::new(Symbol::new("SortExpression", 2)),
            data_abstraction: ManuallyDrop::new(Symbol::new("SortExpression", 2)),
            data_untyped_identifier: ManuallyDrop::new(Symbol::new("SortExpression", 2)),

            data_appl: vec![],
        }
    }
    
    pub fn is_sort_expression(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.sort_expression
    }

    pub fn is_bool_sort(&self, _term: &ATermRef<'_>) -> bool {
        true
    }

    pub fn is_basic_sort(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.basic_sort
    }

    pub fn is_data_function_sort(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.function_sort
    }

    pub fn is_data_variable(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.data_variable
    }

    pub fn is_data_expression(&mut self, term: &ATermRef<'_>) -> bool {
        self.is_data_variable(term)
            || self.is_data_function_symbol(term)
            || self.is_data_machine_number(term)
            || self.is_data_application(term)
            || self.is_data_abstraction(term)
            || self.is_data_where_clause(term)
    }

    pub fn is_data_function_symbol(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.data_function_symbol
    }

    pub fn is_data_machine_number(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.data_machine_number
    }

    pub fn is_data_where_clause(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.data_where_clause
    }

    pub fn is_data_abstraction(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.data_abstraction
    }

    pub fn is_data_untyped_identifier(&self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == **self.data_untyped_identifier
    }

    /// Returns true iff the given term is a data application.
    pub fn is_data_application(&mut self, term: &ATermRef<'_>) -> bool {
        *term.get_head_symbol() == *self.get_data_application_symbol(term.get_head_symbol().arity())
    }

    pub fn get_data_application_symbol(&mut self, arity: usize) -> &SymbolRef<'_> {
        // It can be that data_applications are created without create_data_application in the mcrl2 ffi.
        while self.data_appl.len() <= arity {
            let symbol = Symbol::new(
                "DataAppl",
                self.data_appl.len(),
            );

            self.data_appl.push(symbol);
        }

        &self.data_appl[arity]
    }
}

pub fn is_sort_expression(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_sort_expression(term))
}

pub fn is_bool_sort(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_bool_sort(term))
}

pub fn is_basic_sort(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_basic_sort(term))
}

pub fn is_data_function_sort(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_function_sort(term))
}

pub fn is_data_variable(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_variable(term))
}

pub fn is_data_expression(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow_mut(|ds| ds.is_data_expression(term))
}

pub fn is_data_function_symbol(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_function_symbol(term))
}

pub fn is_data_machine_number(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_machine_number(term))
}

pub fn is_data_where_clause(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_where_clause(term))
}

pub fn is_data_abstraction(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_abstraction(term))
}

pub fn is_data_untyped_identifier(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_untyped_identifier(term))
}

pub fn is_data_application(term: &ATermRef<'_>) -> bool {
    DATA_SYMBOLS.with_borrow_mut(|ds| ds.is_data_application(term))
}

pub fn true_term() -> ATerm {
    unimplemented!()
}

pub fn false_term() -> ATerm {
    unimplemented!()
}

pub fn  get_data_function_symbol_index(_term: &ATermRef<'_>) -> usize {
    unimplemented!()
}