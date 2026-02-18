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

/// The BDD representing the support variables of a BDD function.
pub type BDDSupport = BDDFunction;

/// Computes the support (set of variables) of the given BDD function.
///
/// # Details
///
/// The `support` is the variables on which the `BDD` is defined, and other
/// variables are irrelevant or don't care, formally:
///
/// > support(f) = { x_i | exists x_0, ..., x_{i-1}, x_{i+1}, ..., x_n : f(x_0, ..., x_{i-1}, true, x_{i+1}, ..., x_n) != f(x_0, ..., x_{i-1}, false, x_{i+1}, ..., x_n) }
pub fn support(manager_ref: &BDDManagerRef, function: &BDDFunction) -> Result<Vec<VarNo>, OutOfMemory> {
    let mut result = HashSet::new();
    support_rec(manager_ref, function, &mut result);
    Ok(result.into_iter().collect())
}

/// Recursive implementation of [support].
fn support_rec(manager_ref: &BDDManagerRef, function: &BDDFunction, result: &mut HashSet<VarNo>) {
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
