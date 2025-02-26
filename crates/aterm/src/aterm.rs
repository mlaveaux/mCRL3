use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::ops::Deref;

use mcrl3_utilities::PhantomUnsend;

use crate::is_empty_list;
use crate::is_int;
use crate::is_list;
use crate::SymbolRef;
use crate::THREAD_TERM_POOL;

use super::global_aterm_pool::GLOBAL_TERM_POOL;

/// This represents a lifetime bound reference to an existing ATerm that is
/// protected somewhere statically.
///
/// Can be 'static if the term is protected in a container or ATerm. That means
/// we either return &'a ATermRef<'static> or with a concrete lifetime
/// ATermRef<'a>. However, this means that the functions for ATermRef cannot use
/// the associated lifetime for the results parameters, as that would allow us
/// to acquire the 'static lifetime. This occasionally gives rise to issues
/// where we look at the argument of a term and want to return it's name, but
/// this is not allowed since the temporary returned by the argument is dropped.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ATermRef<'a> {
    index: usize,
    marker: PhantomData<&'a ()>,
}

/// These are safe because terms are never modified. Garbage collection is
/// always performed with exclusive access and uses relaxed atomics to perform
/// some interior mutability.
unsafe impl Send for ATermRef<'_> {}
unsafe impl Sync for ATermRef<'_> {}

impl Default for ATermRef<'_> {
    fn default() -> Self {
        ATermRef {
            index: 0,
            marker: PhantomData,
        }
    }
}

impl<'a> ATermRef<'a> {
    /// Protects the reference on the thread local protection pool.
    pub fn protect(&self) -> ATerm {
        if self.is_default() {
            ATerm::default()
        } else {
            THREAD_TERM_POOL.with_borrow(|tp| {
                tp.protect(&self.copy())
            })
        }
    }

    /// This allows us to extend our borrowed lifetime from 'a to 'b based on
    /// existing parent term which has lifetime 'b.
    ///
    /// The main usecase is to establish transitive lifetimes. For example given
    /// a term t from which we borrow `u = t.arg(0)` then we cannot have
    /// u.arg(0) live as long as t since the intermediate temporary u is
    /// dropped. However, since we know that u.arg(0) is a subterm of `t` we can
    /// upgrade its lifetime to the lifetime of `t` using this function.
    ///
    /// # Safety
    ///
    /// This function might only be used if witness is a parent term of the
    /// current term.
    pub fn upgrade<'b: 'a>(&'a self, parent: &ATermRef<'b>) -> ATermRef<'b> {
        debug_assert!(
            parent.iter().any(|t| t.copy() == *self),
            "Upgrade has been used on a witness that is not a parent term"
        );

        ATermRef::new(self.index)
    }

    /// A private unchecked version of [`ATermRef::upgrade`] to use in iterators.
    pub(crate) unsafe fn upgrade_unchecked<'b: 'a>(&'a self) -> ATermRef<'b> {
        ATermRef::new(self.index)
    }
}

impl ATermRef<'_> {
    /// Creates a new term reference from the given index.
    pub(crate) fn new(index: usize) -> Self {
        ATermRef {
            index,
            marker: PhantomData,
        }
    }

    /// Returns the index of the term.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the indexed argument of the term
    pub fn arg(&self, index: usize) -> ATermRef<'_> {
        self.require_valid();
        debug_assert!(
            index < self.get_head_symbol().arity(),
            "arg({index}) is not defined for term {:?}",
            self
        );

        GLOBAL_TERM_POOL.lock().get_argument(self, index).upgrade(self)
    }

    /// Returns the list of arguments as a collection
    pub fn arguments(&self) -> ATermArgs<'_> {
        self.require_valid();

        ATermArgs::new(self.copy())
    }

    /// Makes a copy of the term with the same lifetime as itself.
    pub fn copy(&self) -> ATermRef<'_> {
        ATermRef::new(self.index)
    }

    /// Returns whether the term is the default term (not initialised)
    pub fn is_default(&self) -> bool {
        self.index == 0
    }

    /// Returns the function of an ATermRef
    pub fn get_head_symbol(&self) -> &SymbolRef<'_> {
        self.require_valid();

        THREAD_TERM_POOL.with_borrow(|tp| {
            unsafe {
                std::mem::transmute(tp.get_head_symbol(self))
            }
        })
    }

    /// Returns true iff this is an aterm_list
    pub fn is_list(&self) -> bool {
        is_list(self.get_head_symbol())
    }

    /// Returns true iff this is the empty aterm_list
    pub fn is_empty_list(&self) -> bool {
        is_empty_list(self.get_head_symbol())
    }

    /// Returns true iff this is a aterm_int
    pub fn is_int(&self) -> bool {
        is_int(self.get_head_symbol())
    }

    /// Returns an iterator over all arguments of the term that runs in pre order traversal of the term trees.
    pub fn iter(&self) -> TermIterator<'_> {
        TermIterator::new(self.copy())
    }

    /// Panics if the term is default
    fn require_valid(&self) {
        debug_assert!(
            !self.is_default(),
            "This function can only be called on valid terms, i.e., not default terms"
        );
    }
}

