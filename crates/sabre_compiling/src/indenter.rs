use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

/// An indentation manager that maintains the current indentation level and provides
/// methods for formatting text with proper indentation.
///
/// The indentation level can be increased with `indent()`, which returns an `Indent`
/// guard that automatically decreases the indentation when dropped.
#[derive(Debug)]
pub struct IndentFormatter<'a, W: Write> {
    /// The current indentation level (number of tabs), wrapped in `Rc<RefCell>` for interior mutability
    level: Rc<RefCell<usize>>,
    /// The underlying writer to which indented content will be written
    writer: &'a mut W,
    /// The string used for a single level of indentation
    indent_str: String,
}

impl<'a, W: Write> IndentFormatter<'a, W> {
    /// Creates a new IndentFormatter with zero indentation.
    pub fn new(writer: &'a mut W) -> Self {
        Self::with_indent_str(writer, "  ".to_string())
    }

    /// Creates a new IndentFormatter with zero indentation and specified indentation string.
    pub fn with_indent_str(writer: &'a mut W, indent_str: String) -> Self {
        Self {
            writer,
            level: Rc::new(RefCell::new(0)),
            indent_str,
        }
    }

    /// Increases the indentation level and returns a guard that will
    /// decrease it when dropped.
    pub fn indent(&self) -> Indent {
        // Increase the indentation level
        let mut level_ref = self.level.borrow_mut();
        *level_ref += 1;
        drop(level_ref); // Release the borrow before creating the guard

        Indent {
            level: Rc::clone(&self.level),
        }
    }

    /// Returns the current indentation level.
    pub fn level(&self) -> usize {
        *self.level.borrow()
    }
}

impl<W: Write> Write for IndentFormatter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Convert the byte slice to a string slice
        let s = std::str::from_utf8(buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        let parts = s.split('\n');
        let mut first = true;

        // Handle the remaining parts
        for part in parts {
            // Write the newline that split() removed
            if !first {
                self.writer.write("\n".as_bytes())?;
            }

            if !part.is_empty() {
                // Add indentation at line start
                for _ in 0..self.level() {
                    // Add indentation at line start
                    self.writer.write(self.indent_str.as_bytes())?;
                }
                self.writer.write(part.as_bytes())?;
            }

            first = false;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

/// A guard object that decreases indentation when dropped.
/// Created by calling `IndentFormatter::indent()`.
///
/// Uses interior mutability to avoid requiring a mutable reference to the IndentFormatter.
#[derive(Debug)]
pub struct Indent {
    /// Reference-counted cell containing the indentation level
    level: Rc<RefCell<usize>>,
}

impl Drop for Indent {
    /// Decreases the indentation level when this guard is dropped.
    fn drop(&mut self) {
        let mut level_ref = self.level.borrow_mut();
        debug_assert!(*level_ref > 0, "Indentation level cannot go below zero");
        *level_ref -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the indenter correctly handles multi-line strings
    #[test]
    fn test_multiline_string() {
        let mut buffer = Vec::new();
        {
            let mut formatter = IndentFormatter::new(&mut buffer);

            // First level indent
            let _indent1 = formatter.indent();

            // Write a multi-line string at once
            write!(formatter, "First line\nSecond line\nThird line").unwrap();
        }

        let result = String::from_utf8(buffer).unwrap();
        let expected = "  First line\n  Second line\n  Third line";
        assert_eq!(result, expected, "Multiline indentation incorrect");
    }
}
