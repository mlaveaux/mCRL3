use merc_ldd::Ldd;
use merc_ldd::Value;
use merc_ldd::union;
use oxidd::BooleanFunction;
use oxidd::Function;
use oxidd::ManagerRef;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;

use merc_ldd::DataRef;
use merc_ldd::LddRef;
use merc_ldd::Storage;
use merc_ldd::height;
use merc_utilities::MercError;
use oxidd_core::util::EdgeDropGuard;

/// Converts an LDD representing a set of vectors into a BDD representing the
/// same set by bitblasting the vector elements.
pub fn ldd_to_bdd_simple(
    storage: &mut Storage,
    manager_ref: &BDDManagerRef,
    ldd: &LddRef<'_>,
) -> Result<BDDFunction, MercError> {
    let highest = compute_highest(storage, ldd);
    let bits = compute_bits(&highest);
    let bits_dd = merc_ldd::singleton(storage, &bits);

    ldd_to_bdd(storage, manager_ref, ldd, &bits_dd, 0)
}

/// Converts an LDD representing a set of vectors into a BDD representing the
/// same set by bitblasting the vector elements.
///
/// # Details
///
/// The `bits` should be a singleton LDD containing the result of
/// [compute_bits]. The conversion works recursively by processing the LDD node
/// by node, and introducing bits number of BDD variables for each layer in the
/// LDD. Note that `first_variable` indicates the level of the first variable,
/// and the next bits are placed at consecutive BDD layers. These variables
/// *must* already exist in the given BDD manager.
pub fn ldd_to_bdd(
    storage: &mut Storage,
    manager_ref: &BDDManagerRef,
    ldd: &LddRef<'_>,
    bits: &LddRef<'_>,
    first_variable: u32,
) -> Result<BDDFunction, MercError> {
    // Base cases
    if **storage.empty_set() == *ldd {
        return Ok(manager_ref.with_manager_shared(|manager| BDDFunction::f(manager)));
    }
    if **storage.empty_vector() == *ldd {
        return Ok(manager_ref.with_manager_shared(|manager| BDDFunction::t(manager)));
    }

    // TODO: Implement caching
    let DataRef(value, down, right) = storage.get_ref(ldd);
    let DataRef(bits_value, bits_down, _bits_right) = storage.get_ref(bits); // Is singleton so right is ignored.

    let right = ldd_to_bdd(storage, manager_ref, &right, bits, first_variable)?;

    // Skip bits_value variables for current bits
    let mut down = ldd_to_bdd(storage, manager_ref, &down, &bits_down, first_variable + bits_value)?;

    // Encode current value per bit, starting from least significant bit since it's computed bottom up.
    for i in 0..bits_value {
        let bit = bits_value - i - 1;
        if value & (1 << i) != 0 {
            // bit is 1
            down = manager_ref.with_manager_shared(|manager| {
                BDDFunction::var(manager, first_variable + bit)?.ite(&down, &BDDFunction::f(manager))
            })?;
        } else {
            // bit is 0
            down = manager_ref.with_manager_shared(|manager| {
                BDDFunction::var(manager, first_variable + bit)?.ite(&BDDFunction::f(manager), &down)
            })?;
        }
    }

    Ok(down.or(&right)?)
}

/// Converts a BDD representing a set of bitblasted vectors back into an LDD
/// representing the same set, i.e., the inverse of [ldd_to_bdd].
pub fn bdd_to_ldd(
    storage: &mut Storage,
    manager_ref: &BDDManagerRef,
    bdd: &BDDFunction,
    bits_dd: &LddRef<'_>,
    bit: Value,
    value: Value,
) -> Result<Ldd, MercError> {
    // Base cases
    if manager_ref.with_manager_shared(|manager| {
        *bdd.as_edge(manager) == *EdgeDropGuard::new(manager, BDDFunction::t_edge(manager))
    }) {
        // TODO: Can this be avoided?
        let empty_set = storage.empty_set().clone();
        let empty_vector = storage.empty_vector().clone();

        return Ok(storage.insert(value, &empty_vector, &empty_set));
    }
    if !bdd.satisfiable() {
        return Ok(storage.empty_set().clone());
    }

    // TODO: Implement caching

    // Read the bits required per layer
    let DataRef(num_bits, bits_down, _bits_right) = storage.get_ref(bits_dd);

    if num_bits == bit {
        // We reached the last bit for this layer
        let down = bdd_to_ldd(storage, manager_ref, bdd, &bits_down, 0, 0)?;
        let right = storage.empty_set().clone();
        Ok(storage.insert(value, &down, &right))
    } else {
        let (high, low) = bdd.cofactors().ok_or("Failed to compute cofactors")?;

        // Recurse for high and low cofactors
        let high = bdd_to_ldd(
            storage,
            manager_ref,
            &high,
            bits_dd,
            bit + 1,
            value | (1 << (num_bits - bit - 1)),
        )?;
        let low = bdd_to_ldd(storage, manager_ref, &low, bits_dd, bit + 1, value)?;

        Ok(union(storage, &high, &low))
    }
}

