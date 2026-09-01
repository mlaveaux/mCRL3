use std::cmp::Ordering;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;

use merc_utilities::Span;

/// A value of type `T` paired with the source [Span] it originates from.
///
/// Equality, ordering and hashing deliberately ignore the [Span] and consider
/// only `node`, so two structurally identical values at different source
/// locations compare and hash equal. Many passes rely on this structural
/// equality (hash maps, deduplication, `assert_eq!` in tests).
#[derive(Clone, Debug, Default)]
pub struct Spanned<T> {
    /// The wrapped value.
    pub node: T,
    /// The source location the value originates from.
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Transforms the wrapped value while preserving the span.
    pub fn map<U>(self, function: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            node: function(self.node),
            span: self.span,
        }
    }
}

/// Wraps `node` together with its source `span`.
pub fn respan<T>(span: Span, node: T) -> Spanned<T> {
    Spanned { node, span }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<T: Eq> Eq for Spanned<T> {}

impl<T: PartialOrd> PartialOrd for Spanned<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T: Ord> Ord for Spanned<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.node.cmp(&other.node)
    }
}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state);
    }
}
