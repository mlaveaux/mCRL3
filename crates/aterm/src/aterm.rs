use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::sync::Arc;

use delegate::delegate;

use mcrl3_unsafety::StablePointer;
use mcrl3_utilities::MCRL3Error;
use mcrl3_utilities::PhantomUnsend;
use mcrl3_utilities::ProtectionIndex;
use parking_lot::Mutex;

use crate::ATermIntRef;
use crate::Markable;
use crate::Marker;
use crate::SharedTerm;
use crate::SharedTermProtection;
use crate::Symb;
use crate::SymbolRef;
use crate::THREAD_TERM_POOL;
use crate::is_int_term;

/// The ATerm trait represents a first-order term in the ATerm library.
/// It provides methods to manipulate and access the term's properties.
pub trait Term<'a, 'b> {
    /// Protects the term from garbage collection
    fn protect(&self) -> ATerm;

    /// Returns the indexed argument of the term
    fn arg(&'b self, index: usize) -> ATermRef<'a>;

    /// Returns the list of arguments as a collection
    fn arguments(&'b self) -> ATermArgs<'a>;

    /// Makes a copy of the term with the same lifetime as itself.
    fn copy(&'b self) -> ATermRef<'a>;

    /// Returns the function of an ATermRef
    fn get_head_symbol(&'b self) -> SymbolRef<'a>;

    /// Returns an iterator over all arguments of the term that runs in pre order traversal of the term trees.
    fn iter(&'b self) -> TermIterator<'a>;

    /// Returns a unique index of the term in the term pool
    fn index(&self) -> usize;

    /// Returns the shared ptr of the term in the term pool
    fn shared(&self) -> &ATermIndex;

    /// Returns the annotation of the term
    fn annotation(&self) -> Option<usize>;
}

/// Type alias for [ATerm] indices, representing a non-zero index into the term pool.
pub type ATermIndex = StablePointer<SharedTerm>;

/// This represents a lifetime bound reference to an existing [ATerm].
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ATermRef<'a> {
    shared: ATermIndex,
    marker: PhantomData<&'a ()>,
}

/// Check that the ATermRef is the same size as a usize.
#[cfg(not(debug_assertions))]
const _: () = assert!(std::mem::size_of::<ATermRef>() == std::mem::size_of::<usize>());

/// Since we have NonZero we can use a niche value optimisation for option.
#[cfg(not(debug_assertions))]
const _: () = assert!(std::mem::size_of::<Option<ATermRef>>() == std::mem::size_of::<usize>());

/// These are safe because terms are immutable. Garbage collection is
/// always performed with exclusive access, and reference terms have no thread-local state.
unsafe impl Send for ATermRef<'_> {}
unsafe impl Sync for ATermRef<'_> {}

impl ATermRef<'_> {
    /// Creates a new term reference from the given index.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check if the index is valid for the given lifetime.
    pub unsafe fn from_index(shared: &ATermIndex) -> Self {
        ATermRef {
            shared: shared.copy(),
            marker: PhantomData,
        }
    }
}

impl<'a, 'b> Term<'a, 'b> for ATermRef<'a> {
    fn protect(&self) -> ATerm {
        THREAD_TERM_POOL.with_borrow(|tp| tp.protect(&self.copy()))
    }

    fn arg(&self, index: usize) -> ATermRef<'a> {
        debug_assert!(
            index < self.get_head_symbol().arity(),
            "arg({index}) is not defined for term {:?}",
            self
        );

        self.shared().arguments()[index].borrow().copy()
    }

    fn arguments(&self) -> ATermArgs<'a> {
        ATermArgs::new(self.copy())
    }

    fn copy(&self) -> ATermRef<'a> {
        unsafe { ATermRef::from_index(self.shared()) }
    }

    fn get_head_symbol(&'b self) -> SymbolRef<'a> {
        unsafe { std::mem::transmute::<SymbolRef<'b>, SymbolRef<'a>>(self.shared().symbol().copy()) }
    }

    fn iter(&self) -> TermIterator<'a> {
        TermIterator::new(self.copy())
    }

    fn index(&self) -> usize {
        self.shared.address()
    }

    fn shared(&self) -> &ATermIndex {
        &self.shared
    }

    fn annotation(&self) -> Option<usize> {
        self.shared().annotation()
    }
}

impl Markable for ATermRef<'_> {
    fn mark(&self, marker: &mut Marker) {
        marker.mark(self);
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        term == self
    }

    fn len(&self) -> usize {
        1
    }
}

