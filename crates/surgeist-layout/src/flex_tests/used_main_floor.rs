//! Used Flex main sizes preserve the real padding/border floor after min/max.
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{self, Phase};
use crate::*;

fn px<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn negative_limit<S: LayoutScalar>() -> SizingCalculationOf<S> {
    SizingCalculationOf::value(LengthPercentageOf::px(px(-20.0)).unwrap())
}

fn qualify<S: LayoutScalar>(grow: f64) {
    for right_margin in [0.0, -2.0] {
        let root_size = Size::new(px(100.0), px(20.0));
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    size: root_size.map(PreferredSizeOf::px),
                    align_items: Some(AlignItems::FlexStart),
                    align_content: Some(AlignContent::FlexStart),
                    justify_content: Some(AlignContent::FlexStart),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    box_sizing: BoxSizing::ContentBox,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
                    min_size: Size::new(MinSizeOf::calculation(negative_limit()), MinSizeOf::ZERO),
                    max_size: Size::new(MaxSizeOf::calculation(negative_limit()), MaxSizeOf::NONE),
                    flex_basis: FlexBasisOf::px(S::ZERO),
                    flex_grow: FlexGrowOf::try_new(px(grow)).unwrap(),
                    flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
                    padding: Edges {
                        left: LengthOf::px(px(3.0)),
                        right: LengthOf::px(px(4.0)),
                        ..Edges::all(LengthOf::ZERO)
                    },
                    border: Edges {
                        left: LengthOf::px(px(2.0)),
                        right: LengthOf::px(px(1.0)),
                        ..Edges::all(LengthOf::ZERO)
                    },
                    margin: Edges {
                        right: LengthAutoOf::px(px(right_margin)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(px(10.0)), PreferredSizeOf::px(px(10.0))),
                    min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    flex_grow: FlexGrowOf::ZERO,
                    flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let host = root_size.map(|value| Some(NonNegativeFiniteOf::new(value).unwrap()));
        let frame = crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            host,
            OrthogonalFallbackOf::new(
                host,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        let request = frame.request(root_size.map(|value| {
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).unwrap()),
                SizingOperation::Normal,
            )
        }));
        let batch = compute_layout(&tree, 0, request)
            .expect("native Flex used sizes retain the content-box zero floor");
        // The actual content basis is0; padding3+4 and border2+1 require an
        // outer10 even when the authored numeric min/max calculations are -20.
        // Grow0 freezes the hypothetical size. Grow1 remains bounded by the
        // same used maximum after free-space distribution. Signed margin is
        // outside that box and changes only the following child's flow origin.
        for (phase, entries) in [
            (
                "unrounded",
                readback::continuous_custody(&batch, Phase::Unrounded).as_slice(),
            ),
            (
                "final",
                readback::continuous_custody(&batch, Phase::Final).as_slice(),
            ),
        ] {
            for (node, size, location, content) in [
                (0, root_size, Point::ZERO, root_size),
                (
                    1,
                    Size::new(px(10.0), px(10.0)),
                    Point::ZERO,
                    Size::new(S::ZERO, px(10.0)),
                ),
                (
                    2,
                    Size::new(px(10.0), px(10.0)),
                    Point::new(px(10.0 + right_margin), S::ZERO),
                    Size::new(px(10.0), px(10.0)),
                ),
            ] {
                let actual = entries
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap_or_else(|| panic!("{phase} node{node}: missing native output"))
                    .output();
                assert_eq!(
                    actual.size, size,
                    "{phase} node{node} grow{grow} margin{right_margin}: border size"
                );
                assert_eq!(
                    actual.location, location,
                    "{phase} node{node}: normal position"
                );
                assert_eq!(
                    actual.content_box_size(),
                    content,
                    "{phase} node{node}: real content box"
                );
                assert_eq!(
                    actual.source_index,
                    SourceIndex::new(usize::from(node == 2)),
                    "{phase} node{node}: source child rank"
                );
                let scroll = actual
                    .scroll_geometry
                    .expect("native boxes retain canonical scroll facts");
                assert_eq!(
                    scroll.border_box().size(),
                    size,
                    "{phase} node{node}: canonical border"
                );
                if node == 1 {
                    assert_eq!(
                        actual.padding,
                        Edges::new(S::ZERO, px(4.0), S::ZERO, px(3.0))
                    );
                    assert_eq!(
                        actual.border,
                        Edges::new(S::ZERO, px(1.0), S::ZERO, px(2.0))
                    );
                    assert_eq!(
                        actual.margin,
                        Edges::new(S::ZERO, px(right_margin), S::ZERO, S::ZERO)
                    );
                } else {
                    assert_eq!(
                        (actual.padding, actual.border, actual.margin),
                        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
                    );
                }
            }
        }
    }
}

#[test]
fn flex_used_main_size_respects_content_box_floor_frozen_f32() {
    qualify::<f32>(0.0);
}
#[test]
fn flex_used_main_size_respects_content_box_floor_frozen_f64() {
    qualify::<f64>(0.0);
}
#[test]
fn flex_used_main_size_respects_content_box_floor_distributed_f32() {
    qualify::<f32>(1.0);
}
#[test]
fn flex_used_main_size_respects_content_box_floor_distributed_f64() {
    qualify::<f64>(1.0);
}
