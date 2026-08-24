#[cfg(debug_assertions)]
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::Hash;
use std::mem::transmute;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use merc_unsafety::ProtectionIndex;
use merc_utilities::PhantomUnsend;

use crate::Markable;
use crate::Symb;
use crate::SymbolRef;
use crate::Term;
use crate::Transmutable;
use crate::aterm::ATermRef;
use crate::storage::GcMutex;
use crate::storage::GcMutexGuard;
use crate::storage::GcMutexReadGuard;
use crate::storage::SendContainerProtectionSet;
use crate::storage::THREAD_TERM_POOL;

/// A container of objects, typically either terms or objects containing terms,
/// that implement [Markable]. These store [ATermRef]`<'static>` values that are
/// protected during garbage collection by being in the container itself.
pub struct Protected<C> {
    container: Arc<GcMutex<C>>,
    root: ProtectionIndex,

    // Protected is not Send because it uses thread-local state for its protection
    // mechanism.
    _unsend: PhantomUnsend,
}

impl<C: Markable + Send + Sync + Transmutable + 'static> Protected<C> {
    /// Creates a new Protected container from a given container.
    pub fn new(container: C) -> Protected<C> {
        let shared = Arc::new(GcMutex::new(container));

        let root = THREAD_TERM_POOL.with(|tp| tp.protect_container(shared.clone()));

        Protected {
            container: shared,
            root,
            _unsend: Default::default(),
        }
    }

    /// Provides mutable access to the underlying container, returning a [ProtectedWriteGuard].
    pub fn write(&mut self) -> ProtectedWriteGuard<'_, C> {
        // SAFETY: Protected is `!Send` so it is only ever used from one thread,
        // and `write` takes `&mut self`, so no other guard from this handle
        // overlaps. The only other access is the global garbage collector,
        // which only accesses the container when this handle is dropped, so it
        // cannot overlap either.
        let mutex = unsafe { &mut *(Arc::as_ptr(&self.container) as *mut GcMutex<C>) };
        ProtectedWriteGuard::new(mutex.lock_mut())
    }

    /// Provides immutable access to the underlying container, returning a [ProtectedReadGuard].
    pub fn read(&self) -> ProtectedReadGuard<'_, C> {
        ProtectedReadGuard::new(self.container.lock())
    }
}

impl<C: Default + Markable + Send + Sync + Transmutable + 'static> Default for Protected<C> {
    fn default() -> Self {
        Protected::new(Default::default())
    }
}

impl<C: Clone + Markable + Send + Sync + Transmutable + 'static> Clone for Protected<C> {
    fn clone(&self) -> Self {
        Protected::new(self.container.lock().clone())
    }
}

impl<C: Hash + Markable> Hash for Protected<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.container.lock().hash(state)
    }
}

impl<C: PartialEq + Markable> PartialEq for Protected<C> {
    fn eq(&self, other: &Self) -> bool {
        self.container.lock().eq(&other.container.lock())
    }
}

impl<C: PartialOrd + Markable> PartialOrd for Protected<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let c: &C = &other.container.lock();
        self.container.lock().partial_cmp(c)
    }
}

impl<C: Debug + Markable> Debug for Protected<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c: &C = &self.container.lock();
        write!(f, "{c:?}")
    }
}

impl<C: Eq + PartialEq + Markable> Eq for Protected<C> {}
impl<C: Ord + PartialEq + PartialOrd + Markable> Ord for Protected<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let c: &C = &other.container.lock();
        self.container.lock().partial_cmp(c).unwrap()
    }
}

impl<C> Drop for Protected<C> {
    fn drop(&mut self) {
        THREAD_TERM_POOL.with(|tp| {
            tp.drop_container(self.root);
        });
    }
}

/// A [`Protected`]-like container that can be sent to, and used from, a
/// different thread than the one that created it.
///
/// # Details
///
/// [`Protected`] registers its container in the *calling* thread's own
/// protection set and later unregisters it through that same thread-local
/// lookup, so it can only ever be dropped on the thread that created it.
/// Instead, this type registers its container in a *globally-scanned*
/// protection set, so it can implement `Send`.
pub struct ProtectedSend<C> {
    container: Arc<GcMutex<C>>,
    root: ProtectionIndex,

    /// A shared handle to the protection set this container was registered in,
    /// kept.
    protection_set: SendContainerProtectionSet,
}

impl<C: Markable + Send + Sync + Transmutable + 'static> ProtectedSend<C> {
    /// Creates a new protected container from a given container.
    pub fn new(container: C) -> ProtectedSend<C> {
        let shared = Arc::new(GcMutex::new(container));

        let protection_set = THREAD_TERM_POOL.with(|tp| tp.send_container_protection_set().clone());
        // Inserting the clone into the (globally-scanned) send-container protection set makes it
        // a GC root until this `GlobalProtected` is dropped.
        let root = protection_set.lock().expect("Lock poisoned!").protect(shared.clone());

        ProtectedSend {
            container: shared,
            root,
            protection_set,
        }
    }

    /// Provides mutable access to the underlying container, returning a [ProtectedWriteGuard].
    pub fn write(&mut self) -> ProtectedWriteGuard<'_, C> {
        // SAFETY: `write` takes `&mut self`, so no other guard from this handle overlaps, and
        // ownership (hence any access) is confined to a single thread at a time. The only other
        // access is the global garbage collector, which -- as for `Protected` -- only reaches
        // the container's contents through `GcMutex::lock`, so it cannot overlap either.
        let mutex = unsafe { &mut *(Arc::as_ptr(&self.container) as *mut GcMutex<C>) };
        ProtectedWriteGuard::new(mutex.lock_mut())
    }

    /// Provides immutable access to the underlying container, returning a [ProtectedReadGuard].
    pub fn read(&self) -> ProtectedReadGuard<'_, C> {
        ProtectedReadGuard::new(self.container.lock())
    }
}

