//! Basic shapes, path data, clip-path and shape-outside values.
//!
//! # Shape-outside
//!
//! Authored Shapes 1 adds three independent noninherited longhands:
//! `shape-outside` borrows [`CssShapeOutside`], `shape-image-threshold` borrows
//! [`CssOpacityValue`](crate::values::effects::CssOpacityValue), and `shape-margin` borrows
//! [`CssNumeric<CssLengthPercentage, CssNonNegative>`](crate::numeric::CssNumeric). [`CssShapeOutsideShape`] retains
//! an optional [`CssShapeBox`] with only the four visual/margin boxes. Omitted
//! boxes stay distinct from explicit margin-box, and canonical pairs emit the
//! shape first. Threshold values remain unclamped; ordinary margin literals
//! are nonnegative while actual math defers range handling. Geometry, resource
//! loading, percentage bases and computed threshold clamping remain downstream.
//!
//! # Clip-path and basic shapes
//!
//! Clip-path is a noninherited terminal with [`CssClipPath::None`](crate::values::effects::CssClipPath::None) initial and the
//! shared ordinary, CSS-wide, pending-reentry and normalization lifecycle. It accepts
//! all seven [`CssBoxEdgeKeyword`](crate::values::common::CssBoxEdgeKeyword) alternatives alone or with one shape in either
//! authored order. [`CssClipPathShape`](crate::values::effects::CssClipPathShape) retains optional reference-box omission;
//! its serializer emits shape before box without resolving their geometry.
//! [`CssEllipseRadii`] contains two independent [`CssEllipseRadius`] components,
//! each an extent or a nonnegative length-percentage. [`CssEllipseShape::radii`](crate::values::shape::CssEllipseShape::radii)
//! distinguishes an omitted pair. Polygon round is an optional signed pure length,
//! following optional fill-rule; percentages and reversed modifiers are rejected.
//!
//! Clip-path, its composition, basic shapes and all eight function models provide
//! bounded specified serializers preserving optional omissions, shape offset/radius arities, units,
//! exact magnitudes and symbolic math through shared cumulative child providers.
//! Circle and ellipse positions retain the full imported Cartesian, named-flow, and
//! relative-flow grammar. [`CssRectShape`] retains four signed edges or `auto`;
//! [`CssXywhShape`] retains signed offsets and nonnegative width and height. Both
//! retain optional round radii. [`CssPathShape`] composes optional [`CssFillRule`]
//! with checked [`CssPathData`], preserving the decoded SVG spelling and string origin.
//! Its complete SVG scanner admits move-only paths and rejects malformed suffixes
//! without computing geometry. [`CssShapeFunction`] retains a full starting position
//! and a nonempty ordered [`CssShapeCommandList`]. Curves and smooth commands couple
//! absolute or relative endpoints with correspondingly typed controls; arcs retain
//! signed radii and optional sweep, size and strict angles. Specified output retains
//! authored omissions and emits the required first comma. Reference-box/default resolution,
//! clipping and
//! painting belong to downstream owners.

pub(crate) mod clip_path_serialization;
pub(crate) mod path_data;
pub(crate) mod shape_commands;
pub(crate) mod shape_outside_serialization;
pub(crate) mod shape_serialization;
pub(crate) mod shapes;

#[doc(inline)]
pub use path_data::CssPathData;
#[doc(inline)]
pub use shape_commands::{
    CssShapeAbsoluteControlPoint, CssShapeAbsoluteControlPointRef, CssShapeArc, CssShapeArcRadii,
    CssShapeArcSize, CssShapeArcSweep, CssShapeCommand, CssShapeCommandList,
    CssShapeConstructionError, CssShapeControlAnchor, CssShapeCoordinatePair, CssShapeCurve,
    CssShapeCurveRef, CssShapeEndpoint, CssShapeHorizontalLine, CssShapeRelativeControlPoint,
    CssShapeSmooth, CssShapeSmoothRef, CssShapeVerticalLine,
};
#[doc(inline)]
pub use shapes::{
    CssBasicShape, CssCircleRadius, CssCircleShape, CssEllipseRadii, CssEllipseRadius,
    CssEllipseShape, CssFillRule, CssInsetShape, CssInsetShapeOffsets, CssPathShape,
    CssPolygonPoint, CssPolygonPointList, CssPolygonShape, CssRectShape, CssShapeBox,
    CssShapeFunction, CssShapeOutside, CssShapeOutsideShape, CssXywhShape,
};
