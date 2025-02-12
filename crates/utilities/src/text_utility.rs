use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Read};
use regex::Regex;

/// Indents each line of text by the specified number of spaces.
pub fn indent_text(text: &str, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    text.lines()
        .map(|line| format!("{}{}", indent_str, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Wraps text to the specified width, preserving word boundaries.
pub fn wrap_text(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut line_length = 0;

    for word in text.split_whitespace() {
        if line_length + word.len() + 1 > width && line_length > 0 {
            writeln!(&mut result).unwrap();
            line_length = 0;
        }

        if line_length > 0 {
            write!(&mut result, " ").unwrap();
            line_length += 1;
        }

        write!(&mut result, "{}", word).unwrap();
        line_length += word.len();
    }

    result
}

/// Splits text into paragraphs separated by one or more blank lines.
pub fn split_paragraphs(text: &str) -> Vec<String> {
    let re = Regex::new(r"\n\s*\n").unwrap();
    re.split(text)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Splits text using the specified separators.
pub fn split(text: &str, separators: &str) -> Vec<String> {
    text.split(|c| separators.contains(c))
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Reads text from a file, optionally warning instead of failing if the file is not found.
pub fn read_text(filename: &str, warn: bool) -> io::Result<String> {
    match File::open(filename) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(content)
        }
        Err(e) if warn => {
            eprintln!("Warning: Could not open input file: {}", filename);
            Ok(String::new())
        }
        Err(e) => Err(e),
    }
}

/// Removes comments from text (from '%' until end of line).
pub fn remove_percentage_comments(text: &str) -> String {
    let re = Regex::new(r"%[^\n]*\n").unwrap();
    re.replace_all(text, "\n").into_owned()
}

/// Removes all whitespace from text.
pub fn remove_whitespace(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// Performs regular expression replacement in text.
pub fn regex_replace(pattern: &str, replacement: &str, text: &str) -> String {
    match Regex::new(pattern) {
        Ok(re) => re.replace_all(text, replacement).into_owned(),
        Err(_) => text.to_string(),
    }
}

/// Removes empty lines and trims remaining lines.
pub fn trim_text(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Removes C-style comments (// and /* */) from text.
pub fn remove_comments(text: &str) -> String {
    let mut result = String::new();
    let mut in_comment = false;

    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('/', Some(&'/')) => {
                // Skip line comment
                for ch in chars.by_ref() {
                    if ch == '\n' {
                        result.push(ch);
                        break;
                    }
                }
            },
            ('/', Some(&'*')) => {
                chars.next(); // consume *
                in_comment = true;
            },
            ('*', Some(&'/')) if in_comment => {
                chars.next(); // consume /
                in_comment = false;
            },
            (c, _) if !in_comment => {
                result.push(c);
            },
            _ => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_text() {
        let text = "line1\nline2\nline3";
        let expected = "  line1\n  line2\n  line3";
        assert_eq!(indent_text(text, 2), expected);
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a long text that needs to be wrapped";
        let wrapped = wrap_text(text, 20);
        assert!(wrapped.lines().all(|line| line.len() <= 20));
    }

    #[test]
    fn test_split_paragraphs() {
        let text = "Para 1\n\nPara 2\n\n\nPara 3";
        let paragraphs = split_paragraphs(text);
        assert_eq!(paragraphs, vec!["Para 1", "Para 2", "Para 3"]);
    }

    #[test]
    fn test_split() {
        let text = "a,b;c d";
        let parts = split(text, ", ;");
        assert_eq!(parts, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_remove_percentage_comments() {
        let text = "code\n% comment\nmore code";
        assert_eq!(remove_percentage_comments(text), "code\n\nmore code");
    }

    #[test]
    fn test_remove_whitespace() {
        let text = "a b\tc\n d";
        assert_eq!(remove_whitespace(text), "abcd");
    }

    #[test]
    fn test_regex_replace() {
        let text = "hello world";
        assert_eq!(
            regex_replace(r"(\w+)\s+(\w+)", "$2 $1", text),
            "world hello"
        );
    }

    #[test]
    fn test_remove_comments() {
        let text = "code // line comment\nmore /* block comment */ code";
        let expected = "code \nmore  code";
        assert_eq!(remove_comments(text), expected);
    }
}