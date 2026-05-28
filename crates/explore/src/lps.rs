//! Trait abstractions for Linear Process Specifications (LPSs).
//!
//! An LPS describes an LTS implicitly by giving an initial state vector and a
//! collection of condition action effect summands. The exploration algorithm in
//! [`crate::explore`] enumerates the state space by repeatedly applying these
//! summands to discovered states.

use std::hash::Hash;

use merc_lts::TransitionLabel;
use merc_utilities::MercError;

/// A Linear Process Specification trait.
pub trait LPS {
    /// The explicit state vector type.
    ///
    /// Typically a `Vec<T>` for some primitive `T`, but kept generic so
    /// callers can pick a representation that suits their domain (e.g. a
    /// fixed-size array or a packed integer).
    type State: Clone + Eq + Hash;

    /// The action label produced for each enumerated transition.
    type Label: TransitionLabel;

    /// A single condition action effect summand of the LPS.
    type Summand: Summand<State = Self::State, Label = Self::Label>;

    /// Returns the initial state of the LPS.
    fn initial_state(&self) -> Self::State;

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
    /// The explicit state vector type, matching [`LPS::State`].
    type State;

    /// The action label type, matching [`LPS::Label`].
    type Label;

    /// Enumerate every outgoing transition produced by this summand from
    /// `state`. For each transition, `report(label, next_state)` is invoked
    /// exactly once with borrowed values; the callback clones only when it
    /// needs to retain them. Errors from the callback or from the summand
    /// itself are propagated to the caller.
    fn enumerate<F>(&self, state: &Self::State, report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &Self::State) -> Result<(), MercError>;
}
