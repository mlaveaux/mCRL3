#![allow(non_camel_case_types)]

use std::ffi::c_char;
use std::ffi::CStr;
use std::mem;
use std::ops::Deref;
use std::ptr;
use std::ptr::NonNull;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use mcrl3_aterm::is_empty_list_term;
use mcrl3_aterm::is_int_term;
use mcrl3_aterm::is_list_term;
use mcrl3_aterm::ATermIndex;
use mcrl3_aterm::ATermInt;
use mcrl3_aterm::ATermList;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::SharedTerm;
use mcrl3_aterm::Symb;
use mcrl3_aterm::SymbolRef;
use mcrl3_aterm::Term;
use mcrl3_aterm::TermOrAnnotation;
use mcrl3_aterm::GLOBAL_TERM_POOL;
use mcrl3_aterm::THREAD_TERM_POOL;

/// The is the underlying shared aterm that is pointed to by the term.
#[repr(C)]
pub struct unprotected_aterm_t {
    ptr: *const std::ffi::c_void,
}

/// This keeps track of the root index for a term.
#[repr(C)]
pub struct root_index_t {
    index: usize,
}

/// This is a pair that is used as return value for some functions.
#[repr(C)]
pub struct aterm_t {
    term: unprotected_aterm_t,
    root: root_index_t,
}

/// The pointer to a shared function symbol, and the root index for its protection.
#[repr(C)]
pub struct function_symbol_t {
    ptr: *const std::ffi::c_void,
    root: root_index_t,
}

/// Returns true iff the term is an integer term.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_is_int(term: unprotected_aterm_t) -> bool {
    unsafe {
        is_int_term(&term_to_aterm_ref(term.ptr))
    }
}

