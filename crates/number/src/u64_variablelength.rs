use std::io::Read;
use std::io::Write;

use bitstream_io::BitRead;
use bitstream_io::BitReader;
use bitstream_io::BitWrite;
use bitstream_io::BitWriter;
use bitstream_io::Endianness;

use merc_utilities::MercError;

/// The maximum number of bytes needed to encode a value of type T in most
/// significant bit encoding.
///
/// The encoding stores 7 payload bits per byte, so a `T` of `n` bits needs
/// `ceil(n / 7)` bytes. `((size_of::<T>() + 1) * 8) / 7` computes that ceiling
/// (e.g. 10 bytes for `u64`).
pub const fn encoding_size<T>() -> usize {
    ((std::mem::size_of::<T>() + 1) * 8) / 7
}

/// Encodes a given unsigned variable-length integer using the most significant bit (MSB) algorithm.
///
/// # Details
///
/// Implementation taken from <https://techoverflow.net/2013/01/25/efficiently-encoding-variable-length-integers-in-cc/>
pub fn write_u64_variablelength<W: Write, E: Endianness>(
    stream: &mut BitWriter<W, E>,
    mut value: u64,
) -> Result<(), MercError> {
    // While more than 7 bits of data are left, occupy the last output byte
    // and set the next byte flag.
    while value > 0b01111111 {
        stream.write::<8, u8>((value as u8 & 0b01111111) | 0b10000000)?;

        // Remove the seven bits we just wrote from value.
        value >>= 7;
    }

    stream.write::<8, u8>(value as u8)?;
    Ok(())
}

/// Decodes an unsigned variable-length integer using the MSB algorithm.
pub fn read_u64_variablelength<R: Read, E: Endianness>(stream: &mut BitReader<R, E>) -> Result<u64, MercError> {
    let mut value: u64 = 0;
    for i in 0..encoding_size::<u64>() {
        let byte = stream.read::<8, u8>()?;

        // Take 7 bits (mask 0b01111111) from byte and shift it before the bits already written to value.
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
    use super::BitReader;
    use super::BitWriter;
    use super::encoding_size;
    use super::read_u64_variablelength;
    use super::write_u64_variablelength;

    use bitstream_io::BigEndian;
    use rand::RngExt;

    use merc_utilities::random_test;

    /// Round-trips `value` through the encoder and decoder and asserts equality.
    fn roundtrip(value: u64) {
        let mut stream: [u8; encoding_size::<u64>()] = [0; encoding_size::<u64>()];
        let mut writer = BitWriter::<_, BigEndian>::new(&mut stream[0..]);
        write_u64_variablelength(&mut writer, value).unwrap();

        let mut reader = BitReader::<_, BigEndian>::new(&stream[0..]);
        assert_eq!(read_u64_variablelength(&mut reader).unwrap(), value);
    }

    #[test]
    fn test_encoding_size() {
        assert_eq!(encoding_size::<u8>(), 2);
        assert_eq!(encoding_size::<u16>(), 3);
        assert_eq!(encoding_size::<u32>(), 5);
        assert_eq!(encoding_size::<u64>(), 10);
    }

    #[test]
    fn test_boundary_encoding() {
        // Edge values and the per-byte continuation boundaries where the
        // encoded length grows by one byte (7 payload bits per byte).
        roundtrip(0);
        roundtrip(u64::MAX);
        for shift in [7, 14, 21, 28, 35, 42, 49, 56, 63] {
            roundtrip((1u64 << shift) - 1);
            roundtrip(1u64 << shift);
        }
    }

    #[test]
    fn test_random_integer_encoding() {
        random_test(1000, |rng| {
            roundtrip(rng.random());
        });
    }
}
