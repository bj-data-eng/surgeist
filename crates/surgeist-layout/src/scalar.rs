use core::fmt::Debug;
use core::ops::{Add, Div, Mul, Neg, Sub};
use surgeist_text::inline_box::InlineBoxEdgeComponents;

/// Numeric precision contract for one layout computation.
pub trait LayoutScalar:
    private::Sealed
    + Copy
    + Clone
    + Debug
    + Default
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + 'static
{
    const ZERO: Self;
    const ONE: Self;
    const INFINITY: Self;
    const NAN: Self;
    const EPSILON: Self;

    fn from_f32(value: f32) -> Self;
    fn from_f64(value: f64) -> Self;
    fn from_usize(value: usize) -> Self;
    fn abs(self) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn is_finite(self) -> bool;
    fn floor_to_usize_saturating(self) -> usize;
    fn to_f64(self) -> f64;
}

/// Failure of one required finite geometry contribution, before scalar narrowing.
pub(crate) struct GeometryAddError {
    pub(crate) value: f64,
    pub(crate) reason: crate::LayoutTextNumericError,
}
pub(crate) fn checked_geometry_add(base: f64, delta: f64) -> Result<f64, GeometryAddError> {
    let result = base + delta;
    if !result.is_finite() {
        return Err(GeometryAddError {
            value: result,
            reason: crate::LayoutTextNumericError::NonFinite,
        });
    }
    if delta != 0.0 && result == base {
        return Err(GeometryAddError {
            value: delta,
            reason: crate::LayoutTextNumericError::NonZeroLost,
        });
    }
    Ok(result)
}

pub(crate) fn canonical_zero<S: LayoutScalar>(value: S) -> S {
    if value == S::ZERO { S::ZERO } else { value }
}

#[inline]
pub(crate) fn round_layout_coordinate<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

macro_rules! impl_layout_scalar {
    ($ty:ty, $bits:ty, $edge_components:ident) => {
        impl private::Sealed for $ty {
            const MAX_NONNEGATIVE_FINITE_RANK: u64 = <$ty>::MAX.to_bits() as u64;
            const FRACTION_BITS: usize = <$ty>::MANTISSA_DIGITS as usize - 1;
            const SMALLEST_SUBNORMAL_EXPONENT: usize =
                (<$ty>::MANTISSA_DIGITS as i32 - <$ty>::MIN_EXP) as usize;

            fn nonnegative_finite_rank(self) -> u64 {
                if self == 0.0 {
                    0
                } else {
                    u64::from(self.to_bits())
                }
            }

            fn from_nonnegative_finite_rank(rank: u64) -> Self {
                // Every rank is bounded by MAX_NONNEGATIVE_FINITE_RANK.
                <$ty>::from_bits(<$bits>::try_from(rank).expect("bounded finite rank"))
            }

            fn inline_box_edge_components(
                margin: Self,
                border: Self,
                padding: Self,
            ) -> surgeist_text::Result<InlineBoxEdgeComponents> {
                InlineBoxEdgeComponents::$edge_components(margin, border, padding)
            }
        }

        impl LayoutScalar for $ty {
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const INFINITY: Self = <$ty>::INFINITY;
            const NAN: Self = <$ty>::NAN;
            const EPSILON: Self = <$ty>::EPSILON;

            fn from_f32(value: f32) -> Self {
                value as Self
            }

            fn from_f64(value: f64) -> Self {
                value as Self
            }

            fn from_usize(value: usize) -> Self {
                value as Self
            }

            fn abs(self) -> Self {
                <$ty>::abs(self)
            }

            fn min(self, other: Self) -> Self {
                <$ty>::min(self, other)
            }

            fn max(self, other: Self) -> Self {
                <$ty>::max(self, other)
            }

            fn floor(self) -> Self {
                <$ty>::floor(self)
            }

            fn ceil(self) -> Self {
                <$ty>::ceil(self)
            }

            fn round(self) -> Self {
                <$ty>::round(self)
            }

            fn is_finite(self) -> bool {
                <$ty>::is_finite(self)
            }

            fn floor_to_usize_saturating(self) -> usize {
                if <$ty>::is_nan(self) || self <= 0.0 {
                    0
                } else if !<$ty>::is_finite(self) || self >= usize::MAX as Self {
                    usize::MAX
                } else {
                    <$ty>::floor(self) as usize
                }
            }

            fn to_f64(self) -> f64 {
                self as f64
            }
        }
    };
}

impl_layout_scalar!(f32, u32, from_f32);
impl_layout_scalar!(f64, u64, from_f64);

mod private {
    use surgeist_text::inline_box::InlineBoxEdgeComponents;

    /// Per-lane numeric facts that private layout algorithms need. Only `f32`
    /// and `f64` implement it.
    pub trait Sealed {
        /// The largest rank of a finite non-negative value of this lane.
        const MAX_NONNEGATIVE_FINITE_RANK: u64;
        /// Explicit significand bits of the lane's binary format.
        const FRACTION_BITS: usize;
        /// Magnitude of the binary exponent of the smallest subnormal value.
        const SMALLEST_SUBNORMAL_EXPONENT: usize;

