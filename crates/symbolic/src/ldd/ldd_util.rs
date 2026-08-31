use std::cmp::Ordering;

use oxidd::ManagerRef;
use oxidd::ldd::LDDFunction;
use oxidd::ldd::LDDManagerRef;
use oxidd::ldd::Value;
use oxidd::util::AllocResult;
use rustc_hash::FxHashMap;

/// Computes the intersection `a ∩ b`.
///
/// # Details
///
/// `LDDFunction` has no native `intersect`, so this is implemented as two `minus` traversals
/// (`a \ (a \ b)`), with no shared apply cache of its own.
pub fn intersect(a: &LDDFunction, b: &LDDFunction) -> AllocResult<LDDFunction> {
    a.minus(&a.minus(b)?)
}

/// Computes the interleaved cartesian product `{ a₀b₀a₁b₁… | a ∈ set_a, b ∈ set_b }`.
///
/// # Details
///
/// Used to build a strategy over the doubled, interleaved state vector `[from₀,
/// to₀, from₁, to₁, …]` out of two vector sets over the plain (non-interleaved)
/// vector. The recursion is:
///
/// > `merge(⊤, b) = b`
/// > `merge(a, ⊤) = a`
/// > `merge(∅, _) = merge(_, ∅) = ∅`
/// > `merge(a, b) = node(a.value, merge(b, a.down), merge(a.right, b))`, otherwise
///
/// where `⊤` is the singleton set containing only the empty vector and `∅` is
/// the empty set. Note the down-branch swaps `a` and `b`: each level of the
/// result takes its value from whichever operand is "in front", which is what
/// produces the interleaving.
///
/// [`LDDFunction::make_node`] does not apply the LDD reduction rule (unlike
/// Sylvan's `lddmc_makenode`), so this function must apply it itself: a node
/// whose `down` branch became empty is not a valid node and collapses to its
/// `right` sibling.
pub fn merge(manager: &LDDManagerRef, a: &LDDFunction, b: &LDDFunction) -> AllocResult<LDDFunction> {
    let mut cache = FxHashMap::default();
    merge_rec(manager, a, b, &mut cache)
}

fn merge_rec(
    manager: &LDDManagerRef,
    a: &LDDFunction,
    b: &LDDFunction,
    cache: &mut FxHashMap<(usize, usize), LDDFunction>,
) -> AllocResult<LDDFunction> {
    if a.is_empty_vector() {
        return Ok(b.clone());
    }
    if b.is_empty_vector() {
        return Ok(a.clone());
    }
    if a.is_empty() || b.is_empty() {
        return manager.with_manager_shared(LDDFunction::empty_set);
    }

    let key = (a.id(), b.id());
    if let Some(cached) = cache.get(&key) {
        return Ok(cached.clone());
    }

    let (value, down, right) = a
        .node()
        .expect("checked above: neither the empty set nor the empty vector");

    let down_result = merge_rec(manager, b, &down, cache)?;
    let right_result = merge_rec(manager, &right, b, cache)?;

    let result = if down_result.is_empty() {
        // A node with an empty `down` branch is not canonical; it collapses to `right`.
        right_result
    } else {
        manager.with_manager_shared(|m| LDDFunction::make_node(m, value, &down_result, &right_result))?
    };

    cache.insert(key, result.clone());
    Ok(result)
}

/// Restricts `set` to the vectors whose element at `level` equals `value`.
pub fn fix_element(manager: &LDDManagerRef, set: &LDDFunction, level: usize, value: Value) -> AllocResult<LDDFunction> {
    if level == 0 {
        let mut current = set.clone();
        loop {
            match current.node() {
                None => return manager.with_manager_shared(LDDFunction::empty_set),
                Some((v, down, right)) => match v.cmp(&value) {
                    Ordering::Equal => {
                        let empty = manager.with_manager_shared(LDDFunction::empty_set)?;
                        return manager.with_manager_shared(|m| LDDFunction::make_node(m, value, &down, &empty));
                    }
                    Ordering::Greater => return manager.with_manager_shared(LDDFunction::empty_set),
                    Ordering::Less => current = right,
                },
            }
        }
    } else {
        match set.node() {
            None => Ok(set.clone()),
            Some((v, down, right)) => {
                let new_down = fix_element(manager, &down, level - 1, value)?;
                let new_right = fix_element(manager, &right, level, value)?;

                if new_down.is_empty() {
                    Ok(new_right)
                } else {
                    manager.with_manager_shared(|m| LDDFunction::make_node(m, v, &new_down, &new_right))
                }
            }
        }
    }
}

