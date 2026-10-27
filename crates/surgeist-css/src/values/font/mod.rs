//! Font shorthand and longhand values, variants, settings and palettes.
//!
//! # Typography, font families, and font-face
//!
//! Family lists, the `@font-face` family descriptor, and `local()` names follow
//! the selected September 7, 2026 Fonts 4 grammar. [`CssFontFamilyName`] preserves
//! decoded identifier tokens and distinguishes literal names from all fifteen
//! typed generic families. Family and font wrappers expose their
//! `families()` and `font()` models; use
//! [`CssFontValue`] and [`CssExplicitFont`] for explicit or system fonts.
//! Other typography includes four-ASCII-character OpenType tags, non-negative
//! feature indices, synthesis, and variant longhands. Individual metadata
//! records identify the implemented grammar and its dated source.
//!
//! ```
//! use surgeist_css::values::font::{CssFontValue, CssSystemFont};
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("font: menu; font-weight: 725");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::Font(font) = report.syntax()[0]
//!     .known().expect("known font")
//!     .property_value().expect("ordinary font")
//! else { panic!("expected font") };
//! assert!(matches!(font.value(), CssFontValue::System(CssSystemFont::Menu)));
//! assert_eq!(font.as_css(), "menu");
//! ```
//!
//! # Authored font-synthesis
//!
//! [`CssFontSynthesis`] represents `none` or a checked set of weight, style,
//! small-caps, and position capabilities. Intrinsic expansion contributes four
//! inherited longhands: selected capabilities become `auto`, omitted ones become
//! `none`. Their initial values are `auto`; only [`CssFontSynthesisStyle`] admits
//! `oblique-only`. Font matching and glyph synthesis remain downstream.
//!
//! ```
//! use surgeist_css::values::font::{CssFontSynthesis, CssFontSynthesisValues};
//!
//! let value = CssFontSynthesis::Values(
//!     CssFontSynthesisValues::try_new(true, false, false, true).unwrap(),
//! );
//! assert_eq!(value.serialize_specified(Default::default()).unwrap(), "weight position");
//! ```
//!
//! # Authored font palettes
//!
//! [`CssFontPalette`] retains keywords, symbolic names and checked ordered mixes.
//! [`CssFontPaletteMix`] shares color interpolation and percentage providers,
//! accepts one or more recursive palette components, and checks their complete
//! structural depth. Its specified output preserves symbolic values without
//! selecting a font or evaluating a palette. The property is inherited, initially
//! `normal`, and independent of the `font` shorthand.
//!
//! ```
//! use surgeist_css::values::font::{CssFontPalette, CssFontPaletteMix, CssFontPaletteMixComponent};
//!
//! let value = CssFontPalette::Mix(Box::new(CssFontPaletteMix::try_new(None, vec![
//!     CssFontPaletteMixComponent::new(CssFontPalette::Light, None),
//!     CssFontPaletteMixComponent::new(CssFontPalette::Dark, None),
//! ]).unwrap()));
//! assert_eq!(value.serialize_specified(Default::default()).unwrap(), "palette-mix(light, dark)");
//! ```

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
