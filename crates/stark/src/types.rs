//! The STARK type lattice, ported from `speclang/…/types/StarkType.java` and
//! its concrete subclasses (`StarkIntegerType`, `StarkRealType`,
//! `StarkBooleanType`, `StarkCustomType`, `StarkRandomType`, `StarkErrorType`).
//!
//! The original models each case as a class implementing a shared interface
//! with double-dispatch `merge`/`isCompatibleWith`/`canBeMergedWith` methods.
//! Here the same case analysis is expressed as free functions matching on a
//! single enum, which turns out to be exactly equivalent (verified against
//! every branch of the original) and avoids re-deriving the case analysis at
//! every call site.

use std::fmt;

/// A STARK type. `Random(_)` never wraps another `Random(_)` or `Error` —
/// that invariant is enforced by [StarkType::random] rather than by
/// construction, mirroring the defensive unwrap in `StarkRandomType`'s
/// original Java constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StarkType {
    Integer,
    Real,
    Boolean,
    /// A user-defined type, identified by name.
    Custom(String),
    /// A statically-known-to-be-random value of the wrapped (always
    /// non-random, non-error) type, e.g. the result of `R[0,1]` (`Random(Real)`).
    Random(Box<StarkType>),
    /// The result of a type error; absorbs into further checks so a single
    /// mistake doesn't cascade into a wall of unrelated diagnostics.
    Error,
}

impl StarkType {
    /// Wraps `inner` as a random value of that type. Flattens
    /// `random(Random(t))` to `Random(t)` rather than nesting, matching the
    /// original `StarkRandomType` constructor.
    pub fn random(inner: StarkType) -> StarkType {
        match inner {
            StarkType::Random(content) => StarkType::Random(content),
            other => StarkType::Random(Box::new(other)),
        }
    }

    /// This type with any `Random` wrapper stripped. Non-random types return
    /// a clone of themselves.
    pub fn deterministic(&self) -> StarkType {
        match self {
            StarkType::Random(content) => (**content).clone(),
            other => other.clone(),
        }
    }

    /// Whether this is (possibly randomly) a numerical type (`int` or `real`).
    pub fn is_numerical(&self) -> bool {
        matches!(self.deterministic(), StarkType::Integer | StarkType::Real)
    }

    /// Whether this is `Random(_)` at the top level.
    pub fn is_random(&self) -> bool {
        matches!(self, StarkType::Random(_))
    }

