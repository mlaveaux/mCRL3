//! Errors from whole-process-specification type checking ([`crate::ProcessSpecification`]).

use merc_syntax::Span;

use crate::InferenceError;
use crate::WellTypedError;

/// An error type checking a whole process specification: an action, process, or `init`
/// declaration that doesn't type check, on top of everything [`WellTypedError`]/[`InferenceError`]
/// already cover for the data-specification subtree.
///
/// `#[non_exhaustive]`: more variants may be added (communication sort-compatibility checking,
/// for one), so a caller matching on this needs a catch-all arm.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    /// An error resolving or checking the data specification itself, or an `act`/`proc`/`glob`
    /// sort against it.
    #[error(transparent)]
    WellTyped(#[from] WellTypedError),
    /// A Phase-3 inference error checking an action argument, a process-instantiation argument,
    /// or a `sum`/`dist` condition or time bound.
    #[error(transparent)]
    Inference(#[from] InferenceError),

    #[error("an anonymous structured sort is not allowed in an action, process, or global variable declaration")]
    AnonymousStructInDeclaration { span: Span },

    #[error("'{name}' is declared as both an action and a process")]
    ActionAndProcessConflict { name: String, span: Span },
    #[error("the parameter '{name}' of process '{process}' is declared more than once")]
    DuplicateProcessParameter { process: String, name: String, span: Span },
    #[error("the global variable '{name}' is declared more than once")]
    DuplicateGlobalVariable { name: String, span: Span },

    #[error("no action or process named '{name}' takes {arity} argument(s)")]
    UndeclaredActionOrProcess { name: String, arity: usize, span: Span },
    #[error("no overload of '{name}' accepts these arguments")]
    NoMatchingOverload {
        name: String,
        span: Span,
        #[source]
        cause: Box<ProcessError>,
    },
    #[error("the use of '{name}' is ambiguous between {count} declarations")]
    AmbiguousActionOrProcess { name: String, count: usize, span: Span },

    #[error("'{name}' is not a parameter of process '{process}'")]
    UnknownProcessParameter { process: String, name: String, span: Span },

    #[error("the action '{name}' is not declared")]
    UndeclaredAction { name: String, span: Span },
}

impl ProcessError {
    /// The span of the offending construct. Mirrors [`WellTypedError::span`]/
    /// [`InferenceError::span`], which every variant here ultimately delegates to or carries
    /// directly — always `Some` except through a `WellTypedError::Custom`.
    pub fn span(&self) -> Option<&Span> {
        match self {
            ProcessError::WellTyped(error) => error.span(),
            ProcessError::Inference(error) => Some(error.span()),
            ProcessError::AnonymousStructInDeclaration { span }
            | ProcessError::ActionAndProcessConflict { span, .. }
            | ProcessError::DuplicateProcessParameter { span, .. }
            | ProcessError::DuplicateGlobalVariable { span, .. }
            | ProcessError::UndeclaredActionOrProcess { span, .. }
            | ProcessError::NoMatchingOverload { span, .. }
            | ProcessError::AmbiguousActionOrProcess { span, .. }
            | ProcessError::UnknownProcessParameter { span, .. }
            | ProcessError::UndeclaredAction { span, .. } => Some(span),
        }
    }

    /// Renders this error's message, followed by a caret-annotated source snippet, the same way
    /// [`WellTypedError::render`]/[`InferenceError::render`] do. `source` must be the original
    /// specification text this error was raised against.
    pub fn render(&self, source: &str) -> String {
        match self.span() {
            Some(span) => format!("{self}\n{}", span.render(source)),
            None => self.to_string(),
        }
    }
}
