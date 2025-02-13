//! Mathematical utility functions.

/// Computes the base 2 logarithm of n (ceiling), by checking the leftmost bit.
///
/// # Arguments
/// * `n` - The number to compute the logarithm of
///
/// # Panics
/// Panics if n is zero
///
/// # Examples
/// ```
/// use number::ceil_log2;
///
/// assert_eq!(ceil_log2(1), 1);
/// assert_eq!(ceil_log2(2), 2);
/// assert_eq!(ceil_log2(3), 2);
/// assert_eq!(ceil_log2(4), 3);
/// ```
pub fn ceil_log2(mut n: usize) -> usize {
    assert!(n > 0, "Cannot compute logarithm of zero");

    let mut result = 0;
    while n != 0 {
        n >>= 1;
        result += 1;
    }
    result
}

/// Calculates n^m for usize numbers using fast exponentiation.
///
/// # Arguments
/// * `n` - The base number
/// * `m` - The exponent
///
/// # Examples
/// ```
/// use number::power;
///
/// assert_eq!(power(2, 3), 8);
/// assert_eq!(power(3, 2), 9);
/// assert_eq!(power(5, 0), 1);
/// ```
pub fn power(mut n: usize, mut m: usize) -> usize {
    let mut result = 1;

    // Invariant: result * n^m = n_initial^m_initial
    while m > 0 {
        if m % 2 == 1 {
            result *= n;
        }
        n *= n;
        m /= 2;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceil_log2() {
        assert_eq!(ceil_log2(1), 1);
        assert_eq!(ceil_log2(2), 2);
        assert_eq!(ceil_log2(3), 2);
        assert_eq!(ceil_log2(4), 3);
        assert_eq!(ceil_log2(5), 3);
        assert_eq!(ceil_log2(7), 3);
        assert_eq!(ceil_log2(8), 4);
        assert_eq!(ceil_log2(9), 4);
    }

    #[test]
    #[should_panic(expected = "Cannot compute logarithm of zero")]
    fn test_ceil_log2_zero() {
        ceil_log2(0);
    }

    #[test]
    fn test_power() {
        assert_eq!(power(2, 0), 1);
        assert_eq!(power(2, 1), 2);
        assert_eq!(power(2, 3), 8);
        assert_eq!(power(3, 2), 9);
        assert_eq!(power(5, 3), 125);
    }
}
