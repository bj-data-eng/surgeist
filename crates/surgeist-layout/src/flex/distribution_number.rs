//! Flex distribution arithmetic with a wider binary exponent.
//!
//! Ordinary operations retain the scalar lane's rounding. Intermediate
//! arithmetic can carry a wider exponent until a target is frozen. Each round
//! starts from scalar extents and factors, and shrink weights multiply two
//! inputs before the line sums them.

use crate::LayoutScalar;
use core::cmp::Ordering;
use core::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy)]
pub(super) struct DistributionNumber<S> {
    value: S,
    exponent: i32,
}

impl<S: LayoutScalar> DistributionNumber<S> {
    pub(super) const ZERO: Self = Self::scalar(S::ZERO);

    const fn scalar(value: S) -> Self {
        Self { value, exponent: 0 }
    }

    pub(super) fn finite(value: S) -> Result<Self, S> {
        if value.is_finite() {
            Ok(Self::scalar(value))
        } else {
            Err(value)
        }
    }

    pub(super) fn sign(self) -> Ordering {
        self.value
            .partial_cmp(&S::ZERO)
            .expect("a distribution significand is finite")
    }

    pub(super) fn abs(self) -> Self {
        Self {
            value: self.value.abs(),
            ..self
        }
    }

    pub(super) fn magnitude_less(self, other: Self) -> bool {
        if self.exponent == 0 && other.exponent == 0 {
            return self.value.abs() < other.value.abs();
        }
        if self.value == S::ZERO {
            return other.value != S::ZERO;
        }
        if other.value == S::ZERO {
            return false;
        }
        let left = self.normalized();
        let right = other.normalized();
        left.exponent < right.exponent
            || left.exponent == right.exponent && left.value.abs() < right.value.abs()
    }

    pub(super) fn min(self, other: Self) -> Self {
        if other.less_than(self) { other } else { self }
    }

    pub(super) fn max(self, other: Self) -> Self {
        if self.less_than(other) { other } else { self }
    }

    fn less_than(self, other: Self) -> bool {
        if self.exponent == 0 && other.exponent == 0 {
            return self.value < other.value;
        }
        let left_sign = self.sign();
        let right_sign = other.sign();
        if left_sign != right_sign {
            return left_sign < right_sign;
        }
        match left_sign {
            Ordering::Less => other.magnitude_less(self),
            Ordering::Equal => false,
            Ordering::Greater => self.magnitude_less(other),
        }
    }

    pub(super) fn narrow(self) -> S {
        if self.exponent == 0 || self.value == S::ZERO {
            return self.value;
        }
        let value = self.value.to_f64();
        let narrowed = match self.exponent {
            exponent if exponent > 1023 => f64::INFINITY.copysign(value),
            exponent if exponent < -1075 => 0.0_f64.copysign(value),
            exponent if exponent < -1022 => {
                // Scale in the normal range before rounding to a subnormal.
                (value * power_of_two(exponent + 1074)) * f64::from_bits(1)
            }
            exponent => value * power_of_two(exponent),
        };
        S::from_f64(narrowed)
    }

    fn normalized(self) -> Self {
        if self.value == S::ZERO {
            return Self::scalar(self.value);
        }
        let value = self.value.to_f64();
        let bits = value.to_bits();
        let sign = bits & (1_u64 << 63);
        let fraction = bits & ((1_u64 << 52) - 1);
        let encoded_exponent = ((bits >> 52) & 0x7ff) as i32;
        let (significand, exponent) = if encoded_exponent == 0 {
            let highest = 63 - fraction.leading_zeros() as i32;
            (
                ((fraction as f64) / power_of_two(highest)).copysign(value),
                highest - 1074,
            )
        } else {
            (
                f64::from_bits(sign | (1023_u64 << 52) | fraction),
                encoded_exponent - 1023,
            )
        };
        Self {
            value: S::from_f64(significand),
            exponent: self.exponent + exponent,
        }
    }
}

