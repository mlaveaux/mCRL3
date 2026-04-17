use std::fs::File;
use std::io::ErrorKind;
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
pub fn reorder(kahypar_path: &Path, kahypar_ini_path: &Path,graph: &DependencyGraph) -> Result<Vec<usize>, MercError> {
    debug!("Total span: {}", graph.total_span());

    let vertices = (0..graph.num_of_vertices()).collect::<Vec<usize>>();
    let result = mince(kahypar_path, kahypar_ini_path,&vertices, &[], graph)?;
    debug!("Reordered total span: {}", graph.reorder(&result).total_span());
    Ok(result)
}

/// The recursive MINCE algorithm to compute a partitioning of the given dependency graph.
///
/// # Details
///
/// The `vertices` are the indices of the subgraph that we are considering
fn mince(
    kahypar_path: &Path,
    kahypar_ini_path: &Path,
    vertices: &[usize],
    left_context: &[usize],
    graph: &DependencyGraph,
) -> Result<Vec<usize>, MercError> {
    trace!("MINCE called with vertices: {:?}", vertices);
    let partition = partition(kahypar_path, kahypar_ini_path, vertices, left_context, graph)?;

    if partition.len() <= 2 {
        // Base case: a single vertex is already "ordered"
        trace!("MINCE reached base case with vertices: {:?}", vertices);
        return Ok(partition);
    }

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

    let mut left = mince(kahypar_path, kahypar_ini_path, &left_vertices, left_context, graph)?;

    let mut new_left_context = left_context.to_vec();
    new_left_context.extend(&left_vertices);
    let mut right = mince(kahypar_path, kahypar_ini_path, &right_vertices, &new_left_context, graph)?;
    left.append(&mut right);

    // Check that the result is a valid permutation
    if cfg!(debug_assertions) {
        let mut copy = left.clone();
        copy.sort();

        debug_assert_eq!(copy, vertices, "Resulting order is not a valid permutation");
    }

    Ok(left)
}

/// A hypergraph representation with indices, edges, and vertex weights.
pub struct Hypergraph {
    /// Indices into the edges vector marking the start of each hyperedge.
    pub indices: Vec<usize>,
    /// The hyperedges, stored as a flat list of vertex indices.
    pub edges: Vec<usize>,
    /// Weights for each vertex in the hypergraph.
    pub weights: Vec<usize>,
}

/// Constructs a hypergraph from the given dependency graph.
///
/// # Details
///
/// The `vertices` are the indices of the subgraph that we are considering.
fn create_hypergraph(
    vertices: &[usize],
    left_context: &[usize],
    graph: &DependencyGraph,
) -> Result<Hypergraph, MercError> {
    let mut hyperedge_indices = Vec::with_capacity(graph.num_of_relations() + 1);
    let mut hyperedges = Vec::new();
    let mut weights = vec![1; vertices.len() + 2]; // +2 for the pseudo-vertices
    for (index, vertex) in vertices.iter().enumerate() {
        // Calculate the total number of edges that the vertex is involved in, and use it as the weight.
        weights[index] = graph
            .relations()
            .filter(|relation| {
                relation.read_vars().any(|j| j == *vertex) || relation.write_vars().any(|j| j == *vertex)
            })
            .count();
    }

    let mut offset = 0usize;

    // Add two pseudo-vertices to represent the "left" and "right" context of the partition.
    let left_pseudo_vertex = vertices.len();
    let right_pseudo_vertex = vertices.len() + 1;

    // They should not contribute to the cut cost.
    weights[left_pseudo_vertex] = 1;
    weights[right_pseudo_vertex] = 1;

    // Make a hyperedge for every relation
    // Track unique edges as sorted lists of local vertex indices
    let mut seen_edges: Vec<Vec<usize>> = Vec::new();

    for relation in graph.relations() {
        // Collect only variables that are in `vertices`, and use their local indices
        let edge_vars: Vec<usize> = relation
            .read_vars()
            .chain(relation.write_vars())
            .map(|j| {
                match vertices.iter().position(|i| *i == j) {
                    Some(local_index) => local_index,
                    None => {
                        // Variable is not in the current subgraph
                        // Check if it is in the left or right context
                        if left_context.contains(&j) {
                            left_pseudo_vertex
                        } else {
                            right_pseudo_vertex
                        }
                    }
                }
            })
            .collect();

        add_edge(
            &mut hyperedge_indices,
            &mut hyperedges,
            &mut offset,
            &mut seen_edges,
            edge_vars,
        );
    }

    hyperedge_indices.push(offset);
    Ok(Hypergraph {
        indices: hyperedge_indices,
        edges: hyperedges,
        weights,
    })
}