/// Returns an LDD containing all elements of the given iterator over vectors.
pub fn from_iter<'a, I>(manager: &LDDManagerRef, iter: I) -> LDDFunction
where
    I: Iterator<Item = &'a Vec<Value>>,
{
    let mut result = manager
        .with_manager_shared(|m| LDDFunction::empty_set(m))
        .expect("Failed to create the empty set");

    for vector in iter {
        let single = manager
            .with_manager_shared(|m| LDDFunction::singleton(m, vector))
            .expect("Failed to create a singleton");
        result = result.union(&single).expect("Failed to compute the union");
    }

    result
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use oxidd::ldd::RelationProductMeta;
    use oxidd::ldd::Value;

    use merc_utilities::random_test;

    use crate::from_iter;
    use crate::random_vector_set;

    use super::fix_element;
    use super::intersect;
    use super::merge;

    /// Cross-checks [intersect] against a `HashSet` reference implementation.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_intersect() {
        random_test(100, |rng| {
            let manager = oxidd::ldd::new_manager(2048, 1024, 1);

            let a = random_vector_set(rng, 16, 5, 5);
            let b = random_vector_set(rng, 16, 5, 5);

            let ldd_a = from_iter(&manager, a.iter());
            let ldd_b = from_iter(&manager, b.iter());

            let result = intersect(&ldd_a, &ldd_b).unwrap();
            let expected: HashSet<Vec<u32>> = a.intersection(&b).cloned().collect();

            assert_eq!(result.len(), expected.len());
            for vector in &expected {
                assert!(crate::element_of(&manager, vector, &result), "missing {vector:?}");
            }
        })
    }

    /// Cross-checks [merge] against a `HashSet` reference implementation of the
    /// interleaved cartesian product.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_merge() {
        random_test(100, |rng| {
            let manager = oxidd::ldd::new_manager(2048, 1024, 1);

            let a = random_vector_set(rng, 8, 3, 5);
            let b = random_vector_set(rng, 8, 3, 5);

            let ldd_a = from_iter(&manager, a.iter());
            let ldd_b = from_iter(&manager, b.iter());

            let result = merge(&manager, &ldd_a, &ldd_b).unwrap();

            let mut expected: HashSet<Vec<u32>> = HashSet::new();
            for va in &a {
                for vb in &b {
                    let mut interleaved = Vec::with_capacity(va.len() + vb.len());
                    for (x, y) in va.iter().zip(vb.iter()) {
                        interleaved.push(*x);
                        interleaved.push(*y);
                    }
                    expected.insert(interleaved);
                }
            }

            assert_eq!(result.len(), expected.len());
            for vector in &expected {
                assert!(crate::element_of(&manager, vector, &result), "missing {vector:?}");
            }
        })
    }

    /// Cross-checks [fix_element] against a `HashSet` reference implementation.
    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_fix_element() {
        random_test(100, |rng| {
            let manager = oxidd::ldd::new_manager(2048, 1024, 1);

            let length = 5;
            let max_value = 5;
            let set = random_vector_set(rng, 32, length, max_value);
            let ldd = from_iter(&manager, set.iter());

            for level in 0..length {
                for value in 0..max_value {
                    let result = fix_element(&manager, &ldd, level, value).unwrap();
                    let expected: HashSet<Vec<u32>> = set.iter().filter(|v| v[level] == value).cloned().collect();

                    assert_eq!(
                        result.len(),
                        expected.len(),
                        "level {level}, value {value}: {:?} vs {:?}",
                        result.len(),
                        expected.len()
                    );
                    for vector in &expected {
                        assert!(crate::element_of(&manager, vector, &result), "missing {vector:?}");
                    }
                }
            }
        })
    }
}
