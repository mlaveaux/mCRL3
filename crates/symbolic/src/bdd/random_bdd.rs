use oxidd::BooleanFunction;
use oxidd::ManagerRef;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::util::OptBool;
use oxidd::util::OutOfMemory;
use rand::Rng;
use rand::RngExt;

/// Generate `num_vectors` random bitvectors of length `num_vars`.
pub fn random_bitvectors<R: Rng>(rng: &mut R, num_vars: usize, num_vectors: usize) -> Vec<Vec<OptBool>> {
    let mut vectors = Vec::new();
    for _ in 0..rng.random_range(0..num_vectors) {
        let mut vec = Vec::new();
        for _ in 0..num_vars {
            vec.push(if rng.random_bool(0.5) {
                OptBool::True
            } else {
                OptBool::False
            });
        }
        vectors.push(vec);
    }
    vectors
}

/// Constructs a BDD representing the given cube (bitvector) over the given variables.
pub fn bdd_from_cube(
    manager_ref: &BDDManagerRef,
    variables: &[BDDFunction],
    cube: &[OptBool],
) -> Result<BDDFunction, OutOfMemory> {
    assert_eq!(
        variables.len(),
        cube.len(),
        "variables and cube must have the same length"
    );
    let mut bdd = manager_ref.with_manager_shared(|manager| BDDFunction::t(manager));
    for (i, bit) in cube.iter().enumerate() {
        let var = variables[i].clone();
        let literal = match *bit {
            OptBool::True => var,
            OptBool::False => var.not()?,
            OptBool::None => continue,
        };
        bdd = bdd.and(&literal)?;
    }
    Ok(bdd)
}

/// Create a BDD from the given iterator over bitvector.
pub fn bdd_from_iter<'a, I>(
    manager_ref: &BDDManagerRef,
    variables: &[BDDFunction],
    vectors: I,
) -> Result<BDDFunction, OutOfMemory>
where
    I: Iterator<Item = &'a Vec<OptBool>>,
{
    let mut bdd = manager_ref.with_manager_shared(|manager| BDDFunction::f(manager));
    for bits in vectors {
        bdd = bdd.or(&bdd_from_cube(manager_ref, variables, bits)?)?;
    }

    Ok(bdd)
}

/// Create a random BDD over the given variables with the given number of cubes.
pub fn random_bdd<R: Rng>(
    manager_ref: &BDDManagerRef,
    rng: &mut R,
    variables: &[BDDFunction],
    num_of_cubes: usize,
) -> Result<BDDFunction, OutOfMemory> {
    let bitvectors = random_bitvectors(rng, variables.len(), num_of_cubes);
    bdd_from_iter(manager_ref, variables, bitvectors.iter())
}