        /// The rank of a finite non-negative value: its bit pattern, which is
        /// ordered like the value, with zero of either sign at rank zero.
        fn nonnegative_finite_rank(self) -> u64;
        /// The value of a rank not above `MAX_NONNEGATIVE_FINITE_RANK`.
        fn from_nonnegative_finite_rank(rank: u64) -> Self;
        /// The text crate's inline-box edge components for three lane values,
        /// converted without narrowing.
        fn inline_box_edge_components(
            margin: Self,
            border: Self,
            padding: Self,
        ) -> surgeist_text::Result<InlineBoxEdgeComponents>;
    }
}

#[cfg(test)]
mod tests {
    use super::private::Sealed;
    use super::*;

    struct LaneFacts {
        max_rank: u64,
        fraction_bits: usize,
        smallest_subnormal_exponent: usize,
        one_rank: u64,
    }

    // The expectations are the IEEE 754 binary32 and binary64 formats.
    fn assert_rank_facts<S: LayoutScalar>(expected: LaneFacts) {
        assert_eq!(S::MAX_NONNEGATIVE_FINITE_RANK, expected.max_rank);
        assert_eq!(S::FRACTION_BITS, expected.fraction_bits);
        assert_eq!(
            S::SMALLEST_SUBNORMAL_EXPONENT,
            expected.smallest_subnormal_exponent
        );

        assert_eq!(S::ZERO.nonnegative_finite_rank(), 0);
        assert_eq!((-S::ZERO).nonnegative_finite_rank(), 0);
        assert_eq!(S::from_nonnegative_finite_rank(0), S::ZERO);

        let smallest = S::from_nonnegative_finite_rank(1);
        // Two factors keep each power of two representable in binary64.
        let exponent = expected.smallest_subnormal_exponent as i32;
        assert_eq!(
            smallest.to_f64(),
            2f64.powi(-(exponent / 2)) * 2f64.powi(-(exponent - exponent / 2))
        );
        assert_eq!(smallest.nonnegative_finite_rank(), 1);

        assert_eq!(S::ONE.nonnegative_finite_rank(), expected.one_rank);
        assert_eq!(S::from_nonnegative_finite_rank(expected.one_rank), S::ONE);

        let largest = S::from_nonnegative_finite_rank(expected.max_rank);
        assert!(largest.is_finite());
        assert_eq!(largest.nonnegative_finite_rank(), expected.max_rank);
        assert!(!S::from_nonnegative_finite_rank(expected.max_rank + 1).is_finite());
    }

    #[test]
    fn f32_rank_items_follow_the_binary32_format() {
        assert_rank_facts::<f32>(LaneFacts {
            max_rank: 0x7f7f_ffff,
            fraction_bits: 23,
            smallest_subnormal_exponent: 149,
            one_rank: 0x3f80_0000,
        });
    }

    #[test]
    fn f64_rank_items_follow_the_binary64_format() {
        assert_rank_facts::<f64>(LaneFacts {
            max_rank: 0x7fef_ffff_ffff_ffff,
            fraction_bits: 52,
            smallest_subnormal_exponent: 1074,
            one_rank: 0x3ff0_0000_0000_0000,
        });
    }

    #[test]
    fn f32_edge_components_keep_the_f32_lane_without_narrowing_loss() {
        let subnormal = f32::from_bits(1);
        let actual = <f32 as Sealed>::inline_box_edge_components(1.5, 0.25, subnormal).unwrap();
        assert_eq!(
            actual,
            InlineBoxEdgeComponents::from_f32(1.5, 0.25, subnormal).unwrap()
        );
        assert_ne!(
            actual,
            InlineBoxEdgeComponents::from_f64(1.5, 0.25, f64::from(subnormal)).unwrap()
        );
        assert!(<f32 as Sealed>::inline_box_edge_components(f32::NAN, 0.0, 0.0).is_err());
        assert!(<f32 as Sealed>::inline_box_edge_components(0.0, -1.0, 0.0).is_err());
    }

    #[test]
    fn f64_edge_components_keep_the_f64_lane() {
        let subnormal = f64::from_bits(1);
        let actual = <f64 as Sealed>::inline_box_edge_components(1.5, 0.25, subnormal).unwrap();
        assert_eq!(
            actual,
            InlineBoxEdgeComponents::from_f64(1.5, 0.25, subnormal).unwrap()
        );
        assert_ne!(
            actual,
            InlineBoxEdgeComponents::from_f32(1.5, 0.25, 0.0).unwrap()
        );
        assert!(<f64 as Sealed>::inline_box_edge_components(f64::INFINITY, 0.0, 0.0).is_err());
        assert!(<f64 as Sealed>::inline_box_edge_components(0.0, 0.0, -1.0).is_err());
    }
}
