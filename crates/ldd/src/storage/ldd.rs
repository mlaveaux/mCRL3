
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use super::protection_set::ProtectionSet;

/// Every Ldd points to its root node in the Storage instance for maximal
/// sharing. These Ldd instances can only be created from the storage.
pub struct Ldd
{
    ldd: LddRef<'static>, // Reference in the node table.
    root: usize, // Index in the root set.
    protection_set: Rc<RefCell<ProtectionSet<usize>>>,
}

impl Ldd
{
    pub fn new(protection_set: &Rc<RefCell<ProtectionSet<usize>>>, index: usize) -> Ldd
    {
        let root = protection_set.borrow_mut().protect(index);
        Ldd { protection_set: Rc::clone(protection_set), ldd: LddRef::new(index), root }
    }

    pub fn index(&self) -> usize
    {
        self.ldd.index
    }
}

impl Deref for Ldd {
    type Target = LddRef<'static>;

    fn deref(&self) -> &Self::Target {
        &self.ldd
    }
}

impl<'a> Borrow<LddRef<'a>> for Ldd {
    fn borrow(&self) -> &LddRef<'a> {
        &self.ldd
    }
}

impl Clone for Ldd
{
    fn clone(&self) -> Self
    {
        Ldd::new(&self.protection_set, self.index())
    }
}

impl Drop for Ldd
{
    fn drop(&mut self)
    {
        self.protection_set.borrow_mut().unprotect(self.root);
    }
}

impl PartialEq for Ldd
{
    fn eq(&self, other: &Self) -> bool
    {
        debug_assert!(Rc::ptr_eq(&self.protection_set, &other.protection_set), "Both LDDs should refer to the same storage."); 
        self.index() == other.index()
    }
}

impl Debug for Ldd
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "index: {}", self.index())
    }
}

impl Hash for Ldd
{    
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index().hash(state);
    }
}

impl Eq for Ldd {}

/// The LddRef is a reference to an existing [Ldd] instance. This can be used to
/// avoid explicit protections that are performed when creating an [Ldd]
/// instance.
/// 
/// # Implementation notes
/// 
/// It is important to note that the lifetime carried by an LddRef is only used
/// to enforce lifetime constraints in several places, in a similar way as used
/// for mutex guards or iterators. The lifetime should *not* be used to derive
/// lifetimes of return values and instead these should be derived from `self`.
/// If this is implemented correctly in the internal implementation then the
/// LddRef can never be misused.
#[derive(Hash, PartialEq, Eq, Debug)]
pub struct LddRef<'a>
{
    index: usize, // Index in the node table.
    marker: PhantomData<&'a ()>
}

impl<'a> LddRef<'a>
{
    /// TODO: This function should only be called by Storage and [Ldd]
    pub fn new(index: usize) -> LddRef<'a>
    {
        LddRef { index, marker: PhantomData }
    }

    pub fn index(&self) -> usize
    {
        self.index
    }
    
    /// Returns an LddRef with the same lifetime as itself.
    pub fn borrow(&self) -> LddRef<'_>
    {
        LddRef::new(self.index())        
    }
}

impl PartialEq<Ldd> for LddRef<'_>
{
    fn eq(&self, other: &Ldd) -> bool
    { 
        self.index == other.index()
    }
}