impl<S: LayoutScalar> Add for DistributionNumber<S> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.exponent == 0 && other.exponent == 0 {
            let value = self.value + other.value;
            if value.is_finite() {
                return Self::scalar(value);
            }
        }
        if self.value == S::ZERO {
            return other;
        }
        if other.value == S::ZERO {
            return self;
        }

        let left = self.normalized();
        let right = other.normalized();
        let exponent = left.exponent.max(right.exponent);
        let left = left.value * S::from_f64(power_of_two(left.exponent - exponent));
        let right = right.value * S::from_f64(power_of_two(right.exponent - exponent));
        Self {
            value: left + right,
            exponent,
        }
        .normalized()
    }
}

impl<S: LayoutScalar> Neg for DistributionNumber<S> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            value: -self.value,
            ..self
        }
    }
}

impl<S: LayoutScalar> Sub for DistributionNumber<S> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl<S: LayoutScalar> Mul for DistributionNumber<S> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        if self.exponent == 0 && other.exponent == 0 {
            let value = self.value * other.value;
            if normal_result(value) || self.value == S::ZERO || other.value == S::ZERO {
                return Self::scalar(value);
            }
        }

        let left = self.normalized();
        let right = other.normalized();
        Self {
            value: left.value * right.value,
            exponent: left.exponent + right.exponent,
        }
        .normalized()
    }
}

impl<S: LayoutScalar> Div for DistributionNumber<S> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        debug_assert!(other.value != S::ZERO);
        if self.exponent == 0 && other.exponent == 0 {
            let value = self.value / other.value;
            if normal_result(value) || self.value == S::ZERO {
                return Self::scalar(value);
            }
        }

        let left = self.normalized();
        let right = other.normalized();
        Self {
            value: left.value / right.value,
            exponent: left.exponent - right.exponent,
        }
        .normalized()
    }
}

fn normal_result<S: LayoutScalar>(value: S) -> bool {
    let minimum = if core::mem::size_of::<S>() == core::mem::size_of::<f32>() {
        S::from_f32(f32::MIN_POSITIVE)
    } else {
        S::from_f64(f64::MIN_POSITIVE)
    };
    value.is_finite() && value.abs() >= minimum
}

fn power_of_two(exponent: i32) -> f64 {
    if exponent < -1074 {
        0.0
    } else if exponent < -1022 {
        f64::from_bits(1_u64 << (exponent + 1074))
    } else {
        f64::from_bits(((exponent + 1023) as u64) << 52)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number<S: LayoutScalar>(value: S) -> DistributionNumber<S> {
        DistributionNumber::finite(value).unwrap()
    }

    fn assert_recovers_tiny_share<S: LayoutScalar>(largest_exponent: i32, precision: i32) {
        let large = S::from_f64(power_of_two(largest_exponent));
        let tiny = S::from_f64(power_of_two(-precision));
        let total = number(large) + number(tiny);
        assert_eq!((number(tiny) / total).narrow(), S::ZERO);
        assert_eq!((number(large) * number(tiny) / total).narrow(), tiny);
    }

    #[test]
    fn small_ratio_keeps_its_representable_share() {
        assert_recovers_tiny_share::<f32>(127, 24);
        assert_recovers_tiny_share::<f64>(1023, 52);
    }

    fn assert_cancellation_recovers_finite_geometry<S: LayoutScalar>(largest: S) {
        let extent = number(largest);
        let sum = extent + extent;
        assert!(!sum.narrow().is_finite());
        assert_eq!((sum - extent).narrow(), largest);
        assert_eq!((extent - sum).narrow(), -largest);
    }

    #[test]
    fn finite_space_survives_an_occupied_sum_outside_the_scalar_range() {
        assert_cancellation_recovers_finite_geometry(f32::MAX);
        assert_cancellation_recovers_finite_geometry(f64::MAX);
    }
}
