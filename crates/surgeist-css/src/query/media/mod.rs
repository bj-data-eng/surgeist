//! Media queries, media features and custom media.

pub(crate) mod construction;
pub(crate) mod custom_media;
pub(crate) mod feature_name;
pub(crate) mod media_features;
pub(crate) mod syntax;

#[doc(inline)]
pub use construction::{
    CssMediaConstructionError, CssUnknownMediaFeature, CssUnknownMediaFeatureReason,
    CssUnknownMediaFeatureRef,
};
#[doc(inline)]
pub use custom_media::{CssCustomMediaBody, CssCustomMediaName, CssCustomMediaReference};
#[doc(inline)]
pub use feature_name::CssMediaFeatureName;
#[doc(inline)]
pub use media_features::{
    CssMediaColorGamut, CssMediaDynamicRange, CssMediaEnvironmentBlending, CssMediaFeatureKind,
    CssMediaFeatureQuery, CssMediaGrid, CssMediaGridRef, CssMediaInteger, CssMediaInvertedColors,
    CssMediaLength, CssMediaNavigationControls, CssMediaOverflowBlock, CssMediaOverflowInline,
    CssMediaRange, CssMediaRangeRef, CssMediaRatio, CssMediaReducedDataPreference,
    CssMediaResolution, CssMediaResolutionRef, CssMediaScripting, CssMediaUpdate,
};
#[doc(inline)]
pub use syntax::{
    CssColorSchemePreference, CssContrastPreference, CssDefinedFalseMediaReason, CssDisplayMode,
    CssForcedColorsMode, CssGridMode, CssHoverCapability, CssMediaLeaf, CssMediaQuery,
    CssMediaQueryList, CssMediaQueryModifier, CssMediaType, CssNeverMediaQuery, CssOrientation,
    CssPointerCapability, CssReducedMotionPreference, CssReducedTransparencyPreference,
    CssScanMode, CssTypedMediaQuery, CssUnknownMediaType,
};
