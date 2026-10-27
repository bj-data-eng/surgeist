//! `@font-palette-values` rules and descriptors.

pub(crate) mod font_palette_serialization;
pub(crate) mod rule;

#[doc(inline)]
pub use rule::{
    CssFontPaletteBase, CssFontPaletteDescriptor, CssFontPaletteDescriptorKind,
    CssFontPaletteDescriptorValue, CssFontPaletteDescriptorValueRef, CssFontPaletteName,
    CssFontPaletteOverride, CssFontPaletteValuesError, CssFontPaletteValuesRule,
};
