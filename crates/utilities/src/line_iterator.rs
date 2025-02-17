use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use streaming_iterator::StreamingIterator;

/// A lending iterator over the lines of a type implementing Read.
pub struct LineIterator<T: Read> {
    reader: BufReader<T>,
    buffer: String,
    end: bool,
}

impl<T: Read> LineIterator<T> {
    pub fn new(reader: T) -> LineIterator<T> {
        LineIterator {
            reader: BufReader::new(reader),
            buffer: String::new(),
            end: false,
        }
    }
}

impl<T: Read> StreamingIterator for LineIterator<T> {
    type Item = String;

    fn advance(&mut self) {
        self.buffer.clear();
        match self.reader.read_line(&mut self.buffer) {
            Ok(n) if n > 0 => {
                if self.buffer.ends_with('\n') {
                    self.buffer.pop();
                    if self.buffer.ends_with('\r') {
                        self.buffer.pop();
                    }
                }
            }
            Ok(_) => self.end = true,
            Err(_) => self.end = true,
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.end {
            None
        } else {
            Some(&self.buffer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn test_line_iterator_basic() {
        let data = "line1\nline2\nline3";
        let cursor = Cursor::new(data);
        let mut line_iterator = LineIterator::new(cursor);

        let mut lines = Vec::new();
        while let Some(line) = line_iterator.next() {
            lines.push(line.clone());
        }

        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_line_iterator_empty() {
        let data = "";
        let cursor = Cursor::new(data);
        let mut line_iterator = LineIterator::new(cursor);

        let mut lines = Vec::new();
        while let Some(line) = line_iterator.next() {
            lines.push(line.clone());
        }

        assert!(lines.is_empty());
    }

    #[test]
    fn test_line_iterator_single_line() {
        let data = "single line";
        let cursor = Cursor::new(data);
        let mut line_iterator = LineIterator::new(cursor);

        let mut lines = Vec::new();
        while let Some(line) = line_iterator.next() {
            lines.push(line.clone());
        }

        assert_eq!(lines, vec!["single line"]);
    }

    #[test]
    fn test_line_iterator_with_carriage_return() {
        let data = "line1\r\nline2\r\nline3";
        let cursor = Cursor::new(data);
        let mut line_iterator = LineIterator::new(cursor);

        let mut lines = Vec::new();
        while let Some(line) = line_iterator.next() {
            lines.push(line.clone());
        }

        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }
}