impl fmt::Display for ATermRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for ATermRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if is_int_term(self) {
            write!(f, "{}", Into::<ATermIntRef>::into(self.copy()))?;
        } else if self.arguments().is_empty() {
            write!(f, "{}", self.get_head_symbol().name())?;
        } else {
            // Format the term with its head symbol and arguments, avoiding trailing comma
            write!(f, "{:?}(", self.get_head_symbol())?;

            let mut args = self.arguments().peekable();
            while let Some(arg) = args.next() {
                write!(f, "{:?}", arg)?;
                if args.peek().is_some() {
                    write!(f, ", ")?;
                }
            }

            write!(f, ")")?;
        }

        Ok(())
    }
}

/// The protected version of [ATermRef], mostly derived from it.
pub struct ATerm {
    term: ATermRef<'static>,

    /// The root of the term in the protection set
    root: ProtectionIndex,

    // ATerm is not Send because it uses thread-local state for its protection
    // mechanism. However, it can be Sync since terms are immutable, and unlike
    // `Rc` cloning results in a local protected copy.
    _marker: PhantomUnsend,
}

impl ATerm {
    /// Creates a new term with the given symbol and arguments.
    pub fn with_args<'a, 'b>(symbol: &'b impl Symb<'a, 'b>, args: &'b [impl Term<'a, 'b>]) -> ATerm {
        THREAD_TERM_POOL.with_borrow(|tp| tp.create_term(symbol, args))
    }

    /// Creates a new term with the given symbol and an iterator over the arguments.
    pub fn with_iter<'a, 'b, 'c, 'd, I, T>(symbol: &'b impl Symb<'a, 'b>, iter: I) -> ATerm
    where
        I: IntoIterator<Item = T>,
        T: Term<'c, 'd>,
    {
        THREAD_TERM_POOL.with_borrow(|tp| tp.create_term_iter(symbol, iter))
    }

    /// Creates a new term with the given symbol and a head term, along with a list of arguments.
    pub fn with_iter_head<'a, 'b, 'c, 'd, 'e, 'f, I, T>(
        symbol: &'b impl Symb<'a, 'b>,
        head: &'d impl Term<'c, 'd>,
        iter: I,
    ) -> ATerm
    where
        I: IntoIterator<Item = T>,
        T: Term<'e, 'f>,
    {
        THREAD_TERM_POOL.with_borrow(|tp| tp.create_term_iter_head(symbol, head, iter))
    }

    /// Creates a new term using the pool
    pub fn constant(symbol: &SymbolRef<'_>) -> ATerm {
        THREAD_TERM_POOL.with_borrow(|tp| tp.create_constant(symbol))
    }

    /// Constructs a term from the given string.
    pub fn from_string(text: &str) -> Result<ATerm, MCRL3Error> {
        THREAD_TERM_POOL.with_borrow(|tp| tp.from_string(text))
    }

    /// Returns a borrow from the term
    pub fn get(&self) -> ATermRef<'_> {
        self.term.copy()
    }

    /// Returns the root of the term
    pub fn root(&self) -> ProtectionIndex {
        self.root
    }

    /// Creates a new term from the given reference and protection set root
    /// entry.
    pub(crate) fn from_index(term: &ATermIndex, root: ProtectionIndex) -> ATerm {
        unsafe {
            ATerm {
                term: ATermRef::from_index(term),
                root,
                _marker: PhantomData,
            }
        }
    }
}

impl<'a, 'b> Term<'a, 'b> for ATerm
where
    'b: 'a,
{
    delegate! {
        to self.term {
            fn protect(&self) -> ATerm;
            fn arg(&self, index: usize) -> ATermRef<'a>;
            fn arguments(&self) -> ATermArgs<'a>;
            fn copy(&self) -> ATermRef<'a>;
            fn get_head_symbol(&self) -> SymbolRef<'a>;
            fn iter(&self) -> TermIterator<'a>;
            fn index(&self) -> usize;
            fn shared(&self) -> &ATermIndex;
            fn annotation(&self) -> Option<usize>;
        }
    }
}

impl Markable for ATerm {
    fn mark(&self, marker: &mut Marker) {
        marker.mark(&self.term);
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        *term == self.term
    }

    fn len(&self) -> usize {
        1
    }
}

impl Drop for ATerm {
    fn drop(&mut self) {
        THREAD_TERM_POOL.with_borrow(|tp| tp.drop(self))
    }
}

impl Clone for ATerm {
    fn clone(&self) -> Self {
        self.copy().protect()
    }
}

impl<'a> Borrow<ATermRef<'a>> for ATerm {
    fn borrow(&self) -> &ATermRef<'a> {
        &self.term
    }
}

impl fmt::Display for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.copy())
    }
}

impl fmt::Debug for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.copy())
    }
}

impl Hash for ATerm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.term.hash(state)
    }
}

impl PartialEq for ATerm {
    fn eq(&self, other: &Self) -> bool {
        self.term.eq(&other.term)
    }
}

impl PartialOrd for ATerm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.term.cmp(&other.term))
    }
}

