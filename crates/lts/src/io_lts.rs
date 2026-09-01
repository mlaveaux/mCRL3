#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

use log::debug;
use log::info;
use merc_aterm::ATerm;
use merc_aterm::ATermInt;
use merc_aterm::ATermList;
use merc_aterm::ATermRead;
use merc_aterm::ATermStreamable;
use merc_aterm::ATermWrite;
use merc_aterm::BinaryATermReader;
use merc_aterm::BinaryATermWriter;
use merc_aterm::Symbol;
use merc_aterm::is_list_term;
use merc_data::Mcrl2DataSpecification;
use merc_io::LargeFormatter;
use merc_io::TimeProgress;
use merc_utilities::MercError;

use crate::LTS;
use crate::LabelledTransitionSystem;
use crate::LtsAction;
use crate::LtsBuilder;
use crate::LtsBuilderMem;
use crate::LtsMultiAction;
use crate::StateIndex;

/// Loads a labelled transition system from the binary 'lts' format of the mCRL2 toolset,
/// together with the data specification that its action arguments are typed over.
///
/// # Errors
///
/// Returns an error if the stream does not start with the LTS marker, contains a
/// probabilistic transition (not supported), is missing its initial state, or is
/// otherwise malformed.
pub fn read_lts<R: Read>(
    reader: R,
    read_state_labels: bool,
) -> Result<
    (
        LabelledTransitionSystem<LtsMultiAction<LtsAction>>,
        Mcrl2DataSpecification,
    ),
    MercError,
> {
    info!("Reading LTS in .lts format...");

    let mut reader = BinaryATermReader::new(BufReader::new(reader))?;

    if reader.read_aterm()? != Some(lts_marker()) {
        return Err("Stream does not contain a labelled transition system (LTS).".into());
    }

    // Read the data specification, parameters, and actions.
    let data_spec = Mcrl2DataSpecification::read(&mut reader)?;
    let _parameters = reader.read_aterm()?;
    let _actions = reader.read_aterm()?;

    // Use a cache to avoid translating the same multi-action multiple times.
    let mut multi_actions: HashMap<ATerm, LtsMultiAction<LtsAction>> = HashMap::new();

    // The initial state is not known yet.
    let mut initial_state: Option<StateIndex> = None;
    let mut builder = LtsBuilderMem::new(Vec::new(), Vec::new());

    let progress = TimeProgress::new(
        |num_of_transitions| {
            info!("Read {num_of_transitions} transitions...");
        },
        1,
    );

    let mut state_labels = Vec::new();

    loop {
        let term = reader.read_aterm()?;
        match term {
            Some(t) => {
                if t == transition_marker() {
                    let from: ATermInt = reader.read_aterm()?.ok_or("Missing from state")?.into();
                    let label = reader.read_aterm()?.ok_or("Missing transition label")?;
                    let to: ATermInt = reader.read_aterm()?.ok_or("Missing to state")?.into();

                    if let Some(multi_action) = multi_actions.get(&label) {
                        // Multi-action already exists in the cache.
                        builder.add_transition(
                            StateIndex::new(from.value()),
                            multi_action,
                            StateIndex::new(to.value()),
                        )?;
                    } else {
                        // New multi-action found, add it to the builder.
                        let multi_action = LtsMultiAction::from_mcrl2_aterm(label.clone())?;
                        multi_actions.insert(label.clone(), multi_action.clone());
                        builder.add_transition(
                            StateIndex::new(from.value()),
                            &multi_action,
                            StateIndex::new(to.value()),
                        )?;
                    }

                    progress.print(builder.num_of_transitions());
                } else if t == probabilistic_transition_mark() {
                    return Err("Probabilistic transitions are not supported yet.".into());
                } else if is_list_term(&t) {
                    if read_state_labels {
                        let t: ATermList<ATerm> = t.into();
                        debug!("Read state label: {:?}", t.to_vec());
                        state_labels.push(t);
                    }
                } else if t == initial_state_marker() {
                    let length = ATermInt::from(reader.read_aterm()?.ok_or("Missing initial state length")?).value();
                    if length != 1 {
                        return Err("Initial state length greater than 1 is not supported.".into());
                    }

                    initial_state = Some(StateIndex::new(
                        ATermInt::from(reader.read_aterm()?.ok_or("Missing initial state index")?).value(),
                    ));
                    debug!("Initial state: {:?}", initial_state);
                } else {
                    return Err(format!("Unexpected term in LTS stream: {}", t).into());
                }
            }
            None => break, // The default constructed term indicates the end of the stream.
        }
    }
    info!("Finished reading LTS.");

    Ok((
        // Deduplicate: the binary `.lts` format is a plain transition relation and, unlike
        // merc's own internal representation, does not guarantee that a state has no two
        // identical outgoing `(label, to)` transitions -- e.g. mCRL2's `ltscombine` legitimately
        // emits such duplicates for its unreduced output.
        builder.finish(initial_state.ok_or("Missing initial state")?, true),
        data_spec,
    ))
}

