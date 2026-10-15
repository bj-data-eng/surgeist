//! Intrinsic Color-well decisions from HTML877d
//! #update-a-color-well-control-color, steps 2 and 4.
//!
//! Consumers establish an actual Input in the Color state, admit the selected
//! reads and service CSS/Color operations. These scalar decisions own no input,
//! identities, facts, controller, tree, storage or publication.

use crate::HtmlLegacyColor;

/// Which actual value the Color update must read before CSS parsing.
///
/// ValueAttribute means the exact null-namespace `value` content attribute;
/// missing and specified-empty remain distinct at that lookup. CurrentValue
/// does not request or inspect the content attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputColorValueSource {
    CurrentValue,
    ValueAttribute,
}

/// Selects the actual input from the established dirty value flag.
///
/// This selection neither reads a string nor changes the dirty flag.
pub const fn input_color_value_source(dirty_value: bool) -> HtmlInputColorValueSource {
    if dirty_value {
        HtmlInputColorValueSource::CurrentValue
    } else {
        HtmlInputColorValueSource::ValueAttribute
    }
}

/// The replacement value for an actual standard-invalid CSS color parse.
///
/// This is the existing checked opaque sRGB RGB8 value domain, not legacy-color
/// parsing or spelling. The consumer constructs its numerical Color from these
/// channels, preserves the CSS failure checkpoint, runs
/// HtmlInputColorSerialization and resolved CSS serialization, then installs the
/// resulting current value. It does not change the content value or dirty flag.
///
/// Unavailable facts, unsupported capabilities, numerical/protocol failures,
/// admission refusal and cancellation are errors outside this source branch;
/// none select this fallback.
pub const fn input_color_standard_failure_fallback() -> HtmlLegacyColor {
    HtmlLegacyColor::new(0, 0, 0)
}
