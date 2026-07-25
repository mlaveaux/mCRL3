use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::rc::Rc;

use merc_syntax::ConstructorId;
use merc_syntax::DefId;
use merc_syntax::EqnSpecId;
use merc_syntax::EqnVarId;
use merc_syntax::EquationId;
use merc_syntax::MapId;
use merc_syntax::UntypedDataSpecification;

use crate::EquationTyping;
use crate::InferenceError;
use crate::ResolvedSortId;
use crate::Signature;
use crate::SortInterner;

/// The context shared by all type-checking queries.
///
/// It owns the [SortInterner] and one [QueryCache] per query. Each semantic
/// fact is a memoized function on this context, so passes pull their
/// dependencies lazily and results are shared.
pub(crate) struct TypeCheckContext {
    pub(crate) sorts: SortInterner,

    pub(crate) sort_of_def: QueryCache<DefId, ResolvedSortId>,
    /// The memoized resolved sort of each constructor declaration, keyed by
    /// [ConstructorId]. Populated lazily by `query_sort_of_constructor`.
    pub(crate) sort_of_constructor: QueryCache<ConstructorId, ResolvedSortId>,
    /// The memoized resolved sort of each map declaration, keyed by [MapId].
    /// Populated lazily by `query_sort_of_map`.
    pub(crate) sort_of_map: QueryCache<MapId, ResolvedSortId>,
    /// The memoized resolved sort of each equation variable, keyed by
    /// `(EqnSpecId, EqnVarId)`. Populated lazily by
    /// `query_sort_of_equation_var`.
    pub(crate) sort_of_equation_var: QueryCache<(EqnSpecId, EqnVarId), ResolvedSortId>,

    /// The signature of the specification.
    pub(crate) signature: Option<Rc<Signature>>,
    /// The resolved signature of the system-defined specification.
    pub(crate) system_signature: Option<Rc<Signature>>,

    /// The memoized results of `query_equation_typing`, keyed by the id of the
    /// enclosing equation specification block and the equation's own id
    /// within it.
    pub(crate) equation_typing: QueryCache<(EqnSpecId, EquationId), Result<Rc<EquationTyping>, InferenceError>>,
}

impl TypeCheckContext {
    pub(crate) fn new() -> Self {
        TypeCheckContext {
            sorts: SortInterner::new(),
            sort_of_def: QueryCache::new(),
            sort_of_constructor: QueryCache::new(),
            sort_of_map: QueryCache::new(),
            sort_of_equation_var: QueryCache::new(),
            signature: None,
            system_signature: None,
            equation_typing: QueryCache::new(),
        }
    }
}

impl TypeCheckContext {
    /// The declared name of the sort that [DefId] `def` resolves to, whether a
    /// user sort (looked up in `spec`) or a system-internal one such as
    /// `@NatPair` (looked up in `system`), or `None` when it is out of range of
    /// both.
    ///
    /// This is the single place aware that a system-internal `DefId` continues
    /// the user sort numbering: it indexes `system.sort_declarations` offset by
    /// the user sort count, the layout `resolve_system_signature` establishes.
    /// The names are derived from the specifications on demand rather than
    /// cached, so nothing here needs to stay in sync with them.
    pub(crate) fn sort_name<'a>(
        &'a self,
        spec: &'a UntypedDataSpecification,
        system: &'a UntypedDataSpecification,
        def: DefId,
    ) -> Option<&'a str> {
        if let Some(decl) = spec.sort_declarations.get(*def) {
            return Some(&decl.identifier);
        }

        let system_index = (*def).checked_sub(spec.sort_declarations.len())?;
        system
            .sort_declarations
            .get(system_index)
            .map(|decl| decl.identifier.as_str())
    }
}

impl Default for TypeCheckContext {
    fn default() -> Self {
        TypeCheckContext::new()
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

    /// Returns the cached value for `key` if it has already been computed,
    /// or `None` if it is not yet in the cache (or still in progress).
    /// Use this for read-only access after the pipeline has populated the cache.
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        match self.entries.get(key)? {
            QueryEntry::Done(v) => Some(v),
            QueryEntry::InProgress => None,
        }
    }

    /// Iterates the values of every entry that has finished computing. Used
    /// for read-only sweeps over the whole cache after the pipeline has run,
    /// rather than looking up one key at a time.
    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.values().filter_map(|entry| match entry {
            QueryEntry::Done(value) => Some(value),
            QueryEntry::InProgress => None,
        })
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
