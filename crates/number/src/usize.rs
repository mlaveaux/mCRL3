// Helper functions for arithmetic operations
pub(crate) fn add_single_number(n1: usize, n2: usize, carry: &mut usize) -> usize {
    debug_assert!(*carry <= 1);

    // Use wrapping operations to handle overflow correctly
    let result = n1.wrapping_add(n2).wrapping_add(*carry);
    // Set carry to 1 if we had an overflow, 0 otherwise
    *carry = if *carry == 0 {
        if result < n1 { 1 } else { 0 } // Overflow if result wrapped around
    } else if result > n1 {
        0
    } else {
        1
    };

    result
}

/// Calculate <carry,result>:=n1*n2+carry, where the lower bits of the calculation
// are stored in the result, and the higher bits are stored in carry.
pub(crate) fn multiply_single_number(n1: usize, n2: usize, carry: &mut usize) -> usize {
    // Split numbers into high and low parts to avoid overflow
    let bits = usize::BITS as usize / 2;
    let mask = (1 << bits) - 1;

    // Extract high and low parts of both numbers
    let n1_low = n1 & mask;
    let n1_high = n1 >> bits;
    let n2_low = n2 & mask;
    let n2_high = n2 >> bits;

    // Multiply parts and accumulate carries
    let mut local_carry = 0;
    // First multiply low parts
    let mut result = add_single_number(n1_low * n2_low, *carry, &mut local_carry);
    let mut cumulative_carry = local_carry;

    // Add product of n1_high * n2_low shifted left
    local_carry = 0;
    result = add_single_number(result, (n1_high * n2_low) << bits, &mut local_carry);
    cumulative_carry += local_carry;

    // Add product of n1_low * n2_high shifted left
    local_carry = 0;
    result = add_single_number(result, (n1_low * n2_high) << bits, &mut local_carry);
    cumulative_carry += local_carry;

    // Calculate final carry including high parts
    *carry = cumulative_carry;
    *carry += (n1_high * n2_low) >> bits; // Add overflow from cross products
    *carry += (n1_low * n2_high) >> bits;
    *carry += n1_high * n2_high; // Add product of high parts

    result
}

/// Divides a single number with remainder, handling the carry.
/// Returns (quotient, remainder).
pub(crate) fn divide_single_number(p: usize, q: usize, mut remainder: usize) -> (usize, usize) {
    debug_assert!(q > remainder);
    let bits = usize::BITS as usize / 2;

    // Split dividend into high and low parts
    let p_high = p >> bits;
    let p_low = p & ((1 << bits) - 1);

    // First handle the high part combined with previous remainder
    let high_part = p_high + (remainder << bits);
    let result_high = high_part / q;
    let remainder_high = high_part % q;

    // Then handle the low part with remainder from high part
    let low_part = p_low + (remainder_high << bits);
    let result_low = low_part / q;
    remainder = low_part % q;

    // Ensure our bit splitting worked correctly
    debug_assert!(result_high < (1 << bits));
    debug_assert!(result_low < (1 << bits));

    // Combine results and return with new remainder
    ((result_high << bits) | result_low, remainder)
}
