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

    /// Prepares the implementation for enumerating transitions from `state`.
    ///
    /// The exploration loop calls this exactly once before iterating over the
    /// summands of a given source state. Implementations typically use it to
    /// stage a substitution in their enumeration backend.
    fn prepare(&self, state: &[Self::Value]);

    /// Returns the state-level metadata for the given source `state`.
    fn state_info(&self, state: &[Self::Value]) -> Self::StateInfo;
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

    /// Enumerate every outgoing transition produced by this summand from the
    /// state vector `state`.
    ///
    /// For each transition, `report(label, next_state)` is invoked exactly once
    /// with borrowed values.
    fn enumerate<F>(&self, state: &[Self::Value], report: F) -> Result<(), MercError>
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
