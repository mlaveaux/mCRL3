use std::fmt;


/// Formats bytes into human-readable format using decimal units (GB, MB, KB, bytes)
pub struct BytesFormatter(pub usize);

impl fmt::Display for BytesFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const KB: usize = 1_000;
        const MB: usize = 1_000_000;
        const GB: usize = 1_000_000_000;
        
        // Choose appropriate unit based on size for best readability
        if self.0 >= GB {
            write!(f, "{:.2} GB", self.0 as f64 / GB as f64)
        } else if self.0 >= MB {
            write!(f, "{:.2} MB", self.0 as f64 / MB as f64)
        } else if self.0 >= KB {
            write!(f, "{:.2} KB", self.0 as f64 / KB as f64)
        } else {
            write!(f, "{} bytes", self.0)
        }
    }
}

pub struct LargeFormatter(pub usize);

impl fmt::Display for LargeFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {        
        let num_str = self.0.to_string();
        let mut result = String::new();
        
        // Process digits in groups of 3 from right to left for readability
        for (i, ch) in num_str.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(' ');
            }
            result.push(ch);
        }
        
        // Reverse back to original order since we processed right-to-left
        let formatted: String = result.chars().rev().collect();
        
        debug_assert!(!formatted.is_empty(), "Formatted string should not be empty");
        debug_assert!(formatted.chars().all(|c| c.is_ascii_digit() || c == ' '), 
                     "Result should only contain digits and spaces");
        
        write!(f, "{}", formatted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_formatter_small_numbers() {
        assert_eq!(format!("{}", LargeFormatter(0)), "0");
        assert_eq!(format!("{}", LargeFormatter(123)), "123");
    }

    #[test]
    fn test_large_formatter_thousands() {
        assert_eq!(format!("{}", LargeFormatter(1234)), "1 234");
        assert_eq!(format!("{}", LargeFormatter(12345)), "12 345");
        assert_eq!(format!("{}", LargeFormatter(123456)), "123 456");
    }

    #[test]
    fn test_large_formatter_millions() {
        assert_eq!(format!("{}", LargeFormatter(1234567)), "1 234 567");
        assert_eq!(format!("{}", LargeFormatter(12345678)), "12 345 678");
    }

    #[test]
    fn test_large_formatter_edge_cases() {
        assert_eq!(format!("{}", LargeFormatter(1000)), "1 000");
        assert_eq!(format!("{}", LargeFormatter(1000000)), "1 000 000");
    }
}