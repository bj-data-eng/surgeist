//! Used-color operations over original source or checked authored values and facts.

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
pub(crate) mod palette;
pub(crate) mod profile;
pub(crate) mod profile_conversion;
pub(crate) mod relative;
pub(crate) mod resolution;
pub(crate) mod syntax_pass;
pub(crate) mod tokens;
