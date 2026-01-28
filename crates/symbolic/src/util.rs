use std::collections::HashSet;

use oxidd::BooleanFunction;
use oxidd::Function;
use oxidd::HasLevel;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::VarNo;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::util::OutOfMemory;

/// Computes the support (set of variables) of the given BDD function.
pub fn support(manager_ref: &BDDManagerRef, function: &BDDFunction) -> Result<Vec<VarNo>, OutOfMemory> {
    let mut result = HashSet::new();
    support_rec(manager_ref, function, &mut result);
    Ok(result.into_iter().collect())
}

/// Recursive implementation of [support].
pub fn support_rec(manager_ref: &BDDManagerRef, function: &BDDFunction, result: &mut HashSet<VarNo>) {
    manager_ref.with_manager_shared(|manager| {
        let node = manager.get_node(function.as_edge(manager)).unwrap_inner();
        result.insert(node.level());

        // Recurse into cofactors
        if let Some((low, high)) = function.cofactors() {
            support_rec(manager_ref, &low, result);
            support_rec(manager_ref, &high, result);
        }
    })
}
