//! Basic shapes, path data, clip-path and shape-outside values.

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
