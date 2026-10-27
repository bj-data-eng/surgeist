//! Media queries, media features and custom media.
//!
//! # Media, supports, imports, and prelude recovery
//!
//! Media syntax preserves unknown conditions separately from malformed-member recovery.
//! A structurally valid unknown feature or value is retained as
//! [`CssMediaLeaf::UnknownFeature`] with no diagnostic. Arbitrary checked enclosures
//! use [`CssMediaLeaf::GeneralEnclosed`] when no preceding grammar branch applies.
//! Both preserve unknown truth under negation; query evaluation belongs downstream.
//! A reserved or structurally
//! malformed list member becomes [`CssMediaQuery::Never`] and emits
//! [`CssRecoveryAction::ReplaceMediaQueryWithNever`](crate::diagnostic::CssRecoveryAction::ReplaceMediaQueryWithNever), allowing later comma siblings to survive.
//!
//! ```
//! use surgeist_css::query::CssConditionKind;
//! use surgeist_css::query::media::{CssMediaLeaf, CssMediaQuery};
//! use surgeist_css::diagnostic::CssRecoveryAction;
//! use surgeist_css::rule::CssRule;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet("@media (future-mode: active), ???, print {}");
//! let [CssRule::Media(media)] = report.syntax().rules() else {
//!     panic!("expected retained media rule");
//! };
//! assert!(matches!(
//!     media.query().queries(),
//!     [
//!         CssMediaQuery::Condition(condition),
//!         CssMediaQuery::Never(_),
//!         CssMediaQuery::Typed(_),
//!     ] if matches!(
//!         condition.kind(),
//!         CssConditionKind::Leaf(CssMediaLeaf::UnknownFeature(_))
//!     )
//! ));
//! assert!(matches!(
//!     report.diagnostics(),
//!     [diagnostic]
//!         if diagnostic.action() == CssRecoveryAction::ReplaceMediaQueryWithNever
//! ));
//! ```

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
