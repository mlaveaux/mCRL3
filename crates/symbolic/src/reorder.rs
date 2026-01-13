use merc_utilities::MercError;
use mt_kahypar::{Context, Hypergraph};

/// Computes a variable reordering for symbolic transition relations using the MINCE algorithm.
pub fn reorder(graph: &DependencyGraph) -> Result<Vec<usize>, MercError> {
    let context = Context::builder().build()?;

    let vertices = (0..graph.num_of_vertices()).collect::<Vec<usize>>();
    mince(&context, &vertices, graph)
}

/// The recursive MINCE algorithm to compute a partitioning of the given dependency graph.
///
/// # Details
///
/// The `vertices` are the indices of the subgraph that we are considering
pub fn mince(context: &Context, vertices: &[usize], graph: &DependencyGraph) -> Result<Vec<usize>, MercError> {
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

    let mut left = mince(context, &left_vertices, graph)?;

    let right_vertices: Vec<usize> = vertices
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| if partition[i] == 1 { Some(v) } else { None })
        .collect();

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
pub fn create_hypergraph<'a>(
    context: &'a Context,
    vertices: &[usize],
    graph: &DependencyGraph,
) -> Result<Hypergraph<'a>, MercError> {
    let mut hyperedge_indices = Vec::with_capacity(graph.num_of_relations() + 1);
    let mut hyperedges = Vec::new();

    let mut offset = 0usize;

    // Make a hyperedge for every relation
    for relation in graph.relations() {
        // Collect only variables that are in `vertices`, and use their indices
        let edge_vars: Vec<usize> = relation
            .read_vars()
            .chain(relation.write_vars())
            .filter_map(|j| vertices.iter().position(|i| *i == j))
            .collect();

        if edge_vars.len() <= 1 {
            // Ignore self-loops and empty edges
            continue;
        }

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

/// Represents a dependency graph between variables used in symbolic transition relations.
pub struct DependencyGraph {
    /// The list of relations in the dependency graph.
    relations: Vec<Relation>,

    /// The number of vertices
    num_of_vertices: usize,
}

impl DependencyGraph {
    /// Creates a new dependency graph from the given relations.
    pub fn new(relations: Vec<Relation>) -> Self {
        let num_of_vertices = relations
            .iter()
            .flat_map(|rel| rel.read_vars.iter().chain(rel.write_vars.iter()))
            .copied()
            .max()
            .map_or(0, |max_index| max_index + 1);

        DependencyGraph {
            relations,
            num_of_vertices,
        }
    }

    /// Returns the number of vertices in the dependency graph.
    pub fn num_of_vertices(&self) -> usize {
        self.num_of_vertices
    }

    /// Number of relations in the dependency graph.
    pub fn num_of_relations(&self) -> usize {
        self.relations.len()
    }

    /// Returns an iterator over the relations in the dependency graph.
    pub fn relations(&self) -> impl Iterator<Item = &Relation> {
        self.relations.iter()
    }
}

/// A single relation in the dependency graph containing read and write
/// dependencies onto variables, given by their indices.
struct Relation {
    read_vars: Vec<usize>,
    write_vars: Vec<usize>,
}

impl Relation {
    /// Returns an iterator over the read variables in this relation.
    pub fn read_vars(&self) -> impl Iterator<Item = usize> + '_ {
        self.read_vars.iter().copied()
    }

    /// Returns an iterator over the write variables in this relation.
    pub fn write_vars(&self) -> impl Iterator<Item = usize> + '_ {
        self.write_vars.iter().copied()
    }
}

/// Parses a dependency graph as output by
/// [lpreach](https://mcrl2.org/web/user_manual/tools/release/lpsreach.html) and
/// [pbessolvesymbolic](https://mcrl2.org/web/user_manual/tools/release/pbessolvesymbolic.html)
/// flag `--info`.
pub fn parse_compacted_dependency_graph(input: &str) -> DependencyGraph {
    let mut relations = Vec::new();

    for line in input.lines() {
        // Keep only pattern characters, ignoring indices/whitespace
        let pattern: Vec<char> = line.chars().filter(|c| matches!(c, '+' | '-' | 'r' | 'w')).collect();

        if pattern.is_empty() {
            continue;
        }

        let mut read_vars = Vec::new();
        let mut write_vars = Vec::new();

        for (col, ch) in pattern.into_iter().enumerate() {
            match ch {
                '+' => {
                    read_vars.push(col);
                    write_vars.push(col);
                }
                'r' => read_vars.push(col),
                'w' => write_vars.push(col),
                '-' => {}
                _ => {}
            }
        }

        relations.push(Relation { read_vars, write_vars });
    }

    DependencyGraph::new(relations)
}

#[cfg(test)]
mod tests {
    use crate::parse_compacted_dependency_graph;

    #[test]
    fn test_parse_abp_dependency_graph() {
        let input = "1 +w---------
2 ---+++-----
3 ------++---
4 --------++-
5 ------+w+w+
6 ---+ww--+w-
7 ---+++--+wr
8 +-----+w---
9 +rr+ww-----
10 +++---++---";

        let graph = parse_compacted_dependency_graph(input);

        assert_eq!(graph.relations.len(), 10);
    }
}
