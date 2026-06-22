//! Small, deterministic checks of the term pool's unsafe paths — construction, shared-pointer
//! identity, argument access, protection across garbage collection, and the `Send` wrapper.
//!
//! Unlike the randomized stress tests (which are `#[cfg_attr(miri, ignore)]` because they are far
//! too slow), these are cheap enough to run under miri, so they exercise the pointer/transmute and
//! protection-set code with Stacked/Tree Borrows checking.

use merc_aterm::ATerm;
use merc_aterm::ATermSend;
use merc_aterm::Symb;
use merc_aterm::Symbol;
use merc_aterm::Term;
use merc_aterm::storage::THREAD_TERM_POOL;

/// Builds the term `f(a, g(a))` from freshly created symbols on every call.
fn build_sample() -> ATerm {
    let a = ATerm::constant(&Symbol::new("a", 0));
    let g = ATerm::with_args(&Symbol::new("g", 1), &[a.copy()]).protect();
    ATerm::with_args(&Symbol::new("f", 2), &[a.copy(), g.copy()]).protect()
}

#[test]
fn test_miri_maximal_sharing() {
    // Two structurally equal terms must resolve to the same shared node: structural equality
    // implies pointer (index) equality.
    let first = build_sample();
    let second = build_sample();
    assert_eq!(first, second);
    assert_eq!(
        first.index(),
        second.index(),
        "maximal sharing should reuse the same node"
    );

    // A structurally different term is a distinct node.
    let other = ATerm::constant(&Symbol::new("a", 0));
    assert_ne!(first.index(), other.index());
}

#[test]
fn test_miri_term_arguments() {
    let term = build_sample();

    assert_eq!(term.get_head_symbol().name(), "f");
    assert_eq!(term.get_head_symbol().arity(), 2);
    assert_eq!(term.arguments().len(), 2);

    // arg(0) is the constant `a`.
    let arg0 = term.arg(0);
    assert_eq!(arg0.get_head_symbol().name(), "a");
    assert_eq!(arg0.arguments().len(), 0);

    // arg(1) is `g(a)`, whose only argument is again `a`, shared with arg(0).
    let arg1 = term.arg(1);
    assert_eq!(arg1.get_head_symbol().name(), "g");
    let nested = arg1.arg(0);
    assert_eq!(nested.get_head_symbol().name(), "a");
    assert_eq!(
        nested.index(),
        arg0.index(),
        "the inner `a` is the same shared node as arg(0)"
    );
}

#[test]
fn test_miri_protection_survives_gc() {
    // A protected term must survive forced garbage collection with its structure intact.
    let term = build_sample();

    THREAD_TERM_POOL.with(|tp| {
        tp.force_collect_garbage();
        tp.force_collect_garbage();
    });

    assert_eq!(term.get_head_symbol().name(), "f");
    assert_eq!(term.arg(1).arg(0).get_head_symbol().name(), "a");

    // Rebuilding the same term after GC still yields the same shared node.
    assert_eq!(term.index(), build_sample().index());
}

#[test]
fn test_miri_send_roundtrip() {
    // The `Send` wrapper keeps its term alive across GC, even on a single thread, and converting
    // back to a protected `ATerm` yields the original shared node.
    let term = build_sample();
    let send = ATermSend::from(build_sample());

    THREAD_TERM_POOL.with(|tp| tp.force_collect_garbage());

    assert_eq!(send.get_head_symbol().name(), "f");
    assert_eq!(send.protect().index(), term.index());
}

#[test]
fn test_miri_term_iterator() {
    // The subterm iterator traverses every edge without deduplicating shared nodes, so `f(a, g(a))`
    // visits f, a, g(a) and the shared `a` again: four nodes in total.
    let term = build_sample();
    assert_eq!(term.iter().count(), 4);
}

/// `ATermArgs` previously did not override `size_hint`, so its lower bound was 0
/// and there was no upper bound. Adapters such as `Skip` and `zip` use `size_hint`
/// to implement their own `ExactSizeIterator::len`, so without the fix they would
/// return 0 regardless of the actual remaining count.
#[test]
fn test_aterm_args_size_hint_is_exact() {
    // `h(a, b, c)` has arity 3, giving a three-element `ATermArgs` iterator.
    let a = ATerm::constant(&Symbol::new("a_sh", 0));
    let b = ATerm::constant(&Symbol::new("b_sh", 0));
    let c = ATerm::constant(&Symbol::new("c_sh", 0));
    let term = ATerm::with_args(&Symbol::new("h_sh", 3), &[a.copy(), b.copy(), c.copy()]).protect();

    // Fresh iterator: all 3 arguments remain.
    let mut iter = term.arguments();
    assert_eq!(iter.size_hint(), (3, Some(3)), "size_hint before advancing");
    assert_eq!(iter.len(), 3, "ExactSizeIterator::len before advancing");

    // Consume one argument; 2 remain.
    iter.next();
    assert_eq!(iter.size_hint(), (2, Some(2)), "size_hint after one next()");
    assert_eq!(iter.len(), 2, "ExactSizeIterator::len after one next()");

    // `skip` is built on top of `size_hint`; with the fix it must still report the
    // correct remaining length.
    let after_skip = term.arguments().skip(1);
    assert_eq!(
        after_skip.len(),
        2,
        "skip(1) on a 3-argument iterator must report len 2"
    );
}
