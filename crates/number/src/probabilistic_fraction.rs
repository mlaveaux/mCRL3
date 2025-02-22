use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::str::FromStr;

use crate::big_numbers::BigNatural;

/// A fraction type for probabilistic transitions that maintains precision
/// as long as the numerator and denominator fit in the available bits.
#[derive(Debug, Clone)]
pub struct ProbabilisticFraction {
    numerator: BigNatural,
    denominator: BigNatural,
}

thread_local! {
    static BUFFER1: RefCell<BigNatural> = RefCell::new(BigNatural::new());
    static BUFFER2: RefCell<BigNatural> = RefCell::new(BigNatural::new());
    static BUFFER3: RefCell<BigNatural> = RefCell::new(BigNatural::new());
}

impl ProbabilisticFraction {
    /// Creates a new fraction from numerator and denominator.
    ///
    /// # Preconditions
    /// * denominator must not be zero
    /// * numerator must not exceed denominator
    ///
    /// # Postconditions
    /// * fraction is normalized (no common factors)
    pub fn new(mut numerator: BigNatural, mut denominator: BigNatural) -> Self {
        assert!(!denominator.is_zero(), "Denominator must not be zero");
        assert!(numerator <= denominator, "Numerator must not exceed denominator");

        // Remove common factors
        Self::remove_common_factors(&mut numerator, &mut denominator);

        let result = Self { numerator, denominator };
        debug_assert!(result.is_normalized(), "Fraction must be normalized");
        result
    }

    /// Creates a fraction from string representations of numerator and denominator.
    pub fn from_str_pair(num: &str, den: &str) -> Result<Self, String> {
        let numerator = BigNatural::from_str(num)?;
        let denominator = BigNatural::from_str(den)?;
        if numerator > denominator {
            return Err("Numerator must not exceed denominator".into());
        }
        Ok(Self { numerator, denominator })
    }

    /// Returns the constant zero (0/1).
    pub fn zero() -> Self {
        static ZERO: once_cell::sync::Lazy<ProbabilisticFraction> = once_cell::sync::Lazy::new(|| {
            ProbabilisticFraction::new(BigNatural::from_usize(0), BigNatural::from_usize(1))
        });
        ZERO.clone()
    }

    /// Returns the constant one (1/1).
    pub fn one() -> Self {
        static ONE: once_cell::sync::Lazy<ProbabilisticFraction> = once_cell::sync::Lazy::new(|| {
            ProbabilisticFraction::new(BigNatural::from_usize(1), BigNatural::from_usize(1))
        });
        ONE.clone()
    }

    /// Returns the numerator of the fraction.
    pub fn numerator(&self) -> &BigNatural {
        &self.numerator
    }

    /// Returns the denominator of the fraction.
    pub fn denominator(&self) -> &BigNatural {
        &self.denominator
    }

    /// Removes common factors from numerator and denominator.
    fn reduce(&mut self) {
        let gcd = Self::greatest_common_divisor(self.numerator.clone(), self.denominator.clone());
        if !gcd.is_number(1) {
            self.numerator = &self.numerator / &gcd;
            self.denominator = &self.denominator / &gcd;
        }
    }

    /// Calculates the greatest common divisor of two numbers.
    fn greatest_common_divisor(mut a: BigNatural, mut b: BigNatural) -> BigNatural {
        while !b.is_zero() {
            let t = b.clone();
            b = &a % &b;
            a = t;
        }
        a
    }

    // Add new helper methods for fraction reduction
    fn remove_common_factors(num: &mut BigNatural, den: &mut BigNatural) {
        BUFFER1.with_borrow(|b1| {
            BUFFER2.with_borrow(|b2| {
                BUFFER3.with_borrow(|b3| {
                    let gcd = Self::greatest_common_divisor(num.clone(), den.clone());
                    if !gcd.is_number(1) {
                        let mut quotient = b1.clone();
                        let mut remainder = b2.clone();

                        num.div_mod(&gcd, &mut quotient, &mut remainder);
                        *num = quotient.clone();

                        den.div_mod(&gcd, &mut quotient, &mut remainder);
                        *den = quotient;
                    }
                })
            })
        })
    }

