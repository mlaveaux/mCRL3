use mcrl3_aterm::Term;
use mcrl3_data::DataExpression;
use mcrl3_utilities::{CachePolicy, FunctionCache, LruPolicy};

use crate::RewriteEngine;

/// A cached rewriter that uses a cache to store previously computed results.
struct CachedRewriter<R: RewriteEngine> {
    cache: FunctionCache<DataExpression, 
        DataExpression, 
        LruPolicy<DataExpression>, 
        Fn(DataExpression) -> DataExpression>,
}

impl<R: RewriteEngine> CachedRewriter<R> {
    /// Creates a new `CachedRewriter` with the given rewriter and cache policy.
    pub fn new(mut rewriter: R) -> Self {
        Self {
            cache: FunctionCache::new(|x| {
                rewriter.rewrite(x)
            }, 1024, LruPolicy::default()),
        }
    }
}

impl<R: RewriteEngine> RewriteEngine for CachedRewriter<R> {
    fn rewrite(&mut self, term: &DataExpression) -> DataExpression {
        self.cache.apply(term).protect().into()
    }
}