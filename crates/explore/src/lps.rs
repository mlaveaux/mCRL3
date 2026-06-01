//! Trait abstractions for Linear Process Specifications (LPSs).
//!
//! An LPS describes an LTS implicitly by giving an initial state vector and a
//! collection of condition action effect summands. The exploration algorithm in
//! [`crate::explore`] enumerates the state space by repeatedly applying these
//! summands to discovered states.

use std::hash::Hash;

use merc_lts::TransitionLabel;
use merc_utilities::MercError;

use crate::Slot;

/// A Linear Process Specification trait.
pub trait LPS {
    /// The type of the values stored at each position of a state vector.
    ///
    /// The value must be a [`Slot`] so state vectors can be stored compactly in
    /// the discovered set's hash-consed sequence forest.
    type Value: Slot;

    /// The action label produced for each enumerated transition.
    type Label: TransitionLabel;

    /// A single condition action effect summand of the LPS.
    type Summand: Summand<Value = Self::Value, Label = Self::Label>;

    /// Returns the initial state vector of the LPS.
    fn initial_state(&self) -> Vec<Self::Value>;

    /// Returns the summands that together define the transition relation.
    fn summands(&self) -> &[Self::Summand];
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
}
