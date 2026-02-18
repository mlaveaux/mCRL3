use std::collections::HashSet;

use oxidd::Edge;
use oxidd::Function;
use oxidd::HasLevel;
use oxidd::InnerNode;
use oxidd::LevelNo;
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
use rustc_hash::FxHashMap;

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
/// `f[x <- x+1]` renamings.
///
/// # Details
///
/// In general substitution is defined as follows,
///
/// > f[x <- g] = (!g ∧ f[x <- false]) ∨ (g ∧ f[x <- true])
///
/// but its computation can be fairly expensive.Restricting the substitution to
/// only renaming variables from 'x' to 'x+1' allows for a more efficient
/// implementation, as follows:
///
/// > f[x <- x+1] = (!x+1 ∧ f[x <- false]) ∨ (x+1 ∧ f[x <- true])
/// >             = make_node(x+1, f[x <- true][x+1 <- true], f[x <- false][x+1 <- false])
///
/// This function checks whether the inputs satisfy this restriction and then
/// performs the renaming.
pub fn variable_rename(
    manager_ref: &BDDManagerRef,
    function: &BDDFunction,
    substitution: &[(VarNo, VarNo)],
) -> Result<BDDFunction, OutOfMemory> {
    // Every subsitution must be from a lower variable to a higher variable.
    for (from, to) in substitution {
        debug_assert!(from + 1 == *to, "Variable renaming must be from 'x' to 'x+1'");
    }

    let mut cache = FxHashMap::default();

    manager_ref.with_manager_shared(|manager| -> Result<BDDFunction, OutOfMemory> {
        Ok(BDDFunction::from_edge(
            manager,
            variable_rename_edge(manager, &mut cache, function.as_edge(manager).borrowed(), substitution)?,
        ))
    })
}

/// Implementation of [variable_rename].
pub fn variable_rename_edge<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    cache: &mut FxHashMap<BDDFunction, BDDFunction>,
    function: Borrowed<EdgeOfFunc<'id, BDDFunction>>,
    substitution: &[(VarNo, VarNo)],
) -> Result<EdgeOfFunc<'id, BDDFunction>, OutOfMemory> {
    let node = match manager.get_node(&function) {
        Node::Terminal(terminal) => return manager.get_terminal(terminal),
        Node::Inner(node) => node,
    };

    if let Some(cached) = cache.get(&BDDFunction::from_edge(manager, manager.clone_edge(&function))) {
        return Ok(manager.clone_edge(cached.as_edge(manager)));
    }

    let (from, to) = match substitution.first() {
        None => {
            // No variables to substitute, so remains identity.
            return Ok(manager.clone_edge(&function));
        }
        Some((from, to)) => (from, to),
    };

    // TODO: Caching

    let result = if node.level() == *from {
        let (high, low) = collect_children(node);
        // Rename variable from 'from' to 'to'.
        let high = EdgeDropGuard::new(manager, variable_rename_edge(manager, cache, high, &substitution[1..])?);
        let low = EdgeDropGuard::new(manager, variable_rename_edge(manager, cache, low, &substitution[1..])?);

        let high_high = match manager.get_node(&high) {
            Node::Inner(node) => {
                if node.level() == *to {
                    // There are f[x <- true][x+1 <- true]
                    collect_children(node).0
                } else {
                    high.borrowed()
                }
            }
            Node::Terminal(_terminal) => high.borrowed(),
        };

        let low_low = match manager.get_node(&low) {
            Node::Inner(node) => {
                if node.level() == *to {
                    // There are f[x <- false][x+1 <- false]
                    collect_children(node).1
                } else {
                    low.borrowed()
                }
            }
            Node::Terminal(_terminal) => low.borrowed(),
        };

        reduce(
            manager,
            *to,
            manager.clone_edge(&high_high),
            manager.clone_edge(&low_low),
        )
    } else if node.level() > *from {
        // We are past the substitution point, so just continue.
        variable_rename_edge(manager, cache, function.borrowed(), &substitution[1..])
    } else {
        // node.level() < *from, in this case we keep the variable as is.
        let (high, low) = collect_children(node);
        let high = variable_rename_edge(manager, cache, high, substitution)?;
        let low = variable_rename_edge(manager, cache, low, substitution)?;

        reduce(manager, node.level(), high, low)
    }?;

    cache.insert(
        BDDFunction::from_edge(manager, manager.clone_edge(&function)),
        BDDFunction::from_edge(manager, manager.clone_edge(&result)),
    );

    Ok(result)
}