impl<C: Default + Markable + Send + Sync + Transmutable + 'static> Default for ProtectedSend<C> {
    fn default() -> Self {
        ProtectedSend::new(Default::default())
    }
}

impl<C> Drop for ProtectedSend<C> {
    fn drop(&mut self) {
        let mut guard = match self.protection_set.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        // SAFETY: `self.root` was protected when this instance was created and
        // `Drop` runs exactly once, so the root is unprotected exactly once.
        unsafe {
            guard.unprotect(self.root);
        }
    }
}

pub struct ProtectedWriteGuard<'a, C: Markable> {
    reference: GcMutexGuard<'a, C>,

    /// Terms that have been protected during the lifetime of this guard.
    #[cfg(debug_assertions)]
    protected: RefCell<Vec<ATermRef<'static>>>,

    /// Symbols that have been protected during the lifetime of this guard.
    #[cfg(debug_assertions)]
    protected_symbols: RefCell<Vec<SymbolRef<'static>>>,
}

impl<'a, C: Markable> ProtectedWriteGuard<'a, C> {
    fn new(reference: GcMutexGuard<'a, C>) -> Self {
        #[cfg(debug_assertions)]
        return ProtectedWriteGuard {
            reference,
            protected: RefCell::new(vec![]),
            protected_symbols: RefCell::new(vec![]),
        };

        #[cfg(not(debug_assertions))]
        return ProtectedWriteGuard { reference };
    }

    /// Yields a term to insert into the container.
    ///
    /// # Safety
    ///
    /// The invariant to uphold is that the resulting term MUST be inserted into
    /// the container. This is checked in debug mode, but not in release mode.
    /// If this invariant is violated, undefined behaviour may occur during
    /// garbage collection.
    pub unsafe fn protect<'b, T: Term<'a, 'b>>(&self, term: &'b T) -> ATermRef<'static> {
        unsafe {
            // Store terms that are marked as protected to check if they are
            // actually in the container when the protection is dropped.
            #[cfg(debug_assertions)]
            self.protected
                .borrow_mut()
                .push(transmute::<ATermRef<'_>, ATermRef<'static>>(term.copy()));

            transmute::<ATermRef<'_>, ATermRef<'static>>(term.copy())
        }
    }

    /// Yields a symbol to insert into the container.
    ///
    /// # Safety
    ///
    /// The invariant to uphold is that the resulting symbol MUST be inserted
    /// into the container.
    pub unsafe fn protect_symbol<'b, S: Symb<'a, 'b>>(&self, symbol: &'b S) -> SymbolRef<'static> {
        unsafe {
            // Store symbols that are marked as protected to check if they are
            // actually in the container when the protection is dropped.
            #[cfg(debug_assertions)]
            self.protected_symbols
                .borrow_mut()
                .push(transmute::<SymbolRef<'_>, SymbolRef<'static>>(symbol.copy()));

            transmute::<SymbolRef<'_>, SymbolRef<'static>>(symbol.copy())
        }
    }
}

#[cfg(debug_assertions)]
impl<C: Markable> Drop for ProtectedWriteGuard<'_, C> {
    fn drop(&mut self) {
        {
            for term in self.protected.borrow().iter() {
                debug_assert!(
                    self.reference.contains_term(term),
                    "Term was protected but not actually inserted"
                );
            }

            for symbol in self.protected_symbols.borrow().iter() {
                debug_assert!(
                    self.reference.contains_symbol(symbol),
                    "Symbol was protected but not actually inserted"
                );
            }
        }
    }
}

impl<'a, C: Markable + Transmutable + 'a> Deref for ProtectedWriteGuard<'a, C> {
    type Target = C::Target<'a>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: 'a is the lifetime of the underlying lock guard, which `self` borrows.
        unsafe { self.reference.transmute_lifetime() }
    }
}

impl<C: Markable + Transmutable> DerefMut for ProtectedWriteGuard<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: 'a is the lifetime of the underlying lock guard, which `self` borrows.
        unsafe { self.reference.deref_mut().transmute_lifetime_mut() }
    }
}

pub struct ProtectedReadGuard<'a, C> {
    reference: GcMutexReadGuard<'a, C>,
}

impl<'a, C> ProtectedReadGuard<'a, C> {
    fn new(reference: GcMutexReadGuard<'a, C>) -> Self {
        Self { reference }
    }
}

impl<'a, C: Transmutable> Deref for ProtectedReadGuard<'a, C> {
    type Target = C::Target<'a>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: 'a is the lifetime of the underlying lock guard, which `self` borrows.
        unsafe { self.reference.transmute_lifetime() }
    }
}

#[cfg(test)]
mod tests {
    use crate::ATerm;
    use crate::ATermRef;
    use crate::Protected;

    #[test]
    fn test_aterm_container() {
        merc_utilities::test_logger();

        let t = ATerm::from_string("f(g(a),b)").unwrap();

        // First test the trait for a standard container.
        let mut container = Protected::<Vec<ATermRef<'static>>>::new(vec![]);

        for _ in 0..1000 {
            let mut write = container.write();
            write.push(t.get());
        }
    }
}
