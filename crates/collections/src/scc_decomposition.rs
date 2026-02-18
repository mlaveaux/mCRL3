use log::debug;
use log::trace;
use merc_utilities::MercIndex;

use crate::BlockIndex;
use crate::Graph;
use crate::IndexedPartition;
use merc_io::LargeFormatter;

/// Computes the strongly connected component partitioning of the given LTS.
pub fn scc_decomposition<F, G>(graph: &G, filter: F) -> IndexedPartition
where
    F: Fn(G::VertexIndex, G::LabelIndex, G::VertexIndex) -> bool,
    G: Graph,
    G::VertexIndex: MercIndex<Target = usize>,
{
    let mut partition = IndexedPartition::new(graph.num_of_vertices());

    // The stack for the depth first search.
    let mut stack = Vec::new();

    // Keep track of already visited states.
    let mut state_info: Vec<Option<StateInfo>> = vec![None; graph.num_of_vertices()];

    let mut smallest_index = 0;
    let mut next_block_number = BlockIndex::new(0);

    // The outer depth first search used to traverse all the states.
    for state_index in graph.iter_vertices() {
        if state_info[state_index.index()].is_none() {
            trace!("State {}", state_index.index());

            strongly_connect(
                state_index,
                graph,
                &filter,
                &mut partition,
                &mut smallest_index,
                &mut next_block_number,
                &mut stack,
                &mut state_info,
            )
        }
    }

    trace!("SCC partition {partition}");
    debug!(
        "Found {} strongly connected components",
        LargeFormatter(partition.num_of_blocks())
    );
    partition
}

/// Information about a state during the SCC computation.
#[derive(Clone, Debug)]
struct StateInfo {
    /// A unique index for every state.
    index: usize,

    /// Keeps track of the lowest state that can be reached on the stack.
    lowlink: usize,

    /// Keeps track of whether this state is on the stack.
    on_stack: bool,
}

/// Tarjan's strongly connected components algorithm.
///
/// The `filter` can be used to determine which (from, label, to) edges should
/// to be connected.
///
/// The `smallest_index`, `stack` and `indices` are updated in each recursive
/// call to keep track of the current SCC.
#[allow(clippy::too_many_arguments)]
fn strongly_connect<F, G>(
    vertex_index: G::VertexIndex,
    lts: &G,
    filter: &F,
    partition: &mut IndexedPartition,
    smallest_index: &mut usize,
    next_block_number: &mut BlockIndex,
    stack: &mut Vec<G::VertexIndex>,
    state_info: &mut Vec<Option<StateInfo>>,
) where
    F: Fn(G::VertexIndex, G::LabelIndex, G::VertexIndex) -> bool,
    G: Graph,
    G::VertexIndex: MercIndex<Target = usize>,
{
    trace!("Visiting state {}", vertex_index.index());

    state_info[vertex_index.index()] = Some(StateInfo {
        index: *smallest_index,
        lowlink: *smallest_index,
        on_stack: true,
    });

    *smallest_index += 1;

    // Start a depth first search from the current state.
    stack.push(vertex_index);

    // Consider successors of the current state.
    for (label, to) in lts.outgoing_edges(vertex_index) {
        if filter(vertex_index, label, to) {
            if let Some(meta) = &mut state_info[to.index()] {
                if meta.on_stack {
                    // Successor w is in stack S and hence in the current SCC
                    // If w is not on stack, then (v, w) is an edge pointing to an SCC already found and must be ignored
                    // v.lowlink := min(v.lowlink, w.lowlink);
                    let w_index = state_info[to.index()]
                        .as_ref()
                        .expect("The state must be visited in the recursive call")
                        .index;
                    let info = state_info[vertex_index.index()]
                        .as_mut()
                        .expect("This state was added before");
                    info.lowlink = info.lowlink.min(w_index);
                }
            } else {
                // Successor w has not yet been visited; recurse on it
                strongly_connect(
                    to,
                    lts,
                    filter,
                    partition,
                    smallest_index,
                    next_block_number,
                    stack,
                    state_info,
                );

                // v.lowlink := min(v.lowlink, w.lowlink);
                let w_lowlink = state_info[to.index()]
                    .as_ref()
                    .expect("The state must be visited in the recursive call")
                    .lowlink;
                let info = state_info[vertex_index.index()]
                    .as_mut()
                    .expect("This state was added before");
                info.lowlink = info.lowlink.min(w_lowlink);
            }
        }
    }

    let info = state_info[vertex_index.index()]
        .as_ref()
        .expect("This state was added before");
    if info.lowlink == info.index {
        // Start a new strongly connected component.
        while let Some(index) = stack.pop() {
            let info = state_info[index.index()].as_mut().expect("This state was on the stack");
            info.on_stack = false;

            trace!("Added state {} to block {}", index.index(), next_block_number);
            partition.set_block(index.index(), *next_block_number);

            if index == vertex_index || stack.is_empty() {
                *next_block_number = BlockIndex::new(next_block_number.value() + 1);
                break;
            }
        }
    }
}