/// Specialized substitution function for variables renaming that only works for
/// `f[x+1 <- x]` renamings, similar to [variable_rename].
///
/// # Details
///
/// We can derive the following:
///
/// > f[x+1 <- x] = (!x ∧ f[x+1 <- false]) ∨ (x ∧ f[x+1 <- true])
/// >             = make_node(x, f[x <- true][x+1 <- true] , f[x+1 <- false][x <- false])
pub fn variable_rename_reverse(
    manager_ref: &BDDManagerRef,
    function: &BDDFunction,
    substitution: &[(VarNo, VarNo)],
) -> Result<BDDFunction, OutOfMemory> {
    // Every subsitution must be from a lower variable to a higher variable.
    for (from, to) in substitution {
        debug_assert!(*from == to + 1, "Variable renaming must be from 'x+1' to 'x'");
    }

    let mut cache = FxHashMap::default();

    manager_ref.with_manager_shared(|manager| -> Result<BDDFunction, OutOfMemory> {
        Ok(BDDFunction::from_edge(
            manager,
            variable_rename_reverse_edge(manager, &mut cache, function.as_edge(manager).borrowed(), substitution)?,
        ))
    })
}

/// Implementation of [variable_rename].
pub fn variable_rename_reverse_edge<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    cache: &mut FxHashMap<BDDFunction, BDDFunction>,
    function: Borrowed<EdgeOfFunc<'id, BDDFunction>>,
    substitution: &[(VarNo, VarNo)],
) -> Result<EdgeOfFunc<'id, BDDFunction>, OutOfMemory> {
    let node = match manager.get_node(&function) {
        Node::Terminal(terminal) => return manager.get_terminal(terminal),
        Node::Inner(node) => node,
    };

    if let Some(cached) = cache.get(&BDDFunction::from_edge(manager, manager.clone_edge(&function))) {
        return Ok(manager.clone_edge(cached.as_edge(manager)));
    }

    let (from, to) = match substitution.first() {
        None => {
            // No variables to substitute, identity.
            return Ok(manager.clone_edge(&function));
        }
        Some((from, to)) => (from, to),
    };

    let result = if node.level() == *to {
        // Build node at level `to` using cofactors of `from` where present.
        let (high, low) = collect_children(node);

        let high = EdgeDropGuard::new(
            manager,
            variable_rename_reverse_edge(manager, cache, high, &substitution[1..])?,
        );
        let low = EdgeDropGuard::new(
            manager,
            variable_rename_reverse_edge(manager, cache, low, &substitution[1..])?,
        );

        let high_high = match manager.get_node(&high) {
            Node::Inner(node) if node.level() == *from => collect_children(node).0,
            _ => high.borrowed(),
        };
        let low_low = match manager.get_node(&low) {
            Node::Inner(node) if node.level() == *from => collect_children(node).1,
            _ => low.borrowed(),
        };

        reduce(
            manager,
            *to,
            manager.clone_edge(&high_high),
            manager.clone_edge(&low_low),
        )
    } else if node.level() == *from {
        // TODO: Why is this case needed?
        // `x+1` appears before `x`: rename this node to level `to`.
        let (high, low) = collect_children(node);
        let high = variable_rename_reverse_edge(manager, cache, high, &substitution[1..])?;
        let low = variable_rename_reverse_edge(manager, cache, low, &substitution[1..])?;
        reduce(manager, *to, high, low)
    } else if node.level() > *from {
        // Past both `to` and `from`, drop this substitution.
        variable_rename_reverse_edge(manager, cache, function.borrowed(), &substitution[1..])
    } else {
        // Recurse normally, keeping the substitution.
        let (high, low) = collect_children(node);
        let high = variable_rename_reverse_edge(manager, cache, high, substitution)?;
        let low = variable_rename_reverse_edge(manager, cache, low, substitution)?;
        reduce(manager, node.level(), high, low)
    }?;

    cache.insert(
        BDDFunction::from_edge(manager, manager.clone_edge(&function)),
        BDDFunction::from_edge(manager, manager.clone_edge(&result)),
    );

    Ok(result)
}

