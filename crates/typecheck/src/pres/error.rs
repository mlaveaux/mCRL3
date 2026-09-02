//! Errors from whole-PRES-specification type checking ([`crate::PresSpecification`]).

use merc_syntax::Span;

use crate::InferenceError;
use crate::WellTypedError;

/// An error type checking a whole PRES specification: a global variable, equation, or `init`
/// declaration that doesn't type check, on top of everything [`WellTypedError`]/[`InferenceError`]
/// already cover for the data-specification subtree.
///
/// `#[non_exhaustive]`: mirrors [`crate::PbesError`] — new, additive surface expected to grow
/// without that being a breaking change for a caller who matches on it.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PresError {
    /// An error resolving or checking the data specification itself, or a `glob`/equation-parameter
    /// sort against it.
    #[error(transparent)]
    WellTyped(#[from] WellTypedError),
    /// A Phase-3 inference error checking a real-valued data expression embedded via `val(...)`, a
    /// `PropVarInst` argument, or an `inf`/`sup`/`sum` binder.
    #[error(transparent)]
    Inference(#[from] InferenceError),

    #[error("an anonymous structured sort is not allowed in a PRES declaration")]
    AnonymousStructInDeclaration { span: Span },

    #[error("the parameter '{name}' of equation '{equation}' is declared more than once")]
    DuplicateEquationParameter { equation: String, name: String, span: Span },
    #[error("the global variable '{name}' is declared more than once")]
    DuplicateGlobalVariable { name: String, span: Span },
    #[error("the propositional variable '{name}' is declared more than once")]
    DuplicatePropositionalVariable { name: String, span: Span },

    #[error("no propositional variable named '{name}' is declared")]
    UndeclaredPropositionalVariable { name: String, span: Span },
    #[error("'{name}' expects {expected} argument(s), found {found}")]
    ArityMismatch {
        name: String,
        expected: usize,
        found: usize,
        span: Span,
    },
}

impl PresError {
    /// The span of the offending construct. Mirrors [`crate::PbesError::span`] — every variant
    /// here ultimately delegates to or carries a span directly, always `Some` except through a
    /// `WellTypedError::Custom`.
    pub fn span(&self) -> Option<&Span> {
        match self {
            PresError::WellTyped(error) => error.span(),
            PresError::Inference(error) => Some(error.span()),
            PresError::AnonymousStructInDeclaration { span }
            | PresError::DuplicateEquationParameter { span, .. }
            | PresError::DuplicateGlobalVariable { span, .. }
            | PresError::DuplicatePropositionalVariable { span, .. }
            | PresError::UndeclaredPropositionalVariable { span, .. }
            | PresError::ArityMismatch { span, .. } => Some(span),
        }
    }

    /// Renders this error's message, followed by a caret-annotated source snippet, the same way
    /// [`crate::PbesError::render`] does. `source` must be the original specification text this
    /// error was raised against.
    pub fn render(&self, source: &str) -> String {
        match self.span() {
            Some(span) => format!("{self}\n{}", span.render(source)),
            None => self.to_string(),
        }
    }
}
