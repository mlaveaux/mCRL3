use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::rc::Rc;

use merc_syntax::DefId;
use merc_syntax::EqnSpecId;
use merc_syntax::EquationId;

use crate::EquationTyping;
use crate::InferenceError;
use crate::ResolvedSortId;
use crate::Signature;
use crate::SortInterner;
use crate::SystemSortNames;

/// The context shared by all type-checking queries.
///
/// It owns the [SortInterner] and one [QueryCache] per query, following the
/// rustc query model: each semantic fact is a memoized function on this
/// context, so passes pull their dependencies lazily and results are shared
/// (see `docs/typecheck.md` §5). The fields are `pub(crate)` so a query can
/// borrow its own cache and the interner disjointly.
pub(crate) struct TypeckContext {
    pub(crate) sorts: SortInterner,
    pub(crate) sort_of_def: QueryCache<DefId, ResolvedSortId>,
    /// The memoized result of `query_signature`, computed once per context; a
    /// context serves a single specification, so there is no key. A plain
    /// [Option] rather than a [QueryCache]: the query cannot re-enter itself,
    /// and its error is not `Clone`, so the cache contract of storing failures
    /// cannot be met — the pipeline aborts on failure instead. Behind an [Rc]
    /// so inference can hold the signature while mutating the context (it
    /// interns binder sorts mid-walk).
    pub(crate) signature: Option<Rc<Signature>>,
    /// The resolved signature of the system-defined specification, computed by
    /// `resolve_system_signature` under the same regime as
    /// [TypeckContext::signature].
    pub(crate) system_signature: Option<Rc<Signature>>,
    /// The display names of the system-internal sorts (`@NatPair`, ...),
    /// filled by the same call as [TypeckContext::system_signature]; consulted
    /// by `display_sort` for debug logging.
    pub(crate) system_sort_names: Option<SystemSortNames>,
    /// The memoized results of `query_equation_typing`, keyed by the id of the
    /// enclosing equation specification block and the equation's own id
    /// within it. Failures are stored too, as the cache contract requires.
    pub(crate) equation_typing: QueryCache<(EqnSpecId, EquationId), Result<Rc<EquationTyping>, InferenceError>>,
}

impl TypeckContext {
    pub(crate) fn new() -> Self {
        TypeckContext {
            sorts: SortInterner::new(),
            sort_of_def: QueryCache::new(),
            signature: None,
            system_signature: None,
            system_sort_names: None,
            equation_typing: QueryCache::new(),
        }
    }
}

impl Default for TypeckContext {
    fn default() -> Self {
        TypeckContext::new()
    }
}

/// The error returned when a query transitively depends on itself.
///
/// Queries detect cycles through the cache lock state, so a cyclic definition
/// (for example a sort alias that refers to itself) surfaces as this error
/// instead of unbounded recursion.
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
#[error("cyclic query dependency")]
pub(crate) struct CyclicQuery;

/// A memoization table for a single query.
///
/// A query first calls [QueryCache::get_or_lock]; a `Some` result is a cache
/// hit and a `None` result locks the key, obliging the caller to compute the
/// value and store it with [QueryCache::unlock]. Re-entering a locked key
/// means the query depends on itself and fails with [CyclicQuery].
///
/// A locked key must always be unlocked, so fallible queries must store their
/// failure as part of the value (`V = Result<T, E>`) rather than returning
/// early; otherwise the key stays locked and later lookups misreport the
/// failure as a [CyclicQuery].
pub(crate) struct QueryCache<K, V> {
    entries: HashMap<K, QueryEntry<V>>,
}

enum QueryEntry<V> {
    InProgress,
    Done(V),
}

impl<K: Eq + Hash, V> QueryCache<K, V> {
    pub(crate) fn new() -> Self {
        QueryCache {
            entries: HashMap::new(),
        }
    }

    /// Returns the cached value for `key`, or locks the key when it has not
    /// been computed yet. After a `Ok(None)` the caller must call
    /// [QueryCache::unlock] with the computed value.
    pub(crate) fn get_or_lock(&mut self, key: K) -> Result<Option<&V>, CyclicQuery> {
        match self.entries.entry(key) {
            Entry::Occupied(entry) => match entry.into_mut() {
                QueryEntry::Done(value) => Ok(Some(value)),
                QueryEntry::InProgress => Err(CyclicQuery),
            },
            Entry::Vacant(entry) => {
                entry.insert(QueryEntry::InProgress);
                Ok(None)
            }
        }
    }

    /// Stores the computed value for a key previously locked by
    /// [QueryCache::get_or_lock] and returns a reference to it.
    pub(crate) fn unlock(&mut self, key: K, value: V) -> &V {
        match self.entries.entry(key) {
            Entry::Occupied(mut entry) => {
                assert!(
                    matches!(entry.get(), QueryEntry::InProgress),
                    "unlock called on a key that was already computed"
                );
                entry.insert(QueryEntry::Done(value));
                match entry.into_mut() {
                    QueryEntry::Done(value) => value,
                    QueryEntry::InProgress => unreachable!("the entry was just set to Done"),
                }
            }
            Entry::Vacant(_) => panic!("unlock called on a key that was never locked"),
        }
    }
}

impl<K: Eq + Hash, V> Default for QueryCache<K, V> {
    fn default() -> Self {
        QueryCache::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::CyclicQuery;
    use crate::QueryCache;

    #[test]
    fn test_query_cache_miss_then_hit() {
        let mut cache: QueryCache<u32, String> = QueryCache::new();

        assert_eq!(cache.get_or_lock(1), Ok(None));
        assert_eq!(cache.unlock(1, "one".to_string()), "one");
        assert_eq!(cache.get_or_lock(1), Ok(Some(&"one".to_string())));
    }

    #[test]
    fn test_query_cache_detects_cycle() {
        let mut cache: QueryCache<u32, String> = QueryCache::new();

        assert_eq!(cache.get_or_lock(1), Ok(None));
        assert_eq!(cache.get_or_lock(1), Err(CyclicQuery));
    }

    #[test]
    #[should_panic(expected = "never locked")]
    fn test_query_cache_unlock_without_lock_panics() {
        let mut cache: QueryCache<u32, String> = QueryCache::new();
        cache.unlock(1, "one".to_string());
    }
}
