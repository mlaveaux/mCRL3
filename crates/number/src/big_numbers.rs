use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::str::FromStr;

/// A big natural number implementation that stores numbers as a vector of machine-sized words.
/// Numbers are stored with the least significant word first, and there are no trailing zeros.
#[derive(Debug, Clone, Eq, Hash)]
pub struct BigNatural {
    /// Numbers are stored as words, with the most significant number last.
    /// The representation is unique: no trailing zeros are allowed.
    digits: Vec<usize>,
}

impl BigNatural {
    /// Creates a new BigNatural set to zero.
    pub fn new() -> Self {
        Self { digits: Vec::new() }
    }

    /// Creates a new BigNatural from a machine-sized number.
    pub fn from_usize(n: usize) -> Self {
        if n == 0 {
            Self::new()
        } else {
            Self { digits: vec![n] }
        }
    }

    /// Returns true if this number equals zero.
    pub fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    /// Returns true if this number equals the given machine-sized number.
    pub fn is_number(&self, n: usize) -> bool {
        if n == 0 {
            self.digits.is_empty()
        } else {
            self.digits.len() == 1 && self.digits[0] == n
        }
    }

    /// Sets the number to zero.
    pub fn clear(&mut self) {
        self.digits.clear();
    }

    /// Removes trailing zeros from the internal representation.
    fn is_well_defined(&mut self) {
        debug_assert!(self.digits.is_empty() || *self.digits.last().unwrap() != 0);
    }

    // Helper functions for arithmetic operations
    fn add_single_number(n1: usize, n2: usize, carry: &mut usize) -> usize {
        debug_assert!(*carry <= 1);
        // Use wrapping operations to handle overflow correctly
        let result = n1.wrapping_add(n2).wrapping_add(*carry);
        // Set carry to 1 if we had an overflow, 0 otherwise
        *carry = if *carry == 0 {
            if result < n1 { 1 } else { 0 }  // Overflow if result wrapped around
        } else {
            if result > n1 { 0 } else { 1 }  // No overflow if result increased
        };
        result
    }

    fn subtract_single_number(n1: usize, n2: usize, carry: &mut usize) -> usize {
        debug_assert!(*carry <= 1);
        let result = n1.wrapping_sub(n2).wrapping_sub(*carry);
        *carry = if *carry == 0 {
            if result > n1 { 1 } else { 0 }
        } else {
            if result < n1 { 0 } else { 1 }
        };
        result
    }

    fn multiply_single_number(n1: usize, n2: usize, carry: &mut usize) -> usize {
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
        let mut result = Self::add_single_number(n1_low * n2_low, *carry, &mut local_carry);
        let mut cumulative_carry = local_carry;
        
        // Add product of n1_high * n2_low shifted left
        local_carry = 0;
        result = Self::add_single_number(result, (n1_high * n2_low) << bits, &mut local_carry);
        cumulative_carry += local_carry;
        
        // Add product of n1_low * n2_high shifted left
        local_carry = 0;
        result = Self::add_single_number(result, (n1_low * n2_high) << bits, &mut local_carry);
        cumulative_carry += local_carry;
        
        // Calculate final carry including high parts
        *carry = cumulative_carry;
        *carry += (n1_high * n2_low) >> bits;  // Add overflow from cross products
        *carry += (n1_low * n2_high) >> bits;
        *carry += n1_high * n2_high;  // Add product of high parts
        
        result
    }

