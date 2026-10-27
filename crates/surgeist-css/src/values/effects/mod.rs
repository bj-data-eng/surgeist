//! Filters, shadows, masks, clips and opacity values.
//!
//! # Shadows
//!
//! [`CssShadow`] requires blur before spread, and [`CssBoxShadowList`] is nonempty.
//! Box-shadow is a noninherited terminal with [`CssBoxShadow::None`] initial;
//! ordinary, CSS-wide, and pending values use the shared expansion and normalization
//! lifecycle. Length components form one contiguous group, while color and inset
//! may precede or follow that group. [`CssDropShadow`] has signed offsets and an
//! optional nonnegative `standard_deviation()`, with optional color on either side.
//! Its third length is a standard deviation; box-shadow retains `blur_radius()`.
//!
//! `serialize_specified()` and `serialize_specified()` on all four
//! shadow models retain authored omissions and explicit zero/currentcolor. Box
//! shadows emit color, offsets, blur, spread, then inset; lists use comma-space;
//! drop shadows emit the full `drop-shadow()` function. Numeric and color children
//! use their existing canonical providers with one cumulative resource budget.
//! These operations preserve symbolic math/colors and diagnostic origins.
//!
//! ```
//! use surgeist_css::values::shape::CssBasicShape;
//! use surgeist_css::values::effects::{CssClipPath, CssFilterFunction, CssFilter};
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::values::transform::{CssTransformFunction, CssTransform};
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents(concat!(
//!     "transform: translate3d(10%, 2px, 4em) rotate(45deg); ",
//!     "filter: blur(2px) drop-shadow(red 1px 2px 3px); ",
//!     "clip-path: polygon(round 2px, 0 0, 100% 0)",
//! ));
//! assert!(report.is_clean());
//!
//! let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
//!     .known().expect("known transform")
//!     .property_value().expect("ordinary transform")
//! else { panic!("expected transform") };
//! assert!(matches!(
//!     transform.value(),
//!     CssTransform::Functions(functions)
//!         if matches!(functions.functions()[0], CssTransformFunction::Translate3d(_))
//! ));
//!
//! let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[1]
//!     .known().expect("known filter")
//!     .property_value().expect("ordinary filter")
//! else { panic!("expected filter") };
//! assert!(matches!(
//!     filter.value(),
//!     CssFilter::Functions(functions)
//!         if matches!(functions.functions()[1], CssFilterFunction::DropShadow(_))
//! ));
//!
//! let CssKnownPropertyValueRef::ClipPath(clip) = report.syntax()[2]
//!     .known().expect("known clip path")
//!     .property_value().expect("ordinary clip path")
//! else { panic!("expected clip-path") };
//! assert!(matches!(
//!     clip.value(),
//!     CssClipPath::BasicShape(shape)
//!         if matches!(shape.shape(), CssBasicShape::Polygon(polygon) if polygon.round().is_some())
//! ));
//! ```
//!
//! # Filters
//!
//! Both filter properties are noninherited terminals with an intrinsic `none`
//! initial. [`CssFilterHueRotate`] retains an omitted angle separately from its
//! effective `0deg`; [`CssFilterBlur`] likewise retains omission and effective `0px`.
//! [`CssFilter`], [`CssFilterFunctionList`] and [`CssFilterFunction`] serialize
//! specified values under one cumulative resource budget. They retain authored
//! order, omitted arguments, explicit defaults, exact checked amounts and symbolic
//! children. Ordinary angle literals keep their finite precision and units;
//! calculations reuse the existing shared projection without filter-specific
//! normalization or clamping. The backdrop source selects only the named property
//! in an immutable exploring draft, with its Level 1 dependency pinned to the
//! selected 2018 publication. It does not select the complete Level 2 module.
//! The four authored SVG filter declarations are also complete CSS grammars:
//! `flood-color` and `lighting-color` reuse [`CssColor`](crate::color::CssColor), while `flood-opacity`
//! reuses the exact [`CssOpacityValue`] without authored range clamping.
//! [`CssColorInterpolationFilters`] distinguishes `Auto`, `Srgb` and `LinearRgb`
//! and emits lowercase specified keywords. Their intrinsic initials are black,
//! 1, white and linearRGB respectively; only color-interpolation-filters inherits.
//! SVG presentation attributes, applicability, computed opacity clamping and
//! filter/lighting execution remain downstream. The selected Filter1 property
//! is nonanimatable; this enum does not select a CSS filter-function color policy.
//!
//! ```
//! use surgeist_css::values::effects::{CssFilter, CssFilterFunction, CssFilterFunctionList, CssFilterHueRotate};
//! let functions = CssFilterFunctionList::try_new(vec![
//!     CssFilterFunction::HueRotate(CssFilterHueRotate::omitted()),
//! ]).unwrap();
//! assert_eq!(
//!     CssFilter::Functions(functions).serialize_specified(Default::default()).unwrap(),
//!     "hue-rotate()",
//! );
//! ```

pub(crate) mod clip;
pub(crate) mod filter_serialization;
pub(crate) mod images;
pub(crate) mod mask_serialization;
pub(crate) mod masking;
pub(crate) mod masking_serialization;
pub(crate) mod opacity_scalar;
pub(crate) mod shadow_serialization;
pub(crate) mod shapes;
pub(crate) mod syntax;

#[doc(inline)]
pub use images::{
    CssFilterImage, CssFilterImageInput, CssFilterImageInputRef, CssFilterImageString,
};
#[doc(inline)]
pub use masking::{
    CssClipRule, CssMaskBorder, CssMaskBoxList, CssMaskClip, CssMaskClipList, CssMaskComposite,
    CssMaskCompositeList, CssMaskLayerBoxes, CssMaskMode, CssMaskModeList, CssMaskType,
};
#[doc(inline)]
pub use opacity_scalar::{CssOpacityScalar, CssOpacityScalarKind};
#[doc(inline)]
pub use shapes::{CssClipPath, CssClipPathShape};
#[doc(inline)]
pub use syntax::{
    CssBoxShadow, CssBoxShadowList, CssClip, CssClipRect, CssColorInterpolationFilters,
    CssDropShadow, CssFilter, CssFilterAmount, CssFilterBlur, CssFilterFunction,
    CssFilterFunctionList, CssFilterHueRotate, CssIsolation, CssMaskLayer, CssMaskList,
    CssOpacityValue, CssShadow,
};
