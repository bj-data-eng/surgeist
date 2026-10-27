//! `@font-palette-values` rules and descriptors.
//!
//! A `@font-palette-values` rule has a dashed-ident [`CssFontPaletteName`] and the descriptors
//! `font-family`, `base-palette` and `override-colors`. The rule retains every valid descriptor
//! occurrence in authored order; palette selection and font loading belong to downstream owners.

pub(crate) mod font_palette_serialization;
pub(crate) mod rule;

#[doc(inline)]
pub use rule::{
    CssFontPaletteBase, CssFontPaletteDescriptor, CssFontPaletteDescriptorKind,
    CssFontPaletteDescriptorValue, CssFontPaletteDescriptorValueRef, CssFontPaletteName,
    CssFontPaletteOverride, CssFontPaletteValuesError, CssFontPaletteValuesRule,
};
