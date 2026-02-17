use std::borrow::Borrow;
use std::f64::consts::LOG10_2;
use std::hash::Hash;
use std::io::BufWriter;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::marker::PhantomData;

use merc_utilities::MercError;

use crate::LtsBuilder;
use crate::StateIndex;
use crate::TransitionLabel;

pub struct AutStream<W: Write, L> {
    writer: BufWriter<W>,

    /// Keep track of the number of transitions added.
    number_of_transitions: usize,

    /// Keep track of the number of states added.
    number_of_states: usize,

    _marker: PhantomData<L>,
}

impl<W: Write, L> AutStream<W, L> {
    /// Creates a new AUT stream writer.
    ///
    /// Note that the writer is buffered internally using a `BufWriter`.
    pub fn new(writer: W) -> Self {
        let mut writer = BufWriter::new(writer);
        // Write a placeholder for the header, which will be filled in later.
        // Reserve enough space for the header using the number of bits of
        // usize. This avoids overwriting transition bytes when the final header
        // is longer.
        let max_usize_digits = (usize::BITS as f64 * LOG10_2).ceil() as usize;
        let header_len = format!("des ({0:<1$}, {0:<1$}, {0:<1$})\n", " ", max_usize_digits).len();
        writer.write_all(" ".repeat(header_len).as_bytes()).unwrap();

        Self {
            writer,
            number_of_transitions: 0,
            number_of_states: 0,
            _marker: PhantomData,
        }
    }
}

impl<W: Write + Seek, L: TransitionLabel> LtsBuilder<L> for AutStream<W, L> {
    type LTS = ();

    fn add_transition<Q>(&mut self, from: StateIndex, label: &Q, to: StateIndex) -> Result<(), MercError>
    where
        L: Borrow<Q>,
        Q: ?Sized + ToOwned<Owned = L> + Eq + Hash,
    {
        self.number_of_transitions += 1;
        self.number_of_states = self.number_of_states.max(from.value() + 1).max(to.value() + 1);

        writeln!(self.writer, "({}, \"{}\", {})", from, label.to_owned(), to)?;
        Ok(())
    }

    fn finish(&mut self, initial_state: StateIndex) -> Result<Self::LTS, MercError> {
        // Flush to ensure all buffered transitions are written
        self.writer.flush()?;

        // Seek to the start and overwrite the header
        self.writer.seek(SeekFrom::Start(0))?;
        writeln!(
            self.writer,
            "des ({}, {}, {})",
            initial_state, self.number_of_transitions, self.number_of_states
        )?;

        // Flush the updated header
        self.writer.flush()?;
        Ok(())
    }

    /// Returns the number of transitions added to the builder.
    fn num_of_transitions(&self) -> usize {
        self.number_of_transitions
    }

    /// Returns the number of states added to the builder.
    fn num_of_states(&self) -> usize {
        self.number_of_states
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use merc_utilities::random_test;

    use crate::AutStream;
    use crate::LTS;
    use crate::LtsBuilder;
    use crate::random_lts_monolithic;
    use crate::read_aut;

    #[test]
    #[cfg_attr(miri, ignore)] // Test is too slow under Miri
    fn test_random_aut_stream_io() {
        random_test(100, |rng| {
            let lts = random_lts_monolithic::<String, _>(rng, 100, 3, 20);

            let mut buffer = Cursor::new(Vec::new());
            {
                let mut stream = AutStream::new(&mut buffer);

                // Write all the transitions to the stream.
                for state_index in lts.iter_states() {
                    for transition in lts.outgoing_transitions(state_index) {
                        stream
                            .add_transition(state_index, &lts.labels()[transition.label.value()], transition.to)
                            .unwrap();
                    }
                }

                stream.finish(lts.initial_state_index()).unwrap();
            }

            // Rewind the buffer to the beginning before reading.
            buffer.set_position(0);
            let result_lts = read_aut(&mut buffer, vec![]).unwrap();

            crate::check_equivalent(&lts, &result_lts);
        })
    }
}
