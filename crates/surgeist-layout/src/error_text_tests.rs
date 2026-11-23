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

fn rect_cause() -> crate::scroll::ScrollRectErrorOf<f64> {
    crate::scroll::ScrollRectErrorOf::NegativeSize {
        axis: PhysicalAxis::Vertical,
        value: -2.0,
    }
}

fn coordinate_cause() -> crate::scroll::ScrollCoordinateErrorOf<f64> {
    crate::scroll::ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
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

/// Every behavior with its words, checked complete by `behavior_index`.
fn behavior_rows() -> Vec<(SizingBehavior, &'static str)> {
    use crate::error::CalcSizeBehaviorBasis as B;
    vec![
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
    ]
}

/// The position of each variant in a list of rows. The exhaustive matches fail
/// to compile when a variant is added, and `assert_complete` fails when the
/// new variant has no row.
fn behavior_index(behavior: SizingBehavior) -> usize {
    use crate::error::CalcSizeBehaviorBasis as B;
    match behavior {
        SizingBehavior::MinContent => 0,
        SizingBehavior::MaxContent => 1,
        SizingBehavior::FitContentFunction => 2,
        SizingBehavior::Stretch => 3,
        SizingBehavior::FitContent => 4,
        SizingBehavior::Contain => 5,
        SizingBehavior::CalcSize(B::Auto) => 6,
        SizingBehavior::CalcSize(B::None) => 7,
        SizingBehavior::CalcSize(B::Content) => 8,
        SizingBehavior::CalcSize(B::MinContent) => 9,
        SizingBehavior::CalcSize(B::MaxContent) => 10,
        SizingBehavior::CalcSize(B::Stretch) => 11,
        SizingBehavior::CalcSize(B::FitContent) => 12,
        SizingBehavior::CalcSize(B::Contain) => 13,
    }
}

fn property_index(property: SizingProperty) -> usize {
    match property {
        SizingProperty::Preferred => 0,
        SizingProperty::Minimum => 1,
        SizingProperty::Maximum => 2,
        SizingProperty::FlexBasis => 3,
    }
}

fn algorithm_index(algorithm: SizingAlgorithm) -> usize {
    match algorithm {
        SizingAlgorithm::Leaf => 0,
        SizingAlgorithm::Block => 1,
        SizingAlgorithm::Flex => 2,
        SizingAlgorithm::Grid => 3,
        SizingAlgorithm::GridLanes => 4,
        SizingAlgorithm::Positioned => 5,
    }
}

fn writing_mode_index(mode: crate::node_input::WritingMode) -> usize {
    use crate::node_input::WritingMode as W;
    match mode {
        W::HorizontalTb => 0,
        W::VerticalRl => 1,
        W::VerticalLr => 2,
        W::SidewaysRl => 3,
        W::SidewaysLr => 4,
    }
}

/// Every index below `rows` appears exactly once.
fn assert_complete(indices: impl IntoIterator<Item = usize>, rows: usize) {
    let mut seen = vec![false; rows];
    for index in indices {
        assert!(
            index < rows,
            "a variant has no row: index {index} of {rows}"
        );
        assert!(!seen[index], "two rows for index {index}");
        seen[index] = true;
    }
    assert!(seen.iter().all(|&covered| covered), "a row is missing");
}

#[test]
fn unsupported_sizing_behavior_names_every_behavior() {
    let rows = behavior_rows();
    assert_complete(rows.iter().map(|(b, _)| behavior_index(*b)), rows.len());
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
    assert_complete(
        properties.iter().map(|(p, _)| property_index(*p)),
        properties.len(),
    );
    for (property, words) in properties {
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
    assert_complete(
        algorithms.iter().map(|(a, _)| algorithm_index(*a)),
        algorithms.len(),
    );
    for (algorithm, words) in algorithms {
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
    use crate::node_input::WritingMode as W;
    let modes = [
        (W::HorizontalTb, "horizontal-tb"),
        (W::VerticalRl, "vertical-rl"),
        (W::VerticalLr, "vertical-lr"),
        (W::SidewaysRl, "sideways-rl"),
        (W::SidewaysLr, "sideways-lr"),
    ];
    assert_complete(
        modes.iter().map(|(m, _)| writing_mode_index(*m)),
        modes.len(),
    );
    for (mode, words) in modes {
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
            grid_axis_label(crate::grid::GridAxisKind::Column),
            grid_axis_label(crate::grid::GridAxisKind::Row),
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
