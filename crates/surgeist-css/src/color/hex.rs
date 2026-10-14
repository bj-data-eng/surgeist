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
