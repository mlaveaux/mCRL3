use std::fmt::Debug;
use std::hash::Hash;
use std::mem::transmute;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use mcrl3_utilities::PhantomUnsend;

use crate::gc_mutex::GcMutex;
use crate::gc_mutex::GcMutexGuard;
use crate::Marker;
use crate::THREAD_TERM_POOL;
use crate::Term;
use crate::aterm::ATermRef;

#[cfg(debug_assertions)]
use std::cell::RefCell;

/// A container of objects, typically either terms or objects containing terms,
/// that are of trait Markable. These store ATermRef<'static> that are protected
/// during garbage collection by being in the container itself.
pub struct Protected<C> {
    container: Arc<GcMutex<C>>,
    root: usize,

    // Protected is not Send because it uses thread-local state for its protection
    // mechanism.
    _unsend: PhantomUnsend,
}

impl<C: Markable + Send + 'static> Protected<C> {
    /// Creates a new Protected container from a given container.
    pub fn new(container: C) -> Protected<C> {
        let shared = Arc::new(GcMutex::new(container));

        let root = THREAD_TERM_POOL.with_borrow(|tp| tp.protect_container(shared.clone()));

        Protected {
            container: shared,
            root,
            _unsend: Default::default(),
        }
    }

    /// Provides mutable access to the underlying container. Use [Protector::protect] of
    /// the resulting guard to be able to insert terms into the container.
    /// Otherwise the borrow checker will note that the [ATermRef] do not
    /// outlive the guard, see [Protector].
    pub fn write(&mut self) -> Protector<'_, C> {
        // The lifetime of ATermRef can be derived from self since it is protected by self, so transmute 'static into 'a.
        Protector::new(self.container.write())
    }

    /// Provides immutable access to the underlying container.
    pub fn read(&self) -> GcMutexGuard<'_, C> {
        // The lifetime of ATermRef can be derived from self since it is protected by self, so transmute 'static into 'a.
        self.container.read()
    }
}

impl<C: Default + Markable + Send + 'static> Default for Protected<C> {
    fn default() -> Self {
        Protected::new(Default::default())
    }
}

impl<C: Clone + Markable + Send + 'static> Clone for Protected<C> {
    fn clone(&self) -> Self {
        Protected::new(self.container.read().clone())
    }
}

impl<C: Hash + Markable> Hash for Protected<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.container.read().hash(state)
    }
}

impl<C: PartialEq + Markable> PartialEq for Protected<C> {
    fn eq(&self, other: &Self) -> bool {
        self.container.read().eq(&other.container.read())
    }
}

impl<C: PartialOrd + Markable> PartialOrd for Protected<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let c: &C = &other.container.read();
        self.container.read().partial_cmp(c)
    }
}

impl<C: Debug + Markable> Debug for Protected<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c: &C = &self.container.read();
        write!(f, "{:?}", c)
    }
}

impl<C: Eq + PartialEq + Markable> Eq for Protected<C> {}
impl<C: Ord + PartialEq + PartialOrd + Markable> Ord for Protected<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let c: &C = &other.container.read();
        self.container.read().partial_cmp(c).unwrap()
    }
}

impl<C> Drop for Protected<C> {
    fn drop(&mut self) {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.drop_container(self.root);
        });
    }
}

/// This trait should be used on all objects and containers related to storing unprotected terms.
pub trait Markable {
    /// Marks all the ATermRefs to prevent them from being garbage collected.
    fn mark(&self, marker: &mut Marker);

    /// Should return true iff the given term is contained in the object. Used for runtime checks.
    fn contains_term(&self, term: &ATermRef<'_>) -> bool;

    /// Returns the number of terms in the instance, used to delay garbage collection.
    fn len(&self) -> usize;

    /// Returns true iff the container is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Markable> Markable for Vec<T> {
    fn mark(&self, marker: &mut Marker) {
        for value in self {
            value.mark(marker);
        }
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        self.iter().any(|v| v.contains_term(term))
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl<T: Markable> Markable for GcMutex<T> {
    fn mark(&self, marker: &mut Marker) {
        self.write().mark(marker);
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        self.read().contains_term(term)
    }

    fn len(&self) -> usize {
        self.read().len()
    }
}

impl<T: Markable> Markable for Option<T> {
    fn mark(&self, marker: &mut Marker) {
        if let Some(value) = self {
            value.mark(marker);
        }
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        if let Some(value) = self {
            value.contains_term(term)
        } else {
            false
        }
    }

    fn len(&self) -> usize {
        if let Some(value) = self {
            value.len()
        } else {
            0
        }
    }
}

// Vec<T> -> Vec<U>

/// This is a helper struct used by TermContainer to protected terms that are
/// inserted into the container before the guard is dropped.
///
/// The reason is that the ATermRef derive their lifetime from the
/// TermContainer. However, when inserting terms with shorter lifetimes we know
/// that their lifetime is extended by being in the container. This is enforced
/// by runtime checks during debug for containers that implement IntoIterator.
pub struct Protector<'a, C: Markable> {
    reference: GcMutexGuard<'a, C>,

    #[cfg(debug_assertions)]
    protected: RefCell<Vec<ATermRef<'static>>>,
}

impl<'a, C: Markable> Protector<'a, C> {
    fn new(reference: GcMutexGuard<'a, C>) -> Protector<'a, C> {
        #[cfg(debug_assertions)]
        return Protector {
            reference,
            protected: RefCell::new(vec![]),
        };

        #[cfg(not(debug_assertions))]
        return Protector { reference };
    }

    /// Yields a term to insert into the container.
    ///
    /// The invariant to uphold is that the resulting term MUST be inserted into the container.
    pub fn protect(&self, term: &impl Term<'a>) -> ATermRef<'static> {
        unsafe {
            // Store terms that are marked as protected to check if they are
            // actually in the container when the protection is dropped.
            #[cfg(debug_assertions)]
            self.protected
                .borrow_mut()
                .push(transmute::<ATermRef<'_>, ATermRef<'static>>(term.copy()));

            transmute(term.copy())
        }
    }
}

/// TODO: Add trait to convert ATermRef<'static> to ATerm<'a>

impl<C: Markable> Deref for Protector<'_, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.reference
    }
}

impl<C: Markable> DerefMut for Protector<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.reference.deref_mut()
    }
}

impl<C: Markable> Drop for Protector<'_, C> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            for term in self.protected.borrow().iter() {
                debug_assert!(
                    self.reference.contains_term(term),
                    "Term was protected but not actually inserted"
                );
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::ATerm;

    use super::*;

    #[test]
    fn test_aterm_container() {
        let _ = mcrl3_utilities::test_logger();

        let t = ATerm::from_string("f(g(a),b)").unwrap();

        // First test the trait for a standard container.
        let mut container = Protected::<Vec<ATermRef>>::new(vec![]);

        for _ in 0..1000 {
            let mut write = container.write();
            let u = write.protect(&t);
            write.push(u);
        }
    }
}