/// Computes the highest value for every layer in the LDD
pub fn compute_highest(storage: &mut Storage, ldd: &LddRef<'_>) -> Vec<u32> {
    let mut result = vec![0; height(storage, ldd)];
    compute_highest_rec(storage, &mut result, ldd, 0);
    result
}

/// Helper function for compute_highest
fn compute_highest_rec(storage: &mut Storage, result: &mut [u32], set: &LddRef<'_>, depth: usize) {
    if set == storage.empty_set() || set == storage.empty_vector() {
        return;
    }

    let DataRef(value, down, right) = storage.get_ref(set);
    compute_highest_rec(storage, result, &right, depth);
    compute_highest_rec(storage, result, &down, depth + 1);

    result[depth] = result[depth].max(value);
}

/// Calculate minimum bits needed to represent the value
/// Use 1 bit if value is 0 to ensure at least 1 bit is written
pub fn required_bits(value: u32) -> u32 {
    (u32::BITS - value.leading_zeros()).max(1)
}

/// Computes the number of bits required to represent the highest value at each layer.
pub fn compute_bits(highest: &[u32]) -> Vec<u32> {
    highest.iter().map(|&h| required_bits(h)).collect()
}

#[cfg(test)]
mod tests {
    use merc_ldd::LddDisplay;
    use merc_ldd::from_iter;
    use merc_ldd::random_vector_set;
    use merc_ldd::singleton;
    use merc_utilities::random_test;

    use crate::FormatConfigSet;
    use crate::create_variables;

    use super::*;

    #[test]
    fn test_random_compute_highest() {
        random_test(100, |rng| {
            let set = random_vector_set(rng, 4, 3, 5);
            let mut storage = Storage::new();
            let ldd = from_iter(&mut storage, set.iter());
            println!("LDD: {}", LddDisplay::new(&storage, &ldd));

            let highest = compute_highest(&mut storage, &ldd);
            println!("Highest: {:?}", highest);
            for (i, h) in highest.iter().enumerate() {
                // Determine the highest value for every vector
                for value in set.iter() {
                    assert!(
                        *h >= value[i],
                        "The highest value for depth {} is {}, but vector has value {}",
                        i,
                        h,
                        value[i]
                    );
                }
            }

            let bits = compute_bits(&highest);
            println!("Bits: {:?}", bits);

            for (i, b) in bits.iter().enumerate() {
                let expected_bits = required_bits(highest[i]);
                assert_eq!(
                    *b, expected_bits,
                    "The number of bits for depth {} is {}, but expected {}",
                    i, b, expected_bits
                );
            }
        })
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Oxidd does not work with miri
    fn test_random_ldd_to_bdd() {
        random_test(100, |rng| {
            let set = random_vector_set(rng, 1, 3, 5);

            let mut storage = Storage::new();
            let ldd = from_iter(&mut storage, set.iter());
            println!("LDD: {}", LddDisplay::new(&storage, &ldd));

            let highest = compute_highest(&mut storage, &ldd);
            let bits = compute_bits(&highest);
            let bits_dd = singleton(&mut storage, &bits);

            let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);

            let total_bits: u32 = bits.iter().sum();
            println!("Total bits: {}", total_bits);
            println!("Bits per layer: {:?}", bits);
            let _variables = create_variables(&manager_ref, total_bits).unwrap();

            let bdd = ldd_to_bdd(&mut storage, &manager_ref, &ldd, &bits_dd, 0).unwrap();
            println!("resulting BDD: {}", FormatConfigSet(&bdd));

            let resulting_ldd = bdd_to_ldd(&mut storage, &manager_ref, &bdd, &bits_dd, 0, 0).unwrap();

            println!("resulting LDD: {}", LddDisplay::new(&storage, &resulting_ldd));
            assert_eq!(ldd, resulting_ldd, "Converted LDD does not match original");
        });
    }
}
