use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Rem;
use std::ops::Sub;
use std::str::FromStr;

use crate::add_single_number;
use crate::divide_single_number;
use crate::multiply_single_number;
use crate::subtract_single_number;

/// A big natural number implementation that stores numbers as a vector of machine-sized words.
/// Numbers are stored with the least significant word first, and there are no trailing zeros.
#[derive(Debug, Clone, Eq)]
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
        if n == 0 { Self::new() } else { Self { digits: vec![n] } }
    }

    /// Returns true if this number equals zero.
    pub fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    /// Sets the number to zero.
    pub fn zero(&mut self) {
        self.digits.clear();
    }

    /// Removes trailing zeros from the internal representation.
    fn is_well_defined(&self) {
        debug_assert!(self.digits.is_empty() || *self.digits.last().unwrap() != 0);
    }

    /// Divides this number by another, storing the result in `quotient` and remainder in `remainder`.
    pub fn div_mod(
        &self,
        other: &Self,
        quotient: &mut Self,
        remainder: &mut Self,
        calculation_buffer_divisor: &mut Self,
    ) {
        self.is_well_defined();
        other.is_well_defined();
        assert!(!other.is_zero(), "Division by zero");

        if self.digits.len() == 1 && other.digits.len() == 1 {
            let n = self.digits[0] / other.digits[0]; // Calculate div.
            if n == 0 {
                quotient.zero();
            } else {
                quotient.digits.resize(1, 0);
                quotient.digits[0] = n;
            }
            let n = self.digits[0] % other.digits[0]; // Calculate mod.
            if n == 0 {
                remainder.zero();
            } else {
                remainder.digits.resize(1, 0);
                remainder.digits[0] = n;
            }

            quotient.is_well_defined();
            remainder.is_well_defined();
            return;
        }

        // The procedure below works bitwise, as no efficient division algorithm has yet been implemented.
        quotient.zero();
        *remainder = self.clone();

        if self.digits.len() < other.digits.len() {
            quotient.is_well_defined();
            remainder.is_well_defined();
            return;
        }

        let no_of_bits_per_digit = usize::BITS as usize;
        calculation_buffer_divisor
            .digits
            .resize(1 + self.digits.len() - other.digits.len(), 0);

        // Place 0 digits at least significant position of the calculation_buffer_divisor to make it of comparable length as the remainder.
        for &digit in &other.digits {
            calculation_buffer_divisor.digits.push(digit);
        }
        calculation_buffer_divisor.normalize();

        for _ in 0..=no_of_bits_per_digit * (self.digits.len() - other.digits.len() + 1) {
            if remainder < calculation_buffer_divisor {
                // We cannot subtract the calculation_buffer_divisor from the remainder.
                quotient.multiply_by(2, 0); // quotient = quotient * 2
            } else {
                // We subtract the calculation_buffer_divisor from the remainder.
                quotient.multiply_by(2, 1); // quotient = quotient * 2 + 1
                remainder.subtract(calculation_buffer_divisor);
            }
            calculation_buffer_divisor.divide_by(2); // Shift the calculation_buffer_divisor one bit to the left.
        }

        quotient.normalize();
        quotient.is_well_defined();
        remainder.is_well_defined();
    }

    /// Subtracts another number from this one.
    /// Assumes this number is larger than the other.
    pub fn subtract(&mut self, other: &Self) {
        assert!(*self >= *other, "Subtraction would result in negative number");

        let mut carry = 0;
        for i in 0..self.digits.len() {
            let n2 = other.digits.get(i).copied().unwrap_or(0);
            self.digits[i] = subtract_single_number(self.digits[i], n2, &mut carry);
        }

        assert!(carry == 0, "Subtraction overflow");
        self.normalize();
    }

    /// Multiplies this number by another, storing the result in `result` and using a calculation buffer.
    /// Initially, `result` must be zero. At the end, `result` equals `self * other + result`.
    /// The `calculation_buffer` does not need to be initialized.
    pub fn multiply(&self, other: &Self, result: &mut Self, calculation_buffer: &mut Self) {
        self.is_well_defined();
        other.is_well_defined();
        result.is_well_defined();

        for (offset, &digit) in other.digits.iter().enumerate() {
            // Clear the calculation buffer and prepare it for the current offset.
            calculation_buffer.zero();
            calculation_buffer.digits.resize(offset, 0);

            // Copy the digits of `self` into the calculation buffer.
            calculation_buffer.digits.extend_from_slice(&self.digits);

            // Multiply the calculation buffer by the current digit.
            calculation_buffer.multiply_by(digit, 0);

            // Add the calculation buffer to the result.
            result.add_impl(calculation_buffer);
        }

        result.is_well_defined();
    }

    /// Divide the current number by n. If there is a remainder return it.
    pub fn divide_by(&mut self, n: usize) -> usize {
        let mut remainder = 0;
        for digit in self.digits.iter_mut().rev() {
            let (quotient, new_remainder) = divide_single_number(*digit, n, remainder);
            *digit = quotient;
            remainder = new_remainder;
        }
        self.normalize();
        remainder
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
            self.digits[i] = add_single_number(self.digits[i], n2, &mut carry);
        }

        if carry > 0 {
            self.digits.push(carry);
        }
    }

    /// Removes trailing zeros from the internal representation.
    fn normalize(&mut self) {
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        debug_assert!(self.digits.is_empty() || *self.digits.last().unwrap() != 0);
    }

    /// Multiplies the current number by `n` and adds the carry.
    pub fn multiply_by(&mut self, n: usize, mut carry: usize) {
        for digit in &mut self.digits {
            *digit = multiply_single_number(*digit, n, &mut carry);
        }
        if carry > 0 {
            // Add an extra digit with the carry.
            self.digits.push(carry);
        }
        self.normalize();
        self.is_well_defined();
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
                let product = multiply_single_number(*digit, 10, &mut carry);
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
            let mut digits = Vec::new();
            while !temp.is_zero() {
                let digit = temp.divide_by(10);
                digits.push(digit);
            }

            for digit in digits.iter().rev() {
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
        let mut buffer = BigNatural::new();
        self.div_mod(other, &mut quotient, &mut remainder, &mut buffer);
        quotient
    }
}

impl Rem for &BigNatural {
    type Output = BigNatural;

    fn rem(self, other: &BigNatural) -> BigNatural {
        let mut quotient = BigNatural::new();
        let mut remainder = BigNatural::new();
        let mut buffer = BigNatural::new();
        self.div_mod(other, &mut quotient, &mut remainder, &mut buffer);
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

impl Hash for BigNatural {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.digits.hash(state);
    }
}

/// Returns true if this number equals the given machine-sized number.
impl PartialEq<usize> for BigNatural {
    fn eq(&self, n: &usize) -> bool {
        if *n == 0 {
            self.digits.is_empty()
        } else {
            self.digits.len() == 1 && self.digits[0] == *n
        }
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
        let mut calculation_buffer = BigNatural::new();
        let mut result = BigNatural::new();
        self.multiply(other, &mut result, &mut calculation_buffer);
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
        assert!(one == 1);
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
