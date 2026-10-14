//! Numerical alpha policy shared by specified and used color serialization.

/// Selects the recoverable two- or three-decimal value for an alpha byte.
///
/// Color4 16.1.1's defining integer-rounding steps take precedence over its
/// conflicting byte-236 example: 236 selects 0.925 and 237 selects 0.93.
/// Formatting and its admission/storage remain with each serializer. This
/// operation applies only to proven byte alpha, never arbitrary floating alpha.
pub fn byte_alpha_for_serialization(alpha: u8) -> f64 {
    let nearest = (u32::from(alpha) * 100 + 127) / 255;
    if (nearest * 255 + 50) / 100 == u32::from(alpha) {
        f64::from(nearest) / 100.0
    } else {
        f64::from((u32::from(alpha) * 1000 + 127) / 255) / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_byte_selects_recoverable_hundredths_or_nearest_thousandth() {
        // Enumerate recoverable hundredths, then minimize exact rational
        // distance; this oracle is independent of the production formula.
        for alpha in 0..=u8::MAX {
            let thousandths = (0..=100_u32)
                .find(|percent| (2 * percent * 255 + 100) / 200 == u32::from(alpha))
                .map_or_else(
                    || {
                        (0..=1000_u32)
                            .min_by_key(|candidate| {
                                (candidate * 255).abs_diff(u32::from(alpha) * 1000)
                            })
                            .unwrap()
                    },
                    |percent| percent * 10,
                );
            assert_eq!(
                byte_alpha_for_serialization(alpha),
                f64::from(thousandths) / 1000.0,
                "alpha {alpha}"
            );
        }
        assert_eq!(byte_alpha_for_serialization(236), 0.925);
        assert_eq!(byte_alpha_for_serialization(237), 0.93);
    }
}
