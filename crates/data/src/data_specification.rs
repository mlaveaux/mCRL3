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

        Ok(DataSpecification { sorts, aliases, constructors, mappings, equations })
    }
}
