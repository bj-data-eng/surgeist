//! Numerical alpha policy shared by specified and used color serialization.

/// Selects the recoverable two- or three-decimal value for an alpha byte.
///
/// Color4 16.1.1's defining integer-rounding steps take precedence over its
/// conflicting byte-236 example: 236 selects 0.925 and 237 selects 0.93.
/// Formatting and its admission/storage remain with each serializer. This
/// operation applies only to proven byte alpha, never arbitrary floating alpha.
pub(crate) fn byte_alpha_for_serialization(alpha: u8) -> f64 {
    let nearest = (u32::from(alpha) * 100 + 127) / 255;
    if (nearest * 255 + 50) / 100 == u32::from(alpha) {
        f64::from(nearest) / 100.0
    } else {
        f64::from((u32::from(alpha) * 1000 + 127) / 255) / 1000.0
    }
}
