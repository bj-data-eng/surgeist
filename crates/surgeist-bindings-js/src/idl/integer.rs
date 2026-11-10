//! Engine-independent WebIDL `ConvertToInt`, after ECMAScript `ToNumber`.
//!
//! Follows `#js-integer-types` and `#js-integer-types-abstract-ops` in WebIDL
//! revision `0620cf171d1b97138847088748e363cae4005156`. All mathematical
//! operations in that algorithm are exact, including the 64-bit modulo.

/// IDL integer type selected by binding metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerKind {
    /// Signed 8-bit IDL byte.
    Byte,
    /// Unsigned 8-bit IDL octet.
    Octet,
    /// Signed 16-bit IDL short.
    Short,
    /// Unsigned 16-bit IDL unsigned short.
    UnsignedShort,
    /// Signed 32-bit IDL long.
    Long,
    /// Unsigned 32-bit IDL unsigned long.
    UnsignedLong,
    /// Signed 64-bit IDL long long.
    LongLong,
    /// Unsigned 64-bit IDL unsigned long long.
    UnsignedLongLong,
}

/// Conversion policy selected by the IDL extended attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum IntegerMode {
    /// Truncate toward zero and reduce modulo the destination's bit width.
    /// Nonfinite numbers become zero.
    Wrap,
    /// Reject nonfinite numbers, then truncate and reject out-of-range values.
    /// Both 64-bit types use the safe-integer upper bound of 2^53 - 1;
    /// the signed lower bound is its negative, and the unsigned lower bound is zero.
    EnforceRange,
    /// Clamp to the destination's range, then round to nearest with ties to even.
    /// NaN becomes zero. Both 64-bit types use the same safe-integer bounds
    /// as [`Self::EnforceRange`].
    Clamp,
}

/// Converted IDL value, retaining its declared integer type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerValue {
    Byte(i8),
    Octet(u8),
    Short(i16),
    UnsignedShort(u16),
    Long(i32),
    UnsignedLong(u32),
    LongLong(i64),
    UnsignedLongLong(u64),
}

/// `EnforceRange` rejected a nonfinite or out-of-range number.
///
/// The engine boundary maps this semantic failure to WebIDL's `TypeError`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IntegerOutOfRange;

impl IntegerKind {
    fn bounds(self) -> (f64, f64) {
        const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
        match self {
            Self::Byte => (-128.0, 127.0),
            Self::Octet => (0.0, 255.0),
            Self::Short => (-32_768.0, 32_767.0),
            Self::UnsignedShort => (0.0, 65_535.0),
            Self::Long => (-2_147_483_648.0, 2_147_483_647.0),
            Self::UnsignedLong => (0.0, 4_294_967_295.0),
            // ConvertToInt deliberately uses safe-integer bounds for 64 bits.
            Self::LongLong => (-MAX_SAFE_INTEGER, MAX_SAFE_INTEGER),
            Self::UnsignedLongLong => (0.0, MAX_SAFE_INTEGER),
        }
    }

    fn value_from_residue(self, residue: u64) -> IntegerValue {
        // Integer casts retain the low bits and interpret the sign bit for
        // signed destinations: precisely modulo 2^N, then signed adjustment.
        match self {
            Self::Byte => IntegerValue::Byte(residue as i8),
            Self::Octet => IntegerValue::Octet(residue as u8),
            Self::Short => IntegerValue::Short(residue as i16),
            Self::UnsignedShort => IntegerValue::UnsignedShort(residue as u16),
            Self::Long => IntegerValue::Long(residue as i32),
            Self::UnsignedLong => IntegerValue::UnsignedLong(residue as u32),
            Self::LongLong => IntegerValue::LongLong(residue as i64),
            Self::UnsignedLongLong => IntegerValue::UnsignedLongLong(residue),
        }
    }
}

/// Converts an already obtained ECMAScript Number without entering guest code.
pub(crate) fn convert(
    number: f64,
    kind: IntegerKind,
    mode: IntegerMode,
) -> Result<IntegerValue, IntegerOutOfRange> {
    let integer = match mode {
        IntegerMode::Wrap => number,
        IntegerMode::EnforceRange => {
            if !number.is_finite() {
                return Err(IntegerOutOfRange);
            }
            let integer = number.trunc();
            let (lower, upper) = kind.bounds();
            if integer < lower || integer > upper {
                return Err(IntegerOutOfRange);
            }
            integer
        }
        IntegerMode::Clamp => {
            if number.is_nan() {
                0.0
            } else {
                let (lower, upper) = kind.bounds();
                number.clamp(lower, upper).round_ties_even()
            }
        }
    };
    Ok(kind.value_from_residue(integer_modulo_64(integer)))
}

