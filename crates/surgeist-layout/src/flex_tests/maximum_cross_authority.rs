//! A maximum cross size is a bound, not an allocated size or percentage basis.
use super::*;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn maximum_cross_source<S: LayoutScalar>(
    root_height: PreferredSizeOf<S>,
    max_height: Option<S>,
    child_height: PreferredSizeOf<S>,
    alignment: AlignItems,
) -> PublicLayoutTreeOf<S> {
    let mut root = NodeInputOf {
        display: Display::Flex,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        size: Size::new(PreferredSizeOf::px(n(200.0)), root_height),
        align_items: Some(alignment),
        align_content: Some(AlignContent::FlexStart),
        justify_content: Some(AlignContent::FlexStart),
        ..NodeInputOf::default()
    };
    if let Some(height) = max_height {
        root.max_size.height = MaxSizeOf::px(height);
    }
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(0, root)
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::AUTO, child_height),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_grow: FlexGrowOf::ZERO,
                flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .measure(1, Size::new(n(20.0), n(10.0)))
}

fn unbounded_normal_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let width = NonNegativeFiniteOf::new(n(200.0)).unwrap();
    let frame = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::new(Some(width), None),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let axes = Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    );
    let request = frame.request(axes);
    assert_eq!(request.sizing_request(), frame.sizing_request(axes));
    assert_eq!(
        request.context(),
        LayoutRootContextOf::Standalone {
            containing_flow_axes: frame.containing_flow_axes(),
            percentage_parent: frame.percentage_parent(),
        }
    );
    request
}

fn imposed_height_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    // The external parent's inline axis is physical height. Its own allocation,
    // percentage parent and ICB authority are supplied independently.
    let parent = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let width = NonNegativeFiniteOf::new(n(200.0)).unwrap();
    let height = NonNegativeFiniteOf::new(n(100.0)).unwrap();
    let viewport = Size::splat(Some(NonNegativeFiniteOf::new(n(400.0)).unwrap()));
    let allocated = Size::new(None, Some(height));
    let context = FlexItemRootContextOf::new(parent, viewport, allocated);
    let sizing_request = SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
            AxisSizingRequestOf::new(AvailableSpaceOf::Definite(height), SizingOperation::Normal),
        ),
        OrthogonalFallbackOf::new(
            viewport,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let request = LayoutRootRequestOf::flex_item(sizing_request, context);
    assert_eq!(context.percentage_parent(), viewport);
    assert_eq!(context.allocated_size(), allocated);
    assert_eq!(context.parent_flow_axes(), parent);
    assert_eq!(request.sizing_request(), sizing_request);
    assert_eq!(request.context(), LayoutRootContextOf::FlexItem(context));
    request
}

fn assert_cross_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    root_height: f64,
    child_height: f64,
    context: &str,
) {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("final", batch.final_entries()),
    ] {
        for (node, expected) in [
            (0, Size::new(n(200.0), n(root_height))),
            (1, Size::new(n(20.0), n(child_height))),
        ] {
            let actual = entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap_or_else(|| panic!("{context} {phase} node {node}: missing native output"))
                .output();
            assert_eq!(
                actual.size, expected,
                "{context} {phase} node {node}: border size"
            );
            assert_eq!(
                actual.location,
                Point::ZERO,
                "{context} {phase} node {node}: location"
            );
            assert_eq!(
                actual.content_box_size(),
                expected,
                "{context} {phase} node {node}: content box"
            );
            assert_eq!(
                actual.source_index,
                SourceIndex::ZERO,
                "{context} {phase} node {node}: source index"
            );
            assert_eq!(
                (actual.margin, actual.border, actual.padding),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO),
                "{context} {phase} node {node}: edges"
            );
            let scroll = actual.scroll_geometry.unwrap_or_else(|| {
                panic!("{context} {phase} node {node}: missing canonical scroll geometry")
            });
            assert_eq!(
                scroll.flow_axes(),
                axes,
                "{context} {phase} node {node}: scroll axes"
            );
            for (field, rect) in [
                ("border box", scroll.border_box()),
                ("padding box", scroll.padding_box()),
                ("content box", scroll.content_box()),
                ("scrollport", scroll.scrollport()),
                ("scrollable overflow", scroll.scrollable_overflow()),
            ] {
                assert_eq!(
                    rect.origin(),
                    Point::ZERO,
                    "{context} {phase} node {node}: {field} origin"
                );
                assert_eq!(
                    rect.size(),
                    expected,
                    "{context} {phase} node {node}: {field} size"
                );
            }
            assert_eq!(
                scroll.scrollbar_size(),
                Size::ZERO,
                "{context} {phase} node {node}: scrollbars"
            );
            let range = scroll.physical_range();
            assert_eq!(
                (
                    range.x().minimum(),
                    range.x().maximum(),
                    range.y().minimum(),
                    range.y().maximum()
                ),
                (S::ZERO, S::ZERO, S::ZERO, S::ZERO),
                "{context} {phase} node {node}: scroll range"
            );
        }
    }
}

