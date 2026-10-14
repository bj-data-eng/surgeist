//! Binary64 significands with a private, wider exponent for the fixed color paths.
//!
//! This extends range, not precision. Each elementary operation rounds its
//! significand normally; cancellation can therefore discard small contributions.
//! No intermediate is narrowed to a complete binary64 color coordinate.
//!
//! Exponent bound for these fixed, nonrecursive paths: finite inputs start in
//! [-1074, 1023]. Every nonzero fixed coefficient has magnitude between 2^-14
//! and 2^10. A coefficient operation changes the normalized exponent by at most
//! 15; adding two normalized binary64 significands changes the larger exponent
//! by at most 53 on cancellation (or one upward). There are fewer than 50 such
//! operations along any coordinate path. Only the source's power can increase
//! exponent magnitude, by at most three; destination powers have ratio at most
//! one. Even charging 50 operations both before and after the source power gives
//! 3*(1074 + 50*64) + 50*64 + 4 < 20000. Therefore the largest rational-power
//! integer product is below 20000*563, far inside i32. No external input needs
//! an exponent assertion, saturation, or an intermediate-range error.

#[derive(Clone, Copy, Debug)]
pub(super) struct Wide {
    significand: f64,
    exponent: i32,
}

impl Wide {
    pub(super) fn from_f64(value: f64) -> Self {
        if value == 0.0 {
            return Self {
                significand: value,
                exponent: 0,
            };
        }
        let bits = value.to_bits();
        let sign = bits & (1_u64 << 63);
        let fraction = bits & ((1_u64 << 52) - 1);
        let encoded_exponent = ((bits >> 52) & 0x7ff) as i32;
        if encoded_exponent == 0 {
            let highest_bit = 63 - fraction.leading_zeros() as i32;
            Self {
                significand: ((fraction as f64) / power_of_two(highest_bit)).copysign(value),
                exponent: highest_bit - 1074,
            }
        } else {
            Self {
                significand: f64::from_bits(sign | (1023_u64 << 52) | fraction),
                exponent: encoded_exponent - 1023,
            }
        }
    }

    fn normalized(significand: f64, exponent: i32) -> Self {
        let mut result = Self::from_f64(significand);
        if result.significand != 0.0 {
            result.exponent += exponent;
        }
        result
    }

    pub(super) fn abs(self) -> Self {
        Self {
            significand: self.significand.abs(),
            ..self
        }
    }

    pub(super) fn negated(self) -> Self {
        Self {
            significand: -self.significand,
            ..self
        }
    }

    pub(super) fn add(self, other: Self) -> Self {
        if self.significand == 0.0 {
            if other.significand == 0.0 {
                return Self::from_f64(self.significand + other.significand);
            }
            return other;
        }
        if other.significand == 0.0 {
            return self;
        }
        let exponent = self.exponent.max(other.exponent);
        Self::normalized(
            self.significand * power_of_two(self.exponent - exponent)
                + other.significand * power_of_two(other.exponent - exponent),
            exponent,
        )
    }

    pub(super) fn add_fixed(self, value: f64) -> Self {
        self.add(Self::from_f64(value))
    }

    pub(super) fn multiply_fixed(self, value: f64) -> Self {
        let other = Self::from_f64(value);
        Self::normalized(
            self.significand * other.significand,
            self.exponent + other.exponent,
        )
    }

    pub(super) fn divide_fixed(self, value: f64) -> Self {
        let other = Self::from_f64(value);
        Self::normalized(
            self.significand / other.significand,
            self.exponent - other.exponent,
        )
    }

    /// Only the fixed positive rational exponents of these color formulas enter.
    pub(super) fn signed_power(self, numerator: i32, denominator: i32) -> Self {
        if self.significand == 0.0 {
            return self;
        }
        let scaled_exponent = self.exponent * numerator;
        let exponent = scaled_exponent.div_euclid(denominator);
        let remainder = scaled_exponent.rem_euclid(denominator);
        let magnitude = if numerator == 1 && denominator == 3 {
            self.significand.abs().cbrt()
        } else if numerator == 3 && denominator == 1 {
            let magnitude = self.significand.abs();
            magnitude * magnitude * magnitude
        } else {
            self.significand
                .abs()
                .powf(f64::from(numerator) / f64::from(denominator))
        };
        Self::normalized(
            (magnitude * (f64::from(remainder) / f64::from(denominator)).exp2())
                .copysign(self.significand),
            exponent,
        )
    }

    /// All thresholds are nonnegative finite constants.
    pub(super) fn greater_than(self, threshold: f64) -> bool {
        if self.significand <= 0.0 {
            return false;
        }
        if threshold == 0.0 {
            return true;
        }
        let other = Self::from_f64(threshold);
        self.exponent > other.exponent
            || (self.exponent == other.exponent && self.significand > other.significand)
    }

    /// Largest terms first lets an exact large cancellation precede tiny terms.
    pub(super) fn magnitude_order(self, other: Self) -> std::cmp::Ordering {
        let key = |value: Self| {
            if value.significand == 0.0 {
                i32::MIN
            } else {
                value.exponent
            }
        };
        key(self)
            .cmp(&key(other))
            .then_with(|| self.significand.abs().total_cmp(&other.significand.abs()))
    }