/// Exact `IntegerPart(number) modulo 2^64`, with nonfinite values mapped to zero.
fn integer_modulo_64(number: f64) -> u64 {
    let bits = number.to_bits();
    let exponent = (bits >> 52) & 0x7ff;
    // This also handles both zeros, subnormals, NaN, and both infinities.
    if exponent < 1023 || exponent == 0x7ff {
        return 0;
    }

    let significand = (bits & 0x000f_ffff_ffff_ffff) | 0x0010_0000_0000_0000;
    // A normal binary64 is significand * 2^(exponent - 1023 - 52).
    // Right shifting discards the fractional part. Left shifting discards
    // multiples of 2^64; shifts of 64 or more have zero residue.
    let shifted = if exponent <= 1075 {
        significand.checked_shr((1075 - exponent) as u32)
    } else {
        significand.checked_shl((exponent - 1075) as u32)
    };
    let magnitude = shifted.unwrap_or(0);
    if bits >> 63 == 0 {
        magnitude
    } else {
        // The sole wrapping arithmetic implements mathematical modulo 2^64.
        magnitude.wrapping_neg()
    }
}

#[cfg(test)]
mod tests {
    use super::convert;
    use super::{
        IntegerKind as Kind, IntegerMode as Mode, IntegerOutOfRange, IntegerValue as Value,
    };

    const ZERO_VALUES: [(Kind, Value); 8] = [
        (Kind::Byte, Value::Byte(0)),
        (Kind::Octet, Value::Octet(0)),
        (Kind::Short, Value::Short(0)),
        (Kind::UnsignedShort, Value::UnsignedShort(0)),
        (Kind::Long, Value::Long(0)),
        (Kind::UnsignedLong, Value::UnsignedLong(0)),
        (Kind::LongLong, Value::LongLong(0)),
        (Kind::UnsignedLongLong, Value::UnsignedLongLong(0)),
    ];

    #[test]
    fn wrapping_respects_each_width_and_signedness() {
        for (number, kind, expected) in [
            (128.0, Kind::Byte, Value::Byte(-128)),
            (-129.0, Kind::Byte, Value::Byte(127)),
            (257.9, Kind::Byte, Value::Byte(1)),
            (256.0, Kind::Octet, Value::Octet(0)),
            (-1.9, Kind::Octet, Value::Octet(255)),
            (32_768.0, Kind::Short, Value::Short(-32_768)),
            (-32_769.0, Kind::Short, Value::Short(32_767)),
            (65_537.0, Kind::Short, Value::Short(1)),
            (65_536.0, Kind::UnsignedShort, Value::UnsignedShort(0)),
            (-2.9, Kind::UnsignedShort, Value::UnsignedShort(65_534)),
            (2_147_483_648.0, Kind::Long, Value::Long(i32::MIN)),
            (-2_147_483_649.0, Kind::Long, Value::Long(i32::MAX)),
            (4_294_967_297.0, Kind::Long, Value::Long(1)),
            (4_294_967_296.0, Kind::UnsignedLong, Value::UnsignedLong(0)),
            (-1.0, Kind::UnsignedLong, Value::UnsignedLong(u32::MAX)),
            (-1.0, Kind::LongLong, Value::LongLong(-1)),
            (-2.0, Kind::LongLong, Value::LongLong(-2)),
            (
                -1.0,
                Kind::UnsignedLongLong,
                Value::UnsignedLongLong(u64::MAX),
            ),
            (
                -2.0,
                Kind::UnsignedLongLong,
                Value::UnsignedLongLong(u64::MAX - 1),
            ),
            (
                9_223_372_036_854_775_808.0,
                Kind::LongLong,
                Value::LongLong(i64::MIN),
            ),
            (
                -9_223_372_036_854_775_808.0,
                Kind::LongLong,
                Value::LongLong(i64::MIN),
            ),
        ] {
            assert_eq!(
                convert(number, kind, Mode::Wrap),
                Ok(expected),
                "{number}, {kind:?}"
            );
        }
    }

