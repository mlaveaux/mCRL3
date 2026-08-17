use std::path::PathBuf;

use log::debug;

use merc_explore::validate_permutation;
use merc_utilities::MercError;

use crate::DependencyGraph;
use crate::ReadWritePattern;
use crate::Relation;
use crate::reorder;

/// The order in which the positions of a state vector are stored in a decision diagram.
///
/// A variable order is a permutation `order` where `order[level]` is the state vector position stored
/// at level `level` of the diagram. It does not change the represented set of states, only the size of
/// the diagram: the closer the positions that a transition group reads and writes are together, the
/// smaller its relation tends to be.
///
/// The order is applied by permuting the state vector of the LPS itself, with
/// [`merc_explore::PermutedLps`], before it is encoded symbolically. Position `i` of the permuted LPS
/// is then stored at level `i`, so nothing downstream of that step has to know the order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum VariableOrder {
    /// Keeps the positions in the order of the state vector.
    #[default]
    None,

    /// Computes an order with the MINCE algorithm from the read/write matrix, which requires the
    /// [KaHyPar](https://github.com/kahypar/kahypar) hypergraph partitioner.
    ///
    /// This is the reordering that the `merc-sym reorder` command applies to the read/write matrix
    /// reported by mCRL2's `lpsreach --info`, here computed from the read/write matrix directly.
    Mince {
        /// Path to the `KaHyPar` executable.
        kahypar_path: PathBuf,

        /// Path to the `kahypar.ini` configuration file it is invoked with.
        kahypar_ini_path: PathBuf,
    },

    /// An explicit permutation, e.g. `1 3 2 0 4`.
    Explicit(Vec<usize>),
}

impl VariableOrder {
    /// Computes the order for a state vector of `num_positions` positions, whose summands have the
    /// given read/write `patterns`.
    ///
    /// The result is always a permutation of `0..num_positions`; positions that no summand uses are
    /// appended in increasing order, since they do not occur in the dependency graph that MINCE works
    /// on. Fails when [Self::Explicit] is not such a permutation, or when MINCE cannot be run.
    pub fn compute(&self, patterns: &[ReadWritePattern], num_positions: usize) -> Result<Vec<usize>, MercError> {
        let order = match self {
            VariableOrder::None => (0..num_positions).collect(),
            VariableOrder::Mince {
                kahypar_path,
                kahypar_ini_path,
            } => {
                let graph = dependency_graph(patterns);
                let mut order = reorder(kahypar_path, kahypar_ini_path, &graph)?;

                // MINCE only orders the positions that occur in the dependency graph, so the unused
                // trailing positions have to be put back.
                let missing: Vec<usize> = (0..num_positions)
                    .filter(|position| !order.contains(position))
                    .collect();
                order.extend(missing);
                order
            }
            VariableOrder::Explicit(order) => order.clone(),
        };

        validate_permutation(&order, num_positions)?;
        debug!("Variable order = {order:?}");
        Ok(order)
    }
}

/// Returns the dependency graph of the read/write matrix, with one hyper-edge per summand.
fn dependency_graph(patterns: &[ReadWritePattern]) -> DependencyGraph {
    DependencyGraph::new(
        patterns
            .iter()
            .map(|pattern| Relation::new(pattern.read_positions().collect(), pattern.write_positions().collect()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterns() -> Vec<ReadWritePattern> {
        vec![
            ReadWritePattern::from_indices(4, &[0], &[1]).unwrap(),
            ReadWritePattern::from_indices(4, &[1], &[2]).unwrap(),
        ]
    }

    #[test]
    fn test_default_order_is_the_identity() {
        assert_eq!(VariableOrder::None.compute(&patterns(), 4).unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_explicit_order() {
        let order = VariableOrder::Explicit(vec![1, 3, 2, 0]);
        assert_eq!(order.compute(&patterns(), 4).unwrap(), vec![1, 3, 2, 0]);

        // Not a permutation of the positions of the state vector.
        assert!(VariableOrder::Explicit(vec![0, 1, 2]).compute(&patterns(), 4).is_err());
        assert!(
            VariableOrder::Explicit(vec![0, 1, 1, 2])
                .compute(&patterns(), 4)
                .is_err()
        );
        assert!(
            VariableOrder::Explicit(vec![0, 1, 2, 4])
                .compute(&patterns(), 4)
                .is_err()
        );
    }
}
