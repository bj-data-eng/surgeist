//! Known properties, their grammars, metadata, expansion and values.

pub(crate) mod expansion;
pub(crate) mod properties;
pub(crate) mod property_name;
pub(crate) mod property_value;
pub(crate) mod svg_glyph;
pub(crate) mod syntax;

#[doc(inline)]
pub use expansion::{
    CssContributionValueRef, CssContributions, CssCustomPropertyContribution, CssExpansion,
    CssExpansionError, CssFourSideShorthandMetadata, CssInitialValueRef, CssLonghandContribution,
    CssLonghandContributions, CssLonghandInitialValue, CssLonghandMetadata, CssLonghandProperty,
    CssLonghandValue, CssLonghandValueRef, CssPendingSubstitution, CssPropertyKindRef,
    CssPropertyMetadata, CssPropertyMetadataError, CssShorthandMetadata,
    CssSvgGlyphOrientationVerticalContribution, CssUniversalReset, CssUniversalResetMetadata,
    CssUserAgentInitial,
};
#[doc(inline)]
pub use properties::{
    CssKnownDeclaration, CssKnownDeclaredValueRef, CssKnownProperty, CssKnownPropertyValueRef,
    CssPropertyGrammar, CssPropertyValue,
};
#[doc(inline)]
pub use property_name::CssPropertyName;
#[doc(inline)]
pub use property_value::{CssPropertyTarget, CssPropertyTextTarget, CssPropertyValueError};
#[doc(inline)]
pub use svg_glyph::{
    CssSvgGlyphOrientationVerticalDeclaration, CssSvgGlyphOrientationVerticalMetadata,
    CssSvgGlyphOrientationVerticalValue,
};
#[doc(inline)]
pub use syntax::{
    CssAllDeclaredValue, CssCustomPropertyMetadata, CssGuaranteedInvalidInitial, CssPropertyNameRef,
};