    #[test]
    fn wrapping_preserves_exact_residues_near_and_above_two_to_the_64() {
        for (number, expected) in [
            (18_446_744_073_709_549_568.0, u64::MAX - 2047),
            (-18_446_744_073_709_549_568.0, 2048),
            (18_446_744_073_709_551_616.0, 0),
            (-18_446_744_073_709_551_616.0, 0),
            (18_446_744_073_709_555_712.0, 4096),
            (-18_446_744_073_709_555_712.0, u64::MAX - 4095),
            (
                2.0_f64.powi(115) + 2.0_f64.powi(63),
                9_223_372_036_854_775_808,
            ),
            (2.0_f64.powi(116), 0),
            (f64::MAX, 0),
            (-f64::MAX, 0),
        ] {
            assert_eq!(
                convert(number, Kind::UnsignedLongLong, Mode::Wrap),
                Ok(Value::UnsignedLongLong(expected)),
                "{number}"
            );
        }
        assert_eq!(
            convert(18_446_744_073_709_549_568.0, Kind::LongLong, Mode::Wrap),
            Ok(Value::LongLong(-2048))
        );
    }

    #[test]
    fn wrapping_normalizes_nonfinite_zero_and_subinteger_inputs() {
        for (kind, zero) in ZERO_VALUES {
            for number in [
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
                -0.0,
                0.9,
                -0.9,
                f64::MIN_POSITIVE,
                -f64::from_bits(1),
            ] {
                assert_eq!(
                    convert(number, kind, Mode::Wrap),
                    Ok(zero),
                    "{number}, {kind:?}"
                );
            }
        }
    }

    #[test]
    fn enforce_range_truncates_before_testing_bounds() {
        for (number, kind, expected) in [
            (127.9, Kind::Byte, Value::Byte(127)),
            (-128.9, Kind::Byte, Value::Byte(-128)),
            (255.9, Kind::Octet, Value::Octet(255)),
            (-0.9, Kind::Octet, Value::Octet(0)),
            (32_767.9, Kind::Short, Value::Short(32_767)),
            (-32_768.9, Kind::Short, Value::Short(-32_768)),
            (65_535.9, Kind::UnsignedShort, Value::UnsignedShort(65_535)),
            (2_147_483_647.9, Kind::Long, Value::Long(i32::MAX)),
            (-2_147_483_648.9, Kind::Long, Value::Long(i32::MIN)),
            (
                4_294_967_295.9,
                Kind::UnsignedLong,
                Value::UnsignedLong(u32::MAX),
            ),
            (-0.9, Kind::UnsignedLongLong, Value::UnsignedLongLong(0)),
        ] {
            assert_eq!(convert(number, kind, Mode::EnforceRange), Ok(expected));
        }
        for (kind, below, above) in [
            (Kind::Byte, -129.0, 128.0),
            (Kind::Octet, -1.0, 256.0),
            (Kind::Short, -32_769.0, 32_768.0),
            (Kind::UnsignedShort, -1.0, 65_536.0),
            (Kind::Long, -2_147_483_649.0, 2_147_483_648.0),
            (Kind::UnsignedLong, -1.0, 4_294_967_296.0),
        ] {
            for number in [below, above] {
                assert_eq!(
                    convert(number, kind, Mode::EnforceRange),
                    Err(IntegerOutOfRange)
                );
            }
        }
    }