fn stretch_from_actual_line<S: LayoutScalar>() {
    let natural = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        None,
        PreferredSizeOf::AUTO,
        AlignItems::Stretch,
    );
    let natural = compute_layout(&natural, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&natural, 10.0, 10.0, "native natural-height control");

    let definite = maximum_cross_source::<S>(
        PreferredSizeOf::px(n(100.0)),
        Some(n(100.0)),
        PreferredSizeOf::AUTO,
        AlignItems::Stretch,
    );
    let definite = compute_layout(&definite, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&definite, 100.0, 100.0, "authored definite-height control");

    let maximum = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        Some(n(100.0)),
        PreferredSizeOf::AUTO,
        AlignItems::Stretch,
    );
    let imposed = compute_layout(&maximum, 0, imposed_height_request()).unwrap();
    assert_cross_geometry(&imposed, 100.0, 100.0, "actual host-known-height control");

    // The natural line is10; max-height100 only clamps it. Stretch uses that
    // actual line size rather than allocating the maximum in advance.
    let maximum = compute_layout(&maximum, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&maximum, 10.0, 10.0, "maximum-only cross size");
}

fn percentage_requires_actual_parent_size<S: LayoutScalar>() {
    let percentage =
        PreferredSizeOf::value(LengthPercentageOf::from_percent_fraction(n(0.5)).unwrap());
    let definite = maximum_cross_source::<S>(
        PreferredSizeOf::px(n(100.0)),
        Some(n(100.0)),
        percentage.clone(),
        AlignItems::FlexStart,
    );
    let definite = compute_layout(&definite, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(
        &definite,
        100.0,
        50.0,
        "authored definite percentage basis control",
    );

    let maximum = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        Some(n(100.0)),
        percentage,
        AlignItems::FlexStart,
    );
    let imposed = compute_layout(&maximum, 0, imposed_height_request()).unwrap();
    assert_cross_geometry(
        &imposed,
        100.0,
        50.0,
        "actual host-known percentage basis control",
    );

    // A content-dependent parent height does not resolve the item's cyclic
    // percentage against its maximum. The percentage behaves as Auto.
    let maximum = compute_layout(&maximum, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&maximum, 10.0, 10.0, "maximum-only percentage parent");
}

fn percentage_under_auto_parent_behaves_as_auto<S: LayoutScalar>() {
    let automatic = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        None,
        PreferredSizeOf::AUTO,
        AlignItems::FlexStart,
    );
    let automatic = compute_layout(&automatic, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&automatic, 10.0, 10.0, "automatic-height native control");

    // Resolving the line establishes the item's used cross size for layout;
    // it does not resolve the item's own cyclic percentage against its parent.
    let percentage =
        PreferredSizeOf::value(LengthPercentageOf::from_percent_fraction(n(0.5)).unwrap());
    let natural = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        None,
        percentage,
        AlignItems::FlexStart,
    );
    let natural = compute_layout(&natural, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(
        &natural,
        10.0,
        10.0,
        "auto parent keeps cyclic percentage automatic",
    );
}

#[test]
fn max_only_flex_cross_size_stretches_to_natural_line_f32() {
    stretch_from_actual_line::<f32>();
}
#[test]
fn max_only_flex_cross_size_stretches_to_natural_line_f64() {
    stretch_from_actual_line::<f64>();
}
#[test]
fn max_only_flex_cross_size_does_not_resolve_percentage_height_f32() {
    percentage_requires_actual_parent_size::<f32>();
}
#[test]
fn max_only_flex_cross_size_does_not_resolve_percentage_height_f64() {
    percentage_requires_actual_parent_size::<f64>();
}

#[test]
fn auto_height_flex_cross_size_does_not_resolve_percentage_height_f32() {
    percentage_under_auto_parent_behaves_as_auto::<f32>();
}
#[test]
fn auto_height_flex_cross_size_does_not_resolve_percentage_height_f64() {
    percentage_under_auto_parent_behaves_as_auto::<f64>();
}