    pub(super) fn to_f64(self) -> Option<f64> {
        let value = if self.significand == 0.0 {
            self.significand
        } else if self.exponent > 1023 {
            return None;
        } else if self.exponent < -1075 {
            0.0_f64.copysign(self.significand)
        } else if self.exponent < -1022 {
            // The first multiplication is an exact normal power-of-two scale;
            // the second performs the final subnormal rounding, including ties.
            (self.significand * power_of_two(self.exponent + 1074)) * f64::from_bits(1)
        } else {
            self.significand * power_of_two(self.exponent)
        };
        value.is_finite().then_some(value)
    }
}

fn power_of_two(exponent: i32) -> f64 {
    if exponent < -1074 {
        0.0
    } else if exponent < -1022 {
        f64::from_bits(1_u64 << (exponent + 1074))
    } else {
        // Callers only scale bounded significands or align toward zero. The
        // largest requested normal exponent is 1023 in final conversion.
        f64::from_bits(((exponent + 1023) as u64) << 52)
    }
}

#[cfg(test)]
mod tests {
    use super::Wide;

    #[test]
    fn finite_decomposition_preserves_bits_including_extremes() {
        for value in [
            0.0,
            -0.0,
            f64::from_bits(1),
            -f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::from_bits(0x000f_ffff_ffff_ffff),
            f64::MAX,
            -f64::MAX,
            1.0,
            -3.5,
        ] {
            assert_eq!(
                Wide::from_f64(value).to_f64().unwrap().to_bits(),
                value.to_bits()
            );
        }
    }

    #[test]
    fn rational_powers_survive_exponents_outside_binary64() {
        // Powers of two give independent exact exponent expectations.
        let huge = Wide::normalized(1.0, 1080);
        assert_eq!(huge.signed_power(1, 3).to_f64(), Some(2.0_f64.powi(360)));
        assert_eq!(
            Wide::normalized(-1.0, -1080).signed_power(1, 3).to_f64(),
            Some(-2.0_f64.powi(-360))
        );
        let encoded = Wide::normalized(1.0, 1000);
        assert_eq!(
            encoded.signed_power(12, 5).signed_power(5, 12).to_f64(),
            Some(2.0_f64.powi(1000))
        );
        let tiny = Wide::normalized(1.0, -2000);
        assert_eq!(tiny.signed_power(1, 3).exponent, -667);
        assert!((tiny.signed_power(1, 3).significand - 2.0_f64.powf(1.0 / 3.0)).abs() < 1e-15);
    }

    #[test]
    fn final_subnormal_rounding_uses_even_ties_and_retains_sign() {
        assert_eq!(Wide::normalized(1.0, -1075).to_f64().unwrap().to_bits(), 0);
        assert_eq!(
            Wide::normalized(-1.0, -1075).to_f64().unwrap().to_bits(),
            (-0.0_f64).to_bits()
        );
        assert_eq!(Wide::normalized(1.5, -1074).to_f64().unwrap().to_bits(), 2);
        assert_eq!(
            Wide::normalized(1.0 + f64::EPSILON, -1075)
                .to_f64()
                .unwrap()
                .to_bits(),
            1
        );
        assert_eq!(Wide::normalized(1.0, -1100).to_f64(), Some(0.0));
    }

    #[test]
    fn extended_exponents_cancel_before_final_overflow_check() {
        let huge = Wide::normalized(1.5, 3000);
        let tiny = Wide::from_f64(f64::from_bits(1));
        assert_eq!(
            huge.add(huge.negated()).add(tiny).to_f64(),
            Some(f64::from_bits(1))
        );
        assert_eq!(Wide::from_f64(f64::MAX).to_f64(), Some(f64::MAX));
        assert_eq!(Wide::normalized(1.0, 1024).to_f64(), None);
        assert_eq!(Wide::normalized(-1.0, 1024).to_f64(), None);
        let last_significand = 2.0 - f64::EPSILON;
        assert_eq!(
            Wide::normalized(last_significand, 1023).to_f64(),
            Some(f64::MAX)
        );
        assert_eq!(Wide::normalized(2.0, 1023).to_f64(), None);
        assert_eq!(
            Wide::from_f64(f64::MAX)
                .multiply_fixed(1.0 + f64::EPSILON)
                .to_f64(),
            None
        );
    }

    #[test]
    fn skewed_channels_keep_separate_exponents_and_tiny_threshold_signs() {
        let channels = [f64::MAX, f64::from_bits(1), -f64::from_bits(1)].map(Wide::from_f64);
        assert_eq!(
            channels.map(|v| v.to_f64().unwrap().to_bits()),
            [f64::MAX, f64::from_bits(1), -f64::from_bits(1)].map(f64::to_bits)
        );
        assert!(channels[1].greater_than(0.0));
        assert!(channels[2].negated().greater_than(0.0));
        assert!(!channels[2].greater_than(0.0));
        assert!(!channels[1].greater_than(1.0 / 512.0));
    }
}
