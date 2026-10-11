//! Opaque sRGB results of the pinned HTML legacy-color recovery algorithm.
//!
//! The selected HTML edition defines no common simple-color author grammar.
//! This module owns native RGB8 values and legacy recovery; full CSS parsing and
//! contextual color-input operations are separate contracts.

use crate::DocumentString;

/// An opaque sRGB color with exact eight-bit red, green and blue components.
///
/// Native construction and HTML legacy-color recovery share the same valid
/// value domain. Source spelling and recovery provenance belong to the
/// controlled interpretation report rather than this normalized value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HtmlLegacyColor {
    channels: [u8; 3],
}

impl HtmlLegacyColor {
    /// Constructs a color from intrinsically valid RGB8 components.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            channels: [red, green, blue],
        }
    }

    /// Returns red, green and blue in that order.
    pub const fn channels(&self) -> [u8; 3] {
        self.channels
    }

    /// Returns the red component.
    pub const fn red(&self) -> u8 {
        self.channels[0]
    }

    /// Returns the green component.
    pub const fn green(&self) -> u8 {
        self.channels[1]
    }

    /// Returns the blue component.
    pub const fn blue(&self) -> u8 {
        self.channels[2]
    }

    /// Returns an equivalent legacy spelling using lowercase `#rrggbb`.
    ///
    /// This is HTML-compatible serialization of opaque RGB8 sRGB, not a
    /// declaration of a simple-color author grammar in the selected HTML edition.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(format!(
            "#{:02x}{:02x}{:02x}",
            self.red(),
            self.green(),
            self.blue()
        ))
    }
}

/// The two source-defined legacy-color failures, independent of admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LegacyColorParseError {
    EmptyInput,
    Transparent,
}

fn ascii_whitespace(unit: u16) -> bool {
    matches!(unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
}

fn hex_value(code_point: u32) -> Option<u8> {
    match code_point {
        0x30..=0x39 => Some((code_point - 0x30) as u8),
        0x41..=0x46 => Some((code_point - 0x41 + 10) as u8),
        0x61..=0x66 => Some((code_point - 0x61 + 10) as u8),
        _ => None,
    }
}

/// HTML §2.3.6, over exact UTF-16, after the caller admits the original input.
pub(crate) fn parse_legacy_color(
    input: &DocumentString,
) -> Result<HtmlLegacyColor, LegacyColorParseError> {
    // Step 1 precedes trimming: nonempty whitespace recovers to black.
    if input.is_empty() {
        return Err(LegacyColorParseError::EmptyInput);
    }
    let mut units = input.code_units();
    while units.first().is_some_and(|&unit| ascii_whitespace(unit)) {
        units = &units[1..];
    }
    while units.last().is_some_and(|&unit| ascii_whitespace(unit)) {
        units = &units[..units.len() - 1];
    }

    // All 148 opaque names in selected CSS Color 4 §6.1 are ASCII, and the
    // longest (lightgoldenrodyellow) has 20 units. Larger/non-ASCII input cannot
    // match a name. This bounds conversion without losing arbitrary UTF-16.
    if units.len() <= 20 && units.iter().all(|&unit| unit <= 0x7f) {
        let name: String = units.iter().map(|&unit| char::from(unit as u8)).collect();
        if name.eq_ignore_ascii_case("transparent") {
            return Err(LegacyColorParseError::Transparent);
        }
        if let Ok((red, green, blue)) = cssparser::color::parse_named_color(&name) {
            return Ok(HtmlLegacyColor::new(red, green, blue));
        }
    }

    // Exactly '#' plus three ASCII hex digits also proves four code points;
    // a surrogate cannot participate in this branch.
    if let [0x23, red, green, blue] = units
        && let (Some(red), Some(green), Some(blue)) = (
            hex_value(u32::from(*red)),
            hex_value(u32::from(*green)),
            hex_value(u32::from(*blue)),
        )
    {
        return Ok(HtmlLegacyColor::new(red * 17, green * 17, blue * 17));
    }

    // Steps 6 and 7: emit only the transformed 128-point prefix. Valid
    // supplementary pairs become two zeroes; isolated surrogates retain their
    // own code point until the non-hex replacement below. Replacing before
    // truncating is observable when a pair crosses the transformed limit.
    let mut points = Vec::with_capacity(128);
    for decoded in char::decode_utf16(units.iter().copied()) {
        let code_point = match decoded {
            Ok(scalar) => u32::from(scalar),
            Err(error) => u32::from(error.unpaired_surrogate()),
        };
        if code_point > 0xffff {
            points.push(0x30);
            if points.len() < 128 {
                points.push(0x30);
            }
        } else {
            points.push(code_point);
        }
        if points.len() == 128 {
            break;
        }
    }

    // Steps 8–10: remove one initial '#', normalize non-hex points to zero,
    // and pad. Every remaining byte is a numeric nibble (not ASCII spelling).
    let points = points.strip_prefix(&[0x23]).unwrap_or(&points);
    let mut digits: Vec<u8> = points
        .iter()
        .map(|&point| hex_value(point).unwrap_or(0))
        .collect();
    while digits.is_empty() || !digits.len().is_multiple_of(3) {
        digits.push(0);
    }

    // Steps 11–14: equal thirds, final eight, synchronized leading-zero
    // removal, then first two. Moving shared slice offsets avoids copying.
    let component_length = digits.len() / 3;
    let mut red = &digits[..component_length];
    let mut green = &digits[component_length..component_length * 2];
    let mut blue = &digits[component_length * 2..];
    if component_length > 8 {
        let start = component_length - 8;
        red = &red[start..];
        green = &green[start..];
        blue = &blue[start..];
    }
    while red.len() > 2 && red[0] == 0 && green[0] == 0 && blue[0] == 0 {
        red = &red[1..];
        green = &green[1..];
        blue = &blue[1..];
    }
    if red.len() > 2 {
        red = &red[..2];
        green = &green[..2];
        blue = &blue[..2];
    }

    // Steps 15–19: a fallback component may contain just one nibble; it is
    // interpreted as that integer, without the shorthand multiplication.
    let component = |digits: &[u8]| digits.iter().fold(0, |value, &digit| value * 16 + digit);
    Ok(HtmlLegacyColor::new(
        component(red),
        component(green),
        component(blue),
    ))
}