/// Adds an edge to the hypergraph, while ensuring that it is not a self-loop, empty, or duplicated.
fn add_edge(
    hyperedge_indices: &mut Vec<usize>,
    hyperedges: &mut Vec<usize>,
    offset: &mut usize,
    seen_edges: &mut Vec<Vec<usize>>,
    mut edge_vars: Vec<usize>,
) {
    // Deduplicate within-edge vertices and normalize order
    edge_vars.sort_unstable();
    edge_vars.dedup();

    if edge_vars.len() <= 1 {
        // Ignore self-loops and empty edges
        return;
    }

    if seen_edges.iter().any(|e| e == &edge_vars) {
        // Ignore duplicated edges
        return;
    }
    seen_edges.push(edge_vars.clone());
    hyperedge_indices.push(*offset);

    // Add the edge to the hypergraph
    for j in edge_vars {
        hyperedges.push(j);
        *offset += 1;
    }
}

/// Partitions the given hypergraph using the `kahypar` tool.
fn partition(
    kahypar_path: &Path,
    kahypar_ini_path: &Path,
    vertices: &[usize],
    left_context: &[usize],
    graph: &DependencyGraph,
) -> Result<Vec<usize>, MercError> {
    let hypergraph = create_hypergraph(vertices, left_context, graph)?;

    if vertices.len() <= 2 || hypergraph.edges.len() <= 1 {
        return Ok(vertices.to_vec());
    }

    run_kahypar(kahypar_path, kahypar_ini_path, &hypergraph)?;

    let partition = read_partition_file()?;

    debug_assert!(partition.iter().all(|x| *x <= 1), "MINCE only supports bipartitioning");
    Ok(partition)
}

/// Writes `reorder.hgr`, runs KaHyPar, and removes the temporary file again.
fn run_kahypar(kahypar_path: &Path, kahypar_ini_path: &Path, hypergraph: &Hypergraph) -> Result<(), MercError> {
    const HYPERGRAPH_FILE: &str = "reorder.hgr";

    let result = (|| {
        // Create a file to write the hypergraph to disk in hMetis format.
        let mut file = File::create_new(HYPERGRAPH_FILE)
            .map_err(|e| format!("Failed to create file '{HYPERGRAPH_FILE}': {e}"))?;

        // Expected <num_hyperedges> <num_hypernodes> <type> (line 1)
        // type 10 is vertex weights only.
        writeln!(
            &mut file,
            "{} {} 10",
            hypergraph.indices.len() - 1,
            hypergraph.weights.len()
        )?;

        for (from, to) in hypergraph.indices.iter().tuple_windows() {
            let edge = &hypergraph.edges[*from..*to];
            writeln!(&mut file, "{}", edge.iter().map(|i| i + 1).format(" "))?;
        }

        for weight in &hypergraph.weights {
            writeln!(&mut file, "{} ", weight)?;
        }

        file.flush()?;
        cmd!(
            kahypar_path,
            "-h",
            HYPERGRAPH_FILE,
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

        Ok::<(), MercError>(())
    })();

    remove_file_if_exists(HYPERGRAPH_FILE)?;
    result
}

/// Reads KaHyPar's partition output and removes the temporary file again.
fn read_partition_file() -> Result<Vec<usize>, MercError> {
    const PARTITION_FILE: &str = "reorder.hgr.part2.epsilon0.01.seed-1.KaHyPar";

    let result = (|| {
        let partition_file = File::open(PARTITION_FILE)?;
        let mut partition = Vec::new();

        for line in BufReader::new(partition_file).lines() {
            let line = line?;
            let block_id: usize = line.trim().parse()?;
            partition.push(block_id);
        }

        Ok::<Vec<usize>, MercError>(partition)
    })();

    remove_file_if_exists(PARTITION_FILE)?;
    result
}

/// Removes the specified file if it exists, ignoring "file not found" errors.
fn remove_file_if_exists(path: &str) -> Result<(), MercError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
