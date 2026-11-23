//! `Display` and `source()` of the crate-private scroll cause enums. The
//! expected text is written from the meaning of each rejected scroll fact.
use super::*;

fn chain(error: &(dyn std::error::Error + 'static)) -> Vec<String> {
    let mut out = vec![error.to_string()];
    let mut next = error.source();
    while let Some(level) = next {
        out.push(level.to_string());
        next = level.source();
    }
    out
}

const RECT_TEXT: &str = "scroll rectangle vertical size must be non-negative";
const COORDINATE_TEXT: &str = "the horizontal physical scroll offset inf is not finite";

fn rect_cause() -> ScrollRectErrorOf<f64> {
    ScrollRectErrorOf::NegativeSize {
        axis: PhysicalAxis::Vertical,
        value: -2.0,
    }
}

fn coordinate_cause() -> ScrollCoordinateErrorOf<f64> {
    ScrollCoordinateErrorOf::NonFinitePhysicalOffset {
        axis: PhysicalAxis::Horizontal,
        value: f64::INFINITY,
    }
}

/// Checks every row against its chain and that each variant has one row.
fn assert_chains<E>(table: &[(E, &[&str])], row: fn(&E) -> usize, rows: usize)
where
    E: std::error::Error + 'static,
{
    let mut seen = vec![false; rows];
    for (value, expected) in table {
        assert_eq!(&chain(value), expected, "chain of {value}");
        for level in expected.iter() {
            assert!(!level.contains(['{', '}']), "struct syntax in {level}");
        }
        assert!(!seen[row(value)], "two rows for one variant");
        seen[row(value)] = true;
    }
    assert!(seen.iter().all(|&covered| covered), "a variant has no row");
}

#[test]
fn canonical_scroll_geometry_failure_names_the_rejected_fact_and_sources_its_cause() {
    use CanonicalScrollGeometryErrorOf as C;
    let table: [(C<f64>, &[&str]); 10] = [
        (
            C::BoxClipGutter(ScrollBoxClipGutterErrorOf::InvalidClipMargin { value: -1.0 }),
            &[
                "the scroll box clip or gutter is invalid",
                "the overflow clip margin -1 is not finite and non-negative",
            ],
        ),
        (
            C::Contribution(ScrollContributionErrorOf::NonFiniteMargin {
                side: PhysicalSide::Top,
                value: f64::INFINITY,
            }),
            &[
                "a scroll contribution is invalid",
                "the top margin inf of a scroll contribution is not finite",
            ],
        ),
        (
            C::ScrollableOverflow(rect_cause()),
            &["the scrollable overflow rectangle is invalid", RECT_TEXT],
        ),
        (
            C::Range(coordinate_cause()),
            &["the scroll range is invalid", COORDINATE_TEXT],
        ),
        (
            C::RoundedRect {
                fact: CanonicalScrollRectFact::BorderBox,
                source: rect_cause(),
            },
            &["the rounded border box is invalid", RECT_TEXT],
        ),
        (
            C::RoundedRect {
                fact: CanonicalScrollRectFact::TargetBorderBox,
                source: rect_cause(),
            },
            &["the rounded target border box is invalid", RECT_TEXT],
        ),
        (
            C::RoundedContribution(coordinate_cause()),
            &["a rounded scroll contribution is invalid", COORDINATE_TEXT],
        ),
        (
            C::RoundedFinalInFlowEnd {
                side: PhysicalSide::Right,
                value: f64::INFINITY,
            },
            &["the rounded final in-flow end inf on the right side is not finite"],
        ),
        (
            C::RoundedScrollbarWidth { value: -1.0 },
            &["the rounded scrollbar width -1 is invalid"],
        ),
        (
            C::RoundedOptimalRegionInset {
                side: PhysicalSide::Bottom,
                value: f64::NAN,
            },
            &["the rounded optimal viewing region inset NaN on the bottom side is invalid"],
        ),
    ];
    assert_chains(
        &table,
        |error| match error {
            C::BoxClipGutter(_) => 0,
            C::Contribution(_) => 1,
            C::ScrollableOverflow(_) => 2,
            C::Range(_) => 3,
            C::RoundedRect {
                fact: CanonicalScrollRectFact::BorderBox,
                ..
            } => 4,
            C::RoundedRect {
                fact: CanonicalScrollRectFact::TargetBorderBox,
                ..
            } => 5,
            C::RoundedContribution(_) => 6,
            C::RoundedFinalInFlowEnd { .. } => 7,
            C::RoundedScrollbarWidth { .. } => 8,
            C::RoundedOptimalRegionInset { .. } => 9,
        },
        10,
    );
}

#[test]
fn scroll_contribution_failure_names_the_rejected_fact_and_sources_its_cause() {
    use ScrollContributionErrorOf as C;
    let table: [(C<f64>, &[&str]); 6] = [
        (
            C::Rect(rect_cause()),
            &["a scroll contribution rectangle is invalid", RECT_TEXT],
        ),
        (
            C::Coordinate(coordinate_cause()),
            &[
                "a scroll contribution coordinate is invalid",
                COORDINATE_TEXT,
            ],
        ),
        (
            C::NonFiniteMargin {
                side: PhysicalSide::Left,
                value: f64::NAN,
            },
            &["the left margin NaN of a scroll contribution is not finite"],
        ),
        (
            C::NonFiniteFinalInFlowEnd {
                axis: LogicalAxis::Inline,
                value: f64::INFINITY,
            },
            &["the final in-flow end inf on the inline axis is not finite"],
        ),
        (
            C::InvalidTerminalPadding {
                side: PhysicalSide::Bottom,
                value: -1.0,
            },
            &["the terminal padding -1 on the bottom side is not finite and non-negative"],
        ),
        (
            C::NonFiniteTerminalEnd {
                side: PhysicalSide::Top,
                value: f64::NEG_INFINITY,
            },
            &["the terminal end -inf on the top side is not finite"],
        ),
    ];
    assert_chains(
        &table,
        |error| match error {
            C::Rect(_) => 0,
            C::Coordinate(_) => 1,
            C::NonFiniteMargin { .. } => 2,
            C::NonFiniteFinalInFlowEnd { .. } => 3,
            C::InvalidTerminalPadding { .. } => 4,
            C::NonFiniteTerminalEnd { .. } => 5,
        },
        6,
    );
}

#[test]
fn scroll_box_clip_gutter_failure_names_the_rejected_fact_and_sources_its_cause() {
    use ScrollBoxClipGutterErrorOf as C;
    let table: [(C<f64>, &[&str]); 6] = [
        (
            C::Rect(rect_cause()),
            &["a scroll box rectangle is invalid", RECT_TEXT],
        ),
        (
            C::Clip(coordinate_cause()),
            &["the overflow clip is invalid", COORDINATE_TEXT],
        ),
        (
            C::InvalidInset {
                side: PhysicalSide::Top,
                value: -2.0,
            },
            &["the top border or padding inset -2 is not finite and non-negative"],
        ),
        (
            C::InvalidClipMargin { value: f64::NAN },
            &["the overflow clip margin NaN is not finite and non-negative"],
        ),
        (
            C::InvalidPercentageBasis {
                axis: PhysicalAxis::Vertical,
            },
            &["the scrollport size on the vertical axis is not a valid percentage basis"],
        ),
        (
            C::InvalidOptimalRegionInset {
                side: PhysicalSide::Left,
            },
            &["the optimal viewing region inset on the left side cannot be resolved"],
        ),
    ];
    assert_chains(
        &table,
        |error| match error {
            C::Rect(_) => 0,
            C::Clip(_) => 1,
            C::InvalidInset { .. } => 2,
            C::InvalidClipMargin { .. } => 3,
            C::InvalidPercentageBasis { .. } => 4,
            C::InvalidOptimalRegionInset { .. } => 5,
        },
        6,
    );
}

#[test]
fn layout_failure_chain_names_each_level_down_to_the_rejected_coordinate() {
    let error: crate::LayoutErrorOf<u32, f64> = crate::error::layout_own_geometry_error(
        5,
        crate::output::RunMode::PerformLayout,
        CanonicalScrollGeometryErrorOf::Range(coordinate_cause()),
    );
    assert_eq!(
        chain(&error),
        [
            "child layout failed at node 5: scroll geometry rejected",
            "canonical scroll geometry is invalid",
            "the scroll range is invalid",
            COORDINATE_TEXT,
        ]
    );
}
