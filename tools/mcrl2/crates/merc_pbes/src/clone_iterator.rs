// This is a helper trait that is object safe and allows a type-erased iterator
// to be cloned.
pub trait CloneIterator: Iterator {
    /// Clone the iterator into a boxed trait object.
    fn clone_boxed<'a>(&self) -> Box<dyn CloneIterator<Item = Self::Item> + 'a>
    where
        Self: 'a;
}

impl<T, I> CloneIterator for I
where
    I: Iterator<Item = T> + Clone,
{
    fn clone_boxed<'a>(&self) -> Box<dyn CloneIterator<Item = Self::Item> + 'a>
    where
        Self: 'a,
    {
        Box::new(self.clone())
    }
}

impl<T: Clone + 'static> Clone for Box<dyn CloneIterator<Item = T> + '_> {
    fn clone(&self) -> Self {
        // This delegates to clone_boxed(), which is implemented by the concrete
        // iterator type via the CloneIterator blanket impl. The call chain is:
        //   Box<dyn CloneIterator>::clone() -> clone_boxed() -> Box::new(self.clone())
        // where the inner clone() calls Iterator::clone() on the concrete type.
        (**self).clone_boxed()
    }
}