impl Ord for ATerm {
    fn cmp(&self, other: &Self) -> Ordering {
        self.term.cmp(&other.term)
    }
}

impl Eq for ATerm {}

/// A sendable variant of an `ATerm`.
///
/// # Details
///
/// Keeps track of an internal reference to the protection set it was protected from to ensure proper cleanup.
pub struct ATermSend {
    term: ATermRef<'static>,

    /// The root of the term in the protection set
    root: ProtectionIndex,

    /// A shared reference to the protection set that this term was created in.
    protection_set: Arc<Mutex<SharedTermProtection>>,
}

impl ATermSend {
    /// Takes ownership of an `ATerm` and makes it send.
    pub fn from(term: ATerm) -> Self {
        // Copy the information from the term, but forget it since we are taking over the `Drop` responsibility.
        let root = term.root;
        let term_ref: ATermRef<'static> = unsafe { ATermRef::from_index(&term.term.shared) };

        std::mem::forget(term);

        Self {
            term: term_ref,
            root,
            protection_set: THREAD_TERM_POOL.with_borrow(|tp| tp.get_protection_set().clone()),
        }
    }
}

impl Drop for ATermSend {
    fn drop(&mut self) {
        self.protection_set.lock().protection_set.unprotect(self.root);
    }
}

impl<'a, 'b> Term<'a, 'b> for ATermSend
where
    'b: 'a,
{
    delegate! {
        to self.term {
            fn protect(&self) -> ATerm;
            fn arg(&self, index: usize) -> ATermRef<'a>;
            fn arguments(&self) -> ATermArgs<'a>;
            fn copy(&self) -> ATermRef<'a>;
            fn get_head_symbol(&self) -> SymbolRef<'a>;
            fn iter(&self) -> TermIterator<'a>;
            fn index(&self) -> usize;
            fn shared(&self) -> &ATermIndex;
            fn annotation(&self) -> Option<usize>;
        }
    }
}

/// An iterator over the arguments of a term.
pub struct ATermArgs<'a> {
    term: Option<ATermRef<'a>>,
    arity: usize,
    index: usize,
}

impl<'a> ATermArgs<'a> {
    pub fn empty() -> ATermArgs<'static> {
        ATermArgs {
            term: None,
            arity: 0,
            index: 0,
        }
    }

    fn new(term: ATermRef<'a>) -> ATermArgs<'a> {
        let arity = term.get_head_symbol().arity();
        ATermArgs {
            term: Some(term),
            arity,
            index: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.arity == 0
    }
}

impl<'a> Iterator for ATermArgs<'a> {
    type Item = ATermRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.arity {
            let res = Some(self.term.as_ref().unwrap().arg(self.index));

            self.index += 1;
            res
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for ATermArgs<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.arity {
            let res = Some(self.term.as_ref().unwrap().arg(self.arity - 1));

            self.arity -= 1;
            res
        } else {
            None
        }
    }
}

impl ExactSizeIterator for ATermArgs<'_> {
    fn len(&self) -> usize {
        self.arity - self.index
    }
}

/// An iterator over all subterms of the given [ATerm] in preorder traversal, i.e.,
/// for f(g(a), b) we visit f(g(a), b), g(a), a, b.
pub struct TermIterator<'a> {
    queue: VecDeque<ATermRef<'a>>,
}

impl TermIterator<'_> {
    pub fn new(t: ATermRef) -> TermIterator {
        TermIterator {
            queue: VecDeque::from([t]),
        }
    }
}

impl<'a> Iterator for TermIterator<'a> {
    type Item = ATermRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_back() {
            Some(term) => {
                // Put subterms in the queue
                for argument in term.arguments().rev() {
                    self.queue.push_back(argument);
                }

                Some(term)
            }
            None => None,
        }
    }
}

/// Blanket implementation allowing passing borrowed terms as references.
/// TODO: Why is this necessary.
impl<'a, 'b, T: Term<'a, 'b>> Term<'a, 'b> for &'b T {
    fn protect(&self) -> ATerm {
        (*self).protect()
    }

    fn arg(&self, index: usize) -> ATermRef<'a> {
        (*self).arg(index)
    }

    fn arguments(&self) -> ATermArgs<'a> {
        (*self).arguments()
    }

    fn copy(&self) -> ATermRef<'a> {
        (*self).copy()
    }

    fn get_head_symbol(&self) -> SymbolRef<'a> {
        (*self).get_head_symbol()
    }

    fn iter(&self) -> TermIterator<'a> {
        (*self).iter()
    }

    fn index(&self) -> usize {
        (*self).index()
    }

    fn shared(&self) -> &ATermIndex {
        (*self).shared()
    }

    fn annotation(&self) -> Option<usize> {
        (*self).annotation()
    }
}
