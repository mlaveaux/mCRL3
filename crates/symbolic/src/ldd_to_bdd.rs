use oxidd::Edge;
use oxidd::HasLevel;
use oxidd::InnerNode;
use oxidd::Manager;
use oxidd::ManagerRef;
use oxidd::bdd::BDDManagerRef;
use oxidd::util::Borrowed;
use oxidd::util::OutOfMemory;
use oxidd_core::function::EdgeOfFunc;
use oxidd::bdd::BDDFunction;
use oxidd::BooleanFunction;
use oxidd::Function;
use oxidd::util::OptBool;
use oxidd::VarNo;

use merc_ldd::DataRef;
use merc_ldd::height;
use merc_ldd::Ldd;
use merc_ldd::LddRef;
use merc_ldd::Storage;
use merc_ldd::union;
use merc_ldd::Value;
use oxidd_rules_bdd::simple::BDDTerminal;

/// Converts an LDD representing a set of vectors into a BDD representing the
/// same set by bitblasting the vector elements using the given variables as
/// bits.
///
/// # Details
///
/// The `bits_per_layer` should be a singleton LDD containing the result of
/// [compute_bits]. The conversion works recursively by processing the LDD node
/// by node, and introducing bit number of BDD variables (given by
/// `bit_variables`) for each layer in the LDD. These variables *must* already
/// exist in the given BDD manager.
pub fn ldd_to_bdd<'id>(
    storage: &mut Storage,
    manager: &<BDDFunction as Function>::Manager<'id>,
    ldd: &LddRef<'_>,
    bits_per_layer: &LddRef<'_>,
    bit_variables: &[VarNo],
) -> Result<BDDFunction, OutOfMemory> {

    // Base cases
    if **storage.empty_set() == *ldd {
        return Ok(BDDFunction::f(manager));
    }
    if **storage.empty_vector() == *ldd {
        return Ok(BDDFunction::t(manager));
    }

    // TODO: Implement caching

    let DataRef(value, down, right) = storage.get_ref(ldd);
    let DataRef(bits_value, bits_down, _bits_right) = storage.get_ref(bits_per_layer);

    // Right branch does not consume variables at this layer
    let right_bdd = ldd_to_bdd(storage, manager, &right, bits_per_layer, bit_variables)?;

    // Ensure we have enough variables for this layer
    let needed = bits_value as usize;
    if bit_variables.len() < needed {
        panic!("Insufficient variables: need {needed}, have {} for current layer", bit_variables.len());
    }

    // Recurse on down with the remaining variables after consuming this layer
    let mut down_bdd = ldd_to_bdd(storage, manager, &down, &bits_down, &bit_variables[needed..])?;

    // Encode current value using the variables for this layer (MSB to LSB)
    // Current layer variables: vars[0..bits_value]
    for i in 0..bits_value {
        let bit = bits_value - i - 1; // MSB first
        let var_no = bit_variables[bit as usize];
        if value & (1 << i) != 0 {
            // bit is 1
            down_bdd = BDDFunction::var(manager, var_no)?.ite(&down_bdd, &BDDFunction::f(manager))?;
        } else {
            // bit is 0
            down_bdd = BDDFunction::var(manager, var_no)?.ite(&BDDFunction::f(manager), &down_bdd)?;
        }
    }

    Ok(down_bdd.or(&right_bdd)?)
}

/// Converts a BDD representing a set of bitblasted vectors back into an LDD
/// representing the same set, i.e., the inverse of [ldd_to_bdd].
pub fn bdd_to_ldd<'id>(
    storage: &mut Storage,
    manager_ref: &BDDManagerRef,
    bdd: &BDDFunction,
    variables: &[VarNo],
    bits_per_layer: &[Value],
    current_bit: Value,
    current_value: Value,
) -> Result<Ldd, OutOfMemory> {
    manager_ref.with_manager_shared(|manager| {
        let edge = bdd.as_edge(manager);
        bdd_to_ldd_edge(storage, manager, edge.borrowed(), variables, bits_per_layer, current_bit, current_value)
    })
}