    /// Divides a single number with remainder, handling the carry.
    /// Returns (quotient, remainder).
    fn divide_single_number(p: usize, q: usize, mut remainder: usize) -> (usize, usize) {
        debug_assert!(q > remainder);
        let bits = usize::BITS as usize / 2;
        
        // Split dividend into high and low parts
        let p_high = p >> bits;
        let p_low = p & ((1 << bits) - 1);
        
        // First handle the high part combined with previous remainder
        let high_part = (p_high as usize) + ((remainder as usize) << bits);
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

    /// Calculates the greatest common divisor of two numbers.
    pub fn greatest_common_divisor(mut a: usize, mut b: usize) -> usize {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        
        a
    }

    /// Divides this number by another, storing the result in `quotient` and remainder in `remainder`.
    pub fn div_mod(&self, other: &Self, quotient: &mut Self, remainder: &mut Self) {
        assert!(!other.is_zero(), "Division by zero");
        
        // Initialize results
        *quotient = Self::new();
        *remainder = self.clone();
        
        // If dividend is smaller than divisor, quotient is 0
        if self.digits.len() < other.digits.len() {
            return;
        }

        // Handle single-digit division as a special case for efficiency
        if self.digits.len() == 1 && other.digits.len() == 1 {
            let (q, r) = Self::divide_single_number(
                self.digits[0], 
                other.digits[0], 
                0
            );
            if q > 0 {
                quotient.digits.push(q);
            }
            if r > 0 {
                remainder.digits.push(r);
            }
            return;
        }

        // Long division algorithm
        let mut shift = self.digits.len() - other.digits.len();
        let mut divisor = other.clone();
        
        // Align divisor with dividend by adding leading zeros
        for _ in 0..shift {
            divisor.digits.insert(0, 0);
        }

        // Initialize quotient with proper size
        quotient.digits = vec![0; shift + 1];
        
        // Perform division by repeated subtraction
        while shift > 0 {
            while *remainder >= divisor {
                remainder.subtract(&divisor);
                quotient.digits[shift] += 1;
            }
            // Remove one leading zero and continue with next digit
            divisor.digits.remove(0);
            shift -= 1;
        }

        // Handle the last digit (no shift)
        while *remainder >= *other {
            remainder.subtract(other);
            quotient.digits[0] += 1;
        }

        // Remove leading zeros from results
        quotient.normalize();
        remainder.normalize();
    }

    /// Subtracts another number from this one.
    /// Assumes this number is larger than the other.
    pub fn subtract(&mut self, other: &Self) {
        assert!(*self >= *other, "Subtraction would result in negative number");
        
        let mut carry = 0;
        for i in 0..self.digits.len() {
            let n2 = other.digits.get(i).copied().unwrap_or(0);
            self.digits[i] = Self::subtract_single_number(self.digits[i], n2, &mut carry);
        }
        
        assert!(carry == 0, "Subtraction overflow");
        self.normalize();
    }

    /// Multiplies this number by another.
    pub fn multiply(&mut self, other: &Self) {
        if self.is_zero() || other.is_zero() {
            self.clear();
            return;
        }

        let mut result = vec![0; self.digits.len() + other.digits.len()];
        
        for (i, &n1) in self.digits.iter().enumerate() {
            let mut carry = 0;
            for (j, &n2) in other.digits.iter().enumerate() {
                let mut local_carry = 0;
                let prod = Self::multiply_single_number(n1, n2, &mut carry);
                result[i + j] = Self::add_single_number(result[i + j], prod, &mut local_carry);
                
                let mut k = 1;
                let mut c = local_carry;
                while c > 0 {
                    result[i + j + k] = Self::add_single_number(result[i + j + k], c, &mut local_carry);
                    c = local_carry;
                    k += 1;
                }
            }
            if carry > 0 {
                result[i + other.digits.len()] = Self::add_single_number(
                    result[i + other.digits.len()], 
                    carry, 
                    &mut carry
                );
            }
        }

        self.digits = result;
        self.normalize();
    }

    /// Adds another number to this one.
    fn add_impl(&mut self, other: &Self) {
        let mut carry = 0;
        let max_len = self.digits.len().max(other.digits.len());
        
        // Ensure we have enough space
        self.digits.resize(max_len, 0);
        
        // Add corresponding digits
        for i in 0..max_len {
            let n2 = other.digits.get(i).copied().unwrap_or(0);
            self.digits[i] = Self::add_single_number(self.digits[i], n2, &mut carry);
        }
        
        if carry > 0 {
            self.digits.push(carry);
        }
    }

    /// Divide the current number by n. If there is a remainder return it.
    pub fn divide_by(&mut self, n: usize) -> usize {
        let mut remainder = 0;
        for digit in self.digits.iter_mut().rev() {
            let (quotient, new_remainder) = Self::divide_single_number(*digit, n, remainder);
            *digit = quotient;
            remainder = new_remainder;
        }
        self.normalize();
        remainder
    }

    /// Removes trailing zeros from the internal representation.
    pub fn normalize(&mut self) {
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        debug_assert!(self.digits.is_empty() || *self.digits.last().unwrap() != 0);
    }

}

// Standard trait implementations

impl Default for BigNatural {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for BigNatural {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = BigNatural::new();
        for c in s.chars() {
            if !c.is_ascii_digit() {
                return Err(format!("Invalid character in number: {}", c));
            }
            let mut carry = (c as usize) - ('0' as usize);
            let mut temp = result.digits.clone();
            for digit in &mut temp {
                let product = BigNatural::multiply_single_number(*digit, 10, &mut carry);
                *digit = product;
            }
            if carry > 0 {
                temp.push(carry);
            }
            result.digits = temp;
            result.normalize();
        }
        Ok(result)
    }
}

impl fmt::Display for BigNatural {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            write!(f, "0")
        } else {
            let mut temp = self.clone();
            while !temp.is_zero() {
                let digit = temp.divide_by(10);
                write!(f, "{}", digit)?;
            }
            Ok(())
        }
    }
}

