use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use duct::cmd;
use itertools::Itertools;
use log::debug;
use log::trace;

use merc_utilities::MercError;

use crate::DependencyGraph;

/// Default implementation of reorder when `kahypar` feature is not enabled.
pub fn reorder(kahypar_path: &Path, graph: &DependencyGraph) -> Result<Vec<usize>, MercError> {
    debug!("Total span: {}", graph.total_span());

    let vertices = (0..graph.num_of_vertices()).collect::<Vec<usize>>();
    let result = mince(kahypar_path, &vertices, graph)?;
    debug!("Reordered total span: {}", graph.reorder(&result).total_span());
    Ok(result)
}

/// The recursive MINCE algorithm to compute a partitioning of the given dependency graph.
///
/// # Details
///
/// The `vertices` are the indices of the subgraph that we are considering
fn mince(kahypar_path: &Path, vertices: &[usize], graph: &DependencyGraph) -> Result<Vec<usize>, MercError> {
    trace!("MINCE called with vertices: {:?}", vertices);


    let (hypergraph_indices, hypergraph_edges) = create_hypergraph(vertices, graph)?;

    if vertices.len() <= 2 || hypergraph_edges.len() <= 1 {
        // Base case: a single vertex is already "ordered"
        trace!("MINCE reached base case with vertices: {:?}", vertices);
        return Ok(vertices.to_vec());
    }

    let partition = partition(kahypar_path, vertices.len(), hypergraph_indices, hypergraph_edges)?;

    // We kept the indices of vertices in the hypergraph the same as `vertices`, so we can now
    // separate them according to the partitioning.
    let left_vertices: Vec<usize> = vertices
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| if partition[i] == 0 { Some(v) } else { None })
        .collect();

    let right_vertices: Vec<usize> = vertices
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| if partition[i] == 1 { Some(v) } else { None })
        .collect();

    if left_vertices.is_empty() || right_vertices.is_empty() {
        // Cannot partition further, return as is
        trace!("MINCE reached base case with vertices: {:?}", vertices);
        return Ok(vertices.to_vec());
    }

    let mut left = mince(kahypar_path, &left_vertices, graph)?;
    let mut right = mince(kahypar_path, &right_vertices, graph)?;
    left.append(&mut right);

    // Check that the result is a valid permutation
    if cfg!(debug_assertions) {
        let mut copy = left.clone();
        copy.sort();

        debug_assert_eq!(copy, vertices, "Resulting order is not a valid permutation");
    }

    Ok(left)
}

/// Constructs a hypergraph from the given dependency graph. Returns the hypergraph in the form of
/// (hyperedge_indices, hyperedges).
/// 
/// # Details
/// 
/// The `vertices` are the indices of the subgraph that we are considering.
fn create_hypergraph<'a>(vertices: &[usize], graph: &DependencyGraph) -> Result<(Vec<usize>, Vec<usize>), MercError> {
    let mut hyperedge_indices = Vec::with_capacity(graph.num_of_relations() + 1);
    let mut hyperedges = Vec::new();

    let mut offset = 0usize;

    // Make a hyperedge for every relation
    // Track unique edges as sorted lists of local vertex indices
    let mut seen_edges: Vec<Vec<usize>> = Vec::new();

    for relation in graph.relations() {
        // Collect only variables that are in `vertices`, and use their local indices
        let mut edge_vars: Vec<usize> = relation
            .read_vars()
            .chain(relation.write_vars())
            .filter_map(|j| vertices.iter().position(|i| *i == j))
            .collect();

        // Deduplicate within-edge vertices and normalize order
        edge_vars.sort_unstable();
        edge_vars.dedup();

        if edge_vars.len() <= 1 {
            // Ignore self-loops and empty edges
            continue;
        }

        // Ignore duplicated edges
        if seen_edges.iter().any(|e| e == &edge_vars) {
            continue;
        }
        seen_edges.push(edge_vars.clone());

        // Add the edge to the hypergraph
        hyperedge_indices.push(offset);
        for j in edge_vars {
            hyperedges.push(j);
            offset += 1;
        }
    }

    hyperedge_indices.push(offset);
    Ok((hyperedge_indices, hyperedges))
}

/// Partitions the given hypergraph using the `kahypar` tool.
fn partition(
    kahypar_path: &Path,
    num_of_vertices: usize,
    hypergraph_indices: Vec<usize>,
    hypergraph_edges: Vec<usize>,
) -> Result<Vec<usize>, MercError> {
    // Create a file to write the hypergraph to disk in hMetis format.
    let mut file = File::create_new("reorder.hgr")?;

    // Expected <num_hyperedges> <num_hypernodes> <type> (line 1)
    writeln!(&mut file, "{} {} 0", hypergraph_indices.len() - 1, num_of_vertices)?;

    for (from, to) in hypergraph_indices.iter().tuple_windows() {
        let edge = &hypergraph_edges[*from..*to];
        writeln!(&mut file, "{}", edge.iter().map(|i| i + 1).format(" "))?;
    }

    // Get path relative to the current executable
    let mut kahypar_ini_path = std::env::current_exe()?;
    kahypar_ini_path.pop(); // remove the executable filename
    kahypar_ini_path.push("kahypar.ini");

    file.flush()?;
    cmd!(
        kahypar_path,
        "-h",
        "reorder.hgr",
        "-k",
        "2",
        "--objective",
        "cut",
        "--mode",
        "direct",
        "--epsilon",
        "0.01",
        "-w",
        "1",
        "-p",
        kahypar_ini_path
    )
    .run()?;

    // Clean up the hypergraph file.
    std::fs::remove_file("reorder.hgr")?;

    // Read the partitioning result from the output file.
    let partition_file = File::open("reorder.hgr.part2.epsilon0.01.seed-1.KaHyPar")?;

    let mut partition = Vec::new();
    for line in BufReader::new(partition_file).lines() {
        let line = line?;
        let block_id: usize = line.trim().parse()?;
        partition.push(block_id);
    }

    // Clean up the partition file.
    std::fs::remove_file("reorder.hgr.part2.epsilon0.01.seed-1.KaHyPar")?;

    debug_assert!(partition.iter().all(|x| *x <= 1), "MINCE only supports bipartitioning");
    Ok(partition)
}
