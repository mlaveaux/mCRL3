use std::sync::Arc;

use merc_explore::LPS;
use merc_explore::StateEffect;
use merc_explore::Summand;
use merc_utilities::MercError;

use crate::bsgs::Bsgs;
use crate::bsgs::CanonicalizeContext;
use crate::explore_common::ParameterLayoutLPS;

/// Wraps any `LPS<Value = usize>` and canonicalizes every enumerated next-state
/// to the lexicographically smallest orbit representative before passing it to
/// the caller.
///
/// State vectors are laid out as `[eq_idx_0..eq_idx_{offset-1}, param_0..param_{n-1}]`
/// where only positions `param_offset..` are touched by the group action.  Position 0
/// (the equation index) is never permuted.
///
/// # Wrapping order
///
/// When combined with [`merc_explore::CacheLPS`], place the cache *inside* and the
/// quotient *outside*:
/// ```text
/// QuotientLps<CacheLPS<PbesLps>>
/// ```
/// This keeps cache keys narrow (raw, un-canonicalized write positions) and avoids
/// forcing the cache to track all parameters as written.
pub struct QuotientLps<P: ParameterLayoutLPS<Value = usize>> {
    inner: Arc<P>,
    bsgs: Arc<Bsgs>,
    summands: Vec<QuotientSummand<P>>,
    param_offset: usize,
}

/// A single summand of a [`QuotientLps`].
///
/// Delegates enumeration to the corresponding inner summand and canonicalizes
/// each next-state before reporting it.
pub struct QuotientSummand<P: ParameterLayoutLPS<Value = usize>> {
    index: usize,
    inner: Arc<P>,
    bsgs: Arc<Bsgs>,
    param_offset: usize,
    read_positions: Vec<usize>,
}

/// Per-thread enumeration context for a [`QuotientLps`].
pub struct QuotientContext<P: ParameterLayoutLPS<Value = usize>> {
    inner: <P::Summand as Summand>::Context,

    /// Working buffers of [`Bsgs::canonicalize_into`], so that canonicalizing a
    /// next state costs no allocation.
    scratch: CanonicalizeContext,

    /// The canonicalized next state handed to the caller's callback.
    canonical: Vec<usize>,
}

// SAFETY: Neither struct has interior mutability of its own (no UnsafeCell /
// raw pointers). All concurrent access is read-only via `&self`. The only
// non-trivially-Sync field is `Arc<P>`: sharing `&Arc<P>` across threads only
// requires `P: Sync` (which the bound enforces). The stdlib's conservative
// `impl<T: Send + Sync> Sync for Arc<T>` also requires `T: Send` to handle the
// last Arc being dropped on a foreign thread, but `QuotientLps` is not `Send`,
// so that case cannot arise. `Arc<Bsgs>` is unconditionally fine because `Bsgs`
// contains only `usize`, `Vec`, and `HashMap` of plain data, all auto-`Sync`.
unsafe impl<P: ParameterLayoutLPS<Value = usize> + Sync> Sync for QuotientLps<P> {}
unsafe impl<P: ParameterLayoutLPS<Value = usize> + Sync> Sync for QuotientSummand<P> {}

impl<P> QuotientLps<P>
where
    P: ParameterLayoutLPS<Value = usize>,
{
    /// Wraps `inner` in a canonicalizing quotient layer.
    ///
    /// `param_offset` is the first position in the state vector that belongs to
    /// the PBES parameters (always `1` for `PbesSrfLps`, where position 0 is the
    /// equation index).
    pub fn new(inner: P, bsgs: Arc<Bsgs>, param_offset: usize) -> Self {
        let inner = Arc::new(inner);

        let summands = inner
            .summands()
            .iter()
            .enumerate()
            .map(|(i, s)| QuotientSummand {
                index: i,
                inner: Arc::clone(&inner),
                bsgs: Arc::clone(&bsgs),
                param_offset,
                read_positions: s.read_positions().to_vec(),
            })
            .collect();

        QuotientLps {
            inner,
            bsgs,
            summands,
            param_offset,
        }
    }
}

impl<P> LPS for QuotientLps<P>
where
    P: ParameterLayoutLPS<Value = usize>,
{
    type Value = usize;
    type Label = P::Label;
    type StateInfo = P::StateInfo;
    type Summand = QuotientSummand<P>;

    fn initial_state(&self) -> Vec<usize> {
        self.bsgs.canonicalize(&self.inner.initial_state(), self.param_offset)
    }

    fn summands(&self) -> &[Self::Summand] {
        &self.summands
    }

    fn create_context(&self) -> QuotientContext<P> {
        QuotientContext {
            inner: self.inner.create_context(),
            scratch: CanonicalizeContext::default(),
            canonical: Vec::new(),
        }
    }

    fn prepare<'a>(&'a self, context: &mut QuotientContext<P>, state: &'a [usize]) -> impl Iterator<Item = usize> + 'a {
        self.inner.prepare(&mut context.inner, state)
    }

    fn state_info(&self, state: &[usize], context: &QuotientContext<P>) -> P::StateInfo {
        self.inner.state_info(state, &context.inner)
    }
}

impl<P> Summand for QuotientSummand<P>
where
    P: ParameterLayoutLPS<Value = usize>,
{
    type Value = usize;
    type Label = P::Label;
    type Context = QuotientContext<P>;

    fn read_positions(&self) -> &[usize] {
        &self.read_positions
    }

    fn effect(&self) -> StateEffect<'_> {
        // Canonicalization can move a value to any parameter position, and the
        // states it passes through unchanged are of other lengths entirely.
        StateEffect::Opaque
    }

    fn enumerate<F>(&self, context: &mut Self::Context, state: &[usize], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[usize]) -> Result<(), MercError>,
    {
        let bsgs = &self.bsgs;
        let inner = &*self.inner;
        let param_offset = self.param_offset;

        // Destructured so the closure can borrow the canonicalization buffers
        // while the inner summand holds its own context.
        let QuotientContext {
            inner: inner_context,
            scratch,
            canonical,
        } = context;

        self.inner.summands()[self.index].enumerate(inner_context, state, |label, next| {
            match inner.parameter_range(next) {
                Some(range) => {
                    debug_assert_eq!(
                        range.start, param_offset,
                        "the parameter block must start where the group acts"
                    );
                    debug_assert_eq!(range.len(), bsgs.n, "the group must act on the whole parameter block");
                    bsgs.canonicalize_into(next, param_offset, scratch, canonical);
                    report(label, canonical)
                }
                // Sinks and subformula vertices carry no data parameters, so the
                // group does not act on them; permuting their payload would
                // corrupt a priority or an interned formula index.
                None => report(label, next),
            }
        })
    }
}
