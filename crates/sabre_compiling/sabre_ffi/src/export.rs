//! In debug mode the FFI types are not FFI-safe, but in release builds they
//! are.
#![allow(improper_ctypes_definitions)]

use std::ffi::c_void;

use merc_aterm::ATermRef;
use merc_aterm::Term;
use merc_aterm::storage::GlobalTermPool;
use merc_aterm::storage::THREAD_TERM_POOL;
use merc_data::DataExpressionRef;
use merc_sharedmutex::GlobalBfSharedMutex;

use crate::DataExpressionFFI;
use crate::DataExpressionRefFFI;
use crate::DataFunctionSymbolRefFFI;

/// # Safety
///
/// See the documentation in the import module.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn initialize_thread_local_term_pool(global_term_pool: *mut c_void) {
    unsafe {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.set_global_term_pool(global_term_pool as *mut GlobalBfSharedMutex<GlobalTermPool>)
        });
    }
}

/// # Safety
///
/// See the documentation in the import module.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn data_expression_arg<'a>(
    term: &DataExpressionRefFFI<'a>,
    index: usize,
) -> DataExpressionRefFFI<'a> {
    unsafe {
        DataExpressionRefFFI::from_index(
            DataExpressionRef::from(ATermRef::from_index(term.shared()))
                .data_arg(index)
                .shared(),
        )
    }
}

/// # Safety
///
/// See the documentation in the import module.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn data_expression_symbol<'a>(term: &DataExpressionRefFFI<'a>) -> DataFunctionSymbolRefFFI<'a> {
    unsafe {
        DataFunctionSymbolRefFFI::from_index(
            DataExpressionRef::from(ATermRef::from_index(term.shared()))
                .data_function_symbol()
                .shared(),
        )
    }
}

/// # Safety
///
/// See the documentation in the import module.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn data_expression_arity(term: &DataExpressionRefFFI<'_>) -> usize {
    unsafe {
        DataExpressionRef::from(ATermRef::from_index(term.shared()))
            .data_arguments()
            .len()
    }
}

/// # Safety
///
/// See the documentation in the import module.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn data_expression_protect(term: &DataExpressionRefFFI<'_>) -> DataExpressionFFI {
    unsafe {
        let t = ATermRef::from_index(term.shared()).protect();

        let d = DataExpressionFFI::from_index(t.shared(), t.root());

        // We are now responsible for the memory of the data expression.
        std::mem::forget(t);

        d
    }
}
