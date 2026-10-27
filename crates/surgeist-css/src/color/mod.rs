//! Used-color operations over original source or checked authored values and facts.
//!
//! # Authored colors
//!
//! [`CssColor`] is the sole checked authored color graph. It retains named,
//! hexadecimal, system, RGB/HSL/HWB, Lab/LCH, predefined `color()`, relative,
//! custom-profile, `alpha()`, and `color-mix()` branches. Ordinary color numbers,
//! percentages, and angles retain exact lexical coefficients and origins;
//! calculations remain symbolic. Color-bearing property wrappers expose this
//! graph through `value()`; aggregate values that carry a color, such as a background
//! layer, expose it through their own `color()` accessor.
//! [`CssColor::keyword_srgba8`](crate::color::CssColor::keyword_srgba8) exposes the intrinsic encoded sRGB bytes of
//! opaque named colors and transparent black, preserving their keyword identity.
//! It returns `None` for representations outside this fixed keyword projection.
//!
//! ```
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::color::CssSystemColor;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("color: ActiveBorder; border: solid #fff");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("color") };
//! assert_eq!(color.value().system(), Some(CssSystemColor::ActiveBorder));
//! let CssKnownPropertyValueRef::Border(border) = report.syntax()[1]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("border") };
//! assert_eq!(border.value().color().unwrap().hex_value().unwrap().digits(), "fff");
//! ```
//!
//! Checked constructors compose each color family without resolving
//! `currentcolor`, system colors, profiles, relative channels, or a mix. The
//! specified serializer applies cumulative resource limits and returns an atomic
//! canonical string. Authored out-of-range components are retained until their
//! computed-value phase. Opacity remains a separate authored numeric model.
//!
//! # Contrast, device colors and numerical helpers
//!
//! The `background-image` and `mask-image` values are [`CssImageValueList`](crate::values::image::CssImageValueList)s read
//! through `value()`, retaining URL, `none`, gradient and LightDark branches in order.
//! [`CssLightDarkColor::try_new`](crate::color::CssLightDarkColor::try_new) and [`CssLightDarkImage::try_new`](crate::values::image::CssLightDarkImage::try_new) retain two
//! checked branches and enforce the complete composed subtree's 256-level ceiling.
//! Color pairs remain contextual; image pairs may contain two `none` branches.
//! Specified serialization shares the enclosing resource budget and preserves both
//! branches. [`CssColorParse`](crate::color::controlled::CssColorParse) selects color branches from an explicit
//! [`CssColorSchemeFact`]; consumers supply that fact and select image branches
//! and resources.
//! [`CssContrastColor::try_new`](crate::color::CssContrastColor::try_new) retains one checked symbolic color input, enforcing
//! the complete composed depth ceiling. Its input uses ordinary Standalone specified
//! serialization under the enclosing budget. [`CssColorParse`](crate::color::controlled::CssColorParse) resolves
//! admitted source and facts, then uses [`surgeist_color::Color::black_white_contrast`]
//! to choose white or black; authored CSS retains the input unchanged.
//! [`CssDeviceCmykColor::try_new`](crate::color::CssDeviceCmykColor::try_new) retains four ordered, unbounded ink channels
//! and optional alpha. Legacy form is number-only with no alpha; modern form
//! also accepts percentages and `none`. [`CssColor::device_cmyk_value`](crate::color::CssColor::device_cmyk_value) borrows
//! the device-dependent input. Declared Standalone/Mix output scales direct
//! percentages to numbers; Origin preserves their domains and explicit alpha.
//! Calculations remain symbolic, and every child shares the enclosing depth and
//! serialization limits. The numerical owner, [`surgeist_color`], supplies the
//! independent concrete-sample helpers
//! [`surgeist_color::naively_convert_cmyk_to_srgba`] and
//! [`surgeist_color::naively_convert_srgba_to_cmyk`] implement
//! the uncalibrated Color 5 numerical fallback with [`surgeist_color::Finite`] coordinates.
//! They preserve alpha bits and do not resolve authored colors or device profiles.
//! Source resolution consults explicit [`CssDeviceColorFact`] values; those
//! contextual choices remain CSS-owned.
//! [`surgeist_color::convert_lab_to_lch`] and [`surgeist_color::convert_oklab_to_oklch`] provide
//! pure numerical
//! rectangular-to-polar coordinates from Color 4, with space-specific missing-hue
//! thresholds. [`surgeist_color::PolarCoordinates`] holds finite lightness/chroma and
//! normalizes optional hue; [`surgeist_color::convert_lch_to_lab`] and
//! [`surgeist_color::convert_oklch_to_oklab`]
//! invert those coordinates, setting both axes to zero when hue is missing.
//! These helpers use concrete number units independently of authored colors.
//! [`surgeist_color::HslCoordinates`] and [`surgeist_color::HwbCoordinates`] describe concrete
//! HSL and HWB coordinates with optional degree hue and percentage reference
//! units. [`surgeist_color::convert_hsl_to_srgb`], [`surgeist_color::convert_srgb_to_hsl`],
//! [`surgeist_color::convert_hwb_to_srgb`] and [`surgeist_color::convert_srgb_to_hwb`] convert
//! encoded sRGB
//! samples without gamut clipping. Inverse conversion marks powerless hue
//! missing while retaining the other coordinates; forward missing hue uses zero.
//! [`surgeist_color::RectangularCoordinates`] holds finite RGB, XYZ, Lab and Oklab
//! channels with explicit missingness. Its
//! [`convert_to`](surgeist_color::RectangularCoordinates::convert_to)
//! method converts among [`surgeist_color::RectangularSpace`] values without gamut
//! clipping, preserving intermediate range with binary64 significand precision.
//! Same-space conversion retains all bits; cross-space missing channels become
//! zero locally. Alpha and authored expression evaluation remain separate.

