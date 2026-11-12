// Binary64 inputs are dyadics. Their product has at most 106 significant bits.
// The retained fraction is exact for magnitudes of at least 1/8. If both
// products have discarded tails, their fractions cannot reach a half-pixel
// boundary together.
const FRACTION_BITS: u32 = 110;
const ONE: i128 = 1 << FRACTION_BITS;
const HALF: i128 = ONE / 2;

struct ScaledMagnitude {
    whole: Option<u64>,
    fraction: i128,
    has_tail: bool,
}

impl ScaledMagnitude {
    fn product(value: f64, scale: f64) -> Self {
        let (value, value_exponent) = magnitude_parts(value);
        let (scale, scale_exponent) = magnitude_parts(scale);
        let significand = u128::from(value) * u128::from(scale);
        let exponent = value_exponent + scale_exponent;
        if significand == 0 {
            return Self {
                whole: Some(0),
                fraction: 0,
                has_tail: false,
            };
        }
        if exponent >= 0 {
            let shift = exponent as u32;
            let bits = u128::BITS - significand.leading_zeros();
            return Self {
                whole: (bits + shift <= u64::BITS).then(|| (significand as u64) << shift),
                fraction: 0,
                has_tail: false,
            };
        }

        let shift = exponent.unsigned_abs();
        let integer = significand.checked_shr(shift).unwrap_or(0);
        let remainder = significand - integer.checked_shl(shift).unwrap_or(0);
        let (fraction, has_tail) = if shift <= FRACTION_BITS {
            (remainder << (FRACTION_BITS - shift), false)
        } else {
            let discard = shift - FRACTION_BITS;
            let kept = remainder.checked_shr(discard).unwrap_or(0);
            (kept, remainder != kept.checked_shl(discard).unwrap_or(0))
        };
        Self {
            whole: u64::try_from(integer).ok(),
            fraction: fraction as i128,
            has_tail,
        }
    }
}

fn magnitude_parts(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let encoded_exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1 << 52) - 1);
    if encoded_exponent == 0 {
        (fraction, -1074)
    } else {
        ((1 << 52) | fraction, encoded_exponent - 1075)
    }
}

fn phase_floor(phase: i128, negative_tail: bool) -> i128 {
    let whole = phase >> FRACTION_BITS;
    whole - i128::from(negative_tail && phase & (ONE - 1) == 0)
}

// The integral part of the absolute origin cancels between the two rounded
// edges. Only the fractional phase can add a pixel to the integral extent.
pub(super) fn pixel_extent(origin: f64, extent: f64, scale: f64) -> Option<u64> {
    let origin_magnitude = ScaledMagnitude::product(origin, scale);
    let span = ScaledMagnitude::product(extent, scale);
    let phase = if origin.is_sign_negative() {
        HALF - origin_magnitude.fraction
    } else {
        HALF + origin_magnitude.fraction
    };
    let negative_tail = origin.is_sign_negative() && origin_magnitude.has_tail;
    let start = phase_floor(phase, negative_tail);
    let end = phase_floor(phase + span.fraction, negative_tail && !span.has_tail);
    let carry = end - start;
    debug_assert!((0..=1).contains(&carry));
    span.whole?.checked_add(carry as u64)
}
