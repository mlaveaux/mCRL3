use std::io::Read;
use std::io::Write;
use std::io::{self};

use bitstream_io::BigEndian;
use bitstream_io::BitRead;
use bitstream_io::BitReader;
use bitstream_io::BitWrite;
use bitstream_io::BitWriter;
use log::error;

use merc_number::read_u64_variablelength;
use merc_number::write_u64_variablelength;
use merc_utilities::MercError;

/// Trait for writing bit-level data.
pub trait BitStreamWrite {
    /// Writes the least significant bits from a u64 value. The `number_of_bits` must be <= 64.
    fn write_bits(&mut self, value: u64, number_of_bits: u8) -> Result<(), MercError>;

    /// Writes a string prefixed with its length as a variable-width integer.
    fn write_string(&mut self, s: &str) -> Result<(), MercError>;

    /// Writes a u64 value using variable-width encoding.
    fn write_integer(&mut self, value: u64) -> Result<(), MercError>;

    /// Flushes any remaining bits to the underlying writer.
    fn flush(&mut self) -> Result<(), MercError>;
}

/// Trait for reading bit-level data.
pub trait BitStreamRead {
    /// Reads bits into the least significant bits of a u64. The `number_of_bits` must be <= 64.
    fn read_bits(&mut self, number_of_bits: u8) -> Result<u64, MercError>;

    /// Reads a length-prefixed string.
    fn read_string(&mut self) -> Result<String, MercError>;

    /// Reads a variable-width encoded integer.
    fn read_integer(&mut self) -> Result<u64, MercError>;
}

/// Writer for bit-level output operations using an underlying writer.
pub struct BitStreamWriter<W: Write> {
    writer: BitWriter<W, BigEndian>,
}

impl<W: Write> BitStreamWriter<W> {
    /// Creates a new BitStreamWriter wrapping the provided writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer: BitWriter::new(writer),
        }
    }
}

impl<W: Write> Drop for BitStreamWriter<W> {
    fn drop(&mut self) {
        if self.flush().is_err() {
            error!("Error flushing the stream when dropped!")
        }
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
}

impl<W: Write> BitStreamWrite for BitStreamWriter<W> {
    fn write_bits(&mut self, value: u64, number_of_bits: u8) -> Result<(), MercError> {
        if number_of_bits > 64 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "number_of_bits must be <= 64").into());
        }
        Ok(self.writer.write_var(number_of_bits as u32, value)?)
    }

    fn write_string(&mut self, s: &str) -> Result<(), MercError> {
        self.write_integer(s.len() as u64)?;
        self.writer.write_bytes(s.as_bytes())?;
        Ok(())
    }

    fn write_integer(&mut self, value: u64) -> Result<(), MercError> {
        write_u64_variablelength(&mut self.writer, value)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), MercError> {
        self.writer.byte_align()?;
        Ok(self.writer.flush()?)
    }
}

impl<R: Read> BitStreamRead for BitStreamReader<R> {
    fn read_bits(&mut self, number_of_bits: u8) -> Result<u64, MercError> {
        if number_of_bits > 64 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "number_of_bits must be <= 64").into());
        }
        Ok(self.reader.read_var(number_of_bits as u32)?)
    }

    fn read_string(&mut self) -> Result<String, MercError> {
        let length = self.read_integer()?;
        let length_usize: usize = length
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "string length exceeds usize"))?;

        self.text_buffer.clear();
        self.text_buffer
            .try_reserve(length_usize)
            .map_err(|_| io::Error::new(io::ErrorKind::OutOfMemory, "string too large to allocate"))?;
        self.text_buffer.resize(length_usize, 0);
        self.reader.read_bytes(&mut self.text_buffer)?;

        // Validate in place and copy the result out, so `text_buffer` keeps its
        // allocation for reuse on the next call (taking it would reset it to empty).
        std::str::from_utf8(&self.text_buffer)
            .map(str::to_owned)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e).into())
    }

    fn read_integer(&mut self) -> Result<u64, MercError> {
        read_u64_variablelength(&mut self.reader)
    }
}

#[cfg(test)]
mod tests {
    use super::BitStreamRead;
    use super::BitStreamReader;
    use super::BitStreamWrite;
    use super::BitStreamWriter;

    use log::debug;
    use rand::RngExt;
    use rand::distr::Alphanumeric;

    use merc_utilities::random_test;

    /// Decide (arbitrarily) what to write into the bitstream.
    #[derive(Debug)]
    enum Instruction {
        String(String),
        Integer(u64),
        /// (value, num_of_bits), where num_of_bits must be at most 64.
        Bits(u64, u8),
    }

