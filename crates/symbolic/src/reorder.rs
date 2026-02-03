use merc_utilities::MercError;

use crate::DependencyGraph;

/// Default implementation of reorder when `kahypar` feature is not enabled.
#[cfg(not(feature = "kahypar"))]
pub fn reorder(_graph: &DependencyGraph) -> Result<Vec<usize>, MercError> {
    Err("reordering requires the `kahypar` feature to be enabled".into())
}

#[cfg(feature = "kahypar")]
mod inner {
    use super::*;

    use log::trace;

    use mt_kahypar::Context;
    use mt_kahypar::Hypergraph;
    use mt_kahypar::Objective;
    use mt_kahypar::Preset;

    /// Computes a variable reordering for symbolic transition relations using the MINCE algorithm.
    ///
    /// # Details
    ///
    /// The algorithm is described in the following paper:
    ///
    /// > Fadi A. Aloul, Igor L. Markov, Karem A. Sakallah:. MINCE: A Static Global Variable-Ordering Heuristic for SAT Search and BDD Manipulation. J. Univers. Comput. Sci. 10(12): 1562-1596 (2004). [DOI](https://doi.org/10.3217/jucs-010-12-1562)

    #[cfg(feature = "kahypar")]
    pub fn reorder(graph: &DependencyGraph) -> Result<Vec<usize>, MercError> {
        trace!("Starting MINCE with {graph:?}");

        let context = Context::builder()
            .preset(Preset::HighestQuality)
            .epsilon(0.01)
            .k(2)
            .objective(Objective::Cut)
            .build()?;

        let vertices = (0..graph.num_of_vertices()).collect::<Vec<usize>>();
        mince(&context, &vertices, graph)
    }

    /// The recursive MINCE algorithm to compute a partitioning of the given dependency graph.
    ///
    /// # Details
    ///
    /// The `vertices` are the indices of the subgraph that we are considering
    fn mince(context: &Context, vertices: &[usize], graph: &DependencyGraph) -> Result<Vec<usize>, MercError> {
        trace!("MINCE called with vertices: {:?}", vertices);

        let hypergraph = create_hypergraph(context, vertices, graph)?;

        let partition = hypergraph.partition()?;
        debug_assert_eq!(partition.num_blocks(), 2, "MINCE only supports bipartitioning");

        // Compute the two partitions
        let partition = partition.extract_partition();

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

        let mut left = mince(context, &left_vertices, graph)?;
        let mut right = mince(context, &right_vertices, graph)?;
        left.append(&mut right);

        // Check that the result is a valid permutation
        if cfg!(debug_assertions) {
            let mut copy = left.clone();
            copy.sort();

            debug_assert_eq!(copy, vertices, "Resulting order is not a valid permutation");
        }

        Ok(left)
    }

    /// Constructs a hypergraph CSR from the given read/write matrix.
    fn create_hypergraph<'a>(
        context: &'a Context,
        vertices: &[usize],
        graph: &DependencyGraph,
    ) -> Result<Hypergraph<'a>, MercError> {
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

        Ok(Hypergraph::from_adjacency(
            context,
            graph.num_of_vertices(),
            &hyperedge_indices,
            &hyperedges,
            None,
            None,
        )?)
    }
}

#[cfg(feature = "kahypar")]
pub use inner::reorder;
