use merc_ldd::Ldd;
use merc_ldd::Value;
use merc_ldd::union;
use oxidd::BooleanFunction;
use oxidd::Function;
use oxidd::ManagerRef;
use oxidd::VarNo;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;

use merc_ldd::DataRef;
use merc_ldd::LddRef;
use merc_ldd::Storage;
use merc_ldd::height;
use merc_utilities::MercError;
use oxidd::util::OptBool;
use oxidd_core::util::EdgeDropGuard;

/// Converts an LDD representing a set of vectors into a BDD representing the
/// same set by bitblasting the vector elements using the given variables as bits.
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
    vars: &[VarNo],
) -> Result<BDDFunction, MercError> {
    // Base cases
    if **storage.empty_set() == *ldd {
        return Ok(manager_ref.with_manager_shared(|manager| BDDFunction::f(manager)));
    }
    if **storage.empty_vector() == *ldd {
        return Ok(manager_ref.with_manager_shared(|manager| BDDFunction::t(manager)));
    }

    let DataRef(value, down, right) = storage.get_ref(ldd);
    let DataRef(bits_value, bits_down, _bits_right) = storage.get_ref(bits);

    // Right branch does not consume variables at this layer
    let right_bdd = ldd_to_bdd(storage, manager_ref, &right, bits, vars)?;

    // Ensure we have enough variables for this layer
    let needed = bits_value as usize;
    if vars.len() < needed {
        return Err(format!(
            "Insufficient variables: need {needed}, have {} for current layer",
            vars.len()
        )
        .into());
    }

    // Recurse on down with the remaining variables after consuming this layer
    let mut down_bdd = ldd_to_bdd(storage, manager_ref, &down, &bits_down, &vars[needed..])?;

    // Encode current value using the variables for this layer (MSB to LSB)
    // Current layer variables: vars[0..bits_value]
    for i in 0..bits_value {
        let bit = bits_value - i - 1; // MSB first
        let var_no = vars[bit as usize];
        if value & (1 << i) != 0 {
            // bit is 1
            down_bdd = manager_ref.with_manager_shared(|manager| {
                BDDFunction::var(manager, var_no)?.ite(&down_bdd, &BDDFunction::f(manager))
            })?;
        } else {
            // bit is 0
            down_bdd = manager_ref.with_manager_shared(|manager| {
                BDDFunction::var(manager, var_no)?.ite(&BDDFunction::f(manager), &down_bdd)
            })?;
        }
    }

    Ok(down_bdd.or(&right_bdd)?)
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

/// Iterator that yields values reconstructed from a bit cube, from a BDD obtained by
/// [ldd_to_bdd].
pub struct ValuesIter<'a> {
    bits: &'a [OptBool],
    num_of_bits: &'a [u32],
    offset: usize,
    index: usize,
}

impl<'a> ValuesIter<'a> {
    pub fn new(bits: &'a [OptBool], num_of_bits: &'a [u32]) -> Self {
        Self {
            bits,
            num_of_bits,
            offset: 0,
            index: 0,
        }
    }
}

impl<'a> Iterator for ValuesIter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.num_of_bits.len() {
            return None;
        }

        let nb = self.num_of_bits[self.index] as usize;
        if self.offset + nb > self.bits.len() {
            // Malformed input; stop iterating.
            return None;
        }

        let value = to_value(&self.bits[self.offset..self.offset + nb]);
        self.offset += nb;
        self.index += 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.num_of_bits.len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for ValuesIter<'a> {}

/// Reconstruct the value represented by the bits as encoded by
/// [crate::ldd_to_bdd]. So most significant bit first.
pub fn to_value(bits: &[OptBool]) -> u64 {
    let mut value = 0u64;
    for (i, bit) in bits.iter().rev().enumerate() {
        if *bit == OptBool::True {
            value |= 1 << i;
        }
    }

    value
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

/// Calculate minimum bits needed to represent the value
pub fn required_bits_64(value: u64) -> u32 {
    (u64::BITS - value.leading_zeros()).max(1)
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
    use oxidd::Manager;

    use crate::FormatConfigSet;

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
            let set = random_vector_set(rng, 2, 3, 5);

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
            let vars = manager_ref.with_manager_exclusive(|manager| manager.add_vars(total_bits).collect::<Vec<_>>());

            let bdd = ldd_to_bdd(&mut storage, &manager_ref, &ldd, &bits_dd, &vars).unwrap();
            println!("resulting BDD: {}", FormatConfigSet(&bdd));

            let resulting_ldd = bdd_to_ldd(&mut storage, &manager_ref, &bdd, &bits_dd, 0, 0).unwrap();

            println!("resulting LDD: {}", LddDisplay::new(&storage, &resulting_ldd));
            // assert_eq!(ldd, resulting_ldd, "Converted LDD does not match original");
        });
    }
}
