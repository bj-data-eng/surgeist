//! Font shorthand and longhand values, variants, settings and palettes.

pub(crate) mod font_controls;
pub(crate) mod font_palette;
pub(crate) mod font_settings;
pub(crate) mod font_shorthand;
pub(crate) mod font_size;
pub(crate) mod font_style;
pub(crate) mod font_synthesis;
pub(crate) mod font_variant;
pub(crate) mod font_variant_serialization;
pub(crate) mod font_weight;
pub(crate) mod font_width;
pub(crate) mod syntax;

#[doc(inline)]
pub use font_controls::{
    CssFontKerning, CssFontLanguageOverride, CssFontLanguageString, CssFontOpticalSizing,
    CssFontSizeAdjust,
};
#[doc(inline)]
pub use font_palette::{
    CssFontPalette, CssFontPaletteMix, CssFontPaletteMixComponent,
    CssFontPaletteMixConstructionError,
};
#[doc(inline)]
pub use font_settings::{
    CssAuthoredFontFeature, CssAuthoredFontFeatureList, CssAuthoredFontFeatureSettings,
    CssAuthoredFontFeatureValue, CssFontVariation, CssFontVariationList, CssFontVariationSettings,
    CssOpenTypeTag,
};
#[doc(inline)]
pub use font_size::CssFontSize;
#[doc(inline)]
pub use font_style::{
    CssFontFaceObliqueRange, CssFontFaceStyle, CssFontStyle, CssFontStyleKeyword,
    CssObliqueAngleRange,
};
#[doc(inline)]
pub use font_synthesis::{
    CssFontSynthesis, CssFontSynthesisPosition, CssFontSynthesisSmallCaps, CssFontSynthesisStyle,
    CssFontSynthesisValues, CssFontSynthesisWeight,
};
#[doc(inline)]
pub use font_variant::{
    CssFontVariant, CssFontVariantAlternateNames, CssFontVariantAlternateValues,
    CssFontVariantAlternates, CssFontVariantCaps, CssFontVariantEastAsian,
    CssFontVariantEastAsianValues, CssFontVariantEastAsianVariant, CssFontVariantEastAsianWidth,
    CssFontVariantEmoji, CssFontVariantLigatureState, CssFontVariantLigatureValues,
    CssFontVariantLigatures, CssFontVariantNumeric, CssFontVariantNumericFigure,
    CssFontVariantNumericFraction, CssFontVariantNumericSpacing, CssFontVariantNumericValues,
    CssFontVariantPosition, CssFontVariantValue, CssFontVariantValues,
};
#[doc(inline)]
pub use font_weight::{
    CssAbsoluteFontWeight, CssFontFaceWeight, CssFontWeight, CssFontWeightRange,
};
#[doc(inline)]
pub use font_width::{CssFontFaceWidth, CssFontWidth, CssFontWidthKeyword};
#[doc(inline)]
pub use syntax::{
    CssExplicitFont, CssFontFamilyList, CssFontFamilyName, CssFontFamilyNameKind, CssFontValue,
    CssGenericFontFamily, CssSystemFont,
};
