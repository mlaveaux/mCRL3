use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::time::Instant;

use log::debug;
use log::trace;
use regex::Regex;
use streaming_iterator::StreamingIterator;
use thiserror::Error;

use mcrl3_io::LineIterator;
use mcrl3_io::Progress;
use mcrl3_utilities::MCRL3Error;

use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::StateIndex;

#[derive(Error, Debug)]
pub enum IOError {
    #[error("Invalid .aut header {0}")]
    InvalidHeader(&'static str),

    #[error("Invalid transition line")]
    InvalidTransition(),
}

///     `(<from>: Nat, "<label>": Str, <to>: Nat)`
///     `(<from>: Nat, <label>: Str, <to>: Nat)`
fn read_transition(input: &str) -> Result<(&str, &str, &str), MCRL3Error> {
    let start_paren = input.find('(').ok_or(IOError::InvalidTransition())?;
    let start_comma = input.find(',').ok_or(IOError::InvalidTransition())?;

    // Find the comma in the second part
    let start_second_comma = input.rfind(',').ok_or(IOError::InvalidTransition())?;
    let end_paren = input.rfind(')').ok_or(IOError::InvalidTransition())?;

    let from = &input[start_paren + 1..start_comma].trim();
    let label = &input[start_comma + 1..start_second_comma].trim();
    let to = &input[start_second_comma + 1..end_paren].trim();
    // Handle the special case where it has quotes.
    if label.starts_with('"') && label.ends_with('"') {
        return Ok((from, &label[1..label.len() - 1], to));
    }

    Ok((from, label, to))
}

/// Loads a labelled transition system in the Aldebaran format from the given reader.
///
/// The Aldebaran format consists of a header:
///     `des (<initial>: Nat, <num_of_transitions>: Nat, <num_of_states>: Nat)`
///     
/// And one line for every transition:
///     `(<from>: Nat, "<label>": Str, <to>: Nat)`
///     `(<from>: Nat, <label>: Str, <to>: Nat)`
pub fn read_aut(reader: impl Read, mut hidden_labels: Vec<String>) -> Result<LabelledTransitionSystem, MCRL3Error> {
    let start = Instant::now();
    debug!("Reading LTS in .aut format...");

    let mut lines = LineIterator::new(reader);
    lines.advance();
    let header = lines
        .get()
        .ok_or(IOError::InvalidHeader("The first line should be the header"))?;

    // Regex for des (<initial>: Nat, <num_of_states>: Nat, <num_of_transitions>: Nat)
    let header_regex = Regex::new(r#"des\s*\(\s*([0-9]*)\s*,\s*([0-9]*)\s*,\s*([0-9]*)\s*\)\s*"#)
        .expect("Regex compilation should not fail");

    let (_, [initial_txt, num_of_transitions_txt, num_of_states_txt]) = header_regex
        .captures(header)
        .ok_or(IOError::InvalidHeader(
            "does not match des (<init>, <num_of_transitions>, <num_of_states>)",
        ))?
        .extract();

    let initial_state = StateIndex::new(initial_txt.parse()?);
    let num_of_transitions: usize = num_of_transitions_txt.parse()?;
    let num_of_states: usize = num_of_states_txt.parse()?;

    // This is used to keep track of the label to index mapping.
    let mut labels_index: HashMap<String, LabelIndex> = HashMap::new();
    let mut labels: Vec<String> = Vec::new();

    let mut transitions: Vec<(StateIndex, LabelIndex, StateIndex)> = Vec::default();
    let mut progress = Progress::new(
        |value, increment| debug!("Reading transitions {}%...", value / increment),
        num_of_transitions,
    );

    while let Some(line) = lines.next() {
        trace!("{}", line);
        let (from_txt, label_txt, to_txt) = read_transition(line)?;

        // Parse the from and to states, with the given label.
        let from = StateIndex::new(from_txt.parse()?);
        let to =  StateIndex::new(to_txt.parse()?);

        let label_index = *labels_index.entry(label_txt.to_string()).or_insert(LabelIndex::new(labels.len()));

        if *label_index >= labels.len() {
            labels.resize_with(label_index.value() + 1, Default::default);
        }

        trace!("Read transition {} --[{}]-> {}", from, label_txt, to);

        transitions.push((from, label_index, to));

        if labels[*label_index].is_empty() {
            labels[*label_index] = label_txt.to_string();
        }

        progress.add(1);
    }

    // Remove duplicated transitions, it is not clear if they are allowed in the .aut format.
    transitions.sort_unstable();
    transitions.dedup();

    debug!("Finished reading LTS");

    hidden_labels.push("tau".to_string());
    debug!("Time read_aut: {:.3}s", start.elapsed().as_secs_f64());
    Ok(LabelledTransitionSystem::new(
        initial_state,
        Some(num_of_states),
        || transitions.iter().cloned(),
        labels,
        hidden_labels,
    ))
}

/// Write a labelled transition system in plain text in Aldebaran format to the given writer.
pub fn write_aut(writer: &mut impl Write, lts: &LabelledTransitionSystem) -> Result<(), MCRL3Error> {
    writeln!(
        writer,
        "des ({}, {}, {})",
        lts.initial_state_index(),
        lts.num_of_transitions(),
        lts.num_of_states()
    )?;

    for state_index in lts.iter_states() {
        for (label, to) in lts.outgoing_transitions(state_index) {
            writeln!(
                writer,
                "({}, \"{}\", {})",
                state_index,
                if lts.is_hidden_label(*label) {
                    "tau"
                } else {
                    &lts.labels()[label.value()]
                },
                to
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    #[test]
    fn test_reading_aut() {
        let file = include_str!("../../../examples/lts/abp.aut");

        let lts = read_aut(file.as_bytes(), vec![]).unwrap();

        assert_eq!(lts.initial_state_index().value(), 0);
        assert_eq!(lts.num_of_transitions(), 92);
        println!("{}", lts);
    }

    #[test]
    fn test_lts_failure() {
        let wrong_header = "
        des (0,2,                                     
            (0,\"r1(d1)\",1)
            (0,\"r1(d2)\",2)
        ";

        debug_assert!(read_aut(wrong_header.as_bytes(), vec![]).is_err());

        let wrong_transition = "
        des (0,2,3)                           
            (0,\"r1(d1),1)
            (0,\"r1(d2)\",2)
        ";

        debug_assert!(read_aut(wrong_transition.as_bytes(), vec![]).is_err());
    }

    #[test]
    fn test_traversal_lts() {
        let file = include_str!("../../../examples/lts/abp.aut");

        let lts = read_aut(file.as_bytes(), vec![]).unwrap();

        // Check the number of outgoing transitions of the initial state
        assert_eq!(lts.outgoing_transitions(lts.initial_state_index()).count(), 2);
    }

    #[test]
    fn test_writing_lts() {
        let file = include_str!("../../../examples/lts/abp.aut");
        let lts_original = read_aut(file.as_bytes(), vec![]).unwrap();

        // Check that it can be read after writing, and results in the same LTS.
        let mut buffer: Vec<u8> = Vec::new();
        write_aut(&mut buffer, &lts_original).unwrap();

        let lts = read_aut(&buffer[0..], vec![]).unwrap();

        assert!(lts.num_of_states() == lts_original.num_of_states());
        assert!(lts.num_of_labels() == lts_original.num_of_labels());
    }
}