impl fmt::Display for ATermRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.require_valid();
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for ATermRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_default() {
            write!(f, "<default>")?;
        } else {
            // TODO: This is recursive and will overflow the stack for large terms!            
            write!(f, "{}(", self.get_head_symbol())?;

            for arg in self.arguments() {
                write!(f, "{}, ", arg)?;
            }

            write!(f, ")")?;
        }

        Ok(())
    }
}

/// The protected version of [ATermRef], mostly derived from it.
#[derive(Default)]
pub struct ATerm {
    term: ATermRef<'static>,
    root: usize,

    // ATerm is not Send because it uses thread-local state for its protection
    // mechanism.
    _marker: PhantomUnsend,
}

impl ATerm {
    /// Creates a new term using the pool
    pub fn with_args<'a>(symbol: &SymbolRef<'_>, args: &[impl Borrow<ATermRef<'a>>]) -> ATerm {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.create(symbol, args)
        })
    }

    /// Creates a new term using the pool
    pub fn with_iter<'a, I, T>(symbol: &SymbolRef<'_>, iter: I) -> ATerm 
    where
        I: IntoIterator<Item = T>,
        T: Borrow<ATermRef<'a>>
    {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.create_term_iter(symbol, iter)
        })
    }

    /// Creates a new term using the pool
    pub fn constant(symbol: &SymbolRef<'_>) -> ATerm {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.create_constant(symbol)
        })
    }

    /// Constructs a term from the given string.
    pub fn from_string(s: &str) -> Result<ATerm, Box<dyn Error>> {
        THREAD_TERM_POOL.with_borrow(|tp| {
            tp.from_string(s)
        })
    }

    /// Returns the root of the term
    pub(crate) fn root(&self) -> usize {
        self.root
    }
    
    /// Creates a new term from the given reference and protection set root
    /// entry.
    pub(crate) fn new_internal(term: usize, root: usize) -> ATerm {
        ATerm {
            term: ATermRef::new(term),
            root,
            _marker: PhantomData,
        }
    }
}

impl Drop for ATerm {
    fn drop(&mut self) {
        if !self.is_default() {
            THREAD_TERM_POOL.with_borrow(|tp| {
                tp.drop(self)
            })
        }
    }
}

impl Clone for ATerm {
    fn clone(&self) -> Self {
        self.copy().protect()
    }
}

impl Deref for ATerm {
    type Target = ATermRef<'static>;

    fn deref(&self) -> &Self::Target {
        &self.term
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

/// An iterator over the arguments of a term.
#[derive(Default)]
pub struct ATermArgs<'a> {
    term: ATermRef<'a>,
    arity: usize,
    index: usize,
}

impl<'a> ATermArgs<'a> {
    fn new(term: ATermRef<'a>) -> ATermArgs<'a> {
        let arity = term.get_head_symbol().arity();
        ATermArgs { term, arity, index: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.arity == 0
    }
}

impl<'a> Iterator for ATermArgs<'a> {
    type Item = ATermRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.arity {
            let res = unsafe { Some(self.term.arg(self.index).upgrade_unchecked()) };

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
            let res = unsafe { Some(self.term.arg(self.arity - 1).upgrade_unchecked()) };

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
                    unsafe {
                        self.queue.push_back(argument.upgrade_unchecked());
                    }
                }

                Some(term)
            }
            None => None,
        }
    }
}
