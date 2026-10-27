//! `color-scheme` and colour-adjust values.
//!
//! The color-adjustment values `color-scheme`, `forced-color-adjust` and `print-color-adjust`.
//! [`CssColorScheme`] also has a UTF-16 front, [`CssColorScheme::parse_utf16`], whose result is a
//! [`CssColorSchemeParseReport`].

pub(crate) mod color_adjustment;
pub(crate) mod utf16_value;

#[doc(inline)]
pub use color_adjustment::{
    CssColorScheme, CssColorSchemeConstructionError, CssColorSchemeKeyword, CssColorSchemeName,
    CssForcedColorAdjust, CssPrintColorAdjust,
};
#[doc(inline)]
pub use utf16_value::CssColorSchemeParseReport;