/// Write a labelled transition system in binary 'lts' format to the given
/// writer. Requires that the labels are ATerm streamable. Note that the writer
/// is buffered internally using a `BufWriter`.
///
/// # Errors
///
/// Returns an error if writing to `writer` fails, or if a label cannot be
/// converted to its mCRL2 ATerm representation.
///
/// # Details
///
/// This format is built on top the ATerm binary format. The structure is as
/// follows:
///
/// ```plain
///     lts_marker: ATerm
///     data_spec: see [`merc_data::Mcrl2DataSpecification::write`]
///     parameters: ATermList
///     action_labels: ATermList
/// ```
///
/// Afterwards we can write the following elements in any order:
///
/// ```plain
/// initial state:
///    initial_state_marker: ATerm
///    state: ATermInt
///
/// transition:
///     transition_marker: ATerm
///     from: ATermInt
///     label: ATerm (the multi_action)
///     to: ATermInt
/// ```
///
/// state_label (index derived from order of appearance):
///    `state_label: ATermList::<DataExpression>`
pub fn write_lts<L, W>(writer: &mut W, lts: &L, data_spec: &Mcrl2DataSpecification) -> Result<(), MercError>
where
    L: LTS<Label = LtsMultiAction<LtsAction>>,
    W: Write,
{
    info!("Writing LTS in .lts format...");

    let mut writer = BinaryATermWriter::new(BufWriter::new(writer))?;

    writer.write_aterm(&lts_marker())?;

    // Write the data specification, parameters, and actions.
    data_spec.write(&mut writer)?;
    writer.write_aterm(&ATermList::<ATerm>::empty().into())?; // Empty parameters
    writer.write_aterm(&ATermList::<ATerm>::empty().into())?; // Empty action labels

    // Convert the internal multi-actions to the ATerm representation that mCRL2 expects.
    let label_terms = lts
        .labels()
        .iter()
        .map(|label| label.to_mcrl2_aterm())
        .collect::<Result<Vec<ATerm>, MercError>>()?;

    // Write the initial state.
    writer.write_aterm(&initial_state_marker())?;
    writer.write_aterm(&ATermInt::new(1))?; // Length of initial state is 1.
    writer.write_aterm(&ATermInt::new(*lts.initial_state_index()))?;

    let num_of_transitions = lts.num_of_transitions();
    let progress = TimeProgress::new(
        move |written: usize| {
            info!(
                "Wrote {} transitions ({}%)...",
                LargeFormatter(written),
                written * 100 / num_of_transitions
            );
        },
        1,
    );

    let mut written = 0;
    for state in lts.iter_states() {
        for transition in lts.outgoing_transitions(state) {
            writer.write_aterm(&transition_marker())?;
            writer.write_aterm(&ATermInt::new(*state))?;
            writer.write_aterm(&label_terms[transition.label.value()])?;
            writer.write_aterm(&ATermInt::new(*transition.to))?;

            progress.print(written);
            written += 1;
        }
    }

    info!("Finished writing LTS.");
    Ok(())
}

/// Returns the ATerm marker for a labelled transition system.
pub(crate) fn lts_marker() -> ATerm {
    ATerm::constant(&Symbol::new("labelled_transition_system", 0))
}

/// Returns the ATerm marker for a transition.
pub(crate) fn transition_marker() -> ATerm {
    ATerm::constant(&Symbol::new("transition", 0))
}

/// Returns the ATerm marker for the initial state.
pub(crate) fn initial_state_marker() -> ATerm {
    ATerm::constant(&Symbol::new("initial_state", 0))
}

/// Returns the ATerm marker for the probabilistic transition.
fn probabilistic_transition_mark() -> ATerm {
    ATerm::constant(&Symbol::new("probabilistic_transition", 0))
}

#[cfg(test)]
mod tests {
    use merc_data::Mcrl2DataSpecification;
    use merc_utilities::random_test;

    use super::write_lts;
    use crate::LTS;
    use crate::LtsAction;
    use crate::LtsMultiAction;
    use crate::random_lts;
    use crate::read_lts;

    #[test]
    #[cfg_attr(miri, ignore)] // Tests are too slow under miri.
    fn test_read_lts() {
        let (lts, _data_spec) = read_lts(include_bytes!("../../../examples/lts/abp.lts").as_ref(), true).unwrap();

        assert_eq!(lts.num_of_states(), 74);
        assert_eq!(lts.num_of_transitions(), 92);
        assert_eq!(*lts.initial_state_index(), 0);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_random_lts_io() {
        random_test(100, |rng| {
            let lts = random_lts::<LtsMultiAction<LtsAction>, _>(rng, 1000, 3);

            let mut buffer: Vec<u8> = Vec::new();
            write_lts(&mut buffer, &lts, &Mcrl2DataSpecification::default()).unwrap();

            let (result_lts, _data_spec) = read_lts(&buffer[0..], false).unwrap();

            crate::check_equivalent(&lts, &result_lts);
        })
    }
}