/// Returns true iff the term is a list term.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_is_list(term: unprotected_aterm_t) -> bool {
    unsafe {
        is_list_term(&term_to_aterm_ref(term.ptr))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_empty_list() -> unprotected_aterm_t {
    let empty_list = ATermList::<()>::empty();

    unprotected_aterm_t {
        ptr: empty_list.shared().deref() as *const SharedTerm as *const std::ffi::c_void,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_is_empty_list(term: unprotected_aterm_t) -> bool {
    unsafe {
        is_empty_list_term(&term_to_aterm_ref(term.ptr))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_is_defined(term: unprotected_aterm_t) -> bool {
    term.ptr.is_null() == false
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_create_int(value: usize) -> aterm_t {
    let term = ATermInt::new(value);
    
    let term_ptr = term.shared().deref() as *const SharedTerm as *const std::ffi::c_void;
    let root = *term.root().deref();

    aterm_t {
        term: unprotected_aterm_t { ptr: term_ptr },
        root: root_index_t { index: root },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_get_int_value(term: unprotected_aterm_t) -> usize {
    unsafe {
        let shared_term = term_to_aterm_ref(term.ptr);
        if is_int_term(&shared_term) {
            shared_term.annotation().unwrap_or(0)
        } else {
            0 // or handle error
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_protect(term: unprotected_aterm_t) -> root_index_t {
    THREAD_TERM_POOL.with_borrow(|tp| {
        let term = unsafe { tp.protect(&term_to_aterm_ref(term.ptr)) };
        let root = term.root();
        std::mem::forget(term); // Prevent the term from being dropped
        root_index_t { index: *root.deref() }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_unprotect(root: root_index_t) {
    unimplemented!();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_get_argument(term: unprotected_aterm_t, index: usize) -> unprotected_aterm_t {
    unimplemented!();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_create_appl(symbol: function_symbol_t, arguments: *const unprotected_aterm_t, num_arguments: usize) -> aterm_t {
    unimplemented!();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn function_symbol_register_prefix(
    prefix: *const c_char,
    _length: usize
) -> prefix_shared_counter_t {
    let result = GLOBAL_TERM_POOL.write().register_prefix(
        unsafe { CStr::from_ptr(prefix).to_str().expect("Invalid UTF-8 in prefix") },
    );

    prefix_shared_counter_t {
        ptr: Arc::into_raw(result) as *const std::ffi::c_void,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn function_symbol_deregister_prefix(
    prefix: *const std::ffi::c_char,
    length: usize,
) {
    GLOBAL_TERM_POOL.write().remove_prefix(
        unsafe { CStr::from_ptr(prefix).to_str().expect("Invalid UTF-8 in prefix") },
    );
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn function_symbol_is_int(symbol: function_symbol_t) -> bool {
    unimplemented!();
}

/// This is a counter that is used to keep track of the number of references to
/// a prefix.
/// 
/// This is used because Arc is not available in the FFI, so we use a raw
/// pointer to the counter (which is stable because it is an Arc).
#[repr(C)]
pub struct prefix_shared_counter_t {
    ptr: *const std::ffi::c_void,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn shared_counter_value(
    counter: prefix_shared_counter_t,
) -> usize{
    unsafe {
        counter.ptr.cast::<AtomicUsize>()
            .as_ref()
            .expect("Counter pointer is not null")
            .load(Ordering::Relaxed)
    }
}

/// Increases the reference count of the shared counter.
/// 
/// # Safety
/// 
/// The given counter must be a valid pointer that has not been released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shared_counter_add_ref(
    counter: prefix_shared_counter_t,
) {
    unsafe {
        // Clone the Arc to increment the reference count, but forgot it to avoid dropping it.
        let result = Arc::from_raw(counter.ptr.cast::<AtomicUsize>());
        mem::forget(result.clone());
        mem::forget(result);
    }
}

/// Decreases the reference count of the shared counter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shared_counter_unref(
    counter: prefix_shared_counter_t,
) {
    unsafe {
        // Construct the Arc and drop it to decrement the reference count.
        Arc::from_raw(counter.ptr.cast::<AtomicUsize>());
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_get_function_symbol(term: unprotected_aterm_t) -> function_symbol_t {
    unimplemented!();
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn function_symbol_create(name: *const std::ffi::c_char, length: usize, arity: usize, check_for_registered_functions: bool) -> function_symbol_t {
    unimplemented!();
}

type term_deletion_hook_t = extern "C" fn(symbol: unprotected_aterm_t);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn register_deletion_hook(
    symbol: &function_symbol_t,
    deletion_hook: term_deletion_hook_t,
) {
    unimplemented!();
    // GLOBAL_TERM_POOL.write().register_deletion_hook(|term| {
    //     deletion_hook(&unprotected_aterm_t {
    //         ptr: term.shared().deref() as *const SharedTerm as *const std::ffi::c_void,
    //     });
    // });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_pool_is_busy_set() -> bool {
    unimplemented!();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn function_symbol_get_arity(symbol: function_symbol_t) -> usize {
    unsafe {
        let shared_term = term_to_aterm_ref(symbol.ptr);
        shared_term.get_head_symbol().arity() // Assuming arity gives the length of the term
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn function_symbol_get_name(symbol: function_symbol_t) -> *const std::ffi::c_char {
    unsafe {
        let shared_term = term_to_aterm_ref(symbol.ptr);
        shared_term.get_head_symbol().name().as_ptr() as *const std::ffi::c_char
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn container_protect() -> root_index_t {
    unimplemented!();
    // THREAD_TERM_POOL.with_borrow(|tp| {
    //     let root = tp.protect_container();
    //     root_index_t { index: *root.deref() }
    // })
}
    
#[unsafe(no_mangle)]
pub unsafe extern "C" fn container_unprotect(root: root_index_t) {
    unimplemented!();
    // THREAD_TERM_POOL.with_borrow(|tp| {
    //     let root = tp.protect_container();
    //     root_index_t { index: *root.deref() }
    // })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn global_lock_shared() {
    // Forget the guard to prevent it from being dropped.
    mem::forget(GLOBAL_TERM_POOL.read());
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn global_unlock_shared() {
    unsafe { GLOBAL_TERM_POOL.force_unlock_read() };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn global_lock_exclusive() {
    // Forget the guard to prevent it from being dropped.
    mem::forget(GLOBAL_TERM_POOL.write());
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn global_unlock_exclusive() {
    unsafe { GLOBAL_TERM_POOL.force_unlock_write() };
}

/// Can be used during garbage collection to mark a term (and all of its subterms) as being reachable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn term_mark(term: &unprotected_aterm_t) {
    unimplemented!();
}

/// Returns the number of arguments in the term.
unsafe fn term_len(term: unprotected_aterm_t) -> usize {
    // Assuming the pointer is to a SharedTerm, we can get the length from the SharedTerm.
    unsafe {
        let symbol: SymbolRef = ptr::read(term.ptr.cast());
        symbol.arity() // Assuming arity gives the length of the term
    }
}

/// Converts a raw pointer to an `ATermRef`, must ensure that the raw ptr is valid.
/// 
/// Safety: The unprotected_aterm_t must point to a valid term.
unsafe fn term_to_aterm_ref(term: unprotected_aterm_t) -> ATermRef<'static> {
    unsafe {
        let wide_ptr = ptr::slice_from_raw_parts(term.ptr as *const TermOrAnnotation, term_len(ptr));
        ATermRef::from_index(&ATermIndex::from_ptr(NonNull::new_unchecked(wide_ptr as *mut SharedTerm)))
    }
}