    /// Calculate minimum bits needed to represent the value
    /// Use 1 bit if value is 0 to ensure at least 1 bit is written
    pub(super) fn required_bits(value: u64) -> u8 {
        if value == 0 {
            1
        } else {
            64 - value.leading_zeros() as u8
        }
    }

    #[test]
    fn test_arbitrary_bitstream() {
        random_test(100, |rng| {
            let instructions: Vec<Instruction> = (0..100)
                .map(|_| match rng.random_range(0..3) {
                    0 => {
                        let string = rng.sample_iter(&Alphanumeric).take(7).map(char::from).collect();
                        Instruction::String(string)
                    }
                    1 => Instruction::Integer(rng.random()),
                    2 => {
                        let value: u64 = rng.random();
                        Instruction::Bits(value, required_bits(value))
                    }
                    _ => unreachable!("The range is from 0 to 3"),
                })
                .collect();

            let mut buffer = Vec::new();
            {
                let mut writer = BitStreamWriter::new(&mut buffer);

                for inst in &instructions {
                    debug!("Writing {inst:?}");
                    match inst {
                        Instruction::String(string) => {
                            writer.write_string(string).expect("Failed to write into stream")
                        }
                        Instruction::Integer(value) => {
                            writer.write_integer(*value).expect("Failed to write into stream")
                        }
                        Instruction::Bits(value, number_of_bits) => writer
                            .write_bits(*value, *number_of_bits)
                            .expect("Failed to write into stream"),
                    }
                }

                writer.flush().expect("Failed to write into stream");
            }

            let mut reader = BitStreamReader::new(&buffer[..]);

            for inst in &instructions {
                debug!("Checking {inst:?}");
                match inst {
                    Instruction::String(string) => {
                        debug_assert_eq!(
                            reader.read_string().expect("Failed to read from stream"),
                            *string,
                            "Failed to read back the string"
                        )
                    }
                    Instruction::Integer(value) => {
                        debug_assert_eq!(
                            reader.read_integer().expect("Failed to read from stream"),
                            *value,
                            "Failed to read back the integer"
                        )
                    }
                    Instruction::Bits(value, number_of_bits) => {
                        debug_assert_eq!(
                            reader.read_bits(*number_of_bits).expect("Failed to read from stream"),
                            *value,
                            "Failed to read back the bits"
                        )
                    }
                }
            }
        });
    }

    /// Writes the given strings and reads them back, asserting they round-trip.
    fn roundtrip_strings(strings: &[&str]) {
        let mut buffer = Vec::new();
        {
            let mut writer = BitStreamWriter::new(&mut buffer);
            for s in strings {
                writer.write_string(s).expect("Failed to write string");
            }
            writer.flush().expect("Failed to flush");
        }

        let mut reader = BitStreamReader::new(&buffer[..]);
        for s in strings {
            assert_eq!(&reader.read_string().expect("Failed to read string"), s);
        }
    }

    #[test]
    fn test_string_edge_cases() {
        // Empty strings, multi-byte UTF-8, and a long string all in one stream so
        // that the shared `text_buffer` is reused across reads of differing length.
        roundtrip_strings(&["", "a", "", "héllo wörld", "🦀∑≈ç", &"x".repeat(10_000), ""]);
    }

    #[test]
    fn test_string_roundtrip_random_unicode() {
        random_test(100, |rng| {
            let strings: Vec<String> = (0..50)
                .map(|_| {
                    let len = rng.random_range(0..32);
                    (0..len).map(|_| rng.random::<char>()).collect()
                })
                .collect();
            let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
            roundtrip_strings(&refs);
        });
    }

    #[test]
    fn test_read_string_rejects_invalid_utf8() {
        // Manually craft a stream: length 2 followed by an invalid UTF-8 sequence.
        let mut buffer = Vec::new();
        {
            let mut writer = BitStreamWriter::new(&mut buffer);
            writer.write_integer(2).expect("Failed to write length");
            writer.write_bits(0xFF, 8).expect("Failed to write byte");
            writer.write_bits(0xFE, 8).expect("Failed to write byte");
            writer.flush().expect("Failed to flush");
        }

        let mut reader = BitStreamReader::new(&buffer[..]);
        assert!(reader.read_string().is_err(), "Invalid UTF-8 should produce an error");
    }

    #[test]
    fn test_write_bits_rejects_too_many_bits() {
        let mut buffer = Vec::new();
        let mut writer = BitStreamWriter::new(&mut buffer);
        assert!(writer.write_bits(0, 65).is_err(), "More than 64 bits must be rejected");
    }
}