fn assert_flex_cross_descendant_percentages_follow_stretch<S: LayoutScalar>() {
    let px = S::from_f64;
    let layout = |root_height, item_height, align_self, margin_top| {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [4])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(PreferredSizeOf::px(px(100.0)), root_height),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    align_self,
                    size: Size::new(PreferredSizeOf::px(px(20.0)), item_height),
                    margin: Edges {
                        top: margin_top,
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::percent(px(0.5))),
                    ..NodeInputOf::default()
                },
            )
            .style(
                4,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(40.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch =
            compute_layout(&tree, 1, unbounded_normal_request()).expect("flex row layout succeeds");
        [2, 3].map(|node| {
            batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
                .size
                .height
        })
    };

    let auto = || PreferredSizeOf::AUTO;
    let zero = LengthAutoOf::ZERO;
    assert_eq!(
        layout(PreferredSizeOf::px(px(80.0)), auto(), None, zero),
        [px(80.0), px(40.0)]
    );
    assert_eq!(layout(auto(), auto(), None, zero), [px(40.0), px(20.0)]);
    assert_eq!(
        layout(
            auto(),
            PreferredSizeOf::px(px(40.0)),
            Some(AlignItems::Start),
            zero
        ),
        [px(40.0), px(20.0)]
    );
    assert_eq!(
        layout(auto(), auto(), Some(AlignItems::Start), zero),
        [px(40.0), px(40.0)]
    );
    assert_eq!(
        layout(auto(), auto(), None, LengthAutoOf::AUTO),
        [px(40.0), px(40.0)]
    );
}

#[test]
fn flex_cross_descendant_percentages_follow_stretch() {
    assert_flex_cross_descendant_percentages_follow_stretch::<f32>();
    assert_flex_cross_descendant_percentages_follow_stretch::<f64>();
}

fn assert_flex_grid_auto_repeat_uses_initial_block_definiteness<S: LayoutScalar>() {
    let px = S::from_f64;
    let layout = |root_height| {
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1])
            .children(1, [2, 3])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    size: Size::new(PreferredSizeOf::px(px(100.0)), root_height),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    flex_basis: FlexBasisOf::CONTENT,
                    grid_template_columns: vec![TrackComponentOf::px(px(100.0))],
                    grid_template_rows: vec![TrackComponentOf::Repeat(
                        TrackRepetitionOf::auto_fill_components(vec![TrackComponentOf::px(px(
                            20.0,
                        ))])
                        .expect("valid fixed auto-fill repetition"),
                    )],
                    grid_auto_rows: vec![TrackComponentOf::px(px(40.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(2, NodeInputOf::default())
            .style(3, NodeInputOf::default());
        let batch = compute_layout(&tree, 0, unbounded_normal_request())
            .expect("nested Grid layout succeeds");
        [1, 2, 3].map(|node| {
            let output = batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output();
            (output.location.y, output.size.height)
        })
    };

    assert_eq!(
        layout(PreferredSizeOf::px(px(60.0))),
        [
            (S::ZERO, px(60.0)),
            (S::ZERO, px(20.0)),
            (px(20.0), px(20.0))
        ]
    );
    assert_eq!(
        layout(PreferredSizeOf::AUTO),
        [
            (S::ZERO, px(60.0)),
            (S::ZERO, px(20.0)),
            (px(20.0), px(40.0))
        ]
    );
}

#[test]
fn flex_grid_auto_repeat_uses_initial_block_definiteness() {
    assert_flex_grid_auto_repeat_uses_initial_block_definiteness::<f32>();
    assert_flex_grid_auto_repeat_uses_initial_block_definiteness::<f64>();
}

fn assert_flex_lanes_repeat_uses_initial_block_definiteness<S: LayoutScalar>() {
    let px = S::from_f64;
    let zero = PreferredSizeOf::px(S::ZERO);
    let repeat = TrackRepetitionOf::auto_fill_components(vec![
        TrackComponentOf::px(px(10.0)),
        TrackComponentOf::MAX_CONTENT,
    ])
    .expect("valid intrinsic repetition");
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2, 3])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::GridLanes,
                flex_basis: FlexBasisOf::CONTENT,
                max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::px(px(120.0))),
                align_content: Some(AlignContent::Start),
                grid_template_rows: vec![
                    TrackComponentOf::px(px(40.0)),
                    TrackComponentOf::percent(px(0.5)),
                    TrackComponentOf::Repeat(repeat),
                    TrackComponentOf::line_names(["suffix"]),
                    TrackComponentOf::px(px(1.0)),
                ],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(zero.clone(), zero.clone()),
                raw_grid_row: RawGridPlacement::new(RawGridLine::Line(1), RawGridLine::Line(2)),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                position: Position::Absolute,
                size: Size::new(zero.clone(), zero),
                inset: Edges {
                    top: LengthAutoOf::ZERO,
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                raw_grid_row: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "suffix".into(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInputOf::default()
            },
        );
    let batch =
        compute_layout(&tree, 0, unbounded_normal_request()).expect("flex lanes layout succeeds");
    let output = |node| {
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
            .output()
    };
    let used_height = output(1).size.height;
    // Before the block basis is known, the percentage prefix adds no fixed
    // reservation. Seven (10px + 1px floor) groups fit after reserving 41px.
    assert_eq!(
        output(3).location.y,
        px(40.0) + used_height / px(2.0) + px(70.0)
    );
}

#[test]
fn flex_lanes_repeat_uses_initial_block_definiteness() {
    assert_flex_lanes_repeat_uses_initial_block_definiteness::<f32>();
    assert_flex_lanes_repeat_uses_initial_block_definiteness::<f64>();
}
