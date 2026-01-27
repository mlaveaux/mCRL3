//! Iterator over cubes in a BDD.

use std::marker::PhantomData;

use oxidd::BooleanFunction;
use oxidd::Function;
use oxidd::Manager;
use oxidd::bdd::BDDFunction;
use oxidd::util::AllocResult;
use oxidd::util::OptBool;
use oxidd_core::function::EdgeOfFunc;

/// Returns the boolean set difference of two BDD functions: lhs \ rhs.
/// Implemented as lhs AND (NOT rhs).
pub fn minus(lhs: &BDDFunction, rhs: &BDDFunction) -> AllocResult<BDDFunction> {
    rhs.imp_strict(lhs)
}

/// Variant of [minus] that works on edges.
pub fn minus_edge<'id>(
    manager: &<BDDFunction as Function>::Manager<'id>,
    lhs: &EdgeOfFunc<'id, BDDFunction>,
    rhs: &EdgeOfFunc<'id, BDDFunction>,
) -> AllocResult<<<BDDFunction as Function>::Manager<'id> as Manager>::Edge> {
    BDDFunction::imp_strict_edge(manager, rhs, lhs)
}

/// Iterator over all cubes (satisfying assignments) in a BDD.
///
/// The returned cubes contain don't care values (OptBool::None) for variables
/// that can be either true or false without affecting the satisfaction of the
/// BDD.
pub struct CubeIter<'a> {
    /// The BDD to iterate over.
    bdd: BDDFunction,

    _marker: PhantomData<&'a ()>,
}

impl<'a> CubeIter<'a> {
    /// Creates a new cube iterator for the given BDD.
    pub fn new(bdd: &'a BDDFunction) -> Self {
        Self {
            bdd: bdd.clone(),
            _marker: PhantomData,
        }
    }
}

impl Iterator for CubeIter<'_> {
    type Item = Vec<OptBool>;

    fn next(&mut self) -> Option<Self::Item> {
        let cube = self.bdd.pick_cube_dd(|_, _, _| true).unwrap();

        self.bdd = minus(&self.bdd, &cube).expect("Failed to compute BDD difference");

        cube.pick_cube(|_, _, _| true)
    }
}

/// The same as [CubeIter], but iterates over all satisfying assignments without
/// considering don't care values. For the universe BDD, the [CubeIter] yields only
/// one cube with all don't cares, while this iterator yields all possible cubes.
pub struct CubeIterAll<'a> {
    bdd: &'a BDDFunction,

    /// Iterator over the cubes with don't cares.
    iter: CubeIter<'a>,

    /// The current cube returned from CubeIter.
    cube: Option<Vec<OptBool>>,

    /// The cube that is currently being iterated over.
    current_cube: Vec<OptBool>,
}

impl<'a> CubeIterAll<'a> {
    /// Creates a new cube iterator that iterates over the single cube
    pub fn new(bdd: &'a BDDFunction) -> CubeIterAll<'a> {
        let mut iter = CubeIter::new(bdd);
        let cube = iter.next();

        // Initialize the current cube by replacing don't cares with false.
        let mut current_cube = cube.clone().unwrap_or_default();
        for element in current_cube.iter_mut() {
            if *element == OptBool::None {
                *element = OptBool::False;
            }
        }

        Self {
            bdd,
            iter,
            current_cube,
            cube,
        }
    }
}

impl Iterator for CubeIterAll<'_> {
    type Item = Vec<OptBool>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cube) = &self.cube {
            // Yield the current assignment first.
            let result = self.current_cube.clone();

            // Advance to the next assignment for this cube. If overflow,
            // move to the next cube and initialize its current assignment.
            if !increment(&mut self.current_cube, cube) {
                self.cube = self.iter.next();
                if let Some(next_cube) = &self.cube {
                    self.current_cube = next_cube
                        .iter()
                        .map(|element| {
                            if *element == OptBool::None {
                                OptBool::False
                            } else {
                                *element
                            }
                        })
                        .collect();
                } else {
                    // Will return None on subsequent calls.
                }
            }

            return Some(result);
        }

        None
    }
}

/// Perform the binary increment, returns false if overflow occurs.
///
/// Only considers bits for which the `cube` has don't care values, since these
/// are the only ones that can be changed.
fn increment(current_cube: &mut [OptBool], cube: &[OptBool]) -> bool {
    for (index, value) in current_cube.iter_mut().enumerate() {
        if cube[index] == OptBool::None {
            // Set each variable to true until we find one that is false
            if *value == OptBool::False {
                *value = OptBool::True;
                return true;
            }

            *value = OptBool::False;
        }
    }

    // All variables were true, overflow
    false
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;

    use merc_utilities::MercError;
    use merc_utilities::random_test;
    use oxidd::BooleanFunction;
    use oxidd::Manager;
    use oxidd::ManagerRef;
    use oxidd::bdd::BDDFunction;
    use oxidd::util::OptBool;

    use crate::CubeIter;
    use crate::CubeIterAll;
    use crate::FormatConfig;
    use crate::bdd_from_iter;
    use crate::random_bitvectors;

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_cube_iter_all() {
        random_test(100, |rng| {
            let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);
            let set = random_bitvectors(rng, 5, 20);
            println!("Set: {:?}", set.iter().format_with(", ", |v, f| f(&FormatConfig(v))));

            let variables = manager_ref
                .with_manager_exclusive(|manager| -> Result<Vec<BDDFunction>, MercError> {
                    Ok(manager
                        .add_vars(5)
                        .map(|i| BDDFunction::var(manager, i))
                        .collect::<Result<Vec<BDDFunction>, _>>()?)
                })
                .expect("Failed to create variables");

            let bdd = bdd_from_iter(&manager_ref, &variables, set.iter()).unwrap();

            // Check that the cube iterator yields all the expected cubes
            let mut seen = HashSet::new();
            for bits in CubeIterAll::new(&bdd) {
                println!("Cube: {}", FormatConfig(&bits));
                assert!(set.contains(&bits), "Cube {} not in expected set", FormatConfig(&bits));
                assert!(
                    seen.insert(bits.clone()),
                    "Duplicate cube found: {}",
                    FormatConfig(&bits)
                );
            }

            let cubes: Vec<Vec<OptBool>> = CubeIterAll::new(&bdd).collect();
            println!("cubes {cubes:?}");
            for cube in &set {
                let found = cubes.iter().find(|bits| *bits == cube);
                assert!(found.is_some(), "Expected cube {} not found", FormatConfig(cube));
            }
        })
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_cube_iter() {
        random_test(100, |rng| {
            let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);
            let set = random_bitvectors(rng, 5, 20);
            println!("Set: {:?}", set.iter().format_with(", ", |v, f| f(&FormatConfig(v))));

            let variables = manager_ref
                .with_manager_exclusive(|manager| -> Result<Vec<BDDFunction>, MercError> {
                    Ok(manager
                        .add_vars(5)
                        .map(|i| BDDFunction::var(manager, i))
                        .collect::<Result<Vec<BDDFunction>, _>>()?)
                })
                .expect("Failed to create variables");

            let bdd = bdd_from_iter(&manager_ref, &variables, set.iter()).unwrap();

            // Check that it does not yield duplicates.
            let mut seen = HashSet::new();
            for cube in CubeIter::new(&bdd) {
                println!("Cube: {}", FormatConfig(&cube));
                assert!(
                    seen.insert(cube.clone()),
                    "Duplicate cube found: {}",
                    FormatConfig(&cube)
                );
            }
        })
    }
}