    /// Checks if the fraction is properly normalized (no common factors).
    fn is_normalized(&self) -> bool {
        Self::greatest_common_divisor(self.numerator.clone(), self.denominator.clone()).is_number(1)
    }
}

impl PartialEq for ProbabilisticFraction {
    fn eq(&self, other: &Self) -> bool {
        BUFFER1.with_borrow_mut(|b1| {
            BUFFER2.with_borrow_mut(|b2| {
                // self.num * other.den == other.num * self.den
                let mut left = b1.clone();
                let mut right = b2.clone();

                left = &self.numerator * &other.denominator;
                right = &other.numerator * &self.denominator;

                left == right
            })
        })
    }
}

impl Eq for ProbabilisticFraction {}

impl PartialOrd for ProbabilisticFraction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProbabilisticFraction {
    fn cmp(&self, other: &Self) -> Ordering {
        BUFFER1.with_borrow(|b1| {
            BUFFER2.with_borrow(|b2| {
                BUFFER3.with_borrow(|b3| {
                    // Compare self.num * other.den <=> other.num * self.den
                    let mut left = b1.clone();
                    let mut right = b2.clone();

                    left = &self.numerator * &other.denominator;
                    right = &other.numerator * &self.denominator;

                    left.cmp(&right)
                })
            })
        })
    }
}

impl Add for &ProbabilisticFraction {
    type Output = ProbabilisticFraction;

    fn add(self, other: &ProbabilisticFraction) -> ProbabilisticFraction {
        debug_assert!(self.is_normalized(), "Left operand must be normalized");
        debug_assert!(other.is_normalized(), "Right operand must be normalized");

        BUFFER1.with_borrow(|b1| {
            BUFFER2.with_borrow(|b2| {
                BUFFER3.with_borrow(|b3| {
                    let mut num = b1.clone();
                    let mut den = b2.clone();

                    // num = self.num * other.den + other.num * self.den
                    // den = self.den * other.den
                    num = &self.numerator * &other.denominator;
                    let mut temp = b3.clone();
                    temp = &other.numerator * &self.denominator;
                    num.add(&temp);

                    den = &self.denominator * &other.denominator;

                    let mut result = ProbabilisticFraction::new(num, den);
                    result.reduce();
                    debug_assert!(result.is_normalized(), "Result must be normalized");
                    result
                })
            })
        })
    }
}

impl Sub for &ProbabilisticFraction {
    type Output = ProbabilisticFraction;

    fn sub(self, other: &ProbabilisticFraction) -> ProbabilisticFraction {
        debug_assert!(self.is_normalized(), "Left operand must be normalized");
        debug_assert!(other.is_normalized(), "Right operand must be normalized");

        // Assert result will be non-negative
        assert!(self >= other, "Subtraction would result in negative fraction");

        BUFFER1.with_borrow(|b1| {
            BUFFER2.with_borrow(|b2| {
                BUFFER3.with_borrow(|b3| {
                    let mut num = b1.clone();
                    let mut den = b2.clone();

                    // num = self.num * other.den - other.num * self.den
                    // den = self.den * other.den
                    num = &self.numerator * &other.denominator;
                    let mut temp = b3.clone();
                    temp = &other.numerator * &self.denominator;
                    num.subtract(&temp);

                    den = &self.denominator * &other.denominator;

                    let mut result = ProbabilisticFraction::new(num, den);
                    result.reduce();
                    debug_assert!(result.is_normalized(), "Result must be normalized");
                    result
                })
            })
        })
    }
}

impl Mul for &ProbabilisticFraction {
    type Output = ProbabilisticFraction;

