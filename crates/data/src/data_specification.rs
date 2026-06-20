use merc_aterm::ATerm;
use merc_aterm::ATermRead;
use merc_aterm::ATermStreamable;
use merc_aterm::ATermWrite;
use merc_utilities::MercError;

/// Stores the five sections of an mCRL2 data specification as raw terms, enabling lossless
/// round-trip serialization of binary formats that embed a data specification.
#[derive(Default)]
pub struct DataSpecification {
    sorts: Vec<ATerm>,
    aliases: Vec<ATerm>,
    constructors: Vec<ATerm>,
    mappings: Vec<ATerm>,
    equations: Vec<ATerm>,
}

impl ATermStreamable for DataSpecification {
    fn write<W: ATermWrite>(&self, writer: &mut W) -> Result<(), MercError> {
        writer.write_aterm_iter(self.sorts.iter().cloned())?;
        writer.write_aterm_iter(self.aliases.iter().cloned())?;
        writer.write_aterm_iter(self.constructors.iter().cloned())?;
        writer.write_aterm_iter(self.mappings.iter().cloned())?;
        writer.write_aterm_iter(self.equations.iter().cloned())?;
        Ok(())
    }

    fn read<R: ATermRead>(reader: &mut R) -> Result<Self, MercError>
    where
        Self: Sized,
    {
        let sorts = reader.read_aterm_iter()?.collect::<Result<Vec<ATerm>, _>>()?;
        let aliases = reader.read_aterm_iter()?.collect::<Result<Vec<ATerm>, _>>()?;
        let constructors = reader.read_aterm_iter()?.collect::<Result<Vec<ATerm>, _>>()?;
        let mappings = reader.read_aterm_iter()?.collect::<Result<Vec<ATerm>, _>>()?;
        let equations = reader.read_aterm_iter()?.collect::<Result<Vec<ATerm>, _>>()?;

        Ok(DataSpecification {
            sorts,
            aliases,
            constructors,
            mappings,
            equations,
        })
    }
}

#[cfg(test)]
mod tests {
    use merc_aterm::ATerm;
    use merc_aterm::BinaryATermReader;
    use merc_aterm::BinaryATermWriter;

    use super::*;

    #[test]
    fn test_data_specification_roundtrip() {
        // Populate every section, including an empty one, to exercise the length-prefixed framing.
        let spec = DataSpecification {
            sorts: vec![ATerm::from_string("SortId(Nat)").unwrap()],
            aliases: Vec::new(),
            constructors: vec![ATerm::from_string("c").unwrap(), ATerm::from_string("d").unwrap()],
            mappings: vec![ATerm::from_string("f(a)").unwrap()],
            equations: vec![ATerm::from_string("eq(x, y)").unwrap()],
        };

        let mut stream: Vec<u8> = Vec::new();
        let mut writer = BinaryATermWriter::new(&mut stream).unwrap();
        spec.write(&mut writer).unwrap();
        ATermWrite::flush(&mut writer).unwrap();
        drop(writer); // Release the mutable borrow on the stream.

        let mut reader = BinaryATermReader::new(&stream[..]).unwrap();
        let read = DataSpecification::read(&mut reader).unwrap();

        assert_eq!(read.sorts, spec.sorts);
        assert_eq!(read.aliases, spec.aliases);
        assert_eq!(read.constructors, spec.constructors);
        assert_eq!(read.mappings, spec.mappings);
        assert_eq!(read.equations, spec.equations);
    }
}
