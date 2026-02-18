use merc_utilities::MercError;
use oxidd::BooleanFunction;
use oxidd::ManagerRef;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;

use merc_symbolic::minus;

use crate::PG;
use crate::Player;
use crate::Priority;
use crate::VariabilityParityGame;
use crate::VertexIndex;

/// Makes the given variability parity game total by adding edges to true/false nodes as needed.
pub fn make_vpg_total(
    manager_ref: &BDDManagerRef,
    vpg: &VariabilityParityGame,
) -> Result<VariabilityParityGame, MercError> {
    // The universe for totality is the game's overall configuration, not global true.
    let universe = manager_ref.with_manager_shared(|manager| BDDFunction::t(manager));

    // For a total game we need to potentially add new edges to true/false nodes.
    let mut edges = Vec::new();

    // Add the true and false nodes.
    let mut owners = vpg.owners().clone();
    let mut priorities = vpg.priorities().clone();

    // Owner does not matter, priority must be even for true node and odd for false node.
    let mut true_node = None;
    let mut false_node = None;

    for vertex in vpg.iter_vertices() {
        let mut all_outgoing = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));
        for edge in vpg.outgoing_edges(vertex) {
            // Add the original edge.
            edges.push((vertex, edge.label().clone(), edge.to()));

            // Keep track of the overall outgoing configuration.
            all_outgoing = all_outgoing.or(edge.label())?;
        }

        // Missing configurations are those in the universe not covered by any outgoing edge.
        let missing = minus(&universe, &all_outgoing)?;
        if missing.satisfiable() {
            if owners[*vertex] == Player::Odd {
                // Odd player deadlock: add edge to true node for the remaining configurations.
                let node = true_node.get_or_insert_with(|| {
                    let idx = VertexIndex::new(owners.len());
                    owners.push(Player::Even);
                    priorities.push(Priority::new(0)); // Even priority for true node
                    idx
                });
                edges.push((vertex, missing.clone(), *node));
            } else {
                // Even player deadlock: add edge to false node for the remaining configurations.
                let node = false_node.get_or_insert_with(|| {
                    let idx = VertexIndex::new(owners.len());
                    owners.push(Player::Even);
                    priorities.push(Priority::new(1)); // Odd priority for false node
                    idx
                });
                edges.push((vertex, missing.clone(), *node));
            }
        }
    }

    // Add self-loops for sink nodes if they were created
    if let Some(node) = true_node {
        edges.push((node, universe.clone(), node));
    }
    if let Some(node) = false_node {
        edges.push((node, universe.clone(), node));
    }

    Ok(VariabilityParityGame::from_edges(
        manager_ref,
        vpg.initial_vertex(),
        owners,
        priorities,
        vpg.configuration().clone(),
        vpg.variables().clone(),
        || edges.iter().cloned(),
    ))
}
