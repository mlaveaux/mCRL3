use std::path::PathBuf;

use log::debug;

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

/// Returns the inverse of the permutation `order`, i.e. the level at which every state vector position
/// is stored.
pub fn inverse_order(order: &[usize]) -> Vec<usize> {
    let mut inverse = vec![0; order.len()];
    for (level, &position) in order.iter().enumerate() {
        inverse[position] = level;
    }
    inverse
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

/// Checks that `order` is a permutation of `0..num_positions`.
fn validate_permutation(order: &[usize], num_positions: usize) -> Result<(), MercError> {
    let mut seen = vec![false; num_positions];

    for &position in order {
        let assigned = seen
            .get_mut(position)
            .ok_or_else(|| format!("position {position} does not exist, there are {num_positions} position(s)"))?;
        if *assigned {
            return Err(format!("position {position} occurs more than once in the variable order").into());
        }
        *assigned = true;
    }

    if let Some(missing) = seen.iter().position(|assigned| !assigned) {
        return Err(format!("position {missing} does not occur in the variable order").into());
    }

    Ok(())
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

    #[test]
    fn test_inverse_order() {
        assert_eq!(inverse_order(&[1, 3, 2, 0]), vec![3, 0, 2, 1]);
        assert_eq!(inverse_order(&[0, 1, 2]), vec![0, 1, 2]);
    }
}
