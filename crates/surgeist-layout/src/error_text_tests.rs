//! Semantic `Display` text and `source()` of scroll geometry failures, whose
//! cause enums are crate-private, and the words of the private label helpers.
//! The expected text is written from the meaning of each rejected scroll fact
//! or label, not from the current output.
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
fn scroll_geometry_failure_names_the_rejected_scroll_fact_and_sources_its_cause() {
    let cases: [(LayoutScrollGeometryErrorOf<f64>, &str, &str); 5] = [
        (
            geometry_failure(CanonicalScrollGeometryErrorOf::Range(coordinate_cause())),
            "canonical scroll geometry is invalid",
            "the scroll range is invalid",
        ),
        (
            geometry_failure(ScrollContributionErrorOf::Coordinate(coordinate_cause())),
            "a scroll contribution is invalid",
            "a scroll contribution coordinate is invalid",
        ),
        (
            geometry_failure(ScrollBoxClipGutterErrorOf::Clip(coordinate_cause())),
            "a scroll box clip or gutter is invalid",
            "the overflow clip is invalid",
        ),
        (
            geometry_failure(rect_cause()),
            "a scroll rectangle is invalid",
            "scroll rectangle vertical size must be non-negative",
        ),
        (
            geometry_failure(coordinate_cause()),
            "a scroll coordinate is invalid",
            "the horizontal physical scroll offset inf is not finite",
        ),
    ];
    for (geometry, expected, cause) in cases {
        assert_eq!(geometry.to_string(), expected);
        let source = geometry
            .source()
            .unwrap_or_else(|| panic!("{expected}: the rejected fact is the cause"));
        assert_eq!(source.to_string(), cause);
    }
}

fn unsupported(
    property: SizingProperty,
    behavior: SizingBehavior,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
) -> String {
    LayoutUnsupportedCapability::SizingBehavior(UnsupportedSizingBehavior::new(
        property, behavior, algorithm, axis,
    ))
    .to_string()
}

/// Every behavior with its words; the exhaustive matches fail to compile when
/// a behavior or a calc-size basis is added without a row here.
fn behavior_rows() -> Vec<(SizingBehavior, &'static str)> {
    use CalcSizeBehaviorBasis as B;
    let rows = vec![
        (SizingBehavior::MinContent, "min-content"),
        (SizingBehavior::MaxContent, "max-content"),
        (SizingBehavior::FitContentFunction, "fit-content function"),
        (SizingBehavior::Stretch, "stretch"),
        (SizingBehavior::FitContent, "fit-content"),
        (SizingBehavior::Contain, "contain"),
        (
            SizingBehavior::CalcSize(B::Auto),
            "calc-size with the auto basis",
        ),
        (
            SizingBehavior::CalcSize(B::None),
            "calc-size with the none basis",
        ),
        (
            SizingBehavior::CalcSize(B::Content),
            "calc-size with the content basis",
        ),
        (
            SizingBehavior::CalcSize(B::MinContent),
            "calc-size with the min-content basis",
        ),
        (
            SizingBehavior::CalcSize(B::MaxContent),
            "calc-size with the max-content basis",
        ),
        (
            SizingBehavior::CalcSize(B::Stretch),
            "calc-size with the stretch basis",
        ),
        (
            SizingBehavior::CalcSize(B::FitContent),
            "calc-size with the fit-content basis",
        ),
        (
            SizingBehavior::CalcSize(B::Contain),
            "calc-size with the contain basis",
        ),
    ];
    for (behavior, _) in &rows {
        match behavior {
            SizingBehavior::MinContent
            | SizingBehavior::MaxContent
            | SizingBehavior::FitContentFunction
            | SizingBehavior::Stretch
            | SizingBehavior::FitContent
            | SizingBehavior::Contain => {}
            SizingBehavior::CalcSize(basis) => match basis {
                B::Auto
                | B::None
                | B::Content
                | B::MinContent
                | B::MaxContent
                | B::Stretch
                | B::FitContent
                | B::Contain => {}
            },
        }
    }
    rows
}

