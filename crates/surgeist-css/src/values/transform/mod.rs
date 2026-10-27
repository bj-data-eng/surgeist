//! Transform, motion path and coordinate-box values.
//!
//! # Motion path
//!
//! Motion Path's five offset longhands and [`CssOffset`] compose the shared
//! numeric, URL, position and BasicShape owners. [`CssOffsetPath::view`](crate::values::transform::CssOffsetPath::view)
//! distinguishes none, box-only and a path with an optional [`CssCoordBox`].
//! [`CssRay`] retains a strict angle bearing, optional authored size, containment
//! and an optional [`CssPosition`](crate::values::position::CssPosition) from the full symbolic Cartesian, named-flow
//! and relative-flow families. [`CssOffsetRotate`] retains modifier/angle
//! absence without applying tangent rotation. Offset's five ordered reset
//! contributions use central normal/none/zero/auto/auto initials; its specified
//! output emits only authored constituents. Geometry, resources and animation
//! execution remain downstream.
//!
//! # Transform functions
//!
//! The typed transform family covers the selected two-dimensional Transforms 1
//! functions and the selected three-dimensional subset with exact arity,
//! separators, and dimensions. Matrices, rotation axes, scale numbers, and easing
//! coordinates share `CssNumeric<CssNumber>`, which retains exact literals and symbolic
//! math. All scale percentage branches use `CssNumeric<CssPercentage>`.
//! Ordinary cubic-bezier X coordinates are checked exactly against inclusive [0, 1];
//! genuine calculations remain unresolved. The independent `scale` property retains
//! one to three typed number/percentage factors, including symbolic calculations. Transform,
//! filter, gradient, and
//! image-orientation angles share exact `CssNumericLiteral<CssAngle>` values and symbolic
//! Angle-root calculations through the strict `CssNumeric<CssAngle>`. Transforms, filters,
//! and gradient directions admit bare zero through `CssAngleOrZero`; image orientation
//! and shape rotation require an angle dimension or calculation. Raw scalars retain provenance
//! while
//! semantic aggregates compare structure independently of scalar origins.
//! Easing values distinguish keywords,
//! `cubic-bezier()`, `steps()`, and authored `linear()` stops. Box shadows and filter
//! `drop-shadow()` have
//! separate models, filter lists preserve URL/function order, and the selected
//! basic-shape family exposes `inset()`, `circle()`, `ellipse()`, `polygon()`,
//! `rect()`, `xywh()`, `path()`, and `shape()`, including polygon `round <length>` and optional
//! rectangular round radii.

pub(crate) mod motion;
pub(crate) mod motion_serialization;
pub(crate) mod position;
pub(crate) mod rotate;
pub(crate) mod syntax;
pub(crate) mod transform_value_serialization;

#[doc(inline)]
pub use motion::{
    CssCoordBox, CssMotionConstructionError, CssOffset, CssOffsetAnchor, CssOffsetPath,
    CssOffsetPathKind, CssOffsetPathRef, CssOffsetPathValue, CssOffsetPosition, CssOffsetRotate,
    CssOffsetRotateModifier, CssRay, CssRaySize,
};
#[doc(inline)]
pub use position::CssTransformOrigin;
#[doc(inline)]
pub use rotate::{CssRotate, CssRotateAxis, CssRotateValues};
#[doc(inline)]
pub use syntax::{
    CssBackfaceVisibility, CssPerspective, CssScale, CssScaleValues, CssTransform, CssTransformBox,
    CssTransformFunction, CssTransformFunctionKind, CssTransformFunctionList, CssTransformMatrix,
    CssTransformMatrix3d, CssTransformPerspective, CssTransformRotate3d, CssTransformScale,
    CssTransformScale3d, CssTransformScaleComponent, CssTransformSkew, CssTransformStyle,
    CssTransformTranslate, CssTransformTranslate3d, CssTranslate, CssTranslateValues,
};
