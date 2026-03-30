//! Authors: Maurice Laveaux and Sjef van Loo

use std::io::Read;
use std::io::Write;

use itertools::Itertools;
use log::info;
use regex::Regex;
use streaming_iterator::StreamingIterator;
use thiserror::Error;

use merc_io::LineIterator;
use merc_io::TimeProgress;
use merc_utilities::MercError;

use crate::PG;
use crate::ParityGame;
use crate::ParityGameBuilder;
use crate::Player;
use crate::Priority;
use crate::VertexIndex;

#[derive(Error, Debug)]
pub enum IOError {
    #[error("Invalid .pg header {0}")]
    InvalidHeader(&'static str),

    #[error("Invalid line {0}")]
    InvalidLine(&'static str),
}

/// Reads a parity game in textual PGSolver `.pg` format from the given reader.
///
/// # Details
///
/// The format starts with a header, followed by the vertices
///
/// `parity <num_of_vertices>;`
/// `<index> <priority> <owner> <outgoing_vertex>, <outgoing_vertex>, ...;`
pub fn read_pg<R: Read>(reader: R) -> Result<ParityGame, MercError> {
    info!("Reading parity game in .pg format...");

    let mut lines = LineIterator::new(reader);
    lines.advance();
    let header = lines
        .get()
        .ok_or(IOError::InvalidHeader("The first line should be the header"))?;

    // Read the header
    let header_regex = Regex::new(r#"parity\s+([0-9]+)\s*;"#).expect("Regex compilation should not fail");

    let (_, [num_of_vertices_txt]) = header_regex
        .captures(header)
        .ok_or(IOError::InvalidHeader("does not match parity <num_of_vertices>;"))?
        .extract();

    let num_of_vertices: usize = num_of_vertices_txt.parse()?;
    let progress = TimeProgress::new(
        |(amount, total): (usize, usize)| info!("Read {} vertices ({}%)...", amount, amount * 100 / total),
        1,
    );

    // Collect the data in a builder and remove duplicate edges at the end.
    let mut builder = ParityGameBuilder::with_capacity(VertexIndex::new(0), num_of_vertices);
    if num_of_vertices > 0 {
        builder.add_vertex(VertexIndex::new(num_of_vertices - 1), Player::Even, Priority::new(0));
    }

    let mut vertex_count = 0;
    while let Some(line) = lines.next() {
        // Parse the line: <index> <priority> <owner> <outgoing_vertex>, <outgoing_vertex>, ...;
        let mut parts = line.split_whitespace();

        let index: usize = parts
            .next()
            .ok_or(IOError::InvalidLine("Expected at least <index> ...;"))?
            .parse()?;
        let vertex_priority: usize = parts
            .next()
            .ok_or(IOError::InvalidLine("Expected at least <index> <priority> ...;"))?
            .parse()?;
        let vertex_owner: u8 = parts
            .next()
            .ok_or(IOError::InvalidLine(
                "Expected at least <index> <priority> <owner> ...;",
            ))?
            .parse()?;

        let vertex = VertexIndex::new(index);
        builder.add_vertex(vertex, Player::from_index(vertex_owner), Priority::new(vertex_priority));

        for successors in parts {
            // Parse successors (remaining parts, removing trailing semicolon)
            for successor in successors
                .trim_end_matches(';')
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().parse())
            {
                let successor = successor?;
                builder.add_edge(vertex, VertexIndex::new(successor));
            }
        }

        progress.print((vertex_count + 1, num_of_vertices));
        vertex_count += 1;
    }

    Ok(builder.finish(false, true))
}

/// Writes the given parity game to the given writer in .pg format.
pub fn write_pg<W: Write, G: PG>(mut writer: W, game: &G) -> Result<(), MercError> {
    info!("Writing parity game to .pg format...");

    let progress = TimeProgress::new(
        |(index, total): (usize, usize)| info!("Wrote {} vertices ({}%)...", index, index * 100 / total),
        1,
    );

    writeln!(writer, "parity {};", game.num_of_vertices())?;
    for v in game.iter_vertices() {
        let prio = game.priority(v);
        let owner = game.owner(v).to_index();

        write!(writer, "{} {} {} ", v.value(), prio.value(), owner)?;
        write!(
            writer,
            "{}",
            game.outgoing_edges(v).map(|edge| edge.to().value()).format(", ")
        )?;
        writeln!(writer, ";")?;
        progress.print((v.value() + 1, game.num_of_vertices()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_read_pg() {
        let parity_game = read_pg(include_bytes!("../../../../examples/vpg/example.pg") as &[u8]).unwrap();
        assert_eq!(parity_game.num_of_vertices(), 3002);
        assert_eq!(parity_game.num_of_edges(), 3968);
    }
}
