use std::cell::RefCell;
use std::mem::ManuallyDrop;

use mcrl3_aterm::ATermRef;
use mcrl3_aterm::Symb;
use mcrl3_aterm::Symbol;
use mcrl3_aterm::SymbolRef;
use mcrl3_aterm::Term;

thread_local! {
    pub static DATA_SYMBOLS: RefCell<DataSymbols> = RefCell::new(DataSymbols::new());
}

/// Defines default symbols and terms for data elements.
pub struct DataSymbols {
    pub sort_expression_symbol: ManuallyDrop<Symbol>,
    pub data_function_symbol: ManuallyDrop<Symbol>,
    pub data_variable: ManuallyDrop<Symbol>,
    pub data_where_clause: ManuallyDrop<Symbol>,
    pub data_abstraction: ManuallyDrop<Symbol>,

    /// The data application symbol for a given arity.
    data_appl: Vec<Symbol>,
}

impl DataSymbols {
    fn new() -> Self {
        Self {
            sort_expression_symbol: ManuallyDrop::new(Symbol::new("SortId", 1)),
            data_function_symbol: ManuallyDrop::new(Symbol::new("OpId", 3)),
            data_variable: ManuallyDrop::new(Symbol::new("Var", 2)),

            data_where_clause: ManuallyDrop::new(Symbol::new("Where", 2)),
            data_abstraction: ManuallyDrop::new(Symbol::new("Abstraction", 2)),
            data_appl: vec![],
        }
    }

    pub fn is_sort_expression<'a>(&self, term: &impl Term<'a>) -> bool {
        term.get_head_symbol() == **self.sort_expression_symbol
    }

    pub fn is_bool_sort<'a>(&self, _term: &impl Term<'a>) -> bool {
        true
    }

    pub fn is_data_variable<'a>(&self, term: &impl Term<'a>) -> bool {
        term.get_head_symbol() == **self.data_variable
    }

    pub fn is_data_expression<'a>(&mut self, term: &impl Term<'a>) -> bool {
        self.is_data_variable(term)
            || self.is_data_function_symbol(term)
            || self.is_data_machine_number(term)
            || self.is_data_application(term)
            || self.is_data_abstraction(term)
            || self.is_data_where_clause(term)
    }

    pub fn is_data_function_symbol<'a>(&self, term: &impl Term<'a>) -> bool {
        term.get_head_symbol() == **self.data_function_symbol
    }

    pub fn is_data_machine_number<'a>(&self, term: &impl Term<'a>) -> bool {
        term.is_int()
    }

    pub fn is_data_where_clause<'a>(&self, term: &impl Term<'a>) -> bool {
        term.get_head_symbol() == **self.data_where_clause
    }

    pub fn is_data_abstraction<'a>(&self, term: &impl Term<'a>) -> bool {
        term.get_head_symbol() == **self.data_abstraction
    }

    /// Returns true iff the given term is a data application.
    pub fn is_data_application<'a>(&mut self, term: &impl Term<'a>) -> bool {
        term.get_head_symbol() == *self.get_data_application_symbol(term.get_head_symbol().arity())
    }

    pub fn get_data_application_symbol(&mut self, arity: usize) -> &SymbolRef<'_> {
        // It can be that data_applications are created without create_data_application in the mcrl2 ffi.
        while self.data_appl.len() <= arity {
            let symbol = Symbol::new("DataAppl", self.data_appl.len());

            self.data_appl.push(symbol);
        }

        &self.data_appl[arity]
    }
}

pub fn is_sort_expression<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_sort_expression(term))
}

pub fn is_bool_sort<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_bool_sort(term))
}

pub fn is_data_variable<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_variable(term))
}

pub fn is_data_expression<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow_mut(|ds| ds.is_data_expression(term))
}

pub fn is_data_function_symbol<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_function_symbol(term))
}

pub fn is_data_machine_number<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_machine_number(term))
}

pub fn is_data_where_clause<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_where_clause(term))
}

pub fn is_data_abstraction<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow(|ds| ds.is_data_abstraction(term))
}

pub fn is_data_application<'a>(term: &impl Term<'a>) -> bool {
    DATA_SYMBOLS.with_borrow_mut(|ds| ds.is_data_application(term))
}

pub fn get_data_function_symbol_index<'a>(_term: &impl Term<'a>) -> usize {
    unimplemented!()
}