    #[test]
    fn enforce_range_rejects_nonfinite_and_accepts_both_zeros() {
        for (kind, zero) in ZERO_VALUES {
            for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert_eq!(
                    convert(number, kind, Mode::EnforceRange),
                    Err(IntegerOutOfRange)
                );
            }
            assert_eq!(convert(-0.0, kind, Mode::EnforceRange), Ok(zero));
            assert_eq!(convert(0.0, kind, Mode::EnforceRange), Ok(zero));
        }
    }

    #[test]
    fn sixty_four_bit_enforcement_and_clamp_use_safe_integer_bounds() {
        for (kind, lower, upper) in [
            (
                Kind::LongLong,
                Value::LongLong(-9_007_199_254_740_991),
                Value::LongLong(9_007_199_254_740_991),
            ),
            (
                Kind::UnsignedLongLong,
                Value::UnsignedLongLong(0),
                Value::UnsignedLongLong(9_007_199_254_740_991),
            ),
        ] {
            assert_eq!(
                convert(9_007_199_254_740_991.0, kind, Mode::EnforceRange),
                Ok(upper)
            );
            assert_eq!(
                convert(9_007_199_254_740_992.0, kind, Mode::EnforceRange),
                Err(IntegerOutOfRange)
            );
            assert_eq!(
                convert(-9_007_199_254_740_992.0, kind, Mode::EnforceRange),
                Err(IntegerOutOfRange)
            );
            for number in [
                9_007_199_254_740_992.0,
                18_446_744_073_709_551_616.0,
                f64::INFINITY,
            ] {
                assert_eq!(convert(number, kind, Mode::Clamp), Ok(upper));
            }
            for number in [
                -9_007_199_254_740_992.0,
                -18_446_744_073_709_551_616.0,
                f64::NEG_INFINITY,
            ] {
                assert_eq!(convert(number, kind, Mode::Clamp), Ok(lower));
            }
        }
        assert_eq!(
            convert(-9_007_199_254_740_991.0, Kind::LongLong, Mode::EnforceRange),
            Ok(Value::LongLong(-9_007_199_254_740_991))
        );
        assert_eq!(
            convert(-1.0, Kind::UnsignedLongLong, Mode::EnforceRange),
            Err(IntegerOutOfRange)
        );
    }

    #[test]
    fn clamp_rounds_halfway_values_to_even() {
        for (number, expected) in [
            (0.5, 0),
            (1.5, 2),
            (2.5, 2),
            (3.5, 4),
            (-0.5, 0),
            (-1.5, -2),
            (-2.5, -2),
            (-3.5, -4),
            (2.49, 2),
            (2.51, 3),
        ] {
            assert_eq!(
                convert(number, Kind::Long, Mode::Clamp),
                Ok(Value::Long(expected))
            );
        }
        assert_eq!(convert(-1.5, Kind::Octet, Mode::Clamp), Ok(Value::Octet(0)));
        assert_eq!(
            convert(254.5, Kind::Octet, Mode::Clamp),
            Ok(Value::Octet(254))
        );
        assert_eq!(
            convert(255.5, Kind::Octet, Mode::Clamp),
            Ok(Value::Octet(255))
        );
    }

    #[test]
    fn clamp_saturates_every_width_and_maps_nan_to_zero() {
        for (kind, lower, upper, zero) in [
            (
                Kind::Byte,
                Value::Byte(-128),
                Value::Byte(127),
                Value::Byte(0),
            ),
            (
                Kind::Octet,
                Value::Octet(0),
                Value::Octet(255),
                Value::Octet(0),
            ),
            (
                Kind::Short,
                Value::Short(-32_768),
                Value::Short(32_767),
                Value::Short(0),
            ),
            (
                Kind::UnsignedShort,
                Value::UnsignedShort(0),
                Value::UnsignedShort(65_535),
                Value::UnsignedShort(0),
            ),
            (
                Kind::Long,
                Value::Long(i32::MIN),
                Value::Long(i32::MAX),
                Value::Long(0),
            ),
            (
                Kind::UnsignedLong,
                Value::UnsignedLong(0),
                Value::UnsignedLong(u32::MAX),
                Value::UnsignedLong(0),
            ),
            (
                Kind::LongLong,
                Value::LongLong(-9_007_199_254_740_991),
                Value::LongLong(9_007_199_254_740_991),
                Value::LongLong(0),
            ),
            (
                Kind::UnsignedLongLong,
                Value::UnsignedLongLong(0),
                Value::UnsignedLongLong(9_007_199_254_740_991),
                Value::UnsignedLongLong(0),
            ),
        ] {
            for number in [f64::NEG_INFINITY, -f64::MAX] {
                assert_eq!(convert(number, kind, Mode::Clamp), Ok(lower));
            }
            for number in [f64::INFINITY, f64::MAX] {
                assert_eq!(convert(number, kind, Mode::Clamp), Ok(upper));
            }
            for number in [f64::NAN, -0.0, 0.0] {
                assert_eq!(convert(number, kind, Mode::Clamp), Ok(zero));
            }
        }
    }
}