/// Collect the two children (high, low) of a binary node
#[inline]
#[must_use]
pub(crate) fn collect_children<E: Edge, N: InnerNode<E>>(node: &N) -> (Borrowed<'_, E>, Borrowed<'_, E>) {
    debug_assert_eq!(N::ARITY, 2);
    let mut it = node.children();
    let f_then = it.next().unwrap();
    let f_else = it.next().unwrap();
    debug_assert!(it.next().is_none());
    (f_then, f_else)
}

/// Apply the reduction rules, creating a node in `manager` if necessary
#[inline(always)]
pub(crate) fn reduce<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    level: LevelNo,
    t: EdgeOfFunc<'id, BDDFunction>,
    e: EdgeOfFunc<'id, BDDFunction>,
) -> Result<EdgeOfFunc<'id, BDDFunction>, OutOfMemory> {
    // We do not use `DiagramRules::reduce()` here, as the iterator is
    // apparently not fully optimized away.
    if t == e {
        manager.drop_edge(e);
        return Ok(t);
    }
    oxidd_core::LevelView::get_or_insert(
        &mut manager.level(level),
        <<BDDFunction as Function>::Manager<'id> as Manager>::InnerNode::new(level, [t, e]),
    )
}

#[cfg(test)]
mod tests {
    use merc_utilities::random_test;
    use oxidd::BooleanFunction;
    use oxidd::FunctionSubst;
    use oxidd::Manager;
    use oxidd::ManagerRef;
    use oxidd::Subst;
    use oxidd::bdd::BDDFunction;
    use oxidd::util::AllocResult;

    use crate::FormatConfigSet;
    use crate::compute_vars_bdd;
    use crate::random_bdd;
    use crate::support;
    use crate::variable_rename;
    use crate::variable_rename_reverse;

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_bdd_variable_rename() {
        let manager_ref = oxidd::bdd::new_manager(2048, 2048, 1024);

        let vars: Vec<BDDFunction> = manager_ref
            .with_manager_exclusive(|manager| {
                AllocResult::from_iter(manager.add_vars(3).map(|i| BDDFunction::var(manager, i)))
            })
            .unwrap();

        let res = vars[0].and(&vars[1]).unwrap().or(&vars[2]).unwrap();
        let subst = variable_rename(&manager_ref, &res, &[(0, 1), (1, 2)]).unwrap();
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
            let _support = support(&manager_ref, &function).unwrap();

            // TODO: Verify support correctness
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_bdd_renaming() {
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

            let function = random_bdd(&manager_ref, rng, &vars, 24).unwrap();

            let to = compute_vars_bdd(&manager_ref, &[1, 3]).unwrap().0;
            let substitution = Subst::new(&[0, 2], &to);
            println!("input: {}", FormatConfigSet(&function));

            let expected = function.substitute(&substitution).unwrap();
            let renamed = variable_rename(&manager_ref, &function, &[(0, 1), (2, 3)]).unwrap();

            println!("expected: {}", FormatConfigSet(&expected));
            println!("result: {}", FormatConfigSet(&renamed));

            assert!(expected == renamed, "Renaming did not match expected substitution");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_bdd_renaming_reverse() {
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

            let function = random_bdd(&manager_ref, rng, &vars, 24).unwrap();

            let to = compute_vars_bdd(&manager_ref, &[0, 2]).unwrap().0;
            let substitution = Subst::new(&[1, 3], &to);
            println!("input: {}", FormatConfigSet(&function));

            let expected = function.substitute(&substitution).unwrap();
            let renamed = variable_rename_reverse(&manager_ref, &function, &[(1, 0), (3, 2)]).unwrap();

            println!("expected: {}", FormatConfigSet(&expected));
            println!("renamed: {}", FormatConfigSet(&renamed));

            assert!(
                expected == renamed,
                "Renaming with reverse did not match expected substitution"
            );
        });
    }
}
