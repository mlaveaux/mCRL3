use std::error::Error;
use std::fmt;

use merc_utilities::Span;
use thiserror::Error as ThisError;

use crate::types::StarkType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Everything `resolve.rs` and `typecheck.rs` can complain about.
///
/// The `#[error]` messages are the single source of truth for the wording;
/// nothing else in the crate formats a diagnostic message.
#[derive(Clone, Debug, ThisError)]
pub enum DiagnosticKind {
    /// Two top-level declarations share a name. STARK has a single flat
    /// namespace, so this covers a constant clashing with a function just as
    /// much as two constants clashing.
    #[error("duplicate definition of `{name}`")]
    DuplicateDefinition { name: String, first: Span },

    /// Two `state`s in the same component share a name.
    #[error("duplicate controller state `{name}`")]
    DuplicateControllerState { name: String, first: Span },

    /// Two bindings in the *same* `let`/argument frame share a name.
    /// Shadowing an outer scope is legal and never reported.
    #[error("duplicate binding `{name}`")]
    DuplicateBinding { name: String, first: Span },

    #[error("unknown symbol `{name}`")]
    UnknownSymbol { name: String },

    #[error("unknown controller state `{name}`")]
    UnknownControllerState { name: String },

    /// The name resolves, but to the wrong kind of thing — calling a
    /// variable, referencing a function as a value, assigning to a constant.
    #[error("`{name}` is {found}, expected {expected}")]
    IllegalUseOfName {
        name: String,
        found: &'static str,
        expected: &'static str,
    },

    /// `type X = X | Y;`. Needs its own variant because neither name is
    /// registered yet at the point the general duplicate check would run.
    #[error("type `{name}` cannot declare an element with the same name")]
    TypeElementSharesTypeName { name: String },

    /// A state variable was read from an expression evaluated once at load
    /// time, before any variable store exists.
    #[error("`{context}` cannot read state variable `{name}`{}", .via.as_ref().map(|f| format!(" (via function `{f}`)")).unwrap_or_default())]
    StateVariableInStaticExpression {
        name: String,
        context: &'static str,
        via: Option<String>,
    },

    /// A `Ty::Named` annotation that names no declared `type`.
    #[error("unknown type `{name}`")]
    UnknownType { name: String },

    /// `expected.is_compatible_with(actual)` failed. Note this relation is
    /// asymmetric: `real` accepts `int`, but `int` does not accept `real`.
    #[error("expected {expected}, found {found}")]
    TypeMismatch { expected: StarkType, found: StarkType },

    #[error("expected a numerical type, found {found}")]
    NotNumerical { found: StarkType },

    /// Two types that have to meet at a join point have no common supertype.
    /// For example ternary operands must match.
    #[error("cannot merge {left} with {right}")]
    IncompatibleTypes { left: StarkType, right: StarkType },

    /// `R`/`N[..]`/`U[..]` used somewhere randomness is not permitted.
    #[error("random expressions are not allowed here")]
    RandomNotAllowed,

    #[error("`{name}` expects {expected} argument(s), found {found}")]
    ArityMismatch {
        name: String,
        expected: usize,
        found: usize,
    },

    /// A construct that resolves and type-checks but has no IR
    /// representation yet.
    #[error("{construct} is not yet supported by lowering")]
    NotYetSupported { construct: &'static str },
}

impl DiagnosticKind {
    /// A second source location worth showing alongside the primary one, with
    /// the label to introduce it by. For example to render a duplicated
    /// definition.
    pub fn related(&self) -> Option<(&Span, &'static str)> {
        match self {
            DiagnosticKind::DuplicateDefinition { first, .. }
            | DiagnosticKind::DuplicateControllerState { first, .. } => Some((first, "first defined here")),
            DiagnosticKind::DuplicateBinding { first, .. } => Some((first, "first bound here")),
            _ => None,
        }
    }
}

/// A single diagnostic anchored to a source [Span].
///
/// Collecting rather than failing fast: instead of stopping at the first
/// problem, `resolve.rs` and `typecheck.rs` record every diagnostic they find
/// into one [Diagnostics] and only fail at the end, so a single
/// `UntypedStarkSpecification` check reports everything wrong with it in one
/// pass.
///
/// Every diagnostic is a concrete [DiagnosticKind] variant rather than a
/// pre-formatted string, so the message is written once (in the `#[error]`
/// attribute) and callers can still match on *what* went wrong — which the
/// tests do, instead of asserting on message substrings. Each variant carries
/// the data the message interpolates, and the few that reference a second
/// location (a duplicate's original declaration) carry that [Span] too, so
/// [Diagnostic::render] can point at both.
#[derive(Clone, Debug, ThisError)]
#[error("{kind}")]
pub struct Diagnostic {
    pub span: Span,
    pub severity: Severity,
    #[source]
    pub kind: DiagnosticKind,
}

impl Diagnostic {
    pub fn error(span: Span, kind: DiagnosticKind) -> Self {
        Diagnostic {
            span,
            severity: Severity::Error,
            kind,
        }
    }

