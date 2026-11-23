//! Semantic `Display` text and `source()` of scroll geometry failures, whose
//! cause enums are crate-private. The expected text is written from the
//! meaning of each rejected scroll fact, not from the current output.
use super::*;
use crate::scroll::{
    CanonicalScrollGeometryErrorOf, ScrollBoxClipGutterErrorOf, ScrollContributionErrorOf,
};
use std::error::Error as _;

fn geometry_failure<E: Into<ScrollGeometryCauseOf<f64>>>(
    cause: E,
) -> LayoutScrollGeometryErrorOf<f64> {
    let error: LayoutErrorOf<u32, f64> =
        layout_own_geometry_error(5, RunMode::PerformLayout, cause);
    match error.kind() {
        LayoutErrorKindOf::ScrollGeometry(geometry) => *geometry,
        other => panic!("a rejected scroll fact is a scroll geometry failure: {other}"),
    }
}

fn rect_cause() -> crate::ScrollRectErrorOf<f64> {
    crate::ScrollRectErrorOf::NegativeSize {
        axis: PhysicalAxis::Vertical,
        value: -2.0,
    }
}

fn coordinate_cause() -> crate::ScrollCoordinateErrorOf<f64> {
    crate::ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
        axis: PhysicalAxis::Horizontal,
        value: f64::INFINITY,
    }
}

#[test]
fn scroll_geometry_failure_names_the_rejected_scroll_fact() {
    let cases: [(LayoutScrollGeometryErrorOf<f64>, &str); 5] = [
        (
            geometry_failure(CanonicalScrollGeometryErrorOf::Range(coordinate_cause())),
            "canonical scroll geometry is invalid",
        ),
        (
            geometry_failure(ScrollContributionErrorOf::Coordinate(coordinate_cause())),
            "a scroll contribution is invalid",
        ),
        (
            geometry_failure(ScrollBoxClipGutterErrorOf::Clip(coordinate_cause())),
            "a scroll box clip or gutter is invalid",
        ),
        (
            geometry_failure(rect_cause()),
            "a scroll rectangle is invalid",
        ),
        (
            geometry_failure(coordinate_cause()),
            "a scroll coordinate is invalid",
        ),
    ];
    for (geometry, expected) in cases {
        assert_eq!(geometry.to_string(), expected);
        assert!(
            geometry.source().is_some(),
            "{expected}: the rejected fact is the cause"
        );
    }
}
