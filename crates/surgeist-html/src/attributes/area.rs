//! Intrinsic area shape/coords author constraints and image-map coordinate descriptors, from HTML #attr-area-shape, #attr-area-coords and #image-map-processing-model.

pub use crate::internal::attributes::area::{
    HtmlAreaCircle, HtmlAreaCoordinatesAuthorError, HtmlAreaCoordinatesAuthorState,
    HtmlAreaCoordinatesReport, HtmlAreaEmptyReason, HtmlAreaIntegerPoint, HtmlAreaNativeError,
    HtmlAreaPolygon, HtmlAreaProcessedCircle, HtmlAreaProcessedPoint, HtmlAreaProcessedPolygon,
    HtmlAreaProcessedRectangle, HtmlAreaProcessedShape, HtmlAreaRectangle, HtmlAreaShape,
    HtmlAuthoredArea,
};
