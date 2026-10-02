use super::wide::Wide;
use crate::CssPredefinedColorSpace;

pub(super) fn decode(value: Wide, space: CssPredefinedColorSpace) -> Wide {
    use CssPredefinedColorSpace::{A98Rgb, DisplayP3, ProphotoRgb, Rec2020, Srgb};
    match space {
        Srgb | DisplayP3 => {
            if value.abs().greater_than(0.04045) {
                let result = value
                    .abs()
                    .add_fixed(0.055)
                    .divide_fixed(1.055)
                    .signed_power(12, 5);
                with_sign(result, value)
            } else {
                value.divide_fixed(12.92)
            }
        }
        A98Rgb => value.signed_power(563, 256),
        ProphotoRgb => {
            if value.abs().greater_than(1.0 / 32.0) {
                value.signed_power(9, 5)
            } else {
                value.divide_fixed(16.0)
            }
        }
        Rec2020 => value.signed_power(12, 5),
        _ => value,
    }
}

pub(super) fn encode(value: Wide, space: CssPredefinedColorSpace) -> Wide {
    use CssPredefinedColorSpace::{A98Rgb, DisplayP3, ProphotoRgb, Rec2020, Srgb};
    match space {
        Srgb | DisplayP3 => {
            if value.abs().greater_than(0.0031308) {
                let result = value
                    .abs()
                    .signed_power(5, 12)
                    .multiply_fixed(1.055)
                    .add_fixed(-0.055);
                with_sign(result, value)
            } else {
                value.multiply_fixed(12.92)
            }
        }
        A98Rgb => value.signed_power(256, 563),
        ProphotoRgb => {
            if value.abs().greater_than(1.0 / 512.0) {
                value.signed_power(5, 9)
            } else {
                value.multiply_fixed(16.0)
            }
        }
        Rec2020 => value.signed_power(5, 12),
        _ => value,
    }
}

fn with_sign(magnitude: Wide, original: Wide) -> Wide {
    // This comparison includes very small negative values whose full binary64
    // representation would underflow, so their sign never requires narrowing.
    if original.negated().greater_than(0.0) {
        magnitude.negated()
    } else {
        magnitude
    }
}
