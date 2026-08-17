/// Source location information, spanning from start to end in the source text.
#[derive(Clone, Default, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<pest::Span<'_>> for Span {
    fn from(span: pest::Span) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}

impl Span {
    /// The 1-based (line, column) of `self.start` within `source`, counted in
    /// `char`s rather than bytes so the column lines up under multi-byte
    /// UTF-8 text.
    pub fn start_line_col(&self, source: &str) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for ch in source[..self.start.min(source.len())].chars() {
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    /// Renders this span against its `source` text as a caret-annotated
    /// snippet, in the `-->`/`|`/`^^^` style `pest` (see
    /// `extend_parser_error` in `parse.rs`) and `rustc` diagnostics use, so
    /// parser errors and later-pass errors (type errors, …) read
    /// consistently:
    ///
    /// ```text
    ///  --> 1:23
    ///   |
    /// 1 | eqn f = undeclared;
    ///   |         ^^^^^^^^^^
    /// ```
    ///
    /// A span crossing a newline is underlined only up to the end of its
    /// first line; an out-of-range span (e.g. [Span::default] on a synthetic
    /// node) renders against the start of `source`.
    pub fn render(&self, source: &str) -> String {
        let (line, col) = self.start_line_col(source);
        let line_text = source.lines().nth(line - 1).unwrap_or("");

        let span_len = source
            .get(self.start..self.end.max(self.start))
            .map_or(1, |text| text.chars().count())
            .max(1);
        let underline_len = span_len.min(line_text.chars().count().saturating_sub(col - 1).max(1));

        let gutter = " ".repeat(line.to_string().len());
        format!(
            "{gutter}--> {line}:{col}\n{gutter} |\n{line} | {line_text}\n{gutter} | {}{}",
            " ".repeat(col - 1),
            "^".repeat(underline_len),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Span;

    #[test]
    fn test_start_line_col_first_line() {
        let span = Span { start: 4, end: 5 };
        assert_eq!(span.start_line_col("eqn f = x;"), (1, 5));
    }

    #[test]
    fn test_start_line_col_counts_newlines() {
        let source = "sort D;\nmap f: D;\neqn f = undeclared;";
        let start = source.rfind("undeclared").unwrap();
        let span = Span {
            start,
            end: start + "undeclared".len(),
        };
        assert_eq!(span.start_line_col(source), (3, 9));
    }

    #[test]
    fn test_start_line_col_multibyte() {
        // A multi-byte character before the span must not throw off the
        // column, which is counted in `char`s, not bytes.
        let source = "eqn é = x;";
        let start = source.rfind('x').unwrap();
        let span = Span { start, end: start + 1 };
        assert_eq!(span.start_line_col(source), (1, 9));
    }

    #[test]
    fn test_render_single_line() {
        let source = "eqn f = undeclared;";
        let start = source.find("undeclared").unwrap();
        let span = Span {
            start,
            end: start + "undeclared".len(),
        };
        assert_eq!(
            span.render(source),
            " --> 1:9\n  |\n1 | eqn f = undeclared;\n  |         ^^^^^^^^^^"
        );
    }

    #[test]
    fn test_render_later_line() {
        let source = "sort D;\nmap f: D;\neqn f = undeclared;";
        let start = source.rfind("undeclared").unwrap();
        let span = Span {
            start,
            end: start + "undeclared".len(),
        };
        assert_eq!(
            span.render(source),
            " --> 3:9\n  |\n3 | eqn f = undeclared;\n  |         ^^^^^^^^^^"
        );
    }

    #[test]
    fn test_render_clamps_to_line_when_span_crosses_newline() {
        let source = "eqn f = x\n+ y;";
        let start = source.find('x').unwrap();
        // A span spuriously extending past the end of the line is still
        // underlined only up to that line's end.
        let span = Span {
            start,
            end: source.len(),
        };
        assert_eq!(span.render(source), " --> 1:9\n  |\n1 | eqn f = x\n  |         ^");
    }

    #[test]
    fn test_render_default_span_points_at_source_start() {
        let source = "eqn f = 1;";
        let span = Span::default();
        assert_eq!(span.render(source), " --> 1:1\n  |\n1 | eqn f = 1;\n  | ^");
    }
}