pub(crate) mod color_alpha;
pub(crate) mod color_expression;
pub(crate) mod color_scalar;
pub(crate) mod component_name;
pub mod controlled;
pub(crate) mod syntax;

#[doc(inline)]
pub use authored_resolution::controlled::{
    CssAuthoredColorOutcome, CssAuthoredColorStandardFailure,
};
#[doc(inline)]
pub use authored_resolution::ordinary::{CssAuthoredColorOrdinaryLimits, CssAuthoredColorResult};
#[doc(inline)]
pub use authored_resolution::path::{CssColorComponentPath, CssColorPathStep};
#[doc(inline)]
pub use bounded::{
    CssColorError, CssColorFactDefect, CssColorGrammarRequest, CssColorInputKind,
    CssColorInputRange, CssColorSourceDescriptor, CssColorStandardFailure,
    CssColorStandardFailureKind, CssColorUnavailable, CssColorUnsupported,
};
#[doc(inline)]
pub use color_alpha::{CssColorAlphaScalarRef, CssParsedColorAlphaRef};
#[doc(inline)]
pub use color_expression::{
    CssProfileColorCalculation, CssProfileColorExpression, CssProfileColorExpressionRef,
};
#[doc(inline)]
pub use color_scalar::{CssColorNumberLiteral, CssColorPercentageLiteral};
#[doc(inline)]
pub use component_name::CssColorComponentName;
#[doc(inline)]
pub use facts::{
    CssColorFactIdentity, CssColorFacts, CssColorPaletteEntry, CssColorProfileAvailability,
    CssColorProfileBinding, CssColorProfileConversion, CssColorProfileConversionRecord,
    CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact,
};
#[doc(inline)]
pub use ordinary::{
    CssColorInput, CssColorOrdinaryLimits, CssColorOrdinaryOutcome, CssColorOrdinaryResource,
    CssColorOrdinaryResult, CssColorResolveError, parse_used_color,
};
#[doc(inline)]
pub use syntax::{
    CssAbsoluteColorEligibility, CssAbsoluteColorExclusion, CssAlphaColor, CssColor,
    CssColorComponent, CssColorConstructionError, CssColorHue, CssColorInterpolation,
    CssColorInterpolationMethod, CssColorMix, CssColorMixComponent, CssColorMixPercentage,
    CssColorMixWeight, CssColorProfileComponentName, CssColorProfileName, CssColorSyntax,
    CssContrastColor, CssCustomColor, CssDeviceCmykColor, CssHexColor, CssHslColor, CssHwbColor,
    CssLabColor, CssLchColor, CssLightDarkColor, CssNamedColor, CssPredefinedColor,
    CssRelativeColor, CssRelativeColorCalculation, CssRelativeColorChannel,
    CssRelativeColorEnvironment, CssRelativeColorExpression, CssRelativeColorExpressionValue,
    CssRelativeColorFunction, CssRelativeColorResultDomain, CssRelativeCustomColor, CssRgbColor,
    CssSystemColor,
};
pub(crate) mod authored;
#[cfg(test)]
pub(crate) mod authored_input_tests;
pub(crate) mod authored_resolution;

pub(crate) mod bounded;
pub(crate) mod channels;
pub(crate) mod component;
pub(crate) mod component_value;
pub(crate) mod decimal;
pub(crate) mod device;
pub(crate) mod expression;
pub(crate) mod facts;
pub(crate) mod grammar;
pub(crate) mod hex;
pub(crate) mod interpolate;
pub(crate) mod keywords;
pub(crate) mod literal;
pub(crate) mod operation;
pub(crate) mod ordinary;
pub(crate) mod ordinary_defaults;
pub(crate) mod palette;
pub(crate) mod profile;
pub(crate) mod profile_conversion;
pub(crate) mod relative;
pub(crate) mod resolution;
pub(crate) mod syntax_pass;
pub(crate) mod tokens;