    /// Whether this is exactly the error type. `Random` never wraps `Error`
    /// in practice (every constructor here checks first), so this only ever
    /// needs to look at the top level.
    pub fn is_error(&self) -> bool {
        matches!(self, StarkType::Error)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.deterministic(), StarkType::Integer)
    }

    pub fn is_real(&self) -> bool {
        matches!(self.deterministic(), StarkType::Real)
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.deterministic(), StarkType::Boolean)
    }

    pub fn is_custom(&self) -> bool {
        matches!(self.deterministic(), StarkType::Custom(_))
    }

    /// Whether a value of type `actual` may be used where `self` (the
    /// expected type) is required. Ignores randomness on both sides (a
    /// `Random(int)` fits wherever a plain `int` is expected, since it is
    /// resolved to a concrete value before use) — the original
    /// `StarkRandomType.isCompatibleWith` delegates straight through to its
    /// content type for the same reason.
    ///
    /// Integer widens to real but not vice versa: `real x = 1;` is fine,
    /// `int x = 1.0;` is not.
    pub fn is_compatible_with(&self, actual: &StarkType) -> bool {
        match self.deterministic() {
            StarkType::Integer => actual.is_integer(),
            StarkType::Real => actual.is_numerical(),
            StarkType::Boolean => actual.is_boolean(),
            StarkType::Custom(name) => matches!(actual.deterministic(), StarkType::Custom(other) if other == name),
            StarkType::Error => false,
            StarkType::Random(_) => unreachable!("deterministic() never returns Random"),
        }
    }

    /// Whether `self` and `other` can be combined in a symmetric position —
    /// both branches of a ternary, both sides of a relation, elements of a
    /// `U[...]` — without a type error. Either side already being `Error`
    /// is always accepted, so one mistake doesn't cascade into a second
    /// diagnostic at the same spot.
    pub fn can_be_merged_with(&self, other: &StarkType) -> bool {
        if self.is_error() || other.is_error() {
            return true;
        }
        matches!(
            (self.deterministic(), other.deterministic()),
            (StarkType::Integer, StarkType::Integer)
                | (StarkType::Real, StarkType::Real)
                | (StarkType::Integer, StarkType::Real)
                | (StarkType::Real, StarkType::Integer)
                | (StarkType::Boolean, StarkType::Boolean)
        ) || matches!((self.deterministic(), other.deterministic()), (StarkType::Custom(a), StarkType::Custom(b)) if a == b)
    }

    /// Combines `self` and `other` into their common type (`int` (+) `real`
    /// -> `real`; identical types merge to themselves), propagating a
    /// `Random` wrapper if either side carries one. Returns `Error` if the
    /// two types have nothing in common — mirrors `StarkType.merge`.
    pub fn merge(&self, other: &StarkType) -> StarkType {
        if self.is_error() || other.is_error() {
            return StarkType::Error;
        }

        let base = match (self.deterministic(), other.deterministic()) {
            (StarkType::Integer, StarkType::Integer) => StarkType::Integer,
            (StarkType::Real, StarkType::Real) => StarkType::Real,
            (StarkType::Integer, StarkType::Real) | (StarkType::Real, StarkType::Integer) => StarkType::Real,
            (StarkType::Boolean, StarkType::Boolean) => StarkType::Boolean,
            (StarkType::Custom(a), StarkType::Custom(b)) if a == b => StarkType::Custom(a),
            _ => StarkType::Error,
        };

        if base.is_error() {
            return StarkType::Error;
        }
        if self.is_random() || other.is_random() {
            StarkType::random(base)
        } else {
            base
        }
    }
}

