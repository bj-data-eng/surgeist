//! `color-scheme` and colour-adjust values.

pub(crate) mod color_adjustment;
pub(crate) mod utf16_value;

#[doc(inline)]
pub use color_adjustment::{
    CssColorScheme, CssColorSchemeConstructionError, CssColorSchemeKeyword, CssColorSchemeName,
    CssForcedColorAdjust, CssPrintColorAdjust,
};
#[doc(inline)]
pub use utf16_value::CssColorSchemeParseReport;
