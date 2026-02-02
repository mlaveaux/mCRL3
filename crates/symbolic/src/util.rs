use std::collections::HashSet;

use oxidd::BooleanFunction;
use oxidd::Edge;
use oxidd::Function;
use oxidd::HasLevel;
use oxidd::InnerNode;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::Node;
use oxidd::VarNo;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::util::Borrowed;
use oxidd::util::OutOfMemory;
use oxidd_core::function::EdgeOfFunc;
use oxidd_core::util::EdgeDropGuard;

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
    manager_ref.with_manager_shared(|manager| {
        support_edge(manager, function.as_edge(manager).borrowed(), &mut result);
    });
    Ok(result.into_iter().collect())
}

/// Recursive implementation of [support].
fn support_edge<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    function: Borrowed<EdgeOfFunc<'id, BDDFunction>>,
    result: &mut HashSet<VarNo>,
) {
    match manager.get_node(&function) {
        Node::Terminal(_) => (),
        Node::Inner(node) => {
            result.insert(node.level());

            // Recurse into cofactors
            let (high, low) = collect_children(node);
            support_edge(manager, low, result);
            support_edge(manager, high, result);
        }
    }
}

/// Specialized substitution function for variables renaming that only works for
/// f[x <- x+1].
///
/// # Details
///
/// In general substitution is defined as follows, but its computation can be
/// fairly expensive.
///
/// > f[x <- g] = (!g ∧ f[x <- false]) ∨ (g ∧ f[x <- true])
/// 
/// Restricting the substitution to only renaming variables from 'x' to 'x+1'
/// allows for a more efficient implementation. This function checks whether the
/// inputs satisfy this restriction and then performs the renaming.
pub fn variable_rename(manager_ref: &BDDManagerRef, 
    function: &BDDFunction,
    substitution: &[(VarNo, VarNo)]
) -> Result<BDDFunction, OutOfMemory>    
{
    // Every subsitution must be from a lower variable to a higher variable.
    for (from, to) in substitution {
        debug_assert!(from + 1 == *to, "Variable renaming must be from 'x' to 'x+1'");
    }

    manager_ref.with_manager_shared(|manager| -> Result<BDDFunction, OutOfMemory>{
        Ok(BDDFunction::from_edge(manager, variable_rename_edge(manager, function.as_edge(manager).borrowed(), substitution)?))
    })
}

/// Implementation of [variable_rename].
pub fn variable_rename_edge<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    function: Borrowed<EdgeOfFunc<'id, BDDFunction>>,
    substitution: &[(VarNo, VarNo)],
) -> Result<EdgeOfFunc<'id, BDDFunction>, OutOfMemory> {
    match substitution.first() {
        Some((from, to)) => {
            match manager.get_node(&function) {
                Node::Terminal(terminal) => manager.get_terminal(terminal),
                Node::Inner(node) => {
                    let (high, low) = collect_children(node);

                    if node.level() == *from {
                        // Rename variable
                        let high_substituted = EdgeDropGuard::new(manager, variable_rename_edge(manager, high, &substitution[1..])?);
                        let low_substituted = EdgeDropGuard::new(manager, variable_rename_edge(manager, low, &substitution[1..])?);

                        BDDFunction::ite_edge(
                            manager,
                            &EdgeDropGuard::new(manager, BDDFunction::var_edge(manager, *to)?),
                            &high_substituted,
                            &low_substituted,
                        )
                    } else {
                        let high_substituted = EdgeDropGuard::new(manager, variable_rename_edge(manager, high, substitution)?);
                        let low_substituted = EdgeDropGuard::new(manager, variable_rename_edge(manager, low, substitution)?);

                        BDDFunction::ite_edge(
                            manager,
                            &EdgeDropGuard::new(manager, BDDFunction::var_edge(manager, node.level())?),
                            &high_substituted,
                            &low_substituted,
                        )
                    }
                }
            }
        }
        None => {
            // No variables to substitute, so remains identity.
            Ok(manager.clone_edge(&function))
        }
    }
}

/// Collect the two children (high, low) of a binary node
#[inline]
#[must_use]
pub fn collect_children<E: Edge, N: InnerNode<E>>(node: &N) -> (Borrowed<'_, E>, Borrowed<'_, E>) {
    debug_assert_eq!(N::ARITY, 2);
    let mut it = node.children();
    let f_then = it.next().unwrap();
    let f_else = it.next().unwrap();
    debug_assert!(it.next().is_none());
    (f_then, f_else)
}

#[cfg(test)]
mod tests {
    use merc_utilities::random_test;
    use oxidd::{BooleanFunction, Manager, ManagerRef, bdd::BDDFunction, util::AllocResult};

    use crate::{random_bdd, support, variable_rename};

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_bdd_variable_rename() {
        let manager_ref = oxidd::bdd::new_manager(2048, 2048, 1024);

        let vars: Vec<BDDFunction> = manager_ref.with_manager_exclusive(|manager| {
            AllocResult::from_iter(manager.add_vars(3).map(|i| BDDFunction::var(manager, i)))
        }).unwrap();

        let res = vars[0].and(&vars[1]).unwrap().or(&vars[2]).unwrap();
        let subst = variable_rename(&manager_ref, &res, &[(0, 1), (1, 2), (2, 0)]).unwrap();
        assert!(subst.satisfiable());
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_bdd_support() {
        random_test(100, |rng| {
            let manager_ref = oxidd::bdd::new_manager(2048, 2048, 1024);

            let vars = manager_ref
                .with_manager_exclusive(|manager| {
                    manager
                        .add_vars(4)
                        .map(|v| BDDFunction::var(manager, v))
                        .collect::<Result<Vec<BDDFunction>, _>>()
                })
                .unwrap();

            let function = random_bdd(&manager_ref, rng, &vars, 9).unwrap();
            let support = support(&manager_ref, &function).unwrap();

            println!("Support: {:?}", support);
        });
    }
}