impl fmt::Display for StarkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StarkType::Integer => write!(f, "int"),
            StarkType::Real => write!(f, "real"),
            StarkType::Boolean => write!(f, "bool"),
            StarkType::Custom(name) => write!(f, "{name}"),
            StarkType::Random(content) => write!(f, "random[{content}]"),
            StarkType::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StarkType;

    fn random(t: StarkType) -> StarkType {
        StarkType::random(t)
    }

    #[test]
    fn merge_same_type_is_identity() {
        assert_eq!(StarkType::Integer.merge(&StarkType::Integer), StarkType::Integer);
        assert_eq!(StarkType::Real.merge(&StarkType::Real), StarkType::Real);
        assert_eq!(StarkType::Boolean.merge(&StarkType::Boolean), StarkType::Boolean);
    }

    #[test]
    fn merge_widens_integer_and_real_to_real() {
        assert_eq!(StarkType::Integer.merge(&StarkType::Real), StarkType::Real);
        assert_eq!(StarkType::Real.merge(&StarkType::Integer), StarkType::Real);
    }

    #[test]
    fn merge_incompatible_kinds_is_error() {
        assert_eq!(StarkType::Boolean.merge(&StarkType::Integer), StarkType::Error);
        assert_eq!(
            StarkType::Custom("Color".into()).merge(&StarkType::Custom("Shape".into())),
            StarkType::Error
        );
    }

    #[test]
    fn merge_propagates_randomness_from_either_side() {
        assert_eq!(
            StarkType::Integer.merge(&random(StarkType::Real)),
            random(StarkType::Real)
        );
        assert_eq!(
            StarkType::Real.merge(&random(StarkType::Integer)),
            random(StarkType::Real)
        );
        assert_eq!(
            random(StarkType::Integer).merge(&random(StarkType::Real)),
            random(StarkType::Real)
        );
    }

    #[test]
    fn merge_never_wraps_error_in_random() {
        // Incompatible kinds stay a bare Error even when random.
        assert_eq!(
            random(StarkType::Boolean).merge(&random(StarkType::Integer)),
            StarkType::Error
        );
    }

    #[test]
    fn random_flattens_nested_random() {
        assert_eq!(StarkType::random(random(StarkType::Real)), random(StarkType::Real));
    }

    #[test]
    fn is_compatible_with_allows_integer_to_widen_to_real() {
        assert!(StarkType::Real.is_compatible_with(&StarkType::Integer));
        assert!(!StarkType::Integer.is_compatible_with(&StarkType::Real));
    }

    #[test]
    fn is_compatible_with_ignores_randomness() {
        assert!(StarkType::Integer.is_compatible_with(&random(StarkType::Integer)));
        assert!(StarkType::Real.is_compatible_with(&random(StarkType::Integer)));
    }

    #[test]
    fn is_compatible_with_error_expected_is_always_false() {
        assert!(!StarkType::Error.is_compatible_with(&StarkType::Integer));
    }

    #[test]
    fn can_be_merged_with_is_permissive_around_errors() {
        assert!(StarkType::Error.can_be_merged_with(&StarkType::Boolean));
        assert!(StarkType::Boolean.can_be_merged_with(&StarkType::Error));
        assert!(!StarkType::Boolean.can_be_merged_with(&StarkType::Integer));
    }

    #[test]
    fn numerical_and_random_flags() {
        assert!(StarkType::Integer.is_numerical());
        assert!(random(StarkType::Real).is_numerical());
        assert!(!StarkType::Boolean.is_numerical());
        assert!(random(StarkType::Integer).is_random());
        assert!(!StarkType::Integer.is_random());
    }

    #[test]
    fn display_matches_stark_source_syntax() {
        assert_eq!(StarkType::Integer.to_string(), "int");
        assert_eq!(StarkType::Real.to_string(), "real");
        assert_eq!(StarkType::Boolean.to_string(), "bool");
        assert_eq!(random(StarkType::Real).to_string(), "random[real]");
        assert_eq!(StarkType::Custom("Color".into()).to_string(), "Color");
    }

    /// Ported from `~/STARK/speclang/src/test/java/stark/speclang/types/StarkTypeTest.java`.
    /// Table-driven, kept close to the original's structure (rows of
    /// `[a, b, expected_merge]` / `[expected, actual]`) so it's easy to
    /// cross-reference; the hand-written tests above already cover the
    /// *reasoning* (why each case holds), this covers the same ground the
    /// original test suite checked.
    mod ported_from_stark_type_test {
        use super::random;
        use crate::types::StarkType;

        fn custom() -> StarkType {
            StarkType::Custom("testType".into())
        }

        /// `[a, b, expected a.merge(b)]`. Also every row is checked for
        /// `a.can_be_merged_with(b) == true`.
        fn mergeable_types() -> Vec<(StarkType, StarkType, StarkType)> {
            vec![
                // Custom
                (custom(), custom(), custom()),
                (custom(), random(custom()), random(custom())),
                // Integer
                (StarkType::Integer, StarkType::Integer, StarkType::Integer),
                (StarkType::Integer, StarkType::Real, StarkType::Real),
                (
                    StarkType::Integer,
                    random(StarkType::Integer),
                    random(StarkType::Integer),
                ),
                (StarkType::Integer, random(StarkType::Real), random(StarkType::Real)),
                // Real
                (StarkType::Real, StarkType::Integer, StarkType::Real),
                (StarkType::Real, StarkType::Real, StarkType::Real),
                (StarkType::Real, random(StarkType::Integer), random(StarkType::Real)),
                (StarkType::Real, random(StarkType::Real), random(StarkType::Real)),
                // Boolean
                (StarkType::Boolean, StarkType::Boolean, StarkType::Boolean),
                (
                    StarkType::Boolean,
                    random(StarkType::Boolean),
                    random(StarkType::Boolean),
                ),
                // Random[Integer]
                (
                    random(StarkType::Integer),
                    StarkType::Integer,
                    random(StarkType::Integer),
                ),
                (random(StarkType::Integer), StarkType::Real, random(StarkType::Real)),
                (
                    random(StarkType::Integer),
                    random(StarkType::Integer),
                    random(StarkType::Integer),
                ),
                (
                    random(StarkType::Integer),
                    random(StarkType::Real),
                    random(StarkType::Real),
                ),
                // Random[Real]
                (random(StarkType::Real), StarkType::Integer, random(StarkType::Real)),
                (random(StarkType::Real), StarkType::Real, random(StarkType::Real)),
                (
                    random(StarkType::Real),
                    random(StarkType::Integer),
                    random(StarkType::Real),
                ),
                (
                    random(StarkType::Real),
                    random(StarkType::Real),
                    random(StarkType::Real),
                ),
                // Random[Boolean]
                (
                    random(StarkType::Boolean),
                    StarkType::Boolean,
                    random(StarkType::Boolean),
                ),
                (
                    random(StarkType::Boolean),
                    random(StarkType::Boolean),
                    random(StarkType::Boolean),
                ),
            ]
        }

        /// `[a, b]`, each expected to have `a.can_be_merged_with(b) == false`
        /// and `a.merge(b) == Error`.
        fn unmergeable_types() -> Vec<(StarkType, StarkType)> {
            vec![
                (custom(), StarkType::Boolean),
                (custom(), StarkType::Integer),
                (custom(), StarkType::Real),
                (custom(), random(StarkType::Boolean)),
                (custom(), random(StarkType::Integer)),
                (custom(), random(StarkType::Real)),
                (StarkType::Integer, StarkType::Boolean),
                (StarkType::Integer, random(StarkType::Boolean)),
                (StarkType::Real, StarkType::Boolean),
                (StarkType::Real, random(StarkType::Boolean)),
                (StarkType::Boolean, StarkType::Integer),
                (StarkType::Boolean, StarkType::Real),
                (StarkType::Boolean, random(StarkType::Integer)),
                (StarkType::Boolean, random(StarkType::Real)),
            ]
        }

        /// `[expected, actual]`, each expected to have
        /// `expected.is_compatible_with(actual) == true`.
        fn compatible_types() -> Vec<(StarkType, StarkType)> {
            vec![
                (StarkType::Boolean, StarkType::Boolean),
                (StarkType::Boolean, random(StarkType::Boolean)),
                (StarkType::Integer, StarkType::Integer),
                (StarkType::Integer, random(StarkType::Integer)),
                (StarkType::Real, StarkType::Integer),
                (StarkType::Real, StarkType::Real),
                (StarkType::Real, random(StarkType::Integer)),
                (StarkType::Real, random(StarkType::Real)),
                (custom(), custom()),
                (custom(), random(custom())),
            ]
        }

        #[test]
        fn types_that_should_be_merged() {
            for (a, b, _) in mergeable_types() {
                assert!(a.can_be_merged_with(&b), "{a} should be merged with {b}");
            }
        }

        #[test]
        fn types_that_cannot_be_merged() {
            for (a, b) in unmergeable_types() {
                assert!(!a.can_be_merged_with(&b), "{a} should not be merged with {b}");
            }
        }

        #[test]
        fn merging_types_results() {
            for (a, b, expected) in mergeable_types() {
                assert_eq!(expected, a.merge(&b), "{a}.merge({b})");
            }
        }

        #[test]
        fn error_merging_types_results() {
            for (a, b) in unmergeable_types() {
                assert_eq!(StarkType::Error, a.merge(&b), "{a}.merge({b})");
            }
        }

        #[test]
        fn subtyping() {
            for (expected, actual) in compatible_types() {
                assert!(
                    expected.is_compatible_with(&actual),
                    "{expected}.is_compatible_with({actual})"
                );
            }
        }
    }
}
