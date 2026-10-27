//! Proof types for numerical operands and the shared closest-finite rule.
//!
//! NaN and both infinities are rejected when a proof is built, so every
//! operation over these operands is total. A finite result that would leave
//! binary64 becomes the closest supported value (CSS Values 4 §10.12 and the
//! numeric-types range rule; CSS Color 4 conversion). This is a numeric-range
//! rule only; it never maps a colour into a gamut.

/// A finite binary64 value.
///
/// ```
/// use surgeist_color::Finite;
/// assert_eq!(Finite::new(-0.0).map(Finite::get), Some(-0.0));
/// assert_eq!(Finite::new(f64::MAX).map(Finite::get), Some(f64::MAX));
/// assert!(Finite::new(f64::INFINITY).is_none());
/// assert!(Finite::new(f64::NAN).is_none());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Finite(f64);

impl Finite {
    /// Proves `value` finite; `None` for NaN and both infinities.
    #[must_use]
    pub const fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Self(value))
        } else {
            None
        }
    }

    /// The proven value, with its exact bits including a signed zero.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// A finite value in `[0, 1]`, such as alpha or interpolation progress.
///
/// Negative zero lies in the interval and keeps its bits.
///
/// ```
/// use surgeist_color::{Finite, UnitInterval};
/// assert_eq!(UnitInterval::new(0.25).map(UnitInterval::get), Some(0.25));
/// assert!(UnitInterval::new(1.5).is_none());
/// assert!(UnitInterval::new(f64::NAN).is_none());
/// let clamped = UnitInterval::clamped(Finite::new(-3.0).unwrap());
/// assert_eq!(clamped.get(), 0.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct UnitInterval(f64);

impl UnitInterval {
    /// Proves `value` within `[0, 1]`; `None` outside it or when not finite.
    #[must_use]
    pub const fn new(value: f64) -> Option<Self> {
        if value >= 0.0 && value <= 1.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// The nearest value in `[0, 1]`; an in-range value keeps its bits.
    #[must_use]
    pub fn clamped(value: Finite) -> Self {
        Self(value.get().clamp(0.0, 1.0))
    }

    /// The proven value, with its exact bits including a signed zero.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// The signed closest finite value of a kernel result.
///
/// Kernels evaluate finite operands, so an infinity is an overflow of a finite
/// exact value and becomes `f64::MAX` with its sign. They never combine
/// opposite infinities or an infinity with zero, so NaN cannot arise.
pub(crate) fn closest_finite(value: f64) -> Finite {
    if value.is_infinite() {
        Finite(f64::MAX.copysign(value))
    } else {
        Finite::new(value).expect("finite kernel operands never produce NaN")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_finite_keeps_finite_bits_and_saturates_signed_overflow() {
        for value in [0.0, -0.0, 5e-324, -f64::MAX, f64::MAX, 1.5] {
            assert_eq!(closest_finite(value).get().to_bits(), value.to_bits());
        }
        assert_eq!(closest_finite(f64::INFINITY).get(), f64::MAX);
        assert_eq!(closest_finite(f64::NEG_INFINITY).get(), -f64::MAX);
    }

    #[test]
    fn unit_interval_admits_both_zeros_and_one_only() {
        assert_eq!(
            UnitInterval::new(-0.0).unwrap().get().to_bits(),
            (-0.0_f64).to_bits()
        );
        assert_eq!(UnitInterval::new(1.0).unwrap().get(), 1.0);
        for value in [
            -5e-324,
            1.0 + f64::EPSILON,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert_eq!(UnitInterval::new(value), None, "{value}");
        }
        assert_eq!(UnitInterval::clamped(Finite::new(2.0).unwrap()).get(), 1.0);
        assert_eq!(
            UnitInterval::clamped(Finite::new(0.5).unwrap())
                .get()
                .to_bits(),
            0.5_f64.to_bits()
        );
    }
}