/// Recursive implementation of [bdd_to_ldd].
pub fn bdd_to_ldd_edge<'id>(
    storage: &mut Storage,
    manager: &<BDDFunction as Function>::Manager<'id>,
    bdd: Borrowed<EdgeOfFunc<'id, BDDFunction>>,
    variables: &[VarNo],
    bits_per_layer: &[Value],
    current_bit: Value,
    current_value: Value,
) -> Result<Ldd, OutOfMemory> {
    let bdd_node = match manager.get_node(&bdd) {
        oxidd::Node::Inner(node) => node,
        oxidd::Node::Terminal(terminal) => {
            // Base cases
            match terminal {
                BDDTerminal::False => {
                    return Ok(storage.empty_set().clone());
                }
                BDDTerminal::True => {
                    if !variables.is_empty() {
                        // If there are still variables left, we must generate don't cares for the remaining layers
                        let num_bits = bits_per_layer.first().copied().expect("Missing bits per layer for current layer");

                        // There are don't care variables in this BDD that have been skipped, so generate both branches.
                        if num_bits == current_bit {
                            // We reached the last bit for this layer, variable still belongs to next layer.
                            let down = bdd_to_ldd_edge(storage, manager, bdd, &variables, &bits_per_layer[1..], 0, 0)?;
                            let right = storage.empty_set().clone();
                            return Ok(storage.insert(current_value, &down, &right))
                        } else {  
                            debug_assert!(current_bit < num_bits, "Current bit exceeds number of bits for layer");        
                            let high = bdd_to_ldd_edge(storage, manager, bdd.borrowed(), &variables[1..], bits_per_layer, current_bit + 1, current_value | (1 << (num_bits - current_bit - 1)))?;
                            let low = bdd_to_ldd_edge(storage, manager, bdd, &variables[1..], bits_per_layer, current_bit + 1, current_value)?;
                            return Ok(union(storage, &high, &low));
                        }
                    }

                    return Ok(storage.insert_singleton(current_value));
                }
            }
        },
    };

    // TODO: Implement caching

    // Read the bits required per layer
    let num_bits = bits_per_layer.first().copied().expect("Missing bits per layer for current layer");

    let variable_level = variables.first().expect("Missing variable for current layer");
    if *variable_level < bdd_node.level() { 
        // There are don't care variables in this BDD that have been skipped, so generate both branches without cofactors.
        if num_bits == current_bit {
            // We reached the last bit for this layer, variable still belongs to next layer.
            let down = bdd_to_ldd_edge(storage, manager, bdd, &variables, &bits_per_layer[1..], 0, 0)?;
            let right = storage.empty_set().clone();
            return Ok(storage.insert(current_value, &down, &right))
        } else {    
            debug_assert!(current_bit < num_bits, "Current bit exceeds number of bits for layer");
            let high = bdd_to_ldd_edge(storage, manager, bdd.borrowed(), &variables[1..], bits_per_layer, current_bit + 1, current_value | (1 << (num_bits - current_bit - 1)))?;
            let low = bdd_to_ldd_edge(storage, manager, bdd, &variables[1..], bits_per_layer, current_bit + 1, current_value)?;
            return Ok(union(storage, &high, &low));
        }
    }

    debug_assert_eq!(*variable_level, bdd_node.level(), "Levels do not match");
    if num_bits == current_bit {
        // We reached the last bit for this layer
        let down = bdd_to_ldd_edge(storage, manager, bdd, variables, &bits_per_layer[1..], 0, 0)?;
        let right = storage.empty_set().clone();
        Ok(storage.insert(current_value, &down, &right))
    } else {
        debug_assert!(current_bit < num_bits, "Current bit exceeds number of bits for layer");  
        let (bdd_high, bdd_low) = collect_children(bdd_node);

        // Recurse for high and low cofactors
        let high = bdd_to_ldd_edge(
            storage,
            manager,
            bdd_high,
            &variables[1..],
            bits_per_layer,
            current_bit + 1,
            current_value | (1 << (num_bits - current_bit - 1)),
        )?;
        let low = bdd_to_ldd_edge(storage, manager, bdd_low, &variables[1..], bits_per_layer, current_bit + 1, current_value)?;

        Ok(union(storage, &high, &low))
    }
}

/// Computes the highest value for every layer in the LDD
pub fn compute_highest(storage: &mut Storage, ldd: &LddRef<'_>) -> Vec<u32> {
    let mut result = vec![0; height(storage, ldd)];
    compute_highest_rec(storage, &mut result, ldd, 0);
    result
}

/// Collect the two children of a binary node
#[inline]
#[must_use]
fn collect_children<E: Edge, N: InnerNode<E>>(node: &N) -> (Borrowed<'_, E>, Borrowed<'_, E>) {
    debug_assert_eq!(N::ARITY, 2);
    let mut it = node.children();
    let f_then = it.next().unwrap();
    let f_else = it.next().unwrap();
    debug_assert!(it.next().is_none());
    (f_then, f_else)
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
    use oxidd::ManagerRef;

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
            let set = random_vector_set(rng, 50, 3, 5);

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
            let bdd =  manager_ref.with_manager_shared(|manager| ldd_to_bdd(&mut storage, manager, &ldd, &bits_dd, &vars).unwrap() );
            println!("resulting BDD: {}", FormatConfigSet(&bdd));
            let resulting_ldd = bdd_to_ldd(&mut storage, &manager_ref, &bdd, &vars, &bits, 0, 0).unwrap();

            println!("resulting LDD: {}", LddDisplay::new(&storage, &resulting_ldd));
            assert_eq!(ldd, resulting_ldd, "Converted LDD does not match original");
        });
    }
}