    /// Renders this diagnostic against its `source` text, in the same style as
    /// [Span::render], followed by a second annotated snippet when
    /// [DiagnosticKind::related] gives one.
    pub fn render(&self, source: &str) -> String {
        let mut rendered = format!("{}: {}\n{}", self.severity, self.kind, self.span.render(source));
        if let Some((span, label)) = self.kind.related() {
            rendered.push_str(&format!("\nnote: {label}\n{}", span.render(source)));
        }
        rendered
    }
}

/// An accumulator for every [Diagnostic] found while resolving or
/// type-checking a [crate::UntypedStarkSpecification].
#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an error diagnostic at `span`.
    pub fn error(&mut self, span: Span, kind: DiagnosticKind) {
        log::trace!("diagnostic at {}..{}: {}", span.start, span.end, kind);
        self.items.push(Diagnostic::error(span, kind));
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &[Diagnostic] {
        &self.items
    }

    /// Whether any recorded diagnostic matches `predicate` — the way tests
    /// assert on *which* problem was found without depending on wording.
    pub fn any(&self, predicate: impl Fn(&DiagnosticKind) -> bool) -> bool {
        self.items.iter().any(|d| predicate(&d.kind))
    }

    /// Merges another collector's diagnostics into this one.
    pub fn extend(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }

    /// `Ok(value)` if nothing errored, otherwise `Err(self)`.
    pub fn into_result<T>(self, value: T) -> Result<T, Diagnostics> {
        if self.has_errors() { Err(self) } else { Ok(value) }
    }

    /// Renders every diagnostic against `source`, separated by blank lines.
    pub fn render(&self, source: &str) -> String {
        self.items
            .iter()
            .map(|d| d.render(source))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Without the source text there is nothing to underline, so this
        // prints messages only. Prefer `render(source)` wherever the source
        // is still in hand.
        for (index, item) in self.items.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            writeln!(f, "{}: {}", item.severity, item.kind)?;
        }
        Ok(())
    }
}

/// Implemented so that a [Diagnostics] converts into `MercError` for free, via
/// that type's blanket `From` impl.
impl Error for Diagnostics {}

#[cfg(test)]
mod tests {
    use super::DiagnosticKind;
    use super::Diagnostics;
    use merc_utilities::Span;

    fn unknown(name: &str) -> DiagnosticKind {
        DiagnosticKind::UnknownSymbol { name: name.to_string() }
    }

    #[test]
    fn empty_collector_has_no_errors() {
        let diagnostics = Diagnostics::new();
        assert!(!diagnostics.has_errors());
        assert!(diagnostics.into_result(()).is_ok());
    }

    #[test]
    fn recorded_error_fails_into_result() {
        let mut diagnostics = Diagnostics::new();
        diagnostics.error(Span { start: 0, end: 1 }, unknown("boom"));
        assert!(diagnostics.has_errors());
        assert!(diagnostics.into_result(()).is_err());
    }

    #[test]
    fn collects_every_error_not_just_the_first() {
        let mut diagnostics = Diagnostics::new();
        diagnostics.error(Span { start: 0, end: 1 }, unknown("first"));
        diagnostics.error(Span { start: 2, end: 3 }, unknown("second"));
        assert_eq!(diagnostics.items().len(), 2);
    }

    #[test]
    fn render_includes_message_and_caret() {
        let mut diagnostics = Diagnostics::new();
        diagnostics.error(Span { start: 4, end: 5 }, unknown("x"));
        let rendered = diagnostics.render("eqn f = x;");
        assert!(rendered.contains("unknown symbol `x`"));
        assert!(rendered.contains("^"));
    }

    #[test]
    fn render_includes_the_related_span_for_duplicates() {
        let source = "const a = 1;\nconst a = 2;";
        let first = Span { start: 6, end: 7 };
        let second = Span { start: 19, end: 20 };
        let mut diagnostics = Diagnostics::new();
        diagnostics.error(
            second,
            DiagnosticKind::DuplicateDefinition {
                name: "a".to_string(),
                first,
            },
        );

        let rendered = diagnostics.render(source);
        assert!(rendered.contains("duplicate definition of `a`"), "{rendered}");
        assert!(rendered.contains("note: first defined here"), "{rendered}");
        // Both the offending line (2) and the original one (1) are shown.
        assert!(rendered.contains("--> 2:7"), "{rendered}");
        assert!(rendered.contains("--> 1:7"), "{rendered}");
    }

    #[test]
    fn any_matches_on_the_kind_not_the_message() {
        let mut diagnostics = Diagnostics::new();
        diagnostics.error(Span { start: 0, end: 1 }, DiagnosticKind::RandomNotAllowed);
        assert!(diagnostics.any(|kind| matches!(kind, DiagnosticKind::RandomNotAllowed)));
        assert!(!diagnostics.any(|kind| matches!(kind, DiagnosticKind::UnknownSymbol { .. })));
    }
}
