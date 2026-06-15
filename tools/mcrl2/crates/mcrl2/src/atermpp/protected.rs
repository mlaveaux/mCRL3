//! A garbage-collection-safe wrapper that keeps the terms stored in a container
//! alive.
//!
//! Explicit state-space exploration interns the data expressions observed for
//! each state to dense `usize` indices, so state vectors can be passed around as
//! plain integers. When the exploration runs on several threads this interning
//! must be shared, which is awkward with the usual [`ATerm`](crate::ATerm): an
//! `ATerm` carries a *thread-local* protection root and is therefore `!Send`, so
//! it cannot be created on one worker and dropped on another.
//!
//! Instead the interning stores bare [`DataExpressionRef<'static>`] references
//! (an `ATermRef` has no `Drop` and is `Send`/`Sync`) inside a thread-safe
//! [`ConcurrentIndexedSet`], and keeps every stored term alive by registering
//! the set as a [`Markable`] garbage-collection container through
//! [`Protected`]. Garbage collection is stop-the-world, so marking the container
//! never races with the workers that insert into it.

use std::ops::Deref;
use std::sync::Arc;

use merc_unsafety::ConcurrentIndexedSet;
use merc_unsafety::ProtectionIndex;
use merc_utilities::PhantomUnsend;

use super::aterm::ATermRef;
use super::markable::Markable;
use super::markable::Todo;
use super::thread_aterm_pool::THREAD_TERM_POOL;

/// Lets any concurrent set whose values are themselves [`Markable`] act as a
/// garbage-collection container: every interned value is marked so the collector
/// keeps the terms it holds alive.
///
/// The hasher `S` is irrelevant to marking, so the impl is generic over it as
/// well; the interning workloads (the LPS/PBES value mappings storing
/// [`DataExpressionRef<'static>`](crate::DataExpressionRef)) are the
/// `T = DataExpressionRef<'static>` instance of this blanket impl.
impl<T: Markable, S> Markable for ConcurrentIndexedSet<T, S> {
    fn mark(&self, mut todo: Todo) {
        // Indices are dense `0..len`, so iterating by index visits every
        // interned value. Stop-the-world collection means `len` is stable here.
        for index in 0..ConcurrentIndexedSet::len(self) {
            if let Some(value) = self.get_by_index(index) {
                value.mark(todo.as_mut());
            }
        }
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        (0..ConcurrentIndexedSet::len(self))
            .any(|index| self.get_by_index(index).is_some_and(|v| v.contains_term(term)))
    }

    fn len(&self) -> usize {
        ConcurrentIndexedSet::len(self)
    }
}

/// An owning wrapper that keeps the terms held by a [`Markable`] container `T`
/// alive for as long as the wrapper lives.
///
/// Registers `T` as a garbage-collection container on the thread that
/// constructs it; the registration is released on `Drop`. Because both happen
/// through the thread-local protection set, a `Protected` is `!Send` and must be
/// created and dropped on the same thread. It is still `Sync`, so an immutable
/// reference can be shared with worker threads, and the [`Arc`] handles handed
/// out by [`Protected::handle`] are freely `Send`/`Sync`.
pub struct Protected<T: Markable + Send + Sync + 'static> {
    inner: Arc<T>,
    root: ProtectionIndex,
    _unsend: PhantomUnsend,
}

impl<T: Markable + Send + Sync + 'static> Protected<T> {
    /// Wraps `inner` and registers it as a garbage-collection container on the
    /// current thread.
    pub fn new(inner: T) -> Protected<T> {
        let inner = Arc::new(inner);
        let root = THREAD_TERM_POOL
            .with_borrow(|pool| pool.protect_container(inner.clone() as Arc<dyn Markable + Send + Sync>));

        Protected {
            inner,
            root,
            _unsend: PhantomUnsend::default(),
        }
    }

    /// Returns a `Send`/`Sync` handle that keeps the container alive and can be
    /// used to insert from worker threads. The handle holds no protection root,
    /// so it can be moved to and dropped on any thread.
    pub fn handle(&self) -> Arc<T> {
        self.inner.clone()
    }
}

impl<T: Markable + Send + Sync + 'static> Deref for Protected<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: Markable + Send + Sync + 'static> Drop for Protected<T> {
    fn drop(&mut self) {
        THREAD_TERM_POOL.with_borrow(|pool| pool.drop_container(self.root));
    }
}
