use mcrl3_aterm::ATermRef;
use mcrl3_aterm::Term;

use crate::DataExpression;
use crate::DataExpressionRef;
use crate::DataFunctionSymbolRef;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn data_expression_arg<'a>(term: &DataExpressionRef<'a>, index: usize) -> DataExpressionRef<'a> {
    unsafe { DataExpressionRef::from_index(ATermRef::from_index(term.index).arg(index + 1).index()) }
}

/// # Safety
///
/// The resulting data expression ref is an argument of the provided data expression.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn data_expression_symbol<'a>(term: &DataExpressionRef<'a>) -> DataFunctionSymbolRef<'a> {
    unsafe { DataFunctionSymbolRef::from_index(ATermRef::from_index(term.index).arg(0).index()) }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn data_expression_protect<'a>(term: &DataExpressionRef<'a>) -> DataExpression {
    unsafe { 
        let t = ATermRef::from_index(term.index).protect();

        let d = DataExpression::from_index(t.index(), t.root());

        // We are now responsible for the memory of the data expression.
        std::mem::forget(t);

        d
    }
}