use std::collections::HashMap;
use std::ops::ControlFlow;

use merc_collections::BlockIndex;
use merc_collections::BlockPartition;
use merc_collections::Graph;
use merc_collections::scc_decomposition;
use merc_syntax::DefId;
use merc_syntax::SortExpression;
use merc_syntax::UntypedDataSpecification;
use merc_syntax::visit_sort_expr;
use merc_utilities::TagIndex;

/// Returns true iff there is a cycle in the alias declarations.
pub fn has_alias_cycle(spec: &UntypedDataSpecification) -> Result<(), Vec<DefId>> {
    // A mapping that keeps track of the relation between sorts.
    let mut mapping: HashMap<DefId, Vec<DefId>> = HashMap::new();

    for sort_decl in &spec.sort_declarations {
        if let Some(alias) = &sort_decl.expr {
            let mut visited = Vec::new();

            visit_sort_expr::<(), _>(alias, |sort| {
                if let SortExpression::Resolved(_, id) = sort {
                    visited.push(*id);
                }

                ControlFlow::Continue(())
            });

            mapping.insert(sort_decl.id.expect("Name must have been resolved"), visited);
        }
    }

    let scc_partition =
        BlockPartition::<()>::from_indexed_partition(&scc_decomposition(&SortGraph { mapping }, |_, _, _| true));

    for block in (0..scc_partition.len()).map(BlockIndex::new) {
        if scc_partition.block(block).len() > 1 {
            return Err(scc_partition.iter_block(block).map(|id| DefId::new(id)).collect());
        }
    }
    Ok(())
}

/// A graph structure representing the alias declarations in a data
/// specification. The vertices are the sorts, and there is an edge from sort A
/// to sort B if B is contained in the alias of A, i.e. `A -> B` if `A = List(B)`.
struct SortGraph {
    mapping: HashMap<DefId, Vec<DefId>>,
}

/// A unique type for the sorts.
pub struct SortTag;

/// The index type for a sort.
pub type SortIndex = TagIndex<usize, SortTag>;

impl Graph for SortGraph {
    type VertexIndex = SortIndex;

    type LabelIndex = ();

    fn num_of_vertices(&self) -> usize {
        self.mapping.keys().map(|id| **id).max().map_or(0, |max_id| max_id + 1)
    }

    fn iter_vertices(&self) -> impl Iterator<Item = Self::VertexIndex> {
        (0..self.num_of_vertices()).map(SortIndex::new)
    }

    fn outgoing_edges(&self, vertex: Self::VertexIndex) -> impl Iterator<Item = (Self::LabelIndex, Self::VertexIndex)> {
        self.mapping
            .get(&DefId::new(*vertex))
            .into_iter()
            .flat_map(|targets| targets.iter())
            .map(|id| ((), SortIndex::new(**id)))
    }
}

#[cfg(test)]
mod tests {
    use merc_syntax::UntypedDataSpecification;

    use crate::DataSpecification;
    use crate::WellTypedError;

    #[test]
    fn test_trivial_alias_cycle() {
        let spec = UntypedDataSpecification::parse(
            "sort S = T;
            T = U;
            U = S;",
        )
        .unwrap();

        match DataSpecification::from_untyped(spec) {
            Err(WellTypedError::AliasCycle { sorts })
                if sorts == vec!["S".to_string(), "T".to_string(), "U".to_string()] => {}
            Err(other) => panic!("Unexpected error {:?}", other),
            _ => panic!("Expected from_untyped to fail"),
        }
    }
}
