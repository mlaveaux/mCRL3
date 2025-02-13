use bitstream_io::BigEndian;
use bitstream_io::BitRead;
use bitstream_io::BitReader;
use bitstream_io::BitWrite;
use bitstream_io::BitWriter;
use std::io::Read;
use std::io::Write;
use std::io::{self};

/// Writer for bit-level output operations using an underlying writer.
pub struct BitStreamWriter<W: Write> {
    writer: BitWriter<W, BigEndian>,
    integer_buffer: [u8; 10], // For variable-width integers
}

impl<W: Write> BitStreamWriter<W> {
    /// Creates a new BitStreamWriter wrapping the provided writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer: BitWriter::new(writer),
            integer_buffer: [0; 10],
        }
    }

    /// Writes the least significant bits from a u64 value.
    ///
    /// # Preconditions
    /// - number_of_bits must be <= 64
    pub fn write_bits(&mut self, value: u64, number_of_bits: u8) -> io::Result<()> {
        assert!(number_of_bits <= 64);
        self.writer.write(number_of_bits as u32, value)
    }

    /// Writes a string prefixed with its length as a variable-width integer.
    pub fn write_string(&mut self, s: &str) -> io::Result<()> {
        self.write_integer(s.len())?;
        for byte in s.as_bytes() {
            self.writer.write(8, *byte as u64)?;
        }
        Ok(())
    }

    /// Writes a usize value using variable-width encoding.
    pub fn write_integer(&mut self, value: usize) -> io::Result<()> {
        let nr_bytes = encode_variablesize_int(value, &mut self.integer_buffer);
        for i in 0..nr_bytes {
            self.writer.write(8, self.integer_buffer[i] as u64)?;
        }
        Ok(())
    }

    /// Flushes any remaining bits to the underlying writer.
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.byte_align()?;
        self.writer.flush()
    }
}

/// Reader for bit-level input operations from an underlying reader.
pub struct BitStreamReader<R: Read> {
    reader: BitReader<R, BigEndian>,
    text_buffer: Vec<u8>,
}

impl<R: Read> BitStreamReader<R> {
    /// Creates a new BitStreamReader wrapping the provided reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BitReader::new(reader),
            text_buffer: Vec::with_capacity(128),
        }
    }

    /// Reads bits into the least significant bits of a u64.
    ///
    /// # Preconditions
    /// - number_of_bits must be <= 64
    pub fn read_bits(&mut self, number_of_bits: u8) -> io::Result<u64> {
        assert!(number_of_bits <= 64);
        self.reader.read(number_of_bits as u32)
    }

    /// Reads a length-prefixed string.
    pub fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_integer()?;
        self.text_buffer.clear();
        self.text_buffer.reserve(length + 1);

        for _ in 0..length {
            let byte = self.reader.read::<u64>(8)? as u8;
            self.text_buffer.push(byte);
        }

        String::from_utf8(self.text_buffer.clone()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Reads a variable-width encoded integer.
    pub fn read_integer(&mut self) -> io::Result<usize> {
        decode_variablesize_int(self)
    }
}

/// Encodes a usize value using 7 bits per byte for the value and 1 bit to indicate
/// if more bytes follow. Writes the encoded bytes to the provided output buffer.
///
/// # Preconditions
/// - output buffer must have sufficient capacity (10 bytes for 64-bit integers)
///
/// # Returns
/// The number of bytes written to the output buffer
fn encode_variablesize_int(mut value: usize, output: &mut [u8]) -> usize {
    let mut output_size = 0;

    while value > 127 {
        output[output_size] = ((value & 127) as u8) | 128;
        value >>= 7;
        output_size += 1;
    }

    output[output_size] = value as u8;
    output_size + 1
}

/// Decodes a variable-width encoded integer from a BitStreamReader.
///
/// # Errors
/// - Reading from the underlying reader fails
/// - The encoded integer uses too many bytes
fn decode_variablesize_int<R: Read>(reader: &mut BitStreamReader<R>) -> io::Result<usize> {
    let mut value = 0usize;
    let max_bytes = (std::mem::size_of::<usize>() * 8 + 6) / 7;

    for i in 0..max_bytes {
        let byte = reader.read_bits(8)? as u8;
        value |= ((byte & 127) as usize) << (7 * i);

        if byte & 128 == 0 {
            return Ok(value);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Failed to read integer: too many bytes",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_string() -> io::Result<()> {
        let mut buffer = Vec::new();
        {
            let mut writer = BitStreamWriter::new(&mut buffer);
            writer.write_string("Hello, World!")?;
            writer.flush()?;
        }

        let mut reader = BitStreamReader::new(&buffer[..]);
        let result = reader.read_string()?;
        assert_eq!(result, "Hello, World!");
        Ok(())
    }

    #[test]
    fn test_variable_int_encoding() -> io::Result<()> {
        let test_values = vec![0, 127, 128, 16383, 16384, 2097151, 2097152];

        for value in test_values {
            let mut buffer = Vec::new();
            {
                let mut writer = BitStreamWriter::new(&mut buffer);
                writer.write_integer(value)?;
                writer.flush()?;
            }

            let mut reader = BitStreamReader::new(&buffer[..]);
            let result = reader.read_integer()?;
            assert_eq!(result, value);
        }
        Ok(())
    }

    #[test]
    fn test_bitstream() -> io::Result<()> {
        let mut buffer = Vec::new();
        {
            let mut writer = BitStreamWriter::new(&mut buffer);
            writer.write_bits(0b1010, 4)?;
            writer.write_bits(0b11111111, 8)?;
            writer.flush()?;
        }

        let mut reader = BitStreamReader::new(&buffer[..]);
        assert_eq!(reader.read_bits(4)?, 0b1010);
        assert_eq!(reader.read_bits(8)?, 0b11111111);
        Ok(())
    }
}