#[test]
fn unsupported_sizing_behavior_names_every_behavior() {
    let rows = behavior_rows();
    assert_eq!(rows.len(), 14);
    for (behavior, words) in rows {
        assert_eq!(
            unsupported(
                SizingProperty::Preferred,
                behavior,
                SizingAlgorithm::Leaf,
                PhysicalAxis::Horizontal
            ),
            format!(
                "the {words} sizing behavior of the preferred size is not supported by the leaf \
                 algorithm on the horizontal axis"
            )
        );
    }
}

#[test]
fn unsupported_sizing_behavior_names_every_property_algorithm_and_axis() {
    let properties = [
        (SizingProperty::Preferred, "preferred size"),
        (SizingProperty::Minimum, "minimum size"),
        (SizingProperty::Maximum, "maximum size"),
        (SizingProperty::FlexBasis, "flex basis"),
    ];
    for (property, words) in properties {
        match property {
            SizingProperty::Preferred
            | SizingProperty::Minimum
            | SizingProperty::Maximum
            | SizingProperty::FlexBasis => {}
        }
        assert_eq!(
            unsupported(
                property,
                SizingBehavior::Stretch,
                SizingAlgorithm::Leaf,
                PhysicalAxis::Horizontal
            ),
            format!(
                "the stretch sizing behavior of the {words} is not supported by the leaf \
                 algorithm on the horizontal axis"
            )
        );
    }
    let algorithms = [
        (SizingAlgorithm::Leaf, "leaf"),
        (SizingAlgorithm::Block, "block"),
        (SizingAlgorithm::Flex, "flex"),
        (SizingAlgorithm::Grid, "grid"),
        (SizingAlgorithm::GridLanes, "grid lanes"),
        (SizingAlgorithm::Positioned, "positioned"),
    ];
    for (algorithm, words) in algorithms {
        match algorithm {
            SizingAlgorithm::Leaf
            | SizingAlgorithm::Block
            | SizingAlgorithm::Flex
            | SizingAlgorithm::Grid
            | SizingAlgorithm::GridLanes
            | SizingAlgorithm::Positioned => {}
        }
        assert_eq!(
            unsupported(
                SizingProperty::Preferred,
                SizingBehavior::Stretch,
                algorithm,
                PhysicalAxis::Vertical
            ),
            format!(
                "the stretch sizing behavior of the preferred size is not supported by the \
                 {words} algorithm on the vertical axis"
            )
        );
    }
}

#[test]
fn unsupported_text_writing_mode_names_every_writing_mode() {
    use crate::WritingMode as W;
    let modes = [
        (W::HorizontalTb, "horizontal-tb"),
        (W::VerticalRl, "vertical-rl"),
        (W::VerticalLr, "vertical-lr"),
        (W::SidewaysRl, "sideways-rl"),
        (W::SidewaysLr, "sideways-lr"),
    ];
    for (mode, words) in modes {
        match mode {
            W::HorizontalTb | W::VerticalRl | W::VerticalLr | W::SidewaysRl | W::SidewaysLr => {}
        }
        assert_eq!(
            LayoutUnsupportedCapability::TextWritingMode(mode).to_string(),
            format!(
                "the {words} writing mode needs a physical adapter that text-backed runs do not \
                 support"
            )
        );
    }
}

#[test]
fn axis_and_side_labels_are_words() {
    assert_eq!(
        [
            physical_axis_label(PhysicalAxis::Horizontal),
            physical_axis_label(PhysicalAxis::Vertical),
            logical_axis_label(LogicalAxis::Inline),
            logical_axis_label(LogicalAxis::Block),
            physical_side_label(PhysicalSide::Top),
            physical_side_label(PhysicalSide::Right),
            physical_side_label(PhysicalSide::Bottom),
            physical_side_label(PhysicalSide::Left),
            grid_axis_label(crate::GridAxisKind::Column),
            grid_axis_label(crate::GridAxisKind::Row),
        ],
        [
            "horizontal",
            "vertical",
            "inline",
            "block",
            "top",
            "right",
            "bottom",
            "left",
            "column",
            "row"
        ]
    );
}
