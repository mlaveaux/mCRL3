//! Trait abstractions for Linear Process Specifications (LPSs) and similar
//! state-space generators (e.g. PBES in SRF form).

use merc_utilities::MercError;

use crate::Slot;

/// A Linear Process Specification trait.
///
/// An [`LPS`] describes a transition system implicitly by giving an initial
/// state vector and a collection of condition action effect summands.
pub trait LPS {
    /// The type of the values stored at each position of a state vector.
    ///
    /// The value must be a [`Slot`] so state vectors can be stored compactly in
    /// the discovered set's hash-consed sequence forest.
    type Value: Slot;

    /// The label produced for each enumerated transition.
    type Label: Clone;

    /// Metadata reported once per discovered state to the exploration caller.
    ///
    /// Plain LPSs typically use `()`.
    type StateInfo;

    /// A single condition action effect summand of the LPS.
    type Summand: Summand<Value = Self::Value, Label = Self::Label>;

    /// Returns the initial state vector of the LPS.
    fn initial_state(&self) -> Vec<Self::Value>;

    /// Returns the summands that together define the transition relation.
    fn summands(&self) -> &[Self::Summand];

    /// Creates a fresh per-thread enumeration context for driving the summands.
    ///
    /// Each worker thread owns exactly one context; it holds the enumeration
    /// backend and the reusable scratch buffers. The LPS definition itself stays
    /// immutable and shareable by `&self`, so a single LPS can drive several
    /// contexts concurrently. The returned context is threaded through both
    /// [`LPS::prepare`] and [`Summand::enumerate`].
    fn create_context(&self) -> <Self::Summand as Summand>::Context;

    /// Prepares `context` for enumerating transitions from `state` and returns
    /// the indices of the summands that must be explored from `state`.
    ///
    /// The exploration loop calls this exactly once before iterating over the
    /// summands of a given source state, on the same thread (and thus the same
    /// `context`) that will then drive [`Summand::enumerate`]. Implementations
    /// typically use it to stage a substitution in their per-thread enumeration
    /// backend.
    ///
    /// The returned iterator lets implementations restrict enumeration to the
    /// relevant summands (e.g. a PBES in SRF form only enumerates the summands
    /// belonging to the current equation); implementations with no such
    /// restriction return all summand indices.
    ///
    /// The returned iterator may borrow both `self` and `state`, so it must be
    /// fully consumed before `state` is mutated again.
    fn prepare<'a>(
        &'a self,
        context: &mut <Self::Summand as Summand>::Context,
        state: &'a [Self::Value],
    ) -> impl Iterator<Item = usize> + 'a;

    /// Returns the state-level metadata for the given source `state`.
    fn state_info(&self, state: &[Self::Value]) -> Self::StateInfo;
}

impl<P: LPS> LPS for &P {
    type Value = P::Value;
    type Label = P::Label;
    type StateInfo = P::StateInfo;
    type Summand = P::Summand;

    fn initial_state(&self) -> Vec<Self::Value> {
        (**self).initial_state()
    }

    fn summands(&self) -> &[Self::Summand] {
        (**self).summands()
    }

    fn create_context(&self) -> <Self::Summand as Summand>::Context {
        (**self).create_context()
    }

    fn prepare<'a>(
        &'a self,
        context: &mut <Self::Summand as Summand>::Context,
        state: &'a [Self::Value],
    ) -> impl Iterator<Item = usize> + 'a {
        (**self).prepare(context, state)
    }

    fn state_info(&self, state: &[Self::Value]) -> Self::StateInfo {
        (**self).state_info(state)
    }
}

/// A condition action effect summand of an [`LPS`].
///
/// A summand represents a guarded transition: given a current state, it
/// reports each outgoing transition (action label and next state vector) it
/// produces by invoking the `report` callback. Implementations are free to
/// short-circuit when the callback returns an error.
pub trait Summand {
    /// The state vector element type, matching [`LPS::Value`].
    type Value;

    /// The action label type, matching [`LPS::Label`].
    type Label;

    /// Per-thread reusable scratch state threaded through [`Summand::enumerate`].
    ///
    /// Enumeration mutates this context instead of any shared `&self` state, so
    /// the same `&Summand` can be driven from several threads at once as long as
    /// each thread owns a distinct context. Summands that need no scratch state
    /// use `()`.
    ///
    /// Contexts are produced by [`LPS::create_context`] rather than constructed
    /// directly, so a context can own backend state (e.g. an mCRL2 enumerator)
    /// that depends on the LPS definition.
    type Context;

    /// Enumerate every outgoing transition produced by this summand from the
    /// state vector `state`.
    ///
    /// For each transition, `report(label, next_state)` is invoked exactly once
    /// with borrowed values. `context` supplies the per-thread scratch buffers;
    /// callers reuse a single context across summands and states on one thread.
    fn enumerate<F>(&self, context: &mut Self::Context, state: &[Self::Value], report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[Self::Value]) -> Result<(), MercError>;

    /// Returns the indices into the state vector whose values fully determine
    /// this summand's enumeration result (the "gamma" set). Used as the cache
    /// key by [`crate::CacheLPS`].
    ///
    /// An empty slice signals that caching is not applicable for this summand.
    fn read_positions(&self) -> &[usize] {
        &[]
    }

    /// Returns the indices into the state vector that this summand may change.
    /// Every position *not* in this set is passed through unchanged from the
    /// source state to each enumerated next state.
    fn write_positions(&self) -> &[usize] {
        &[]
    }
}
