//! Diagnostics collected during name resolution and type checking.
//!
//! Ported from `parsing/ParseErrorCollector.java`: rather than failing at the
//! first problem, `resolve.rs` and `typecheck.rs` record every diagnostic
//! they find into one [Diagnostics] and only fail at the end, so a single
//! `UntypedStarkSpecification` check reports everything wrong with it in one pass.

use std::error::Error;
use std::fmt;

use merc_utilities::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
}

/// A single diagnostic anchored to a source [Span].
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: Severity,
    pub message: String,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Diagnostic {
            span,
            severity: Severity::Error,
            message: message.into(),
        }
    }

    /// Renders this diagnostic against its `source` text, in the same
    /// `-->`/`|`/`^^^` style parser errors use (see [Span::render]).
    pub fn render(&self, source: &str) -> String {
        format!("{}\n{}", self.message, self.span.render(source))
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
    pub fn error(&mut self, span: Span, message: impl Into<String>) {
        self.items.push(Diagnostic::error(span, message));
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

    /// Merges another collector's diagnostics into this one.
    pub fn extend(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }

    /// `Ok(value)` if nothing errored, otherwise `Err(self)` — the "return
    /// `null` only after collecting every error" pattern from
    /// `SpecificationLoader.load`, but via `Result` instead of a sentinel.
    pub fn into_result<T>(self, value: T) -> Result<T, Diagnostics> {
        if self.has_errors() { Err(self) } else { Ok(value) }
    }

    /// Renders every diagnostic against `source`, separated by blank lines.
    pub fn render(&self, source: &str) -> String {
        self.items.iter().map(|d| d.render(source)).collect::<Vec<_>>().join("\n\n")
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, item) in self.items.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            writeln!(f, "{}", item.message)?;
        }
        Ok(())
    }
}

// Letting `Diagnostics` implement `std::error::Error` means it converts into
// `MercError` for free via that type's blanket `From` impl.
impl Error for Diagnostics {}

#[cfg(test)]
mod tests {
    use super::Diagnostics;
    use merc_utilities::Span;

    #[test]
    fn empty_collector_has_no_errors() {
        let diagnostics = Diagnostics::new();
        assert!(!diagnostics.has_errors());
        assert!(diagnostics.into_result(()).is_ok());
    }

    #[test]
    fn recorded_error_fails_into_result() {
        let mut diagnostics = Diagnostics::new();
        diagnostics.error(Span { start: 0, end: 1 }, "boom");
        assert!(diagnostics.has_errors());
        assert!(diagnostics.into_result(()).is_err());
    }

    #[test]
    fn collects_every_error_not_just_the_first() {
        let mut diagnostics = Diagnostics::new();
        diagnostics.error(Span { start: 0, end: 1 }, "first");
        diagnostics.error(Span { start: 2, end: 3 }, "second");
        assert_eq!(diagnostics.items().len(), 2);
    }

    #[test]
    fn render_includes_message_and_caret() {
        let mut diagnostics = Diagnostics::new();
        diagnostics.error(Span { start: 4, end: 5 }, "unexpected x");
        let rendered = diagnostics.render("eqn f = x;");
        assert!(rendered.contains("unexpected x"));
        assert!(rendered.contains("^"));
    }
}
