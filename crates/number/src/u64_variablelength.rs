use std::error::Error;
use std::io::Read;
use std::io::Write;

use bitstream_io::BitRead;
use bitstream_io::BitReader;
use bitstream_io::BitWrite;
use bitstream_io::BitWriter;
use bitstream_io::LittleEndian;

/// The number of bits needed to represent a value of type T in most significant bit encoding.
#[allow(unused)]
pub fn encoding_size<T>() -> usize {
    ((std::mem::size_of::<T>() + 1) * 8) / 7
}

/// Encodes an unsigned variable-length integer using the most significant bit (MSB) algorithm.
/// This function assumes that the value is stored as little endian.
/// \param value The input value. Any standard integer type is allowed.
/// \param output A pointer to a piece of reserved memory. Must have a minimum size dependent on the input size (32 bit = 5 bytes, 64 bit = 10 bytes).
/// \returns The number of bytes used in the output.
/// \details Implementation taken from <https://techoverflow.net/2013/01/25/efficiently-encoding-variable-length-integers-in-cc/>
#[allow(unused)]
pub fn write_u64_variablelength<W: Write>(
    stream: &mut BitWriter<W, LittleEndian>,
    mut value: u64,
) -> Result<(), Box<dyn Error>> {
    // While more than 7 bits of data are left, occupy the last output byte
    // and set the next byte flag.
    while value > 0b01111111 {
        stream.write(8, (value as u8 & 0b01111111) | 0b10000000)?;

        // Remove the seven bits we just wrote from value.
        value >>= 7;
    }

    stream.write(8, value as u8)?;
    Ok(())
}

///  Decodes an unsigned variable-length integer using the MSB algorithm.
#[allow(unused)]
pub fn read_u64_variablelength<R: Read>(stream: &mut BitReader<R, LittleEndian>) -> Result<u64, Box<dyn Error>> {
    let mut value: u64 = 0;
    for i in 0..encoding_size::<u64>() {
        let byte = stream.read::<u8>(8)?;

        // Take 7 bits (mask 0x01111111) from byte and shift it before the bits already written to value.
        value |= ((byte & 0b01111111) as u64) << (7 * i);

        if byte & 0b10000000 == 0 {
            // If the next-byte flag is not set then we are finished.
            break;
        }
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_encoding() {
        let mut stream: [u8; 10] = [0; 10];
        let mut writer = BitWriter::new(&mut stream[0..]);

        let value = 234678;
        write_u64_variablelength(&mut writer, value).unwrap();
        writer.write(32, 0 as u64).unwrap();

        let mut reader = BitReader::new(&stream[0..]);
        let result = read_u64_variablelength(&mut reader).unwrap();

        assert_eq!(result, value);
    }
}