impl Div for &BigNatural {
    type Output = BigNatural;

    fn div(self, other: &BigNatural) -> BigNatural {
        let mut quotient = BigNatural::new();
        let mut remainder = BigNatural::new();
        self.div_mod(other, &mut quotient, &mut remainder);
        quotient
    }
}

impl Rem for &BigNatural {
    type Output = BigNatural;

    fn rem(self, other: &BigNatural) -> BigNatural {
        let mut quotient = BigNatural::new();
        let mut remainder = BigNatural::new();
        self.div_mod(other, &mut quotient, &mut remainder);
        remainder
    }
}

impl PartialEq for BigNatural {
    fn eq(&self, other: &Self) -> bool {
        if self.digits.len() != other.digits.len() {
            return false;
        }
        self.digits == other.digits
    }
}

impl PartialOrd for BigNatural {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigNatural {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.digits.len().cmp(&other.digits.len()) {
            Ordering::Equal => {
                for (a, b) in self.digits.iter().rev().zip(other.digits.iter().rev()) {
                    match a.cmp(b) {
                        Ordering::Equal => continue,
                        other => return other,
                    }
                }
                Ordering::Equal
            }
            other => other,
        }
    }
}

impl Add for &BigNatural {
    type Output = BigNatural;

    fn add(self, other: &BigNatural) -> BigNatural {
        let mut result = self.clone();
        result.add_impl(other);
        result
    }
}

impl Sub for &BigNatural {
    type Output = BigNatural;

    fn sub(self, other: &BigNatural) -> BigNatural {
        let mut result = self.clone();
        result.subtract(other);
        result
    }
}

impl Mul for &BigNatural {
    type Output = BigNatural;

    fn mul(self, other: &BigNatural) -> BigNatural {
        let mut result = self.clone();
        result.multiply(other);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let zero = BigNatural::new();
        let one = BigNatural::from_usize(1);
        
        assert!(zero.is_zero());
        assert!(one.is_number(1));
        assert_eq!(zero.to_string(), "0");
        assert_eq!(one.to_string(), "1");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(BigNatural::from_str("0").unwrap().to_string(), "0");
        assert_eq!(BigNatural::from_str("123").unwrap().to_string(), "123");
        assert!(BigNatural::from_str("12a3").is_err());
    }

    #[test]
    fn test_division() {
        let a = BigNatural::from_str("1000").unwrap();
        let b = BigNatural::from_str("7").unwrap();
        assert_eq!((&a / &b).to_string(), "142");
        assert_eq!((&a % &b).to_string(), "6");
    }

    #[test]
    fn test_single_digit_division() {
        let mut n = BigNatural::from_str("123").unwrap();
        assert_eq!(n.divide_by(10), 3);
        assert_eq!(n.to_string(), "12");
    }

    #[test]
    fn test_large_division() {
        let a = BigNatural::from_str("123456789123456789").unwrap();
        let b = BigNatural::from_str("987654321").unwrap();
        let result = &a / &b;
        assert_eq!(result.to_string(), "124999998");
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_division_by_zero() {
        let a = BigNatural::from_str("123").unwrap();
        let b = BigNatural::new();
        let _ = &a / &b;
    }

    #[test]
    fn test_add() {
        let a = BigNatural::from_str("123").unwrap();
        let b = BigNatural::from_str("456").unwrap();
        assert_eq!((&a + &b).to_string(), "579");
    }

    #[test]
    fn test_subtract() {
        let a = BigNatural::from_str("456").unwrap();
        let b = BigNatural::from_str("123").unwrap();
        assert_eq!((&a - &b).to_string(), "333");
    }

    #[test]
    fn test_multiply() {
        let a = BigNatural::from_str("123").unwrap();
        let b = BigNatural::from_str("456").unwrap();
        assert_eq!((&a * &b).to_string(), "56088");
    }

    #[test]
    fn test_comparison() {
        let a = BigNatural::from_str("123").unwrap();
        let b = BigNatural::from_str("456").unwrap();
        let c = BigNatural::from_str("123").unwrap();
        
        assert!(a < b);
        assert!(b > a);
        assert!(a == c);
        assert!(a <= c);
        assert!(a >= c);
    }

    #[test]
    #[should_panic(expected = "Subtraction would result in negative number")]
    fn test_negative_subtraction() {
        let a = BigNatural::from_str("123").unwrap();
        let b = BigNatural::from_str("456").unwrap();
        let _ = &a - &b;
    }
}