    fn mul(self, other: &ProbabilisticFraction) -> ProbabilisticFraction {
        debug_assert!(self.is_normalized(), "Left operand must be normalized");
        debug_assert!(other.is_normalized(), "Right operand must be normalized");

        BUFFER1.with_borrow(|b1| {
            BUFFER2.with_borrow(|b2| {
                BUFFER3.with_borrow(|b3| {
                    let mut num = b1.clone();
                    let mut den = b2.clone();

                    // num = self.num * other.num
                    // den = self.den * other.den
                    num = &self.numerator * &other.numerator;
                    den = &self.denominator * &other.denominator;

                    let mut result = ProbabilisticFraction::new(num, den);
                    result.reduce();
                    debug_assert!(result.is_normalized(), "Result must be normalized");
                    result
                })
            })
        })
    }
}

impl Div for &ProbabilisticFraction {
    type Output = ProbabilisticFraction;

    fn div(self, other: &ProbabilisticFraction) -> ProbabilisticFraction {
        debug_assert!(self.is_normalized(), "Left operand must be normalized");
        debug_assert!(other.is_normalized(), "Right operand must be normalized");
        assert!(!other.numerator.is_zero(), "Division by zero");

        BUFFER1.with_borrow(|b1| {
            BUFFER2.with_borrow(|b2| {
                BUFFER3.with_borrow(|b3| {
                    let mut num = b1.clone();
                    let mut den = b2.clone();

                    // num = self.num * other.den
                    // den = self.den * other.num
                    num = &self.numerator * &other.denominator;
                    den = &self.denominator * &other.numerator;

                    let mut result = ProbabilisticFraction::new(num, den);
                    result.reduce();
                    debug_assert!(result.is_normalized(), "Result must be normalized");
                    result
                })
            })
        })
    }
}

impl fmt::Display for ProbabilisticFraction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl Hash for ProbabilisticFraction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.numerator.hash(state);
        self.denominator.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let half = ProbabilisticFraction::from_str_pair("1", "2").unwrap();
        let quarter = ProbabilisticFraction::from_str_pair("1", "4").unwrap();

        assert!(quarter < half);
        assert_eq!(
            &half + &quarter,
            ProbabilisticFraction::from_str_pair("3", "4").unwrap()
        );
        assert_eq!(
            &half - &quarter,
            ProbabilisticFraction::from_str_pair("1", "4").unwrap()
        );
    }

    #[test]
    fn test_reduction() {
        let frac = ProbabilisticFraction::from_str_pair("2", "4").unwrap();
        assert_eq!(frac.to_string(), "1/2");
    }

    #[test]
    #[should_panic(expected = "Numerator must not exceed denominator")]
    fn test_invalid_fraction() {
        ProbabilisticFraction::from_str_pair("5", "3").unwrap();
    }

    #[test]
    fn test_arithmetic() {
        let a = ProbabilisticFraction::from_str_pair("1", "2").unwrap();
        let b = ProbabilisticFraction::from_str_pair("1", "3").unwrap();

        // Test addition
        let sum = &a + &b;
        assert_eq!(sum.to_string(), "5/6");

        // Test subtraction
        let diff = &a - &b;
        assert_eq!(diff.to_string(), "1/6");

        // Test multiplication
        let prod = &a * &b;
        assert_eq!(prod.to_string(), "1/6");

        // Test division
        let quot = &a / &b;
        assert_eq!(quot.to_string(), "3/2");
    }

    #[test]
    fn test_reduction2() {
        let mut frac = ProbabilisticFraction::from_str_pair("2", "4").unwrap();
        assert_eq!(frac.to_string(), "1/2");

        frac = ProbabilisticFraction::from_str_pair("15", "25").unwrap();
        assert_eq!(frac.to_string(), "3/5");
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_division_by_zero() {
        let a = ProbabilisticFraction::from_str_pair("1", "2").unwrap();
        let zero = ProbabilisticFraction::zero();
        let _ = &a / &zero;
    }

    #[test]
    fn test_normalization() {
        let frac = ProbabilisticFraction::from_str_pair("6", "8").unwrap();
        assert!(frac.is_normalized());
        assert_eq!(frac.to_string(), "3/4");
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn test_zero_denominator() {
        ProbabilisticFraction::new(BigNatural::from_usize(1), BigNatural::from_usize(0));
    }
}
