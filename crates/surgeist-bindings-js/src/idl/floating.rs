//! WebIDL floating conversions after ECMAScript `ToNumber`.
//!
//! Implements `#js-float`, `#js-unrestricted-float`, `#js-double`, and
//! `#js-unrestricted-double` at WebIDL revision
//! `0620cf171d1b97138847088748e363cae4005156`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FloatOutOfRange;

pub(crate) fn float(number: f64) -> Result<f32, FloatOutOfRange> {
    // IEEE binary64-to-binary32 narrowing rounds to nearest, ties to even.
    // Infinity is the outcome for the spec's special +/-2^128 values. Check
    // after narrowing: values just above f32::MAX can still round to f32::MAX.
    let rounded = number as f32;
    if rounded.is_finite() {
        Ok(rounded)
    } else {
        Err(FloatOutOfRange)
    }
}

pub(crate) fn unrestricted_float(number: f64) -> f32 {
    if number.is_nan() {
        // The pinned js-to-unrestricted-float algorithm requires this payload.
        f32::from_bits(0x7fc0_0000)
    } else {
        number as f32
    }
}

pub(crate) fn double(number: f64) -> Result<f64, FloatOutOfRange> {
    if number.is_finite() {
        Ok(number)
    } else {
        Err(FloatOutOfRange)
    }
}

pub(crate) fn unrestricted_double(number: f64) -> f64 {
    if number.is_nan() {
        // The pinned js-to-unrestricted-double algorithm requires this payload.
        f64::from_bits(0x7ff8_0000_0000_0000)
    } else {
        number
    }
}

#[cfg(test)]
mod tests {
    use super::{FloatOutOfRange, double, float, unrestricted_double, unrestricted_float};

    #[test]
    fn restricted_float_checks_the_rounded_overflow_boundary() {
        // Max finite binary32 is 2^128 - 2^104; the next candidate is 2^128.
        // Their midpoint chooses the even special candidate and must fail.
        let midpoint = 2.0_f64.powi(128) - 2.0_f64.powi(103);
        let below = f64::from_bits(midpoint.to_bits() - 1);
        for sign in [-1.0, 1.0] {
            assert_eq!(
                float(sign * f64::from(f32::MAX)),
                Ok(sign as f32 * f32::MAX)
            );
            assert_eq!(float(sign * below), Ok(sign as f32 * f32::MAX));
            assert_eq!(float(sign * midpoint), Err(FloatOutOfRange));
            assert_eq!(float(sign * 2.0_f64.powi(128)), Err(FloatOutOfRange));
            assert_eq!(
                unrestricted_float(sign * midpoint),
                sign as f32 * f32::INFINITY
            );
        }
    }

    #[test]
    fn float_narrowing_uses_even_significands_and_preserves_zero_sign() {
        assert_eq!(float(1.0 + 2.0_f64.powi(-24)), Ok(1.0));
        assert_eq!(
            float(1.0 + 3.0 * 2.0_f64.powi(-24)),
            Ok(f32::from_bits(0x3f80_0002))
        );
        assert_eq!(float(2.0_f64.powi(-150)), Ok(0.0));
        assert_eq!(float(3.0 * 2.0_f64.powi(-150)), Ok(f32::from_bits(2)));
        for number in [-0.0, -2.0_f64.powi(-151)] {
            assert_eq!(float(number).map(f32::to_bits), Ok(0x8000_0000));
            assert_eq!(unrestricted_float(number).to_bits(), 0x8000_0000);
        }
        assert_eq!(double(-0.0).map(f64::to_bits), Ok(0x8000_0000_0000_0000));
        assert_eq!(unrestricted_double(-0.0).to_bits(), 0x8000_0000_0000_0000);
    }

    #[test]
    fn restricted_types_reject_nonfinite_values_and_double_retains_finite_values() {
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(float(number), Err(FloatOutOfRange));
            assert_eq!(double(number), Err(FloatOutOfRange));
        }
        for number in [f64::MAX, -f64::MAX, f64::from_bits(1), 1.25] {
            assert_eq!(double(number), Ok(number));
            assert_eq!(unrestricted_double(number), number);
        }
        assert_eq!(unrestricted_float(f64::INFINITY), f32::INFINITY);
        assert_eq!(unrestricted_float(f64::NEG_INFINITY), f32::NEG_INFINITY);
    }

    #[test]
    fn unrestricted_nan_is_canonicalized_to_the_specified_bits() {
        for number in [
            f64::NAN,
            f64::from_bits(0x7ff0_0000_0000_0001),
            f64::from_bits(0xffff_ffff_ffff_ffff),
        ] {
            assert_eq!(unrestricted_float(number).to_bits(), 0x7fc0_0000);
            assert_eq!(unrestricted_double(number).to_bits(), 0x7ff8_0000_0000_0000);
        }
    }
}
