//! One fixed hex-digit update and the shared short/long RGBA projection.

pub(super) fn write_digit(bytes: &mut [u8; 4], length: usize, position: usize, unit: char) -> bool {
    if !unit.is_ascii_hexdigit() || position >= length || !matches!(length, 3 | 4 | 6 | 8) {
        return false;
    }
    let digit = unit.to_digit(16).expect("ASCII hex") as u8;
    if length <= 4 {
        bytes[position] = digit;
    } else {
        bytes[position / 2] = (bytes[position / 2] << 4) | digit;
    }
    true
}
pub(super) fn finish(bytes: [u8; 4], length: usize) -> [u8; 4] {
    if length == 3 || length == 4 {
        [
            bytes[0] * 17,
            bytes[1] * 17,
            bytes[2] * 17,
            if length == 4 { bytes[3] * 17 } else { 255 },
        ]
    } else {
        [
            bytes[0],
            bytes[1],
            bytes[2],
            if length == 8 { bytes[3] } else { 255 },
        ]
    }
}

/// Expands a checked 3, 4, 6 or 8 digit hex color to RGBA bytes; omitted
/// alpha is opaque. Color 4 §5.2 doubles each digit of the short forms.
pub(crate) fn expand(digits: &str) -> Option<[u8; 4]> {
    let mut bytes = [0; 4];
    for (position, unit) in digits.chars().enumerate() {
        if !write_digit(&mut bytes, digits.len(), position, unit) {
            return None;
        }
    }
    Some(finish(bytes, digits.len()))
}
