use std::fmt;


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

impl fmt::Debug for DependencyGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DependencyGraph with {} vertices:", self.num_of_vertices)?;
        for (i, relation) in self.relations.iter().enumerate() {
            writeln!(f, "  {}: {:?}", i, relation)?;
        }
        Ok(())
    }
}

/// A single relation in the dependency graph containing read and write
/// dependencies onto variables, given by their indices.
pub struct Relation {
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

impl fmt::Debug for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.read_vars, self.write_vars)
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